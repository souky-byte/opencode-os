use opencode_client::apis::configuration::Configuration;
use opencode_client::models::Part;
use opencode_core::{Session, SessionPhase, Task, TaskStatus};
use std::sync::Arc;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::activity_store::SessionActivityMsg;
use crate::error::{OrchestratorError, Result};
use crate::prompts::PhasePrompts;
use crate::services::{
    ExecutorContext, FixPhase, ImplementationPhase, MessageParser, PlanningPhase, ReviewPhase,
};

pub use crate::services::executor_context::ExecutorConfig;
pub use crate::services::message_parser::ReviewResult;

#[derive(Debug, Clone)]
pub enum PhaseResult {
    SessionCreated {
        session_id: String,
    },
    PlanCreated {
        session_id: String,
        plan_path: String,
    },
    AwaitingApproval {
        phase: SessionPhase,
    },
    ReviewPassed {
        session_id: String,
    },
    ReviewFailed {
        session_id: String,
        feedback: String,
        iteration: u32,
    },
    FixCompleted {
        session_id: String,
    },
    PhasedImplementationComplete {
        total_phases: u32,
    },
    MaxIterationsExceeded {
        iterations: u32,
    },
    Completed,
}

#[derive(Debug, Clone)]
pub struct StartedExecution {
    pub session_id: Uuid,
    pub opencode_session_id: String,
    pub phase: SessionPhase,
}

pub struct TaskExecutor {
    ctx: ExecutorContext,
}

impl TaskExecutor {
    pub fn new(opencode_config: Arc<Configuration>, config: ExecutorConfig) -> Self {
        Self {
            ctx: ExecutorContext::new(opencode_config, config),
        }
    }

    pub fn with_model(mut self, provider_id: &str, model_id: &str) -> Self {
        self.ctx = self.ctx.with_model(provider_id, model_id);
        self
    }

    pub fn with_workspace_manager(mut self, manager: Arc<vcs::WorkspaceManager>) -> Self {
        self.ctx = self.ctx.with_workspace_manager(manager);
        self
    }

    pub fn with_session_repo(mut self, repo: Arc<db::SessionRepository>) -> Self {
        self.ctx = self.ctx.with_session_repo(repo);
        self
    }

    pub fn with_task_repo(mut self, repo: Arc<db::TaskRepository>) -> Self {
        self.ctx = self.ctx.with_task_repo(repo);
        self
    }

    pub fn with_event_bus(mut self, bus: events::EventBus) -> Self {
        self.ctx = self.ctx.with_event_bus(bus);
        self
    }

    pub fn with_activity_registry(
        mut self,
        registry: crate::activity_store::SessionActivityRegistry,
    ) -> Self {
        self.ctx = self.ctx.with_activity_registry(registry);
        self
    }

    pub fn file_manager(&self) -> &crate::files::FileManager {
        self.ctx.file_manager()
    }

    pub fn opencode_config(&self) -> &Arc<Configuration> {
        &self.ctx.opencode_config
    }

    pub fn transition(&self, task: &mut Task, to: TaskStatus) -> Result<()> {
        self.ctx.transition(task, to)
    }

    pub fn extract_text_from_parts(parts: &[Part]) -> String {
        MessageParser::extract_text_from_parts(parts)
    }

    pub fn parse_message_parts(parts: &[Part]) -> Vec<SessionActivityMsg> {
        MessageParser::parse_message_parts(parts)
    }

    pub fn parse_sse_part(part: &serde_json::Value) -> Option<SessionActivityMsg> {
        MessageParser::parse_sse_part(part)
    }

    pub async fn execute_phase(&self, task: &mut Task) -> Result<PhaseResult> {
        info!(
            task_id = %task.id,
            current_status = %task.status.as_str(),
            "Executing phase for task"
        );

        let result = match task.status {
            TaskStatus::Todo => {
                debug!("Task in TODO, transitioning to PLANNING");
                self.ctx.transition(task, TaskStatus::Planning)?;
                PlanningPhase::run(&self.ctx, task).await
            }
            TaskStatus::Planning => {
                debug!("Task in PLANNING, running planning session");
                PlanningPhase::run(&self.ctx, task).await
            }
            TaskStatus::PlanningReview => {
                if self.ctx.config.require_plan_approval {
                    info!("Plan requires approval, awaiting human review");
                    Ok(PhaseResult::AwaitingApproval {
                        phase: SessionPhase::Planning,
                    })
                } else {
                    debug!("Auto-approving plan, transitioning to IN_PROGRESS");
                    self.ctx.transition(task, TaskStatus::InProgress)?;
                    ImplementationPhase::run(&self.ctx, task).await
                }
            }
            TaskStatus::InProgress => {
                debug!("Task IN_PROGRESS, running implementation session");
                ImplementationPhase::run(&self.ctx, task).await
            }
            TaskStatus::AiReview => {
                debug!("Task in AI_REVIEW, running AI review");
                ReviewPhase::run(&self.ctx, task, 0).await
            }
            TaskStatus::Fix => {
                debug!("Task in FIX, running fix session");
                FixPhase::run(&self.ctx, task).await
            }
            TaskStatus::Review => {
                if self.ctx.config.require_human_review {
                    info!("Implementation requires human review, awaiting approval");
                    Ok(PhaseResult::AwaitingApproval {
                        phase: SessionPhase::Review,
                    })
                } else {
                    debug!("Auto-approving review, transitioning to DONE");
                    self.ctx.transition(task, TaskStatus::Done)?;
                    Ok(PhaseResult::Completed)
                }
            }
            TaskStatus::Done => {
                debug!("Task already DONE");
                Ok(PhaseResult::Completed)
            }
        };

        match &result {
            Ok(phase_result) => info!(
                task_id = %task.id,
                result = ?phase_result,
                "Phase execution completed"
            ),
            Err(e) => tracing::error!(
                task_id = %task.id,
                error = %e,
                "Phase execution failed"
            ),
        }

        result
    }

    pub async fn run_full_cycle(&self, task: &mut Task) -> Result<PhaseResult> {
        info!("Starting full cycle for task {}: {}", task.id, task.title);

        if task.status == TaskStatus::Done {
            return Ok(PhaseResult::Completed);
        }

        if task.status == TaskStatus::Todo {
            self.ctx.transition(task, TaskStatus::Planning)?;
        }

        if task.status == TaskStatus::Planning {
            let result = PlanningPhase::run(&self.ctx, task).await?;
            if self.ctx.config.require_plan_approval {
                return Ok(result);
            }
        }

        if task.status == TaskStatus::PlanningReview {
            self.ctx.transition(task, TaskStatus::InProgress)?;
        }

        if task.status == TaskStatus::InProgress {
            ImplementationPhase::run(&self.ctx, task).await?;
        }

        let mut iteration = 0;
        while task.status == TaskStatus::AiReview
            && iteration < self.ctx.config.max_review_iterations
        {
            let result = ReviewPhase::run(&self.ctx, task, iteration).await?;
            match result {
                PhaseResult::ReviewPassed { .. } => {
                    if self.ctx.config.require_human_review {
                        return Ok(PhaseResult::AwaitingApproval {
                            phase: SessionPhase::Review,
                        });
                    } else {
                        self.ctx.transition(task, TaskStatus::Done)?;
                        return Ok(PhaseResult::Completed);
                    }
                }
                PhaseResult::ReviewFailed { feedback, .. } => {
                    info!(
                        "AI review failed (iteration {}), running fix iteration",
                        iteration
                    );
                    FixPhase::run_iteration(&self.ctx, task, &feedback).await?;
                    iteration += 1;
                }
                _ => return Ok(result),
            }
        }

        if iteration >= self.ctx.config.max_review_iterations {
            warn!(
                "Task {} exceeded max review iterations ({})",
                task.id, self.ctx.config.max_review_iterations
            );
            return Ok(PhaseResult::MaxIterationsExceeded {
                iterations: iteration,
            });
        }

        if task.status == TaskStatus::Review {
            if self.ctx.config.require_human_review {
                return Ok(PhaseResult::AwaitingApproval {
                    phase: SessionPhase::Review,
                });
            }
            self.ctx.transition(task, TaskStatus::Done)?;
        }

        Ok(PhaseResult::Completed)
    }

    pub async fn start_phase_async(&self, task: &mut Task) -> Result<StartedExecution> {
        info!(
            task_id = %task.id,
            current_status = %task.status.as_str(),
            "Starting async phase execution"
        );

        if task.status == TaskStatus::Todo {
            self.ctx.transition(task, TaskStatus::Planning)?;
        }

        match task.status {
            TaskStatus::Todo | TaskStatus::Planning => {
                PlanningPhase::start_async(&self.ctx, task).await
            }
            TaskStatus::PlanningReview | TaskStatus::InProgress => {
                ImplementationPhase::start_async(&self.ctx, task).await
            }
            TaskStatus::AiReview => ReviewPhase::start_async(&self.ctx, task).await,
            TaskStatus::Fix => FixPhase::start_async(&self.ctx, task).await,
            TaskStatus::Review => ReviewPhase::start_async(&self.ctx, task).await,
            TaskStatus::Done => Err(OrchestratorError::ExecutionFailed(
                "Task is already done".to_string(),
            )),
        }
    }

    pub async fn start_fix_with_comments(
        &self,
        task: &Task,
        comments: &[crate::prompts::UserReviewComment],
    ) -> Result<StartedExecution> {
        FixPhase::start_with_comments(&self.ctx, task, comments).await
    }

    pub async fn approve_plan(&self, task: &mut Task) -> Result<()> {
        info!(task_id = %task.id, "Plan APPROVED by human reviewer");

        if task.status != TaskStatus::PlanningReview {
            warn!(
                current_status = %task.status.as_str(),
                "Cannot approve plan - task not in PlanningReview state"
            );
            return Err(OrchestratorError::InvalidTransition {
                from: task.status.as_str().to_string(),
                to: "InProgress (plan approval)".to_string(),
            });
        }
        self.ctx.transition(task, TaskStatus::InProgress)?;
        info!(task_id = %task.id, "Task ready for implementation");
        Ok(())
    }

    pub async fn reject_plan(&self, task: &mut Task, feedback: &str) -> Result<PhaseResult> {
        info!(
            task_id = %task.id,
            feedback_length = feedback.len(),
            "Plan REJECTED by human reviewer, re-planning"
        );

        if task.status != TaskStatus::PlanningReview {
            warn!(
                current_status = %task.status.as_str(),
                "Cannot reject plan - task not in PlanningReview state"
            );
            return Err(OrchestratorError::InvalidTransition {
                from: task.status.as_str().to_string(),
                to: "Planning (plan rejection)".to_string(),
            });
        }
        self.ctx.transition(task, TaskStatus::Planning)?;

        let mut session = Session::new(task.id, SessionPhase::Planning);
        let opencode_session = self
            .ctx
            .opencode_client
            .create_session(&self.ctx.config.repo_path)
            .await?;
        let session_id_str = opencode_session.id.to_string();

        info!(
            opencode_session_id = %session_id_str,
            "OpenCode session created for re-planning"
        );

        session.start(session_id_str.clone());
        self.ctx.persist_session(&session).await?;

        let activity_store = self.ctx.get_activity_store(session.id);

        let prompt = PhasePrompts::replan(task, feedback);
        let response_content = self
            .ctx
            .opencode_client
            .send_prompt(
                &session_id_str,
                &prompt,
                &self.ctx.config.repo_path,
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

        let plan_path = self
            .ctx
            .file_manager
            .write_plan(task.id, &response_content)
            .await?;

        info!(plan_path = %plan_path.display(), "New plan saved");

        session.complete();
        self.ctx.update_session(&session).await?;

        self.ctx.transition(task, TaskStatus::PlanningReview)?;

        info!(task_id = %task.id, "Re-planning completed, awaiting review");

        Ok(PhaseResult::PlanCreated {
            session_id: session_id_str,
            plan_path: plan_path.to_string_lossy().to_string(),
        })
    }

    pub async fn approve_review(&self, task: &mut Task) -> Result<()> {
        info!(task_id = %task.id, "Implementation APPROVED by human reviewer");

        if task.status != TaskStatus::Review {
            warn!(
                current_status = %task.status.as_str(),
                "Cannot approve review - task not in Review state"
            );
            return Err(OrchestratorError::InvalidTransition {
                from: task.status.as_str().to_string(),
                to: "Done (review approval)".to_string(),
            });
        }
        self.ctx.transition(task, TaskStatus::Done)?;
        info!(task_id = %task.id, "Task COMPLETED successfully");
        Ok(())
    }

    pub async fn reject_review(&self, task: &mut Task, feedback: &str) -> Result<PhaseResult> {
        info!(
            task_id = %task.id,
            feedback_length = feedback.len(),
            "Implementation REJECTED by human reviewer, running fix iteration"
        );

        if task.status != TaskStatus::Review {
            warn!(
                current_status = %task.status.as_str(),
                "Cannot reject review - task not in Review state"
            );
            return Err(OrchestratorError::InvalidTransition {
                from: task.status.as_str().to_string(),
                to: "InProgress (review rejection)".to_string(),
            });
        }
        FixPhase::run_iteration(&self.ctx, task, feedback).await
    }

    pub async fn run_fix_iteration(&self, task: &mut Task, feedback: &str) -> Result<PhaseResult> {
        FixPhase::run_iteration(&self.ctx, task, feedback).await
    }

    #[cfg(test)]
    fn parse_review_response(content: &str) -> ReviewResult {
        MessageParser::parse_review_response(content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_parse_review_approved() {
        let content = "## Review\n\nThe code looks good.\n\nAPPROVED";
        let result = TaskExecutor::parse_review_response(content);
        assert_eq!(result, ReviewResult::Approved);
    }

    #[test]
    fn test_parse_review_approved_lowercase() {
        let content = "The implementation is correct. Approved!";
        let result = TaskExecutor::parse_review_response(content);
        assert_eq!(result, ReviewResult::Approved);
    }

    #[test]
    fn test_parse_review_changes_requested() {
        let content = "## Review\n\nCHANGES_REQUESTED\n\n- Fix the error handling\n- Add tests";
        let result = TaskExecutor::parse_review_response(content);
        match result {
            ReviewResult::ChangesRequested(feedback) => {
                assert!(feedback.contains("Fix the error handling"));
            }
            _ => panic!("Expected ChangesRequested"),
        }
    }

    #[test]
    fn test_parse_review_rejected() {
        let content = "REJECTED\n\nThe code has critical issues:\n1. Security vulnerability";
        let result = TaskExecutor::parse_review_response(content);
        match result {
            ReviewResult::ChangesRequested(feedback) => {
                assert!(feedback.contains("Security vulnerability"));
            }
            _ => panic!("Expected ChangesRequested"),
        }
    }

    #[test]
    fn test_parse_review_unclear() {
        let content = "I'm not sure about this implementation.";
        let result = TaskExecutor::parse_review_response(content);
        match result {
            ReviewResult::ChangesRequested(feedback) => {
                assert!(feedback.contains("Manual review required"));
            }
            _ => panic!("Expected ChangesRequested"),
        }
    }

    #[test]
    fn test_parse_review_not_approved() {
        let content = "This is NOT APPROVED due to issues.";
        let result = TaskExecutor::parse_review_response(content);
        match result {
            ReviewResult::ChangesRequested(_) => {}
            ReviewResult::Approved => panic!("Should not be approved when NOT APPROVED is present"),
            ReviewResult::FindingsDetected(_) => {}
        }
    }

    #[test]
    fn test_executor_config_builder() {
        let config = ExecutorConfig::new("/repo")
            .with_plan_approval(false)
            .with_human_review(false)
            .with_max_iterations(5);

        assert_eq!(config.repo_path, PathBuf::from("/repo"));
        assert!(!config.require_plan_approval);
        assert!(!config.require_human_review);
        assert_eq!(config.max_review_iterations, 5);
    }
}
