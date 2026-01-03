//! Review phase with MCP/JSON fallback.
//!
//! The review phase performs AI-driven code review with structured findings.
//! It supports both MCP-based and JSON-based finding extraction.

use async_trait::async_trait;
use opencode_core::{SessionPhase, Task, TaskStatus};
use std::path::PathBuf;
use tracing::{debug, info, warn};
use vcs::Workspace;

use crate::core::{
    McpServerSpec, Phase, PhaseConfig, PhaseMetadata, PhaseOutcome, ResourceRequirements,
    SessionOutput,
};
use crate::error::{OrchestratorError, Result};
use crate::prompts::PhasePrompts;
use crate::services::message_parser::ReviewResult;
use crate::services::{ExecutorContext, MessageParser};

/// Review phase - performs AI-driven code review.
///
/// This phase:
/// 1. Gets the workspace diff
/// 2. Sends a review prompt with the diff to OpenCode
/// 3. Extracts structured findings (via MCP or JSON parsing)
/// 4. Saves findings and transitions based on result
///
/// The phase supports both MCP-based and fallback JSON-based finding extraction.
pub struct ReviewPhase {
    /// Current review iteration (0-indexed)
    iteration: u32,
    /// Whether to use MCP for structured findings
    use_mcp: bool,
}

impl ReviewPhase {
    /// Create a new review phase with MCP support.
    pub fn new(iteration: u32) -> Self {
        Self {
            iteration,
            use_mcp: true,
        }
    }

    /// Create a review phase without MCP (JSON fallback only).
    pub fn without_mcp(iteration: u32) -> Self {
        Self {
            iteration,
            use_mcp: false,
        }
    }

    /// Get the current iteration number.
    pub fn iteration(&self) -> u32 {
        self.iteration
    }

    /// Check if MCP is enabled.
    pub fn uses_mcp(&self) -> bool {
        self.use_mcp
    }

    /// Parse review result from session output.
    async fn parse_review_result(
        &self,
        ctx: &ExecutorContext,
        task: &Task,
        result: &SessionOutput,
    ) -> Result<ReviewResult> {
        // Try to read findings from MCP server output
        if self.use_mcp {
            if let Ok(Some(findings)) = ctx.file_manager.read_findings(task.id).await {
                if !findings.findings.is_empty() {
                    return Ok(ReviewResult::FindingsDetected(findings.findings.len()));
                }
                if findings.approved {
                    return Ok(ReviewResult::Approved);
                }
            }
        }

        // Fallback to parsing response text
        let review_result = MessageParser::parse_review_response(&result.response_text);
        Ok(review_result)
    }

    /// Get workspace diff for review.
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
}

#[async_trait]
impl Phase for ReviewPhase {
    fn phase_type(&self) -> SessionPhase {
        SessionPhase::Review
    }

    fn required_resources(&self) -> ResourceRequirements {
        ResourceRequirements {
            needs_workspace: true,
            needs_mcp_findings: self.use_mcp,
            needs_diff: true,
        }
    }

    async fn build_config(&self, ctx: &ExecutorContext, task: &Task) -> Result<PhaseConfig> {
        let working_dir = ctx.working_dir_for_task(task);

        // Get workspace diff - require workspace, no silent fallback
        let diff = Self::get_workspace_diff(ctx, task).await.map_err(|e| {
            warn!(
                task_id = %task.id,
                error = %e,
                "Failed to get workspace diff"
            );
            OrchestratorError::WorkspaceRequired(task.id)
        })?;

        let prompt = if self.use_mcp {
            PhasePrompts::review_with_mcp(task, &diff)
        } else {
            PhasePrompts::review(task, &diff)
        };

        let mcp_servers = if self.use_mcp {
            vec![McpServerSpec::findings()]
        } else {
            vec![]
        };

        debug!(
            task_id = %task.id,
            iteration = self.iteration,
            use_mcp = self.use_mcp,
            diff_length = diff.len(),
            prompt_length = prompt.len(),
            "Building review phase config"
        );

        Ok(PhaseConfig {
            prompt,
            working_dir,
            mcp_servers,
            skip_status_update: false,
            metadata: PhaseMetadata::Review {
                iteration: self.iteration,
            },
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
                next_status: TaskStatus::AiReview,
            });
        }

        // Save review output
        ctx.file_manager
            .write_review(task.id, &result.response_text)
            .await?;

        // Parse findings
        let review_result = self.parse_review_result(ctx, task, result).await?;

        info!(
            task_id = %task.id,
            iteration = self.iteration,
            result = ?review_result,
            "Review phase completed"
        );

        match review_result {
            ReviewResult::Approved => {
                // Transition to human review
                ctx.transition(task, TaskStatus::Review)?;

                if ctx.config.require_human_review {
                    Ok(PhaseOutcome::AwaitingApproval {
                        phase: SessionPhase::Review,
                    })
                } else {
                    // Auto-approve
                    ctx.transition(task, TaskStatus::Done)?;
                    Ok(PhaseOutcome::Complete)
                }
            }
            ReviewResult::FindingsDetected(count) => {
                info!(
                    task_id = %task.id,
                    finding_count = count,
                    "Findings detected, awaiting action"
                );

                // Check iteration limit
                if self.iteration >= ctx.config.max_review_iterations {
                    warn!(
                        task_id = %task.id,
                        max_iterations = ctx.config.max_review_iterations,
                        "Max review iterations reached"
                    );
                    // Force transition to human review
                    ctx.transition(task, TaskStatus::Review)?;
                    Ok(PhaseOutcome::AwaitingApproval {
                        phase: SessionPhase::Review,
                    })
                } else {
                    Ok(PhaseOutcome::Iterate {
                        feedback: format!("{} issues found", count),
                        iteration: self.iteration,
                    })
                }
            }
            ReviewResult::ChangesRequested(feedback) => {
                info!(
                    task_id = %task.id,
                    feedback_length = feedback.len(),
                    "Changes requested"
                );

                // Check iteration limit
                if self.iteration >= ctx.config.max_review_iterations {
                    warn!(
                        task_id = %task.id,
                        max_iterations = ctx.config.max_review_iterations,
                        "Max review iterations reached"
                    );
                    ctx.transition(task, TaskStatus::Review)?;
                    Ok(PhaseOutcome::AwaitingApproval {
                        phase: SessionPhase::Review,
                    })
                } else {
                    Ok(PhaseOutcome::Iterate {
                        feedback,
                        iteration: self.iteration,
                    })
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_review_phase_new() {
        let phase = ReviewPhase::new(0);
        assert_eq!(phase.iteration(), 0);
        assert!(phase.uses_mcp());
        assert_eq!(phase.phase_type(), SessionPhase::Review);
    }

    #[test]
    fn test_review_phase_without_mcp() {
        let phase = ReviewPhase::without_mcp(1);
        assert_eq!(phase.iteration(), 1);
        assert!(!phase.uses_mcp());
    }

    #[test]
    fn test_review_phase_resources() {
        let phase = ReviewPhase::new(0);
        let resources = phase.required_resources();

        assert!(resources.needs_workspace);
        assert!(resources.needs_mcp_findings);
        assert!(resources.needs_diff);
    }

    #[test]
    fn test_review_phase_resources_without_mcp() {
        let phase = ReviewPhase::without_mcp(0);
        let resources = phase.required_resources();

        assert!(resources.needs_workspace);
        assert!(!resources.needs_mcp_findings);
        assert!(resources.needs_diff);
    }
}
