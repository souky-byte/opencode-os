use opencode_core::SessionPhase;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum OrchestratorError {
    #[error("Invalid state transition from {from} to {to}")]
    InvalidTransition { from: String, to: String },

    #[error("Task not found: {0}")]
    TaskNotFound(String),

    #[error("OpenCode error: {0}")]
    OpenCodeError(String),

    #[error("Database error: {0}")]
    Database(#[from] db::DbError),

    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Session already exists for task: {0}")]
    SessionExists(String),

    // New contextual error types for clean architecture
    #[error("Workspace required but not configured for task: {0}")]
    WorkspaceRequired(Uuid),

    #[error("MCP server connection failed: {server} - {reason}")]
    McpConnectionFailed { server: String, reason: String },

    #[error("Phase {phase:?} failed at iteration {iteration}: {reason}")]
    PhaseExecutionFailed {
        phase: SessionPhase,
        iteration: u32,
        reason: String,
    },

    #[error("Resource acquisition failed: {0}")]
    ResourceAcquisitionFailed(String),

    #[error("Session timeout after {duration_ms}ms")]
    SessionTimeout { duration_ms: u64 },

    #[error("Plan not found for task: {0}")]
    PlanNotFound(Uuid),

    #[error("Findings not found for task: {0}")]
    FindingsNotFound(Uuid),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Serialization error: {0}")]
    Serialization(String),
}

impl OrchestratorError {
    /// Create an MCP connection failed error.
    pub fn mcp_failed(server: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::McpConnectionFailed {
            server: server.into(),
            reason: reason.into(),
        }
    }

    /// Create a phase execution failed error.
    pub fn phase_failed(phase: SessionPhase, iteration: u32, reason: impl Into<String>) -> Self {
        Self::PhaseExecutionFailed {
            phase,
            iteration,
            reason: reason.into(),
        }
    }
}

pub type Result<T> = std::result::Result<T, OrchestratorError>;

/// Execution context for better error reporting.
///
/// This provides additional context about where an error occurred,
/// useful for debugging and logging.
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// Task ID being executed
    pub task_id: Uuid,
    /// Session ID (if created)
    pub session_id: Option<Uuid>,
    /// Current phase
    pub phase: SessionPhase,
    /// Current iteration (for review/fix cycles)
    pub iteration: u32,
    /// Duration of execution so far
    pub duration_ms: u64,
}

impl ExecutionContext {
    /// Create a new execution context.
    pub fn new(task_id: Uuid, phase: SessionPhase) -> Self {
        Self {
            task_id,
            session_id: None,
            phase,
            iteration: 0,
            duration_ms: 0,
        }
    }

    /// Set the session ID.
    pub fn with_session(mut self, session_id: Uuid) -> Self {
        self.session_id = Some(session_id);
        self
    }

    /// Set the iteration number.
    pub fn with_iteration(mut self, iteration: u32) -> Self {
        self.iteration = iteration;
        self
    }

    /// Set the duration.
    pub fn with_duration(mut self, duration_ms: u64) -> Self {
        self.duration_ms = duration_ms;
        self
    }
}

/// Result type with execution context for better error reporting.
#[derive(Debug)]
pub struct ContextualResult<T> {
    /// The actual result
    pub result: Result<T>,
    /// Execution context
    pub context: ExecutionContext,
}

impl<T> ContextualResult<T> {
    /// Create a successful result with context.
    pub fn ok(value: T, context: ExecutionContext) -> Self {
        Self {
            result: Ok(value),
            context,
        }
    }

    /// Create an error result with context.
    pub fn err(error: OrchestratorError, context: ExecutionContext) -> Self {
        Self {
            result: Err(error),
            context,
        }
    }

    /// Check if the result is successful.
    pub fn is_ok(&self) -> bool {
        self.result.is_ok()
    }

    /// Check if the result is an error.
    pub fn is_err(&self) -> bool {
        self.result.is_err()
    }

    /// Unwrap the result, panicking on error.
    pub fn unwrap(self) -> T {
        self.result.unwrap()
    }

    /// Convert to standard Result, discarding context.
    pub fn into_result(self) -> Result<T> {
        self.result
    }
}
