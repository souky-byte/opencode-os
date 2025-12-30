use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    pub number: u64,
    pub title: String,
    pub body: Option<String>,
    pub state: PrState,
    pub head_branch: String,
    pub base_branch: String,
    pub html_url: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub merged_at: Option<DateTime<Utc>>,
    pub ci_status: Option<CiStatus>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PrState {
    Open,
    Closed,
    Merged,
}

impl PrState {
    pub fn as_str(&self) -> &'static str {
        match self {
            PrState::Open => "open",
            PrState::Closed => "closed",
            PrState::Merged => "merged",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub number: u64,
    pub title: String,
    pub body: Option<String>,
    pub state: IssueState,
    pub labels: Vec<String>,
    pub html_url: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IssueState {
    Open,
    Closed,
}

impl IssueState {
    pub fn as_str(&self) -> &'static str {
        match self {
            IssueState::Open => "open",
            IssueState::Closed => "closed",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CiStatus {
    pub state: CiState,
    pub total_count: u32,
    pub checks: Vec<CheckRun>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CiState {
    Pending,
    Success,
    Failure,
    Error,
}

impl CiState {
    pub fn as_str(&self) -> &'static str {
        match self {
            CiState::Pending => "pending",
            CiState::Success => "success",
            CiState::Failure => "failure",
            CiState::Error => "error",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckRun {
    pub name: String,
    pub status: String,
    pub conclusion: Option<String>,
    pub html_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CreatePrRequest {
    pub title: String,
    pub body: String,
    pub head: String,
    pub base: String,
    pub draft: bool,
}

impl CreatePrRequest {
    pub fn new(title: impl Into<String>, head: impl Into<String>, base: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            body: String::new(),
            head: head.into(),
            base: base.into(),
            draft: false,
        }
    }

    pub fn with_body(mut self, body: impl Into<String>) -> Self {
        self.body = body.into();
        self
    }

    pub fn as_draft(mut self) -> Self {
        self.draft = true;
        self
    }
}

#[derive(Debug, Clone)]
pub struct RepoConfig {
    pub owner: String,
    pub repo: String,
}

impl RepoConfig {
    pub fn new(owner: impl Into<String>, repo: impl Into<String>) -> Self {
        Self {
            owner: owner.into(),
            repo: repo.into(),
        }
    }

    pub fn from_full_name(full_name: &str) -> Option<Self> {
        let parts: Vec<&str> = full_name.split('/').collect();
        if parts.len() == 2 {
            Some(Self::new(parts[0], parts[1]))
        } else {
            None
        }
    }

    pub fn from_git_url(url: &str) -> Option<Self> {
        let url = url.trim();

        if let Some(rest) = url.strip_prefix("git@github.com:") {
            let repo_path = rest.trim_end_matches(".git");
            return Self::from_full_name(repo_path);
        }

        if url.contains("github.com") {
            let url = url.trim_end_matches(".git");
            let parts: Vec<&str> = url.split('/').collect();
            if parts.len() >= 2 {
                let repo = parts[parts.len() - 1];
                let owner = parts[parts.len() - 2];
                return Some(Self::new(owner, repo));
            }
        }

        None
    }

    pub async fn from_git_remote(repo_path: &std::path::Path) -> Option<Self> {
        let output = tokio::process::Command::new("git")
            .args(["remote", "get-url", "origin"])
            .current_dir(repo_path)
            .output()
            .await
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let url = String::from_utf8_lossy(&output.stdout);
        Self::from_git_url(&url)
    }
}
