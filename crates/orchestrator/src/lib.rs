pub mod activity_store;
pub mod error;
pub mod executor;
pub mod files;
pub mod mcp_config;
pub mod opencode_events;
pub mod prompts;
pub mod state_machine;

pub use activity_store::{SessionActivityMsg, SessionActivityRegistry, SessionActivityStore};
pub use error::{OrchestratorError, Result};
pub use executor::{ExecutorConfig, PhaseResult, ReviewResult, StartedExecution, TaskExecutor};
pub use files::{
    FileManager, FindingSeverity, FindingStatus, ReviewFinding, ReviewFindings,
};
pub use mcp_config::{expand_env_vars, McpBinarySource, McpServerSpec, PhaseMcpConfig};
pub use opencode_events::{ExecutorEvent, OpenCodeEventSubscriber, SessionStatus as OpenCodeSessionStatus};
pub use state_machine::TaskStateMachine;
