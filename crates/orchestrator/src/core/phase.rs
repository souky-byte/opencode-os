//! Phase trait and related types for unified phase execution.
//!
//! This module defines the core abstraction for all execution phases (Planning,
//! Implementation, Review, Fix). Each phase implements the `Phase` trait which
//! provides a consistent interface for the execution engine.

use async_trait::async_trait;
use opencode_core::{SessionPhase, Task, TaskStatus};
use std::path::PathBuf;
use uuid::Uuid;

use crate::error::Result;
use crate::services::ExecutorContext;

/// Output from a completed session execution.
#[derive(Debug, Clone)]
pub struct SessionOutput {
    /// Local session ID
    pub session_id: Uuid,
    /// OpenCode server session ID
    pub opencode_session_id: String,
    /// Full response text from the AI
    pub response_text: String,
    /// Whether the session completed successfully
    pub success: bool,
    /// Error message if session failed
    pub error: Option<String>,
}

/// Configuration for a phase execution session.
#[derive(Debug, Clone)]
pub struct PhaseConfig {
    /// The prompt to send to OpenCode
    pub prompt: String,
    /// Working directory for the session
    pub working_dir: PathBuf,
    /// MCP servers to connect for this phase
    pub mcp_servers: Vec<McpServerSpec>,
    /// Whether to skip task status update after completion
    pub skip_status_update: bool,
    /// Phase-specific metadata
    pub metadata: PhaseMetadata,
}

/// Specification for an MCP server to connect.
#[derive(Debug, Clone)]
pub struct McpServerSpec {
    /// Server name/identifier
    pub name: String,
    /// Server type
    pub server_type: McpServerType,
}

impl McpServerSpec {
    /// Create a findings MCP server spec.
    pub fn findings() -> Self {
        Self {
            name: "opencode-findings".to_string(),
            server_type: McpServerType::Findings,
        }
    }
}

/// Types of MCP servers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum McpServerType {
    /// Findings server for review/fix phases
    Findings,
}

/// Phase-specific metadata.
#[derive(Debug, Clone)]
pub enum PhaseMetadata {
    /// Planning phase metadata
    Planning,
    /// Implementation phase metadata
    Implementation {
        /// Current phase number (for multi-phase)
        phase_number: Option<u32>,
        /// Total phases (for multi-phase)
        total_phases: Option<u32>,
    },
    /// Review phase metadata
    Review {
        /// Current iteration number
        iteration: u32,
    },
    /// Fix phase metadata
    Fix,
}

impl PhaseMetadata {
    /// Get the session phase type.
    pub fn phase_type(&self) -> SessionPhase {
        match self {
            PhaseMetadata::Planning => SessionPhase::Planning,
            PhaseMetadata::Implementation { .. } => SessionPhase::Implementation,
            PhaseMetadata::Review { .. } => SessionPhase::Review,
            PhaseMetadata::Fix => SessionPhase::Fix,
        }
    }
}

/// Outcome of phase processing after session completion.
#[derive(Debug, Clone)]
pub enum PhaseOutcome {
    /// Phase completed, transition to next status
    Transition {
        /// The next task status to transition to
        next_status: TaskStatus,
    },
    /// Phase requires human approval before continuing
    AwaitingApproval {
        /// Which phase is awaiting approval
        phase: SessionPhase,
    },
    /// Phase needs another iteration (review/fix cycle)
    Iterate {
        /// Feedback for the next iteration
        feedback: String,
        /// Current iteration number
        iteration: u32,
    },
    /// Continue to next phase in multi-phase execution
    Continue,
    /// Task is fully complete
    Complete,
}

/// Resource requirements for a phase.
#[derive(Debug, Clone, Default)]
pub struct ResourceRequirements {
    /// Whether this phase needs a workspace (VCS branch)
    pub needs_workspace: bool,
    /// Whether this phase needs MCP findings server
    pub needs_mcp_findings: bool,
    /// Whether this phase needs workspace diff
    pub needs_diff: bool,
}

/// Core trait that all execution phases must implement.
///
/// Each phase (Planning, Implementation, Review, Fix) implements this trait
/// to provide a consistent interface for the execution engine. The engine
/// handles resource acquisition, session execution, and cleanup while the
/// phase implementations focus on their specific logic.
#[async_trait]
pub trait Phase: Send + Sync {
    /// Get the phase type identifier.
    fn phase_type(&self) -> SessionPhase;

    /// Build the session configuration for this phase.
    ///
    /// This method is called before session execution to prepare the prompt,
    /// working directory, and any other configuration needed.
    async fn build_config(&self, ctx: &ExecutorContext, task: &Task) -> Result<PhaseConfig>;

    /// Process the result after session completion.
    ///
    /// This method is called after the session has completed (successfully or not)
    /// to handle the output, save artifacts, and determine the next action.
    async fn process_result(
        &self,
        ctx: &ExecutorContext,
        task: &mut Task,
        result: &SessionOutput,
    ) -> Result<PhaseOutcome>;

    /// Get the resource requirements for this phase.
    ///
    /// The execution engine uses this to acquire necessary resources before
    /// session execution and ensure proper cleanup afterwards.
    fn required_resources(&self) -> ResourceRequirements {
        ResourceRequirements::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phase_metadata_type() {
        assert_eq!(PhaseMetadata::Planning.phase_type(), SessionPhase::Planning);
        assert_eq!(
            PhaseMetadata::Implementation {
                phase_number: None,
                total_phases: None
            }
            .phase_type(),
            SessionPhase::Implementation
        );
        assert_eq!(
            PhaseMetadata::Review { iteration: 0 }.phase_type(),
            SessionPhase::Review
        );
        assert_eq!(PhaseMetadata::Fix.phase_type(), SessionPhase::Fix);
    }

    #[test]
    fn test_mcp_server_spec_findings() {
        let spec = McpServerSpec::findings();
        assert_eq!(spec.name, "opencode-findings");
        assert_eq!(spec.server_type, McpServerType::Findings);
    }

    #[test]
    fn test_resource_requirements_default() {
        let req = ResourceRequirements::default();
        assert!(!req.needs_workspace);
        assert!(!req.needs_mcp_findings);
        assert!(!req.needs_diff);
    }
}
