use opencode_core::{Session, SessionPhase, Task, TaskStatus};
use std::sync::Arc;
use tracing::{debug, info, warn};

use crate::error::Result;
use crate::executor::{PhaseResult, StartedExecution};
use crate::prompts::PhasePrompts;
use crate::services::ExecutorContext;
use crate::session_runner::{SessionConfig, SessionDependencies, SessionRunner};

pub struct PlanningPhase;

impl PlanningPhase {
    pub async fn run(ctx: &ExecutorContext, task: &mut Task) -> Result<PhaseResult> {
        info!(
            task_id = %task.id,
            task_title = %task.title,
            "Starting PLANNING session"
        );

        let mut session = Session::new(task.id, SessionPhase::Planning);

        let wiki_setup = if let Some(ref wiki_config) = ctx.config.wiki_config {
            match ctx
                .mcp_manager
                .setup_wiki_server(&ctx.config.repo_path, wiki_config)
                .await
            {
                Ok(()) => {
                    info!("Wiki MCP server connected for planning");
                    true
                }
                Err(e) => {
                    warn!(error = %e, "Failed to setup wiki MCP server, continuing without it");
                    false
                }
            }
        } else {
            false
        };

        debug!("Creating OpenCode session for planning");
        let client = ctx.opencode_client_for_phase(SessionPhase::Planning);
        let opencode_session = client.create_session(&ctx.config.repo_path).await?;
        let session_id_str = opencode_session.id.to_string();

        info!(
            opencode_session_id = %session_id_str,
            "OpenCode session created"
        );

        session.start(session_id_str.clone());
        ctx.persist_session(&session).await?;

        let activity_store = ctx.get_activity_store(session.id);
        ctx.emit_session_started(&session, task.id);

        debug!("Generating planning prompt");
        let prompt = PhasePrompts::planning(task);
        debug!(
            prompt_length = prompt.len(),
            "Sending planning prompt to OpenCode"
        );

        let response_content = client
            .send_prompt(
                &session_id_str,
                &prompt,
                &ctx.config.repo_path,
                activity_store.as_deref(),
            )
            .await;

        let response_content = match response_content {
            Ok(content) => {
                if let Some(ref store) = activity_store {
                    store.push_finished(true, None);
                }
                content
            }
            Err(e) => {
                if let Some(ref store) = activity_store {
                    store.push_finished(false, Some(e.to_string()));
                }
                return Err(e);
            }
        };

        info!(
            response_length = response_content.len(),
            "Received planning response"
        );

        let plan_path = ctx
            .file_manager
            .write_plan(task.id, &response_content)
            .await?;

        info!(plan_path = %plan_path.display(), "Plan saved to file");

        session.complete();
        ctx.update_session(&session).await?;
        ctx.emit_session_ended(session.id, task.id, true);

        // Commit plan changes
        ctx.commit_phase_changes(
            task,
            "Planning",
            &format!("Created plan for: {}", task.title),
        )
        .await?;

        if wiki_setup {
            if let Err(e) = ctx.mcp_manager.cleanup_wiki_server(&ctx.config.repo_path).await {
                warn!(error = %e, "Failed to cleanup wiki MCP server");
            }
        }

        ctx.transition(task, TaskStatus::PlanningReview)?;

        info!(
            task_id = %task.id,
            "PLANNING session completed, awaiting review"
        );

        Ok(PhaseResult::PlanCreated {
            session_id: session_id_str,
            plan_path: plan_path.to_string_lossy().to_string(),
        })
    }

    pub async fn start_async(ctx: &ExecutorContext, task: &Task) -> Result<StartedExecution> {
        info!(task_id = %task.id, "Starting planning with SessionRunner");

        let prompt = PhasePrompts::planning(task);
        let client = ctx.opencode_client_for_phase(SessionPhase::Planning);

        let config = SessionConfig {
            task_id: task.id,
            task_status: task.status,
            phase: SessionPhase::Planning,
            prompt,
            working_dir: ctx.config.repo_path.clone(),
            provider_id: client.provider_id().to_string(),
            model_id: client.model_id().to_string(),
            mcp_config: None,
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

        info!(
            task_id = %task.id,
            session_id = %result.session_id,
            opencode_session_id = %result.opencode_session_id,
            "Planning started via SessionRunner"
        );

        Ok(StartedExecution {
            session_id: result.session_id,
            opencode_session_id: result.opencode_session_id,
            phase: SessionPhase::Planning,
        })
    }
}
