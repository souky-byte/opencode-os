use db::SessionRepository;
use events::{Event, EventBus, EventEnvelope};
use opencode::OpenCodeClient;
use opencode_core::{Session, SessionPhase, Task, TaskStatus};
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{info, warn};
use vcs::{Workspace, WorkspaceManager};

use crate::error::{OrchestratorError, Result};
use crate::files::FileManager;
use crate::prompts::PhasePrompts;
use crate::state_machine::TaskStateMachine;

#[derive(Debug, Clone)]
pub struct ExecutorConfig {
    pub require_plan_approval: bool,
    pub require_human_review: bool,
    pub max_review_iterations: u32,
    pub repo_path: PathBuf,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            require_plan_approval: true,
            require_human_review: true,
            max_review_iterations: 3,
            repo_path: PathBuf::from("."),
        }
    }
}

impl ExecutorConfig {
    pub fn new(repo_path: impl Into<PathBuf>) -> Self {
        Self {
            repo_path: repo_path.into(),
            ..Default::default()
        }
    }

    pub fn with_plan_approval(mut self, require: bool) -> Self {
        self.require_plan_approval = require;
        self
    }

    pub fn with_human_review(mut self, require: bool) -> Self {
        self.require_human_review = require;
        self
    }

    pub fn with_max_iterations(mut self, max: u32) -> Self {
        self.max_review_iterations = max;
        self
    }
}

pub struct TaskExecutor {
    opencode: Arc<OpenCodeClient>,
    config: ExecutorConfig,
    file_manager: FileManager,
    workspace_manager: Option<Arc<WorkspaceManager>>,
    session_repo: Option<Arc<SessionRepository>>,
    event_bus: Option<EventBus>,
}

impl TaskExecutor {
    pub fn new(opencode: Arc<OpenCodeClient>, config: ExecutorConfig) -> Self {
        let file_manager = FileManager::new(&config.repo_path);
        Self {
            opencode,
            config,
            file_manager,
            workspace_manager: None,
            session_repo: None,
            event_bus: None,
        }
    }

    pub fn with_workspace_manager(mut self, manager: Arc<WorkspaceManager>) -> Self {
        self.workspace_manager = Some(manager);
        self
    }

    pub fn with_session_repo(mut self, repo: Arc<SessionRepository>) -> Self {
        self.session_repo = Some(repo);
        self
    }

    pub fn with_event_bus(mut self, bus: EventBus) -> Self {
        self.event_bus = Some(bus);
        self
    }

    pub fn transition(&self, task: &mut Task, to: TaskStatus) -> Result<()> {
        let from = task.status;
        TaskStateMachine::validate_transition(&task.status, &to)?;
        task.status = to;
        task.updated_at = chrono::Utc::now();

        self.emit_event(Event::TaskStatusChanged {
            task_id: task.id,
            from_status: from.as_str().to_string(),
            to_status: to.as_str().to_string(),
        });

        Ok(())
    }

    fn emit_event(&self, event: Event) {
        if let Some(ref bus) = self.event_bus {
            bus.publish(EventEnvelope::new(event));
        }
    }

    async fn persist_session(&self, session: &Session) -> Result<()> {
        if let Some(ref repo) = self.session_repo {
            repo.create(session).await?;
        }
        Ok(())
    }

    async fn update_session(&self, session: &Session) -> Result<()> {
        if let Some(ref repo) = self.session_repo {
            repo.update(session).await?;
        }
        Ok(())
    }

    pub async fn execute_phase(&self, task: &mut Task) -> Result<PhaseResult> {
        match task.status {
            TaskStatus::Todo => {
                self.transition(task, TaskStatus::Planning)?;
                self.run_planning_session(task).await
            }
            TaskStatus::Planning => self.run_planning_session(task).await,
            TaskStatus::PlanningReview => {
                if self.config.require_plan_approval {
                    Ok(PhaseResult::AwaitingApproval {
                        phase: SessionPhase::Planning,
                    })
                } else {
                    self.transition(task, TaskStatus::InProgress)?;
                    self.run_implementation_session(task).await
                }
            }
            TaskStatus::InProgress => self.run_implementation_session(task).await,
            TaskStatus::AiReview => self.run_ai_review(task, 0).await,
            TaskStatus::Review => {
                if self.config.require_human_review {
                    Ok(PhaseResult::AwaitingApproval {
                        phase: SessionPhase::Review,
                    })
                } else {
                    self.transition(task, TaskStatus::Done)?;
                    Ok(PhaseResult::Completed)
                }
            }
            TaskStatus::Done => Ok(PhaseResult::Completed),
        }
    }

    pub async fn run_full_cycle(&self, task: &mut Task) -> Result<PhaseResult> {
        info!("Starting full cycle for task {}: {}", task.id, task.title);

        if task.status == TaskStatus::Done {
            return Ok(PhaseResult::Completed);
        }

        if task.status == TaskStatus::Todo {
            self.transition(task, TaskStatus::Planning)?;
        }

        if task.status == TaskStatus::Planning {
            let result = self.run_planning_session(task).await?;
            if self.config.require_plan_approval {
                return Ok(result);
            }
        }

        if task.status == TaskStatus::PlanningReview {
            self.transition(task, TaskStatus::InProgress)?;
        }

        if task.status == TaskStatus::InProgress {
            self.run_implementation_session(task).await?;
        }

        let mut iteration = 0;
        while task.status == TaskStatus::AiReview && iteration < self.config.max_review_iterations {
            let result = self.run_ai_review(task, iteration).await?;
            match result {
                PhaseResult::ReviewPassed { .. } => {
                    if self.config.require_human_review {
                        return Ok(PhaseResult::AwaitingApproval {
                            phase: SessionPhase::Review,
                        });
                    } else {
                        self.transition(task, TaskStatus::Done)?;
                        return Ok(PhaseResult::Completed);
                    }
                }
                PhaseResult::ReviewFailed { feedback, .. } => {
                    info!(
                        "AI review failed (iteration {}), running fix iteration",
                        iteration
                    );
                    self.run_fix_iteration(task, &feedback).await?;
                    iteration += 1;
                }
                _ => return Ok(result),
            }
        }

        if iteration >= self.config.max_review_iterations {
            warn!(
                "Task {} exceeded max review iterations ({})",
                task.id, self.config.max_review_iterations
            );
            return Ok(PhaseResult::MaxIterationsExceeded {
                iterations: iteration,
            });
        }

        if task.status == TaskStatus::Review {
            if self.config.require_human_review {
                return Ok(PhaseResult::AwaitingApproval {
                    phase: SessionPhase::Review,
                });
            }
            self.transition(task, TaskStatus::Done)?;
        }

        Ok(PhaseResult::Completed)
    }

    async fn run_planning_session(&self, task: &mut Task) -> Result<PhaseResult> {
        info!("Running planning session for task {}", task.id);

        let mut session = Session::new(task.id, SessionPhase::Planning);

        let opencode_session = self
            .opencode
            .create_session(Some(format!("Planning: {}", task.title)))
            .await?;

        session.start(opencode_session.id.clone());
        self.persist_session(&session).await?;

        self.emit_event(Event::SessionStarted {
            session_id: session.id,
            task_id: task.id,
        });

        let prompt = PhasePrompts::planning(task);
        let response = self
            .opencode
            .send_message(&opencode_session.id, &prompt, None)
            .await?;

        let plan_path = self
            .file_manager
            .write_plan(task.id, &response.message.content)
            .await?;

        session.complete();
        self.update_session(&session).await?;

        self.emit_event(Event::SessionEnded {
            session_id: session.id,
            task_id: task.id,
            success: true,
        });

        self.transition(task, TaskStatus::PlanningReview)?;

        Ok(PhaseResult::PlanCreated {
            session_id: opencode_session.id,
            plan_path: plan_path.to_string_lossy().to_string(),
        })
    }

    async fn run_implementation_session(&self, task: &mut Task) -> Result<PhaseResult> {
        info!("Running implementation session for task {}", task.id);

        let mut session = Session::new(task.id, SessionPhase::Implementation);

        if let Some(ref wm) = self.workspace_manager {
            let workspace = wm.setup_workspace(&task.id.to_string()).await.map_err(|e| {
                OrchestratorError::ExecutionFailed(format!("Failed to setup workspace: {}", e))
            })?;
            task.workspace_path = Some(workspace.path.to_string_lossy().to_string());

            self.emit_event(Event::WorkspaceCreated {
                task_id: task.id,
                path: workspace.path.to_string_lossy().to_string(),
            });
        }

        let opencode_session = self
            .opencode
            .create_session(Some(format!("Implementation: {}", task.title)))
            .await?;

        session.start(opencode_session.id.clone());
        self.persist_session(&session).await?;

        self.emit_event(Event::SessionStarted {
            session_id: session.id,
            task_id: task.id,
        });

        let plan = if self.file_manager.plan_exists(task.id).await {
            self.file_manager.read_plan(task.id).await.ok()
        } else {
            None
        };

        let prompt = PhasePrompts::implementation_with_plan(task, plan.as_deref());
        let _response = self
            .opencode
            .send_message(&opencode_session.id, &prompt, None)
            .await?;

        session.complete();
        self.update_session(&session).await?;

        self.emit_event(Event::SessionEnded {
            session_id: session.id,
            task_id: task.id,
            success: true,
        });

        self.transition(task, TaskStatus::AiReview)?;

        Ok(PhaseResult::SessionCreated {
            session_id: opencode_session.id,
        })
    }

    async fn run_ai_review(&self, task: &mut Task, iteration: u32) -> Result<PhaseResult> {
        info!(
            "Running AI review session for task {} (iteration {})",
            task.id, iteration
        );

        let mut session = Session::new(task.id, SessionPhase::Review);

        let opencode_session = self
            .opencode
            .create_session(Some(format!("AI Review: {}", task.title)))
            .await?;

        session.start(opencode_session.id.clone());
        self.persist_session(&session).await?;

        self.emit_event(Event::SessionStarted {
            session_id: session.id,
            task_id: task.id,
        });

        let diff = self.get_workspace_diff(task).await?;
        let prompt = PhasePrompts::review(task, &diff);

        let response = self
            .opencode
            .send_message(&opencode_session.id, &prompt, None)
            .await?;

        let _review_path = self
            .file_manager
            .write_review(task.id, &response.message.content)
            .await?;

        session.complete();
        self.update_session(&session).await?;

        let review_result = Self::parse_review_response(&response.message.content);

        self.emit_event(Event::SessionEnded {
            session_id: session.id,
            task_id: task.id,
            success: matches!(review_result, ReviewResult::Approved),
        });

        match review_result {
            ReviewResult::Approved => {
                self.transition(task, TaskStatus::Review)?;
                Ok(PhaseResult::ReviewPassed {
                    session_id: opencode_session.id,
                })
            }
            ReviewResult::ChangesRequested(feedback) => {
                self.transition(task, TaskStatus::InProgress)?;
                Ok(PhaseResult::ReviewFailed {
                    session_id: opencode_session.id,
                    feedback,
                    iteration,
                })
            }
        }
    }

    async fn get_workspace_diff(&self, task: &Task) -> Result<String> {
        if let Some(ref wm) = self.workspace_manager {
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

    fn parse_review_response(content: &str) -> ReviewResult {
        let content_upper = content.to_uppercase();

        if content_upper.contains("APPROVED") && !content_upper.contains("NOT APPROVED") {
            ReviewResult::Approved
        } else if content_upper.contains("CHANGES_REQUESTED")
            || content_upper.contains("CHANGES REQUESTED")
            || content_upper.contains("REJECTED")
        {
            let feedback = content
                .lines()
                .skip_while(|line| {
                    let upper = line.to_uppercase();
                    !upper.contains("CHANGES_REQUESTED")
                        && !upper.contains("CHANGES REQUESTED")
                        && !upper.contains("REJECTED")
                        && !upper.contains("FEEDBACK")
                        && !upper.contains("ISSUES")
                })
                .skip(1)
                .collect::<Vec<_>>()
                .join("\n")
                .trim()
                .to_string();

            if feedback.is_empty() {
                ReviewResult::ChangesRequested(content.to_string())
            } else {
                ReviewResult::ChangesRequested(feedback)
            }
        } else {
            ReviewResult::ChangesRequested(
                "Review response unclear. Manual review required.".to_string(),
            )
        }
    }

    pub async fn run_fix_iteration(&self, task: &mut Task, feedback: &str) -> Result<PhaseResult> {
        info!("Running fix iteration for task {}", task.id);

        let mut session = Session::new(task.id, SessionPhase::Implementation);

        let opencode_session = self
            .opencode
            .create_session(Some(format!("Fix: {}", task.title)))
            .await?;

        session.start(opencode_session.id.clone());
        self.persist_session(&session).await?;

        self.emit_event(Event::SessionStarted {
            session_id: session.id,
            task_id: task.id,
        });

        let prompt = PhasePrompts::fix_issues(task, feedback);
        let _response = self
            .opencode
            .send_message(&opencode_session.id, &prompt, None)
            .await?;

        session.complete();
        self.update_session(&session).await?;

        self.emit_event(Event::SessionEnded {
            session_id: session.id,
            task_id: task.id,
            success: true,
        });

        self.transition(task, TaskStatus::AiReview)?;

        Ok(PhaseResult::SessionCreated {
            session_id: opencode_session.id,
        })
    }

    pub async fn approve_plan(&self, task: &mut Task) -> Result<()> {
        if task.status != TaskStatus::PlanningReview {
            return Err(OrchestratorError::InvalidTransition {
                from: task.status.as_str().to_string(),
                to: "InProgress (plan approval)".to_string(),
            });
        }
        self.transition(task, TaskStatus::InProgress)?;
        Ok(())
    }

    pub async fn reject_plan(&self, task: &mut Task, feedback: &str) -> Result<PhaseResult> {
        if task.status != TaskStatus::PlanningReview {
            return Err(OrchestratorError::InvalidTransition {
                from: task.status.as_str().to_string(),
                to: "Planning (plan rejection)".to_string(),
            });
        }
        self.transition(task, TaskStatus::Planning)?;

        let mut session = Session::new(task.id, SessionPhase::Planning);
        let opencode_session = self
            .opencode
            .create_session(Some(format!("Re-planning: {}", task.title)))
            .await?;

        session.start(opencode_session.id.clone());
        self.persist_session(&session).await?;

        let prompt = PhasePrompts::replan(task, feedback);
        let response = self
            .opencode
            .send_message(&opencode_session.id, &prompt, None)
            .await?;

        let plan_path = self
            .file_manager
            .write_plan(task.id, &response.message.content)
            .await?;

        session.complete();
        self.update_session(&session).await?;

        self.transition(task, TaskStatus::PlanningReview)?;

        Ok(PhaseResult::PlanCreated {
            session_id: opencode_session.id,
            plan_path: plan_path.to_string_lossy().to_string(),
        })
    }

    pub async fn approve_review(&self, task: &mut Task) -> Result<()> {
        if task.status != TaskStatus::Review {
            return Err(OrchestratorError::InvalidTransition {
                from: task.status.as_str().to_string(),
                to: "Done (review approval)".to_string(),
            });
        }
        self.transition(task, TaskStatus::Done)?;
        Ok(())
    }

    pub async fn reject_review(&self, task: &mut Task, feedback: &str) -> Result<PhaseResult> {
        if task.status != TaskStatus::Review {
            return Err(OrchestratorError::InvalidTransition {
                from: task.status.as_str().to_string(),
                to: "InProgress (review rejection)".to_string(),
            });
        }
        self.run_fix_iteration(task, feedback).await
    }
}

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
    MaxIterationsExceeded {
        iterations: u32,
    },
    Completed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReviewResult {
    Approved,
    ChangesRequested(String),
}

#[cfg(test)]
mod tests {
    use super::*;

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
