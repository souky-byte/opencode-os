use db::{SessionRepository, TaskRepository};
use events::{Event, EventBus, EventEnvelope};
use opencode_client::apis::configuration::Configuration;
use opencode_core::{Session, SessionPhase, Task, TaskStatus, UpdateTaskRequest};
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use crate::activity_store::SessionActivityRegistry;
use crate::error::{OrchestratorError, Result};
use crate::executor::{PhaseResult, StartedExecution};
use crate::files::{FileManager, ParsedPlan, PhaseContext, PhaseSummary};
use crate::plan_parser::{extract_phase_summary, parse_plan_phases};
use crate::prompts::PhasePrompts;
use crate::services::{ExecutorContext, OpenCodeClient};
use crate::session_runner::{SessionConfig, SessionDependencies, SessionRunner};

pub struct ImplementationPhase;

impl ImplementationPhase {
    pub async fn run(ctx: &ExecutorContext, task: &mut Task) -> Result<PhaseResult> {
        info!(
            task_id = %task.id,
            task_title = %task.title,
            "Starting IMPLEMENTATION session"
        );

        if ctx.file_manager.plan_exists(task.id).await {
            let plan_content = ctx.file_manager.read_plan(task.id).await?;

            debug!(
                task_id = %task.id,
                plan_length = plan_content.len(),
                plan_preview = %plan_content.chars().take(500).collect::<String>(),
                "Read plan content for phase detection"
            );

            let parsed = parse_plan_phases(&plan_content);

            info!(
                task_id = %task.id,
                phases_count = parsed.phases.len(),
                is_single_phase = parsed.is_single_phase(),
                total_phases = parsed.total_phases(),
                phase_titles = ?parsed.phases.iter().map(|p| &p.title).collect::<Vec<_>>(),
                "Plan parsed for implementation"
            );

            if !parsed.is_single_phase() {
                info!(
                    task_id = %task.id,
                    total_phases = parsed.total_phases(),
                    "Plan has multiple phases, using phased implementation"
                );
                return Self::run_phased(ctx, task, parsed).await;
            } else {
                info!(
                    task_id = %task.id,
                    "Plan has single phase, using single implementation session"
                );
            }
        } else {
            warn!(
                task_id = %task.id,
                "No plan file found for task"
            );
        }

        Self::run_single(ctx, task).await
    }

    async fn run_single(ctx: &ExecutorContext, task: &mut Task) -> Result<PhaseResult> {
        let mut session = Session::new(task.id, SessionPhase::Implementation);

        ctx.setup_workspace(task).await?;

        let working_dir = ctx.working_dir_for_task(task);

        debug!(
            working_dir = %working_dir.display(),
            has_workspace = task.workspace_path.is_some(),
            "Creating OpenCode session for implementation"
        );
        let opencode_session = ctx.opencode_client.create_session(&working_dir).await?;
        let session_id_str = opencode_session.id.to_string();

        info!(
            opencode_session_id = %session_id_str,
            working_dir = %working_dir.display(),
            "OpenCode session created for implementation"
        );

        session.start(session_id_str.clone());
        ctx.persist_session(&session).await?;

        let activity_store = ctx.get_activity_store(session.id);
        ctx.emit_session_started(&session, task.id);

        let plan = if ctx.file_manager.plan_exists(task.id).await {
            debug!("Loading existing plan for implementation");
            ctx.file_manager.read_plan(task.id).await.ok()
        } else {
            debug!("No existing plan found, proceeding without plan");
            None
        };

        debug!(
            has_plan = plan.is_some(),
            "Generating implementation prompt"
        );
        let prompt = PhasePrompts::implementation_with_plan(task, plan.as_deref());
        debug!(
            prompt_length = prompt.len(),
            "Sending implementation prompt to OpenCode"
        );

        let response = ctx
            .opencode_client
            .send_prompt(
                &session_id_str,
                &prompt,
                &working_dir,
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

        info!("Implementation response received from OpenCode");

        session.complete();
        ctx.update_session(&session).await?;

        ctx.emit_session_ended(session.id, task.id, true);

        ctx.transition(task, TaskStatus::AiReview)?;

        info!(
            task_id = %task.id,
            "IMPLEMENTATION session completed, proceeding to AI review"
        );

        Ok(PhaseResult::SessionCreated {
            session_id: session_id_str,
        })
    }

    async fn run_phased(
        ctx: &ExecutorContext,
        task: &mut Task,
        parsed_plan: ParsedPlan,
    ) -> Result<PhaseResult> {
        let mut context = ctx
            .file_manager
            .read_phase_context(task.id)
            .await?
            .unwrap_or_else(|| PhaseContext::new(parsed_plan.total_phases()));

        info!(
            task_id = %task.id,
            current_phase = context.phase_number,
            total_phases = context.total_phases,
            "Running phased implementation"
        );

        ctx.setup_workspace(task).await?;

        let working_dir = ctx.working_dir_for_task(task);

        while !context.is_complete() {
            let phase_idx = (context.phase_number - 1) as usize;
            if phase_idx >= parsed_plan.phases.len() {
                break;
            }

            let current_phase = &parsed_plan.phases[phase_idx];

            info!(
                task_id = %task.id,
                phase = context.phase_number,
                total = context.total_phases,
                phase_title = %current_phase.title,
                "Starting implementation phase"
            );

            let mut session = Session::new_implementation_phase(
                task.id,
                context.phase_number,
                &current_phase.title,
            );

            let opencode_session = ctx.opencode_client.create_session(&working_dir).await?;
            let session_id_str = opencode_session.id.to_string();

            session.start(session_id_str.clone());
            ctx.persist_session(&session).await?;

            let activity_store = ctx.get_activity_store(session.id);
            ctx.emit_session_started(&session, task.id);

            let prompt = PhasePrompts::implementation_phase(task, current_phase, &context);

            let response = ctx
                .opencode_client
                .send_prompt(
                    &session_id_str,
                    &prompt,
                    &working_dir,
                    activity_store.as_deref(),
                )
                .await;

            let response_text = match response {
                Ok(text) => {
                    if let Some(ref store) = activity_store {
                        store.push_finished(true, None);
                    }
                    text
                }
                Err(e) => {
                    if let Some(ref store) = activity_store {
                        store.push_finished(false, Some(e.to_string()));
                    }
                    session.fail();
                    ctx.update_session(&session).await?;
                    return Err(e);
                }
            };

            let summary = Self::extract_or_create_summary(
                &response_text,
                context.phase_number,
                &current_phase.title,
            );

            ctx.file_manager
                .write_phase_summary(task.id, &summary)
                .await?;

            ctx.file_manager
                .mark_phase_complete_in_plan(task.id, context.phase_number)
                .await?;

            ctx.emit_event(Event::PhaseCompleted {
                task_id: task.id,
                session_id: session.id,
                phase_number: context.phase_number,
                total_phases: context.total_phases,
                phase_title: current_phase.title.clone(),
            });

            session.complete();
            ctx.update_session(&session).await?;

            ctx.emit_session_ended(session.id, task.id, true);

            context.advance(summary);
            ctx.file_manager
                .write_phase_context(task.id, &context)
                .await?;

            if !context.is_complete() {
                ctx.emit_event(Event::PhaseContinuing {
                    task_id: task.id,
                    next_phase_number: context.phase_number,
                    total_phases: context.total_phases,
                });
            }
        }

        ctx.transition(task, TaskStatus::AiReview)?;

        info!(
            task_id = %task.id,
            total_phases = context.total_phases,
            "All implementation phases completed, proceeding to AI review"
        );

        Ok(PhaseResult::PhasedImplementationComplete {
            total_phases: context.total_phases,
        })
    }

    fn extract_or_create_summary(
        response: &str,
        phase_number: u32,
        phase_title: &str,
    ) -> PhaseSummary {
        if let Some(extracted) = extract_phase_summary(response) {
            return PhaseSummary::new(
                phase_number,
                phase_title,
                extracted.summary,
                extracted.files_changed,
                extracted.notes,
            );
        }

        info!(
            phase = phase_number,
            "No structured summary found in response, creating basic summary"
        );

        let summary = if response.len() > 500 {
            format!("{}...", &response[..497])
        } else {
            response.to_string()
        };

        PhaseSummary::new(phase_number, phase_title, summary, Vec::new(), None)
    }

    pub async fn start_async(ctx: &ExecutorContext, task: &mut Task) -> Result<StartedExecution> {
        info!(task_id = %task.id, "Starting implementation with SessionRunner");

        ctx.setup_workspace(task).await?;

        let working_dir = ctx.working_dir_for_task(task);

        if ctx.file_manager.plan_exists(task.id).await {
            let plan_content = ctx.file_manager.read_plan(task.id).await?;
            let parsed = parse_plan_phases(&plan_content);

            info!(
                task_id = %task.id,
                phases_count = parsed.phases.len(),
                is_single_phase = parsed.is_single_phase(),
                "Checking plan for phased implementation"
            );

            if !parsed.is_single_phase() {
                info!(
                    task_id = %task.id,
                    total_phases = parsed.total_phases(),
                    "Using phased implementation for multi-phase plan"
                );
                return Self::start_phased_async(ctx, task, parsed, working_dir).await;
            }
        }

        let plan = if ctx.file_manager.plan_exists(task.id).await {
            ctx.file_manager.read_plan(task.id).await.ok()
        } else {
            None
        };
        let prompt = PhasePrompts::implementation_with_plan(task, plan.as_deref());

        let config = SessionConfig {
            task_id: task.id,
            task_status: task.status,
            phase: SessionPhase::Implementation,
            prompt,
            working_dir,
            provider_id: ctx.opencode_client.provider_id().to_string(),
            model_id: ctx.opencode_client.model_id().to_string(),
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

        Ok(StartedExecution {
            session_id: result.session_id,
            opencode_session_id: result.opencode_session_id,
            phase: SessionPhase::Implementation,
        })
    }

    async fn start_phased_async(
        ctx: &ExecutorContext,
        task: &mut Task,
        parsed_plan: ParsedPlan,
        working_dir: PathBuf,
    ) -> Result<StartedExecution> {
        let context = ctx
            .file_manager
            .read_phase_context(task.id)
            .await?
            .unwrap_or_else(|| PhaseContext::new(parsed_plan.total_phases()));

        let phase_idx = (context.phase_number - 1) as usize;
        let current_phase = parsed_plan.phases.get(phase_idx).ok_or_else(|| {
            OrchestratorError::ExecutionFailed(format!(
                "Phase {} not found in plan",
                context.phase_number
            ))
        })?;

        let opencode_session = ctx.opencode_client.create_session(&working_dir).await?;
        let opencode_session_id = opencode_session.id.to_string();

        let mut session =
            Session::new_implementation_phase(task.id, context.phase_number, &current_phase.title);
        session.start(opencode_session_id.clone());
        ctx.persist_session(&session).await?;

        ctx.emit_session_started(&session, task.id);

        let first_session_id = session.id;
        let first_opencode_session_id = opencode_session_id.clone();
        let return_opencode_session_id = first_opencode_session_id.clone();

        let task_id = task.id;
        let task_clone = task.clone();
        let file_manager = ctx.file_manager.clone();
        let session_repo = ctx.session_repo.clone();
        let task_repo = ctx.task_repo.clone();
        let event_bus = ctx.event_bus.clone();
        let activity_registry = ctx.activity_registry.clone();
        let opencode_config = Arc::clone(&ctx.opencode_config);
        let provider_id = ctx.opencode_client.provider_id().to_string();
        let model_id = ctx.opencode_client.model_id().to_string();

        tokio::spawn(async move {
            let mut task = task_clone;
            match Self::run_phased_background(
                &mut task,
                parsed_plan,
                working_dir,
                first_session_id,
                first_opencode_session_id,
                file_manager,
                session_repo,
                task_repo,
                event_bus,
                activity_registry,
                opencode_config,
                provider_id,
                model_id,
            )
            .await
            {
                Ok(_) => {
                    info!(task_id = %task_id, "Phased implementation completed successfully");
                }
                Err(e) => {
                    error!(task_id = %task_id, error = %e, "Phased implementation failed");
                }
            }
        });

        Ok(StartedExecution {
            session_id: first_session_id,
            opencode_session_id: return_opencode_session_id,
            phase: SessionPhase::Implementation,
        })
    }

    #[allow(clippy::too_many_arguments)]
    async fn run_phased_background(
        task: &mut Task,
        parsed_plan: ParsedPlan,
        working_dir: PathBuf,
        first_session_id: uuid::Uuid,
        first_opencode_session_id: String,
        file_manager: FileManager,
        session_repo: Option<Arc<SessionRepository>>,
        task_repo: Option<Arc<TaskRepository>>,
        event_bus: Option<EventBus>,
        activity_registry: Option<SessionActivityRegistry>,
        opencode_config: Arc<Configuration>,
        provider_id: String,
        model_id: String,
    ) -> Result<()> {
        let mut context = file_manager
            .read_phase_context(task.id)
            .await?
            .unwrap_or_else(|| PhaseContext::new(parsed_plan.total_phases()));

        info!(
            task_id = %task.id,
            current_phase = context.phase_number,
            total_phases = context.total_phases,
            "Starting phased implementation"
        );

        let mut is_first_phase = true;

        while !context.is_complete() {
            let phase_idx = (context.phase_number - 1) as usize;
            if phase_idx >= parsed_plan.phases.len() {
                break;
            }

            let current_phase = &parsed_plan.phases[phase_idx];

            info!(
                task_id = %task.id,
                phase = context.phase_number,
                total = context.total_phases,
                phase_title = %current_phase.title,
                "Starting phase"
            );

            let (session_id, opencode_session_id) = if is_first_phase {
                is_first_phase = false;
                (first_session_id, first_opencode_session_id.clone())
            } else {
                let opencode_session =
                    OpenCodeClient::create_session_static(&opencode_config, working_dir.to_str())
                        .await?;
                let new_opencode_session_id = opencode_session.id.to_string();

                let mut session = Session::new_implementation_phase(
                    task.id,
                    context.phase_number,
                    &current_phase.title,
                );
                session.start(new_opencode_session_id.clone());

                if let Some(ref repo) = session_repo {
                    repo.create(&session).await.map_err(|e| {
                        OrchestratorError::ExecutionFailed(format!(
                            "Failed to persist session: {}",
                            e
                        ))
                    })?;
                }

                if let Some(ref bus) = event_bus {
                    bus.publish(EventEnvelope::new(Event::SessionStarted {
                        session_id: session.id,
                        task_id: task.id,
                        phase: session.phase.as_str().to_string(),
                        status: session.status.as_str().to_string(),
                        opencode_session_id: session.opencode_session_id.clone(),
                        created_at: session.created_at,
                    }));
                }

                (session.id, new_opencode_session_id)
            };

            let prompt = PhasePrompts::implementation_phase(task, current_phase, &context);
            let config = SessionConfig {
                task_id: task.id,
                task_status: task.status,
                phase: SessionPhase::Implementation,
                prompt,
                working_dir: working_dir.clone(),
                provider_id: provider_id.clone(),
                model_id: model_id.clone(),
                mcp_config: None,
                implementation_phase: Some((context.phase_number, current_phase.title.clone())),
                skip_task_status_update: true,
            };

            let deps = SessionDependencies::new(
                Arc::clone(&opencode_config),
                session_repo.clone(),
                task_repo.clone(),
                event_bus.clone(),
                activity_registry.clone(),
                file_manager.clone(),
            );

            let opencode_session_id_clone = opencode_session_id.clone();
            let (success, response_text) = SessionRunner::execute_and_complete(
                config,
                deps,
                session_id,
                opencode_session_id_clone,
            )
            .await;

            if !success {
                return Err(OrchestratorError::ExecutionFailed(format!(
                    "Phase {} failed",
                    context.phase_number
                )));
            }

            let summary = Self::extract_or_create_summary(
                &response_text,
                context.phase_number,
                &current_phase.title,
            );

            file_manager.write_phase_summary(task.id, &summary).await?;
            file_manager
                .mark_phase_complete_in_plan(task.id, context.phase_number)
                .await?;

            if let Some(ref bus) = event_bus {
                bus.publish(EventEnvelope::new(Event::PhaseCompleted {
                    task_id: task.id,
                    session_id,
                    phase_number: context.phase_number,
                    total_phases: context.total_phases,
                    phase_title: current_phase.title.clone(),
                }));
            }

            context.advance(summary);
            file_manager.write_phase_context(task.id, &context).await?;

            if !context.is_complete() {
                if let Some(ref bus) = event_bus {
                    bus.publish(EventEnvelope::new(Event::PhaseContinuing {
                        task_id: task.id,
                        next_phase_number: context.phase_number,
                        total_phases: context.total_phases,
                    }));
                }
            }
        }

        task.status = TaskStatus::AiReview;

        if let Some(ref repo) = task_repo {
            let update = UpdateTaskRequest {
                status: Some(TaskStatus::AiReview),
                ..Default::default()
            };
            let _ = repo.update(task.id, &update).await;
        }

        if let Some(ref bus) = event_bus {
            bus.publish(EventEnvelope::new(Event::TaskStatusChanged {
                task_id: task.id,
                from_status: TaskStatus::InProgress.as_str().to_string(),
                to_status: TaskStatus::AiReview.as_str().to_string(),
            }));
        }

        info!(
            task_id = %task.id,
            total_phases = context.total_phases,
            "All phases completed, proceeding to AI review"
        );

        Ok(())
    }
}
