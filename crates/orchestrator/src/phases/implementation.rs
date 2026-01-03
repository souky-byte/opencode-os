//! Implementation phase with multi-phase support.
//!
//! The implementation phase converts the approved plan into working code.
//! It supports both single-phase and multi-phase execution for complex plans.

use async_trait::async_trait;
use opencode_core::{SessionPhase, Task, TaskStatus};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};
use uuid::Uuid;

use crate::core::{
    Phase, PhaseConfig, PhaseMetadata, PhaseOutcome, ResourceRequirements, SessionOutput,
};
use crate::error::Result;
use crate::files::{ParsedPlan, PhaseContext as FilePhaseContext, PhaseSummary};
use crate::plan_parser::parse_plan_phases;
use crate::prompts::PhasePrompts;
use crate::services::ExecutorContext;
use chrono::Utc;

/// Thread-safe atomic phase context for multi-phase execution.
///
/// This ensures proper synchronization when multiple async operations
/// access the phase state concurrently.
pub struct AtomicPhaseContext {
    inner: Arc<RwLock<PhaseContextState>>,
}

/// Internal state for phase tracking.
#[derive(Debug, Clone)]
pub struct PhaseContextState {
    /// Current phase number (1-indexed)
    pub phase_number: u32,
    /// Total number of phases
    pub total_phases: u32,
    /// Completed phase summaries
    pub completed_phases: Vec<PhaseSummary>,
    /// Previous phase summary (for context)
    pub previous_summary: Option<PhaseSummary>,
}

impl AtomicPhaseContext {
    /// Create a new atomic phase context.
    pub fn new(total_phases: u32) -> Self {
        Self {
            inner: Arc::new(RwLock::new(PhaseContextState {
                phase_number: 1,
                total_phases,
                completed_phases: Vec::new(),
                previous_summary: None,
            })),
        }
    }

    /// Load from existing state.
    pub fn from_state(state: PhaseContextState) -> Self {
        Self {
            inner: Arc::new(RwLock::new(state)),
        }
    }

    /// Get the current phase number.
    pub async fn current_phase(&self) -> u32 {
        self.inner.read().await.phase_number
    }

    /// Get the total number of phases.
    pub async fn total_phases(&self) -> u32 {
        self.inner.read().await.total_phases
    }

    /// Check if all phases are complete.
    pub async fn is_complete(&self) -> bool {
        let state = self.inner.read().await;
        state.phase_number > state.total_phases
    }

    /// Advance to the next phase with a summary.
    pub async fn advance(&self, summary: PhaseSummary) {
        let mut state = self.inner.write().await;
        state.completed_phases.push(summary.clone());
        state.previous_summary = Some(summary);
        state.phase_number += 1;
    }

    /// Get a snapshot of the current state.
    pub async fn snapshot(&self) -> PhaseContextState {
        self.inner.read().await.clone()
    }
}

/// Implementation phase - converts plan into working code.
///
/// This phase supports two modes:
/// 1. **Single-phase**: Simple plans executed in one session
/// 2. **Multi-phase**: Complex plans split into multiple sessions
///
/// Multi-phase execution uses `AtomicPhaseContext` for thread-safe
/// state management across async operations.
pub struct ImplementationPhase {
    /// Parsed plan content (if available)
    parsed_plan: Option<ParsedPlan>,
    /// Phase context for multi-phase execution
    phase_context: Option<AtomicPhaseContext>,
    /// Task ID for file operations
    task_id: Uuid,
}

impl ImplementationPhase {
    /// Create a new implementation phase.
    ///
    /// This reads the plan file and determines if multi-phase execution is needed.
    pub async fn new(ctx: &ExecutorContext, task_id: Uuid) -> Result<Self> {
        let parsed_plan = if ctx.file_manager.plan_exists(task_id).await {
            match ctx.file_manager.read_plan(task_id).await {
                Ok(content) => Some(parse_plan_phases(&content)),
                Err(_) => None,
            }
        } else {
            None
        };

        let phase_context = if let Some(ref plan) = parsed_plan {
            if !plan.is_single_phase() {
                // Load existing context or create new
                let state = match ctx.file_manager.read_phase_context(task_id).await {
                    Ok(Some(ctx)) => PhaseContextState {
                        phase_number: ctx.phase_number,
                        total_phases: plan.total_phases(),
                        completed_phases: ctx.completed_phases.clone(),
                        previous_summary: ctx.previous_summary.clone(),
                    },
                    _ => PhaseContextState {
                        phase_number: 1,
                        total_phases: plan.total_phases(),
                        completed_phases: Vec::new(),
                        previous_summary: None,
                    },
                };
                Some(AtomicPhaseContext::from_state(state))
            } else {
                None
            }
        } else {
            None
        };

        Ok(Self {
            parsed_plan,
            phase_context,
            task_id,
        })
    }

    /// Check if this is a multi-phase implementation.
    pub fn is_multi_phase(&self) -> bool {
        self.phase_context.is_some()
    }

    /// Get the current phase number (1 for single-phase).
    pub async fn current_phase(&self) -> u32 {
        match &self.phase_context {
            Some(ctx) => ctx.current_phase().await,
            None => 1,
        }
    }

    /// Get total phases (1 for single-phase).
    pub async fn total_phases(&self) -> u32 {
        match &self.phase_context {
            Some(ctx) => ctx.total_phases().await,
            None => 1,
        }
    }
}

#[async_trait]
impl Phase for ImplementationPhase {
    fn phase_type(&self) -> SessionPhase {
        SessionPhase::Implementation
    }

    fn required_resources(&self) -> ResourceRequirements {
        ResourceRequirements {
            needs_workspace: true,
            needs_mcp_findings: false,
            needs_diff: false,
        }
    }

    async fn build_config(&self, ctx: &ExecutorContext, task: &Task) -> Result<PhaseConfig> {
        let working_dir = ctx.working_dir_for_task(task);

        let (prompt, skip_update, phase_number, total_phases) =
            if let Some(ref phase_ctx) = self.phase_context {
                // Multi-phase: build phase-specific prompt
                let current = phase_ctx.current_phase().await;
                let total = phase_ctx.total_phases().await;
                let plan = self.parsed_plan.as_ref().unwrap();
                let phase = &plan.phases[(current - 1) as usize];
                let state = phase_ctx.snapshot().await;

                // Build the FilePhaseContext for the prompt
                let file_context = FilePhaseContext {
                    phase_number: current,
                    total_phases: total,
                    completed_phases: state.completed_phases.clone(),
                    previous_summary: state.previous_summary.clone(),
                };

                let prompt = PhasePrompts::implementation_phase(task, phase, &file_context);

                (prompt, true, Some(current), Some(total))
            } else {
                // Single phase
                let plan_content = self
                    .parsed_plan
                    .as_ref()
                    .map(|p| p.phases.first().map(|ph| ph.content.as_str()).unwrap_or(""));

                let prompt = PhasePrompts::implementation_with_plan(task, plan_content);
                (prompt, false, None, None)
            };

        let prompt_len = prompt.len();
        debug!(
            task_id = %task.id,
            is_multi_phase = self.is_multi_phase(),
            phase_number = ?phase_number,
            total_phases = ?total_phases,
            prompt_length = prompt_len,
            "Building implementation phase config"
        );

        Ok(PhaseConfig {
            prompt,
            working_dir,
            mcp_servers: vec![],
            skip_status_update: skip_update,
            metadata: PhaseMetadata::Implementation {
                phase_number,
                total_phases,
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
                next_status: TaskStatus::InProgress,
            });
        }

        if let Some(ref phase_ctx) = self.phase_context {
            // Multi-phase: extract summary and advance
            let current = phase_ctx.current_phase().await;
            let plan = self.parsed_plan.as_ref().unwrap();
            let phase = &plan.phases[(current - 1) as usize];

            let summary = extract_or_create_summary(&result.response_text, current, &phase.title);

            // Save phase summary
            ctx.file_manager
                .write_phase_summary(self.task_id, &summary)
                .await?;

            // Mark phase complete in plan
            ctx.file_manager
                .mark_phase_complete_in_plan(self.task_id, current)
                .await?;

            // Advance phase context
            phase_ctx.advance(summary).await;

            // Persist phase context
            let state = phase_ctx.snapshot().await;
            ctx.file_manager
                .write_phase_context(
                    self.task_id,
                    &FilePhaseContext {
                        phase_number: state.phase_number,
                        total_phases: state.total_phases,
                        completed_phases: state.completed_phases.clone(),
                        previous_summary: state.previous_summary.clone(),
                    },
                )
                .await?;

            info!(
                task_id = %task.id,
                phase = current,
                total = state.total_phases,
                "Implementation phase completed"
            );

            if phase_ctx.is_complete().await {
                // All phases done - transition to review
                ctx.transition(task, TaskStatus::AiReview)?;
                Ok(PhaseOutcome::Transition {
                    next_status: TaskStatus::AiReview,
                })
            } else {
                // Continue to next phase
                Ok(PhaseOutcome::Continue)
            }
        } else {
            // Single phase: done - transition to review
            ctx.transition(task, TaskStatus::AiReview)?;

            info!(
                task_id = %task.id,
                "Single-phase implementation completed"
            );

            Ok(PhaseOutcome::Transition {
                next_status: TaskStatus::AiReview,
            })
        }
    }
}

/// Extract or create a summary from the response.
fn extract_or_create_summary(response: &str, phase_number: u32, title: &str) -> PhaseSummary {
    // Try to extract summary from response (look for markers)
    let summary = if let Some(start) = response.find("## Summary") {
        let content = &response[start..];
        let end = content
            .find("\n## ")
            .map(|i| i + start)
            .unwrap_or(response.len());
        response[start..end].trim().to_string()
    } else {
        // Create a default summary
        format!("Completed phase {}: {}", phase_number, title)
    };

    // Try to extract changed files from response
    let files_changed = extract_changed_files(response);

    // Try to extract notes for next phase
    let notes = extract_notes(response);

    PhaseSummary {
        phase_number,
        title: title.to_string(),
        summary,
        files_changed,
        notes,
        completed_at: Utc::now(),
    }
}

/// Extract changed files from response text.
fn extract_changed_files(response: &str) -> Vec<String> {
    let mut files = Vec::new();

    // Look for "Changed files:" or similar patterns
    if let Some(start) = response.find("**Changed files:**") {
        let content = &response[start..];
        for line in content.lines().skip(1) {
            let trimmed = line.trim();
            if trimmed.starts_with('-') {
                let file = trimmed.trim_start_matches('-').trim();
                if !file.is_empty() && !file.starts_with('*') {
                    files.push(file.to_string());
                }
            } else if trimmed.is_empty() || trimmed.starts_with('*') {
                break;
            }
        }
    }

    files
}

/// Extract notes for next phase from response text.
fn extract_notes(response: &str) -> Option<String> {
    if let Some(start) = response.find("**Notes for next phase:**") {
        let content = &response[start + 25..];
        let end = content.find("###").unwrap_or(content.len());
        let notes = content[..end].trim();
        if !notes.is_empty() {
            return Some(notes.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_atomic_phase_context_new() {
        let ctx = AtomicPhaseContext::new(3);

        assert_eq!(ctx.current_phase().await, 1);
        assert_eq!(ctx.total_phases().await, 3);
        assert!(!ctx.is_complete().await);
    }

    #[tokio::test]
    async fn test_atomic_phase_context_advance() {
        let ctx = AtomicPhaseContext::new(2);

        let summary = PhaseSummary::new(1, "Phase 1", "Done", vec![], None);

        ctx.advance(summary).await;

        assert_eq!(ctx.current_phase().await, 2);
        assert!(!ctx.is_complete().await);

        let summary2 = PhaseSummary::new(2, "Phase 2", "Done", vec![], None);

        ctx.advance(summary2).await;

        assert_eq!(ctx.current_phase().await, 3);
        assert!(ctx.is_complete().await);
    }

    #[test]
    fn test_extract_summary_with_marker() {
        let response = "Some content\n## Summary\nThis is the summary\n## Next Section";
        let summary = extract_or_create_summary(response, 1, "Test Phase");

        assert!(summary.summary.contains("Summary"));
    }

    #[test]
    fn test_extract_summary_without_marker() {
        let response = "Some content without summary marker";
        let summary = extract_or_create_summary(response, 1, "Test Phase");

        assert!(summary.summary.contains("Completed phase 1"));
        assert!(summary.summary.contains("Test Phase"));
    }
}
