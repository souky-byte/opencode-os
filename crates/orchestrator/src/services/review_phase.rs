use opencode_core::{Session, SessionPhase, Task, TaskStatus};
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{debug, info, warn};
use uuid::Uuid;
use vcs::Workspace;

use crate::error::{OrchestratorError, Result};
use crate::executor::{PhaseResult, StartedExecution};
use crate::prompts::PhasePrompts;
use crate::services::message_parser::ReviewResult;
use crate::services::{ExecutorContext, MessageParser};
use crate::session_runner::{McpConfig, SessionConfig, SessionDependencies, SessionRunner};

pub struct ReviewPhase;

impl ReviewPhase {
    pub async fn run(
        ctx: &ExecutorContext,
        task: &mut Task,
        iteration: u32,
    ) -> Result<PhaseResult> {
        info!(
            task_id = %task.id,
            iteration = iteration,
            max_iterations = ctx.config.max_review_iterations,
            "Starting AI_REVIEW session with MCP"
        );

        let mut session = Session::new(task.id, SessionPhase::Review);

        debug!("Creating OpenCode session for AI review");
        let opencode_session = ctx
            .opencode_client
            .create_session(&ctx.config.repo_path)
            .await?;
        let session_id_str = opencode_session.id.to_string();

        info!(
            opencode_session_id = %session_id_str,
            "OpenCode session created for AI review"
        );

        session.start(session_id_str.clone());
        ctx.persist_session(&session).await?;

        let activity_store = ctx.get_activity_store(session.id);
        ctx.emit_session_started(&session, task.id);

        let workspace_path = ctx.working_dir_for_task(task);

        if let Err(e) = ctx
            .mcp_manager
            .setup_findings_server(task.id, session.id, &workspace_path)
            .await
        {
            warn!(error = %e, "Failed to add MCP server, falling back to JSON parsing");
            return Self::run_json_fallback(
                ctx,
                task,
                session,
                session_id_str,
                activity_store,
                iteration,
            )
            .await;
        }

        debug!("Getting workspace diff for review");
        let diff = Self::get_workspace_diff(ctx, task).await?;
        debug!(diff_length = diff.len(), "Workspace diff retrieved");

        let prompt = PhasePrompts::review_with_mcp(task, &diff);
        debug!(
            prompt_length = prompt.len(),
            "Sending MCP review prompt to OpenCode"
        );

        let response_content = ctx
            .opencode_client
            .send_prompt(
                &session_id_str,
                &prompt,
                &workspace_path,
                activity_store.as_deref(),
            )
            .await;

        if let Err(e) = ctx
            .mcp_manager
            .cleanup_findings_server(&workspace_path)
            .await
        {
            debug!(error = %e, "MCP cleanup failed");
        }

        let response_content = match response_content {
            Ok(content) => content,
            Err(e) => {
                if let Some(ref store) = activity_store {
                    store.push_finished(false, Some(e.to_string()));
                }
                return Err(e);
            }
        };

        info!(
            response_length = response_content.len(),
            "Received AI review response"
        );

        let _review_path = ctx
            .file_manager
            .write_review(task.id, &response_content)
            .await?;

        session.complete();
        ctx.update_session(&session).await?;

        let review_result = match ctx.file_manager.read_findings(task.id).await {
            Ok(Some(findings)) => {
                info!(
                    approved = findings.approved,
                    finding_count = findings.findings.len(),
                    "AI review findings read from MCP server"
                );

                if findings.approved || findings.findings.is_empty() {
                    ReviewResult::Approved
                } else {
                    ReviewResult::FindingsDetected(findings.findings.len())
                }
            }
            Ok(None) => {
                warn!("No MCP findings file found, falling back to JSON parsing");
                Self::parse_and_save_findings(ctx, &response_content, task.id, session.id).await
            }
            Err(e) => {
                warn!(error = %e, "Failed to read MCP findings, falling back to JSON parsing");
                Self::parse_and_save_findings(ctx, &response_content, task.id, session.id).await
            }
        };

        let success = matches!(review_result, ReviewResult::Approved);

        if let Some(ref store) = activity_store {
            store.push_finished(success, None);
        }

        info!(
            review_result = ?review_result,
            "AI review result processed"
        );

        ctx.emit_session_ended(session.id, task.id, success);

        Self::handle_review_result(ctx, task, review_result, session_id_str, iteration).await
    }

    async fn run_json_fallback(
        ctx: &ExecutorContext,
        task: &mut Task,
        mut session: Session,
        session_id_str: String,
        activity_store: Option<Arc<crate::activity_store::SessionActivityStore>>,
        iteration: u32,
    ) -> Result<PhaseResult> {
        debug!("Getting workspace diff for review");
        let diff = Self::get_workspace_diff(ctx, task).await?;
        debug!(diff_length = diff.len(), "Workspace diff retrieved");

        let prompt = PhasePrompts::review(task, &diff);
        debug!(
            prompt_length = prompt.len(),
            "Sending review prompt to OpenCode"
        );

        let workspace_path = ctx.working_dir_for_task(task);
        let response_content = ctx
            .opencode_client
            .send_prompt(
                &session_id_str,
                &prompt,
                &workspace_path,
                activity_store.as_deref(),
            )
            .await;

        let response_content = match response_content {
            Ok(content) => content,
            Err(e) => {
                if let Some(ref store) = activity_store {
                    store.push_finished(false, Some(e.to_string()));
                }
                return Err(e);
            }
        };

        info!(
            response_length = response_content.len(),
            "Received AI review response"
        );

        let _review_path = ctx
            .file_manager
            .write_review(task.id, &response_content)
            .await?;

        session.complete();
        ctx.update_session(&session).await?;

        let review_result =
            Self::parse_and_save_findings(ctx, &response_content, task.id, session.id).await;

        let success = matches!(review_result, ReviewResult::Approved);

        if let Some(ref store) = activity_store {
            store.push_finished(success, None);
        }

        ctx.emit_session_ended(session.id, task.id, success);

        Self::handle_review_result(ctx, task, review_result, session_id_str, iteration).await
    }

    async fn parse_and_save_findings(
        ctx: &ExecutorContext,
        response_content: &str,
        task_id: Uuid,
        session_id: Uuid,
    ) -> ReviewResult {
        match MessageParser::parse_review_json(response_content, task_id, session_id) {
            Ok(findings) => {
                let _ = ctx.file_manager.write_findings(task_id, &findings).await;
                if findings.approved || findings.findings.is_empty() {
                    ReviewResult::Approved
                } else {
                    ReviewResult::FindingsDetected(findings.findings.len())
                }
            }
            Err(_) => {
                warn!("Falling back to legacy text-based review parsing");
                MessageParser::parse_review_response(response_content)
            }
        }
    }

    async fn handle_review_result(
        ctx: &ExecutorContext,
        task: &mut Task,
        review_result: ReviewResult,
        session_id_str: String,
        iteration: u32,
    ) -> Result<PhaseResult> {
        match review_result {
            ReviewResult::Approved => {
                info!(task_id = %task.id, "AI review APPROVED, proceeding to human review");
                ctx.transition(task, TaskStatus::Review)?;
                Ok(PhaseResult::ReviewPassed {
                    session_id: session_id_str,
                })
            }
            ReviewResult::FindingsDetected(count) => {
                info!(
                    task_id = %task.id,
                    finding_count = count,
                    "AI review found issues, waiting for user action"
                );
                Ok(PhaseResult::ReviewFailed {
                    session_id: session_id_str,
                    feedback: format!(
                        "{} issues found. Review findings and choose to fix or skip.",
                        count
                    ),
                    iteration,
                })
            }
            ReviewResult::ChangesRequested(feedback) => {
                warn!(
                    task_id = %task.id,
                    iteration = iteration,
                    feedback_preview = %feedback.chars().take(200).collect::<String>(),
                    "AI review REJECTED (legacy format), changes requested"
                );
                ctx.transition(task, TaskStatus::InProgress)?;
                Ok(PhaseResult::ReviewFailed {
                    session_id: session_id_str,
                    feedback,
                    iteration,
                })
            }
        }
    }

    async fn get_workspace_diff(ctx: &ExecutorContext, task: &Task) -> Result<String> {
        if let Some(ref wm) = ctx.workspace_manager {
            if let Some(ref workspace_path) = task.workspace_path {
                let workspace = Workspace::new(
                    task.id.to_string(),
                    PathBuf::from(workspace_path),
                    format!("task-{}", task.id),
                );
                return wm
                    .get_diff(&workspace)
                    .await
                    .map_err(|e| OrchestratorError::ExecutionFailed(format!("VCS error: {}", e)));
            }
        }
        Ok("(no workspace configured - diff unavailable)".to_string())
    }

    pub async fn start_async(ctx: &ExecutorContext, task: &Task) -> Result<StartedExecution> {
        info!(task_id = %task.id, "Starting review with SessionRunner");

        let working_dir = ctx.working_dir_for_task(task);

        let mcp_config = if task.status == TaskStatus::AiReview {
            let temp_session_id = Uuid::new_v4();
            match ctx
                .mcp_manager
                .setup_findings_server(task.id, temp_session_id, &working_dir)
                .await
            {
                Ok(_) => {
                    info!(task_id = %task.id, "MCP findings server added for review");
                    Some(McpConfig {
                        workspace_path: working_dir.clone(),
                        setup_success: true,
                    })
                }
                Err(e) => {
                    warn!(error = %e, "Failed to add MCP server, falling back to JSON parsing");
                    None
                }
            }
        } else {
            None
        };

        let diff = Self::get_workspace_diff(ctx, task).await.unwrap_or_else(|e| {
            warn!(error = %e, task_id = %task.id, "Failed to get workspace diff, proceeding without diff");
            String::new()
        });
        let prompt = if mcp_config.is_some() {
            PhasePrompts::review_with_mcp(task, &diff)
        } else {
            PhasePrompts::review(task, &diff)
        };

        let config = SessionConfig {
            task_id: task.id,
            task_status: task.status,
            phase: SessionPhase::Review,
            prompt,
            working_dir,
            provider_id: ctx.opencode_client.provider_id().to_string(),
            model_id: ctx.opencode_client.model_id().to_string(),
            mcp_config,
            implementation_phase: None,
            skip_task_status_update: false,
        };

        let deps = SessionDependencies::new(
            Arc::clone(&ctx.opencode_config),
            ctx.session_repo.clone(),
            ctx.task_repo.clone(),
            ctx.event_bus.clone(),
            ctx.activity_registry.clone(),
            ctx.file_manager.clone(),
        );

        let result = SessionRunner::start(config, deps).await?;

        Ok(StartedExecution {
            session_id: result.session_id,
            opencode_session_id: result.opencode_session_id,
            phase: SessionPhase::Review,
        })
    }
}
