use thiserror::Error;

/// Wiki-specific error types
#[derive(Debug, Error)]
pub enum WikiError {
    #[error("OpenRouter API error: {message}")]
    OpenRouterApi {
        message: String,
        status_code: Option<u16>,
    },

    #[error("OpenRouter rate limited, retry after {retry_after:?}s")]
    RateLimited { retry_after: Option<u64> },

    #[error("Vector store error: {0}")]
    VectorStore(String),

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Index not found for branch: {branch}")]
    IndexNotFound { branch: String },

    #[error("Wiki page not found: {slug}")]
    PageNotFound { slug: String },

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Indexing failed: {0}")]
    IndexingFailed(String),

    #[error("Generation failed: {0}")]
    GenerationFailed(String),

    #[error("Git error: {0}")]
    GitError(String),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Chunk too large: {size} tokens (max: {max})")]
    ChunkTooLarge { size: usize, max: usize },

    #[error("Embedding dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },
}

/// Result type alias for wiki operations
pub type WikiResult<T> = Result<T, WikiError>;
