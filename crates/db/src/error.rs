use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("Migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error("Task not found: {0}")]
    TaskNotFound(Uuid),

    #[error("Session not found: {0}")]
    SessionNotFound(Uuid),
}
