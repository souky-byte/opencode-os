use thiserror::Error;

#[derive(Debug, Error)]
pub enum OpenCodeError {
    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("JSON serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Connection failed: {0}")]
    Connection(String),

    #[error("Event stream error: {0}")]
    EventStream(String),
}

pub type Result<T> = std::result::Result<T, OpenCodeError>;
