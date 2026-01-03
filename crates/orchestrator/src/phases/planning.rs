//! Planning phase implementation.
//!
//! The planning phase generates a detailed implementation plan for the task.
//! After completion, the plan is saved and the task transitions to PlanningReview.

use async_trait::async_trait;
use opencode_core::{SessionPhase, Task, TaskStatus};
use tracing::{debug, info};

use crate::core::{
    Phase, PhaseConfig, PhaseMetadata, PhaseOutcome, ResourceRequirements, SessionOutput,
};
use crate::error::Result;
use crate::prompts::PhasePrompts;
use crate::services::ExecutorContext;

/// Planning phase - generates implementation plan for a task.
///
/// This phase:
/// 1. Sends a planning prompt to OpenCode
/// 2. Saves the generated plan to `.opencode-studio/kanban/plans/{task_id}.md`
/// 3. Transitions to PlanningReview (or auto-approves based on config)
pub struct PlanningPhase;

impl PlanningPhase {
    /// Create a new planning phase.
    pub fn new() -> Self {
        Self
    }
}

impl Default for PlanningPhase {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Phase for PlanningPhase {
    fn phase_type(&self) -> SessionPhase {
        SessionPhase::Planning
    }

    fn required_resources(&self) -> ResourceRequirements {
        // Planning doesn't need workspace or MCP
        ResourceRequirements::default()
    }

    async fn build_config(&self, ctx: &ExecutorContext, task: &Task) -> Result<PhaseConfig> {
        let prompt = PhasePrompts::planning(task);
        let working_dir = ctx.config.repo_path.clone();

        debug!(
            task_id = %task.id,
            prompt_length = prompt.len(),
            "Building planning phase config"
        );

        Ok(PhaseConfig {
            prompt,
            working_dir,
            mcp_servers: vec![],
            skip_status_update: false,
            metadata: PhaseMetadata::Planning,
        })
    }

    async fn process_result(
        &self,
        ctx: &ExecutorContext,
        task: &mut Task,
        result: &SessionOutput,
    ) -> Result<PhaseOutcome> {
        if !result.success {
            return Ok(PhaseOutcome::Transition {
                next_status: TaskStatus::Todo,
            });
        }

        // Save plan to file
        ctx.file_manager
            .write_plan(task.id, &result.response_text)
            .await?;

        info!(
            task_id = %task.id,
            plan_length = result.response_text.len(),
            "Plan saved"
        );

        // Transition to planning review
        ctx.transition(task, TaskStatus::PlanningReview)?;

        if ctx.config.require_plan_approval {
            // Wait for human approval
            Ok(PhaseOutcome::AwaitingApproval {
                phase: SessionPhase::Planning,
            })
        } else {
            // Auto-approve and continue to implementation
            ctx.transition(task, TaskStatus::InProgress)?;
            Ok(PhaseOutcome::Transition {
                next_status: TaskStatus::InProgress,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_planning_phase_type() {
        let phase = PlanningPhase::new();
        assert_eq!(phase.phase_type(), SessionPhase::Planning);
    }

    #[test]
    fn test_planning_phase_resources() {
        let phase = PlanningPhase::new();
        let resources = phase.required_resources();

        assert!(!resources.needs_workspace);
        assert!(!resources.needs_mcp_findings);
        assert!(!resources.needs_diff);
    }

    #[test]
    fn test_planning_phase_default() {
        let phase = PlanningPhase::default();
        assert_eq!(phase.phase_type(), SessionPhase::Planning);
    }
}
