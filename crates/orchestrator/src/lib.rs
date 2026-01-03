pub mod activity_store;
pub mod error;
pub mod executor;
pub mod files;
pub mod mcp_config;
pub mod opencode_events;
pub mod plan_parser;
pub mod prompts;
pub mod services;
pub mod session_runner;
pub mod state_machine;

pub use activity_store::{SessionActivityMsg, SessionActivityRegistry, SessionActivityStore};
pub use error::{OrchestratorError, Result};
pub use executor::{ExecutorConfig, PhaseResult, ReviewResult, StartedExecution, TaskExecutor};
pub use files::{
    FileManager, FindingSeverity, FindingStatus, ParsedPlan, PhaseContext, PhaseSummary, PlanPhase,
    ReviewFinding, ReviewFindings,
};
pub use mcp_config::{expand_env_vars, McpBinarySource, McpServerSpec, PhaseMcpConfig};
pub use opencode_events::{
    ExecutorEvent, OpenCodeEventSubscriber, SessionStatus as OpenCodeSessionStatus,
};
pub use plan_parser::{extract_phase_summary, parse_plan_phases, ExtractedSummary};
pub use prompts::UserReviewComment;
pub use services::{McpManager, MessageParser, OpenCodeClient};
pub use session_runner::{
    McpConfig, SessionConfig, SessionDependencies, SessionResult, SessionRunner,
};
pub use state_machine::TaskStateMachine;
