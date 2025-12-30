use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("Task not found: {0}")]
    TaskNotFound(Uuid),

    #[error("Session not found: {0}")]
    SessionNotFound(Uuid),

    #[error("Invalid task status transition from {from} to {to}")]
    InvalidStatusTransition { from: String, to: String },

    #[error("Validation error: {0}")]
    Validation(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let id = Uuid::new_v4();
        let error = CoreError::TaskNotFound(id);
        assert!(error.to_string().contains(&id.to_string()));
    }
}
