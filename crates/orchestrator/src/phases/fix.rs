//! Fix phase with multiple fix modes.
//!
//! The fix phase addresses review findings and implements corrections.
//! It supports multiple modes: MCP-based, feedback-based, and user comment-based.

use async_trait::async_trait;
use opencode_core::{SessionPhase, Task, TaskStatus};
use tracing::{debug, info};

use crate::core::{
    McpServerSpec, Phase, PhaseConfig, PhaseMetadata, PhaseOutcome, ResourceRequirements,
    SessionOutput,
};
use crate::error::Result;
use crate::prompts::{PhasePrompts, UserReviewComment};
use crate::services::ExecutorContext;

/// Mode of fix operation.
#[derive(Debug, Clone)]
pub enum FixMode {
    /// Fix based on MCP findings server data
    McpFindings,
    /// Fix based on feedback string (from AI review)
    Feedback(String),
    /// Fix based on user-provided review comments
    UserComments(Vec<UserReviewComment>),
}

impl FixMode {
    /// Check if this mode requires MCP.
    pub fn requires_mcp(&self) -> bool {
        matches!(self, FixMode::McpFindings)
    }
}

/// Fix phase - addresses review findings.
///
/// This phase supports three fix modes:
/// 1. **McpFindings**: Uses MCP server to access structured findings
/// 2. **Feedback**: Uses feedback string from previous review
/// 3. **UserComments**: Uses specific user-provided comments
///
/// After fixing, the task transitions back to AiReview for verification.
pub struct FixPhase {
    /// The fix mode determining how issues are addressed
    mode: FixMode,
}

impl FixPhase {
    /// Create a fix phase with MCP findings mode.
    pub fn with_mcp_findings() -> Self {
        Self {
            mode: FixMode::McpFindings,
        }
    }

    /// Create a fix phase with feedback mode.
    pub fn with_feedback(feedback: String) -> Self {
        Self {
            mode: FixMode::Feedback(feedback),
        }
    }

    /// Create a fix phase with user comments mode.
    pub fn with_user_comments(comments: Vec<UserReviewComment>) -> Self {
        Self {
            mode: FixMode::UserComments(comments),
        }
    }

    /// Get the fix mode.
    pub fn mode(&self) -> &FixMode {
        &self.mode
    }
}

#[async_trait]
impl Phase for FixPhase {
    fn phase_type(&self) -> SessionPhase {
        SessionPhase::Fix
    }

    fn required_resources(&self) -> ResourceRequirements {
        ResourceRequirements {
            needs_workspace: true,
            needs_mcp_findings: self.mode.requires_mcp(),
            needs_diff: false,
        }
    }

    async fn build_config(&self, ctx: &ExecutorContext, task: &Task) -> Result<PhaseConfig> {
        let working_dir = ctx.working_dir_for_task(task);

        let prompt = match &self.mode {
            FixMode::McpFindings => PhasePrompts::fix_with_mcp(task),
            FixMode::Feedback(feedback) => PhasePrompts::fix_issues(task, feedback),
            FixMode::UserComments(comments) => PhasePrompts::fix_user_comments(task, comments),
        };

        let mcp_servers = if self.mode.requires_mcp() {
            vec![McpServerSpec::findings()]
        } else {
            vec![]
        };

        debug!(
            task_id = %task.id,
            mode = ?self.mode,
            prompt_length = prompt.len(),
            mcp_servers = mcp_servers.len(),
            "Building fix phase config"
        );

        Ok(PhaseConfig {
            prompt,
            working_dir,
            mcp_servers,
            skip_status_update: false,
            metadata: PhaseMetadata::Fix,
        })
    }

    async fn process_result(
        &self,
        ctx: &ExecutorContext,
        task: &mut Task,
        result: &SessionOutput,
    ) -> Result<PhaseOutcome> {
        if !result.success {
            info!(
                task_id = %task.id,
                error = ?result.error,
                "Fix phase failed"
            );
            return Ok(PhaseOutcome::Transition {
                next_status: TaskStatus::AiReview,
            });
        }

        info!(
            task_id = %task.id,
            mode = ?self.mode,
            response_length = result.response_text.len(),
            "Fix phase completed"
        );

        // After fix, always go back to AI review for verification
        ctx.transition(task, TaskStatus::AiReview)?;

        Ok(PhaseOutcome::Transition {
            next_status: TaskStatus::AiReview,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fix_phase_mcp_mode() {
        let phase = FixPhase::with_mcp_findings();
        assert!(phase.mode().requires_mcp());
        assert_eq!(phase.phase_type(), SessionPhase::Fix);
    }

    #[test]
    fn test_fix_phase_feedback_mode() {
        let phase = FixPhase::with_feedback("Fix the bug".to_string());
        assert!(!phase.mode().requires_mcp());

        if let FixMode::Feedback(fb) = phase.mode() {
            assert_eq!(fb, "Fix the bug");
        } else {
            panic!("Expected Feedback mode");
        }
    }

    #[test]
    fn test_fix_phase_user_comments_mode() {
        let comments = vec![UserReviewComment {
            file_path: "src/main.rs".to_string(),
            line_start: 10,
            line_end: 15,
            side: "RIGHT".to_string(),
            content: "Fix this".to_string(),
        }];

        let phase = FixPhase::with_user_comments(comments);
        assert!(!phase.mode().requires_mcp());

        if let FixMode::UserComments(c) = phase.mode() {
            assert_eq!(c.len(), 1);
            assert_eq!(c[0].file_path, "src/main.rs");
        } else {
            panic!("Expected UserComments mode");
        }
    }

    #[test]
    fn test_fix_phase_resources_mcp() {
        let phase = FixPhase::with_mcp_findings();
        let resources = phase.required_resources();

        assert!(resources.needs_workspace);
        assert!(resources.needs_mcp_findings);
        assert!(!resources.needs_diff);
    }

    #[test]
    fn test_fix_phase_resources_feedback() {
        let phase = FixPhase::with_feedback("Fix".to_string());
        let resources = phase.required_resources();

        assert!(resources.needs_workspace);
        assert!(!resources.needs_mcp_findings);
        assert!(!resources.needs_diff);
    }
}
