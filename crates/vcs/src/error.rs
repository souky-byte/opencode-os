use thiserror::Error;

#[derive(Debug, Error)]
pub enum VcsError {
    #[error("Command execution failed: {0}")]
    CommandFailed(String),

    #[error("Command not found: {0}")]
    CommandNotFound(String),

    #[error("Workspace not found: {0}")]
    WorkspaceNotFound(String),

    #[error("Workspace already exists: {0}")]
    WorkspaceAlreadyExists(String),

    #[error("Invalid workspace path: {0}")]
    InvalidPath(String),

    #[error("VCS not initialized in repository: {0}")]
    NotInitialized(String),

    #[error("Merge conflict: {0}")]
    MergeConflict(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(String),
}

pub type Result<T> = std::result::Result<T, VcsError>;
