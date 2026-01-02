use thiserror::Error;

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
}

pub type Result<T> = std::result::Result<T, OrchestratorError>;
