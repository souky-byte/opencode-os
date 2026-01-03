use opencode_core::{Session, SessionPhase, Task, TaskStatus};
use std::sync::Arc;
use tracing::{debug, info, warn};

use crate::error::{OrchestratorError, Result};
use crate::executor::{PhaseResult, StartedExecution};
use crate::prompts::{PhasePrompts, UserReviewComment};
use crate::services::ExecutorContext;
use crate::session_runner::{McpConfig, SessionConfig, SessionDependencies, SessionRunner};

pub struct FixPhase;

impl FixPhase {
    pub async fn run(ctx: &ExecutorContext, task: &mut Task) -> Result<PhaseResult> {
        info!(
            task_id = %task.id,
            "Starting FIX session with MCP"
        );

        let mut session = Session::new(task.id, SessionPhase::Fix);

        debug!("Creating OpenCode session for fix");
        let opencode_session = ctx
            .opencode_client
            .create_session(&ctx.config.repo_path)
            .await?;
        let session_id_str = opencode_session.id.to_string();

        info!(
            opencode_session_id = %session_id_str,
            "OpenCode session created for fix"
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
            warn!(error = %e, "Failed to add MCP server for fix session");
            session.fail();
            ctx.update_session(&session).await?;

            if let Some(ref store) = activity_store {
                store.push_finished(false, Some(e.to_string()));
            }

            return Err(OrchestratorError::ExecutionFailed(format!(
                "MCP server required for fix session: {}",
                e
            )));
        }

        let prompt = PhasePrompts::fix_with_mcp(task);
        debug!(
            prompt_length = prompt.len(),
            "Sending fix prompt to OpenCode"
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

        let _ = ctx
            .mcp_manager
            .cleanup_findings_server(&workspace_path)
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
            "Received fix session response"
        );

        session.complete();
        ctx.update_session(&session).await?;

        if let Some(ref store) = activity_store {
            store.push_finished(true, None);
        }

        ctx.emit_session_ended(session.id, task.id, true);

        info!(task_id = %task.id, "Fix session completed, transitioning to AI Review");
        ctx.transition(task, TaskStatus::AiReview)?;

        Ok(PhaseResult::FixCompleted {
            session_id: session_id_str,
        })
    }

    pub async fn run_iteration(
        ctx: &ExecutorContext,
        task: &mut Task,
        feedback: &str,
    ) -> Result<PhaseResult> {
        info!(
            task_id = %task.id,
            feedback_length = feedback.len(),
            "Starting FIX iteration based on review feedback"
        );

        let mut session = Session::new(task.id, SessionPhase::Implementation);

        debug!("Creating OpenCode session for fix iteration");
        let opencode_session = ctx
            .opencode_client
            .create_session(&ctx.config.repo_path)
            .await?;
        let session_id_str = opencode_session.id.to_string();

        info!(
            opencode_session_id = %session_id_str,
            "OpenCode session created for fix iteration"
        );

        session.start(session_id_str.clone());
        ctx.persist_session(&session).await?;

        let activity_store = ctx.get_activity_store(session.id);
        ctx.emit_session_started(&session, task.id);

        let prompt = PhasePrompts::fix_issues(task, feedback);
        debug!(
            prompt_length = prompt.len(),
            "Sending fix prompt to OpenCode"
        );

        let workspace_path = ctx.working_dir_for_task(task);
        let response = ctx
            .opencode_client
            .send_prompt(
                &session_id_str,
                &prompt,
                &workspace_path,
                activity_store.as_deref(),
            )
            .await;

        match response {
            Ok(_) => {
                if let Some(ref store) = activity_store {
                    store.push_finished(true, None);
                }
            }
            Err(e) => {
                if let Some(ref store) = activity_store {
                    store.push_finished(false, Some(e.to_string()));
                }
                return Err(e);
            }
        }

        info!("Fix iteration response received from OpenCode");

        session.complete();
        ctx.update_session(&session).await?;

        ctx.emit_session_ended(session.id, task.id, true);

        ctx.transition(task, TaskStatus::AiReview)?;

        info!(
            task_id = %task.id,
            "FIX iteration completed, returning to AI review"
        );

        Ok(PhaseResult::SessionCreated {
            session_id: session_id_str,
        })
    }

    pub async fn start_async(ctx: &ExecutorContext, task: &Task) -> Result<StartedExecution> {
        info!(task_id = %task.id, "Starting fix with SessionRunner");

        let working_dir = ctx.working_dir_for_task(task);

        let mcp_config = Some(McpConfig {
            workspace_path: working_dir.clone(),
            setup_success: true,
        });

        let prompt = PhasePrompts::fix_with_mcp(task);

        let config = SessionConfig {
            task_id: task.id,
            task_status: task.status,
            phase: SessionPhase::Fix,
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
            phase: SessionPhase::Fix,
        })
    }

    pub async fn start_with_comments(
        ctx: &ExecutorContext,
        task: &Task,
        comments: &[UserReviewComment],
    ) -> Result<StartedExecution> {
        info!(
            task_id = %task.id,
            comment_count = comments.len(),
            "Starting fix with user comments"
        );

        let working_dir = ctx.working_dir_for_task(task);

        let mcp_config = Some(McpConfig {
            workspace_path: working_dir.clone(),
            setup_success: true,
        });

        let prompt = PhasePrompts::fix_user_comments(task, comments);

        let config = SessionConfig {
            task_id: task.id,
            task_status: task.status,
            phase: SessionPhase::Fix,
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
            phase: SessionPhase::Fix,
        })
    }
}
