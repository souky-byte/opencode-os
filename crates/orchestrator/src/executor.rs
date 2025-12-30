use opencode::OpenCodeClient;
use opencode_core::{Task, TaskStatus};
use std::sync::Arc;

use crate::error::Result;
use crate::prompts::PhasePrompts;
use crate::state_machine::TaskStateMachine;

pub struct ExecutorConfig {
    pub require_plan_approval: bool,
    pub require_human_review: bool,
    pub max_review_iterations: u32,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            require_plan_approval: true,
            require_human_review: true,
            max_review_iterations: 3,
        }
    }
}

pub struct TaskExecutor {
    opencode: Arc<OpenCodeClient>,
    config: ExecutorConfig,
}

impl TaskExecutor {
    pub fn new(opencode: Arc<OpenCodeClient>) -> Self {
        Self {
            opencode,
            config: ExecutorConfig::default(),
        }
    }

    pub fn with_config(opencode: Arc<OpenCodeClient>, config: ExecutorConfig) -> Self {
        Self { opencode, config }
    }

    pub fn transition(&self, task: &mut Task, to: TaskStatus) -> Result<()> {
        TaskStateMachine::validate_transition(&task.status, &to)?;
        task.status = to;
        task.updated_at = chrono::Utc::now();
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
                    Ok(PhaseResult::AwaitingApproval)
                } else {
                    self.transition(task, TaskStatus::InProgress)?;
                    self.run_implementation_session(task).await
                }
            }
            TaskStatus::InProgress => self.run_implementation_session(task).await,
            TaskStatus::AiReview => self.run_ai_review(task).await,
            TaskStatus::Review => {
                if self.config.require_human_review {
                    Ok(PhaseResult::AwaitingApproval)
                } else {
                    self.transition(task, TaskStatus::Done)?;
                    Ok(PhaseResult::Completed)
                }
            }
            TaskStatus::Done => Ok(PhaseResult::Completed),
        }
    }

    async fn run_planning_session(&self, task: &mut Task) -> Result<PhaseResult> {
        let session = self
            .opencode
            .create_session(Some(format!("Planning: {}", task.title)))
            .await?;

        let prompt = PhasePrompts::planning(task);
        self.opencode
            .send_message(&session.id, &prompt, None)
            .await?;

        self.transition(task, TaskStatus::PlanningReview)?;

        Ok(PhaseResult::SessionCreated {
            session_id: session.id,
        })
    }

    async fn run_implementation_session(&self, task: &mut Task) -> Result<PhaseResult> {
        let session = self
            .opencode
            .create_session(Some(format!("Implementation: {}", task.title)))
            .await?;

        let prompt = PhasePrompts::implementation(task);
        self.opencode
            .send_message(&session.id, &prompt, None)
            .await?;

        self.transition(task, TaskStatus::AiReview)?;

        Ok(PhaseResult::SessionCreated {
            session_id: session.id,
        })
    }

    async fn run_ai_review(&self, task: &mut Task) -> Result<PhaseResult> {
        let session = self
            .opencode
            .create_session(Some(format!("AI Review: {}", task.title)))
            .await?;

        let diff = self.get_workspace_diff(task).await?;
        let prompt = PhasePrompts::review(task, &diff);

        let response = self
            .opencode
            .send_message(&session.id, &prompt, None)
            .await?;

        let review_result = self.parse_review_response(&response.message.content);

        match review_result {
            ReviewResult::Approved => {
                self.transition(task, TaskStatus::Review)?;
                Ok(PhaseResult::ReviewPassed)
            }
            ReviewResult::ChangesRequested(feedback) => {
                self.transition(task, TaskStatus::InProgress)?;
                Ok(PhaseResult::ReviewFailed { feedback })
            }
        }
    }

    async fn get_workspace_diff(&self, task: &Task) -> Result<String> {
        if let Some(ref _workspace_path) = task.workspace_path {
            Ok("(diff would be fetched from VCS)".to_string())
        } else {
            Ok("(no workspace configured)".to_string())
        }
    }

    fn parse_review_response(&self, content: &str) -> ReviewResult {
        if content.contains("APPROVED") {
            ReviewResult::Approved
        } else if content.contains("CHANGES_REQUESTED") {
            let feedback = content
                .lines()
                .skip_while(|line| !line.contains("CHANGES_REQUESTED"))
                .skip(1)
                .collect::<Vec<_>>()
                .join("\n");
            ReviewResult::ChangesRequested(feedback)
        } else {
            ReviewResult::ChangesRequested(
                "Review response unclear. Please review manually.".to_string(),
            )
        }
    }

    pub async fn run_fix_iteration(&self, task: &mut Task, feedback: &str) -> Result<PhaseResult> {
        let session = self
            .opencode
            .create_session(Some(format!("Fix: {}", task.title)))
            .await?;

        let prompt = PhasePrompts::fix_issues(task, feedback);
        self.opencode
            .send_message(&session.id, &prompt, None)
            .await?;

        self.transition(task, TaskStatus::AiReview)?;

        Ok(PhaseResult::SessionCreated {
            session_id: session.id,
        })
    }
}

#[derive(Debug, Clone)]
pub enum PhaseResult {
    SessionCreated { session_id: String },
    AwaitingApproval,
    ReviewPassed,
    ReviewFailed { feedback: String },
    Completed,
}

#[derive(Debug, Clone)]
pub enum ReviewResult {
    Approved,
    ChangesRequested(String),
}
