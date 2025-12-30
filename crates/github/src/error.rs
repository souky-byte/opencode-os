use thiserror::Error;

#[derive(Debug, Error)]
pub enum GitHubError {
    #[error("API error: {0}")]
    Api(String),

    #[error("Authentication failed: {0}")]
    Authentication(String),

    #[error("Repository not found: {owner}/{repo}")]
    RepoNotFound { owner: String, repo: String },

    #[error("Pull request not found: #{number}")]
    PrNotFound { number: u64 },

    #[error("Issue not found: #{number}")]
    IssueNotFound { number: u64 },

    #[error("Rate limit exceeded, resets at {reset_at}")]
    RateLimitExceeded { reset_at: String },

    #[error("Invalid configuration: {0}")]
    Config(String),

    #[error("Network error: {0}")]
    Network(String),
}

impl From<octocrab::Error> for GitHubError {
    fn from(err: octocrab::Error) -> Self {
        match &err {
            octocrab::Error::GitHub { source, .. } => {
                if source.message.contains("rate limit") {
                    GitHubError::RateLimitExceeded {
                        reset_at: "unknown".to_string(),
                    }
                } else {
                    GitHubError::Api(source.message.clone())
                }
            }
            _ => GitHubError::Api(err.to_string()),
        }
    }
}

pub type Result<T> = std::result::Result<T, GitHubError>;
