pub mod client;
pub mod error;
pub mod types;

pub use client::GitHubClient;
pub use error::{GitHubError, Result};
pub use types::{
    CheckRun, CiState, CiStatus, CreatePrRequest, Issue, IssueState, PrState, PullRequest,
    RepoConfig,
};
