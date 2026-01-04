use std::path::Path;
use tokio::process::Command;
use tracing::{debug, info};

use crate::error::{GitHubError, Result};
use crate::types::{CreatePrRequest, PullRequest, RepoConfig};

/// GitHub CLI wrapper that uses the user's local `gh` authentication
pub struct GhCli {
    #[allow(dead_code)]
    repo: RepoConfig,
    /// Working directory for gh commands (usually the repo root)
    cwd: std::path::PathBuf,
}

impl GhCli {
    pub fn new(repo: RepoConfig, cwd: impl AsRef<Path>) -> Self {
        Self {
            repo,
            cwd: cwd.as_ref().to_path_buf(),
        }
    }

    /// Check if gh CLI is available and authenticated
    pub async fn is_available() -> bool {
        let output = Command::new("gh").args(["auth", "status"]).output().await;

        match output {
            Ok(o) => o.status.success(),
            Err(_) => false,
        }
    }

    /// Create a pull request using gh CLI
    pub async fn create_pull_request(&self, request: CreatePrRequest) -> Result<PullRequest> {
        info!(
            "Creating PR via gh CLI: {} ({} -> {})",
            request.title, request.head, request.base
        );

        let mut args = vec![
            "pr".to_string(),
            "create".to_string(),
            "--title".to_string(),
            request.title.clone(),
            "--body".to_string(),
            request.body.clone(),
            "--base".to_string(),
            request.base.clone(),
            "--head".to_string(),
            request.head.clone(),
        ];

        if request.draft {
            args.push("--draft".to_string());
        }

        let output = Command::new("gh")
            .args(&args)
            .current_dir(&self.cwd)
            .output()
            .await
            .map_err(|e| GitHubError::Api(format!("Failed to run gh CLI: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GitHubError::Api(format!("gh pr create failed: {}", stderr)));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let pr_url = stdout.trim().to_string();

        // Extract PR number from URL (e.g., https://github.com/owner/repo/pull/123)
        let number = pr_url
            .split('/')
            .next_back()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);

        debug!("Created PR #{} at {}", number, pr_url);

        Ok(PullRequest {
            number,
            title: request.title,
            body: Some(request.body),
            state: crate::types::PrState::Open,
            head_branch: request.head,
            base_branch: request.base,
            html_url: pr_url,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            merged_at: None,
            ci_status: None,
        })
    }

    /// Get repository info from gh CLI
    pub async fn get_repo_info(cwd: impl AsRef<Path>) -> Result<RepoConfig> {
        let output = Command::new("gh")
            .args(["repo", "view", "--json", "owner,name"])
            .current_dir(cwd.as_ref())
            .output()
            .await
            .map_err(|e| GitHubError::Api(format!("Failed to run gh CLI: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GitHubError::Api(format!("gh repo view failed: {}", stderr)));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        #[derive(serde::Deserialize)]
        struct RepoInfo {
            owner: Owner,
            name: String,
        }

        #[derive(serde::Deserialize)]
        struct Owner {
            login: String,
        }

        let info: RepoInfo = serde_json::from_str(&stdout)
            .map_err(|e| GitHubError::Api(format!("Failed to parse gh output: {}", e)))?;

        Ok(RepoConfig {
            owner: info.owner.login,
            repo: info.name,
        })
    }

    /// Push branch and create PR in one command
    pub async fn push_and_create_pr(&self, request: CreatePrRequest) -> Result<PullRequest> {
        // First push the branch
        info!("Pushing branch {} via git", request.head);

        let push_output = Command::new("git")
            .args(["push", "-u", "origin", &request.head])
            .current_dir(&self.cwd)
            .output()
            .await
            .map_err(|e| GitHubError::Api(format!("Failed to push: {}", e)))?;

        if !push_output.status.success() {
            let stderr = String::from_utf8_lossy(&push_output.stderr);
            // Ignore "already up to date" type errors
            if !stderr.contains("Everything up-to-date") {
                return Err(GitHubError::Api(format!("git push failed: {}", stderr)));
            }
        }

        // Then create the PR
        self.create_pull_request(request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_gh_cli_availability() {
        // This test just checks the function doesn't panic
        let _available = GhCli::is_available().await;
    }
}
