use octocrab::models::IssueState as OctocrabIssueState;
use octocrab::Octocrab;
use tracing::{debug, info};

use crate::error::{GitHubError, Result};
use crate::types::{
    CheckRun, CiState, CiStatus, CreatePrRequest, CreateReviewCommentRequest, DiffSide, FileStatus,
    GitHubUser, Issue, IssueState, Label, PrFile, PrIssueComment, PrReview, PrReviewComment,
    PrState, PullRequest, PullRequestDetail, Reactions, RepoConfig, ReviewState,
};

#[derive(Clone)]
pub struct GitHubClient {
    octocrab: Octocrab,
    repo: RepoConfig,
}

impl GitHubClient {
    pub fn new(token: &str, repo: RepoConfig) -> Result<Self> {
        let octocrab = Octocrab::builder()
            .personal_token(token.to_string())
            .build()
            .map_err(|e| GitHubError::Config(e.to_string()))?;

        Ok(Self { octocrab, repo })
    }

    pub fn from_env(repo: RepoConfig) -> Result<Self> {
        let token = std::env::var("GITHUB_TOKEN")
            .map_err(|_| GitHubError::Authentication("GITHUB_TOKEN not set".to_string()))?;
        Self::new(&token, repo)
    }

    /// Create a new GitHub client using the provided token or falling back to GITHUB_TOKEN env var
    pub fn from_token_or_env(token: Option<String>, repo: RepoConfig) -> Result<Self> {
        let resolved_token = token
            .filter(|t| !t.trim().is_empty())
            .or_else(|| std::env::var("GITHUB_TOKEN").ok())
            .ok_or_else(|| {
                GitHubError::Authentication(
                    "No GitHub token configured. Set GITHUB_TOKEN environment variable or configure token in Settings.".to_string()
                )
            })?;
        Self::new(&resolved_token, repo)
    }

    pub fn repo(&self) -> &RepoConfig {
        &self.repo
    }
}

impl GitHubClient {
    pub async fn create_pull_request(&self, request: CreatePrRequest) -> Result<PullRequest> {
        info!(
            "Creating PR: {} ({} -> {})",
            request.title, request.head, request.base
        );

        let pr = self
            .octocrab
            .pulls(&self.repo.owner, &self.repo.repo)
            .create(&request.title, &request.head, &request.base)
            .body(&request.body)
            .draft(request.draft)
            .send()
            .await?;

        Ok(self.convert_pr(pr))
    }

    pub async fn get_pull_request(&self, number: u64) -> Result<PullRequest> {
        debug!("Getting PR #{}", number);

        let pr = self
            .octocrab
            .pulls(&self.repo.owner, &self.repo.repo)
            .get(number)
            .await?;

        Ok(self.convert_pr(pr))
    }

    pub async fn list_pull_requests(&self, state: Option<PrState>) -> Result<Vec<PullRequest>> {
        debug!("Listing PRs with state: {:?}", state);

        let pulls_handler = self.octocrab.pulls(&self.repo.owner, &self.repo.repo);

        let prs = match state {
            Some(PrState::Open) => {
                pulls_handler
                    .list()
                    .state(octocrab::params::State::Open)
                    .send()
                    .await?
            }
            Some(PrState::Closed) | Some(PrState::Merged) => {
                pulls_handler
                    .list()
                    .state(octocrab::params::State::Closed)
                    .send()
                    .await?
            }
            None => pulls_handler.list().send().await?,
        };

        Ok(prs
            .items
            .into_iter()
            .map(|pr| self.convert_pr(pr))
            .collect())
    }

    pub async fn merge_pull_request(
        &self,
        number: u64,
        commit_message: Option<&str>,
    ) -> Result<()> {
        info!("Merging PR #{}", number);

        let pulls_handler = self.octocrab.pulls(&self.repo.owner, &self.repo.repo);
        let merge_builder = pulls_handler.merge(number);

        match commit_message {
            Some(msg) => merge_builder.message(msg).send().await?,
            None => merge_builder.send().await?,
        };

        Ok(())
    }

    pub async fn close_pull_request(&self, number: u64) -> Result<()> {
        info!("Closing PR #{}", number);

        self.octocrab
            .pulls(&self.repo.owner, &self.repo.repo)
            .update(number)
            .state(octocrab::params::pulls::State::Closed)
            .send()
            .await?;

        Ok(())
    }

    fn convert_pr(&self, pr: octocrab::models::pulls::PullRequest) -> PullRequest {
        let state = if pr.merged_at.is_some() {
            PrState::Merged
        } else {
            match &pr.state {
                Some(s) => match s {
                    OctocrabIssueState::Closed => PrState::Closed,
                    OctocrabIssueState::Open => PrState::Open,
                    _ => PrState::Open,
                },
                None => PrState::Open,
            }
        };

        PullRequest {
            number: pr.number,
            title: pr.title.unwrap_or_default(),
            body: pr.body,
            state,
            head_branch: pr.head.ref_field,
            base_branch: pr.base.ref_field,
            html_url: pr.html_url.map(|u| u.to_string()).unwrap_or_default(),
            created_at: pr.created_at.unwrap_or_default(),
            updated_at: pr.updated_at.unwrap_or_default(),
            merged_at: pr.merged_at,
            ci_status: None,
        }
    }

    fn convert_pr_detail(&self, pr: octocrab::models::pulls::PullRequest) -> PullRequestDetail {
        let state = if pr.merged_at.is_some() {
            PrState::Merged
        } else {
            match &pr.state {
                Some(s) => match s {
                    OctocrabIssueState::Closed => PrState::Closed,
                    OctocrabIssueState::Open => PrState::Open,
                    _ => PrState::Open,
                },
                None => PrState::Open,
            }
        };

        let user = pr
            .user
            .as_ref()
            .map(|u| GitHubUser {
                login: u.login.clone(),
                avatar_url: u.avatar_url.to_string(),
                html_url: u.html_url.to_string(),
            })
            .unwrap_or_else(|| GitHubUser {
                login: "unknown".to_string(),
                avatar_url: String::new(),
                html_url: String::new(),
            });

        let labels = pr
            .labels
            .as_ref()
            .map(|labels| {
                labels
                    .iter()
                    .map(|l| Label {
                        name: l.name.clone(),
                        color: l.color.clone(),
                        description: l.description.clone(),
                    })
                    .collect()
            })
            .unwrap_or_default();

        let requested_reviewers = pr
            .requested_reviewers
            .as_ref()
            .map(|reviewers| {
                reviewers
                    .iter()
                    .map(|u| GitHubUser {
                        login: u.login.clone(),
                        avatar_url: u.avatar_url.to_string(),
                        html_url: u.html_url.to_string(),
                    })
                    .collect()
            })
            .unwrap_or_default();

        PullRequestDetail {
            number: pr.number,
            title: pr.title.clone().unwrap_or_default(),
            body: pr.body.clone(),
            state,
            head_branch: pr.head.ref_field.clone(),
            base_branch: pr.base.ref_field.clone(),
            html_url: pr
                .html_url
                .as_ref()
                .map(|u| u.to_string())
                .unwrap_or_default(),
            created_at: pr.created_at.unwrap_or_default(),
            updated_at: pr.updated_at.unwrap_or_default(),
            merged_at: pr.merged_at,
            ci_status: None,
            user,
            additions: pr.additions.unwrap_or(0) as u32,
            deletions: pr.deletions.unwrap_or(0) as u32,
            changed_files: pr.changed_files.unwrap_or(0) as u32,
            mergeable: pr.mergeable,
            mergeable_state: pr.mergeable_state.as_ref().map(|s| format!("{:?}", s)),
            labels,
            requested_reviewers,
            draft: pr.draft.unwrap_or(false),
            comments_count: pr.comments.unwrap_or(0) as u32,
            review_comments_count: pr.review_comments.unwrap_or(0) as u32,
        }
    }
}

// =============================================================================
// Pull Request Detail & Extended Info
// =============================================================================

impl GitHubClient {
    /// Get detailed information about a PR including additions, deletions, labels, reviewers
    pub async fn get_pull_request_detail(&self, number: u64) -> Result<PullRequestDetail> {
        debug!("Getting PR detail #{}", number);

        let pr = self
            .octocrab
            .pulls(&self.repo.owner, &self.repo.repo)
            .get(number)
            .await?;

        Ok(self.convert_pr_detail(pr))
    }

    /// List pull requests with detailed information
    pub async fn list_pull_requests_detail(
        &self,
        state: Option<PrState>,
    ) -> Result<Vec<PullRequestDetail>> {
        debug!("Listing PRs detail with state: {:?}", state);

        let pulls_handler = self.octocrab.pulls(&self.repo.owner, &self.repo.repo);

        let prs = match state {
            Some(PrState::Open) => {
                pulls_handler
                    .list()
                    .state(octocrab::params::State::Open)
                    .send()
                    .await?
            }
            Some(PrState::Closed) | Some(PrState::Merged) => {
                pulls_handler
                    .list()
                    .state(octocrab::params::State::Closed)
                    .send()
                    .await?
            }
            None => pulls_handler.list().send().await?,
        };

        Ok(prs
            .items
            .into_iter()
            .map(|pr| self.convert_pr_detail(pr))
            .collect())
    }

    /// Get PR diff as raw text
    pub async fn get_pr_diff(&self, number: u64) -> Result<String> {
        debug!("Getting PR diff #{}", number);

        // Use gh CLI to get diff since octocrab doesn't support raw diff format easily
        let output = tokio::process::Command::new("gh")
            .args([
                "pr",
                "diff",
                &number.to_string(),
                "--repo",
                &format!("{}/{}", self.repo.owner, self.repo.repo),
            ])
            .output()
            .await
            .map_err(|e| GitHubError::Api(format!("Failed to run gh command: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GitHubError::Api(format!("gh pr diff failed: {}", stderr)));
        }

        let diff = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(diff)
    }

    /// Get list of files changed in a PR
    pub async fn get_pr_files(&self, number: u64) -> Result<Vec<PrFile>> {
        debug!("Getting PR files #{}", number);

        let url = format!(
            "https://api.github.com/repos/{}/{}/pulls/{}/files",
            self.repo.owner, self.repo.repo, number
        );

        let response: Vec<serde_json::Value> = self
            .octocrab
            .get(&url, None::<&()>)
            .await
            .map_err(|e| GitHubError::Api(e.to_string()))?;

        let files = response
            .into_iter()
            .map(|f| {
                let status_str = f["status"].as_str().unwrap_or("modified");
                let status = match status_str {
                    "added" => FileStatus::Added,
                    "removed" => FileStatus::Removed,
                    "modified" => FileStatus::Modified,
                    "renamed" => FileStatus::Renamed,
                    "copied" => FileStatus::Copied,
                    "changed" => FileStatus::Changed,
                    _ => FileStatus::Modified,
                };

                PrFile {
                    filename: f["filename"].as_str().unwrap_or("").to_string(),
                    status,
                    additions: f["additions"].as_u64().unwrap_or(0) as u32,
                    deletions: f["deletions"].as_u64().unwrap_or(0) as u32,
                    changes: f["changes"].as_u64().unwrap_or(0) as u32,
                    patch: f["patch"].as_str().map(|s| s.to_string()),
                    previous_filename: f["previous_filename"].as_str().map(|s| s.to_string()),
                }
            })
            .collect();

        Ok(files)
    }
}

// =============================================================================
// Pull Request Review Comments (Line Comments)
// =============================================================================

impl GitHubClient {
    /// Get review comments (comments on specific lines of code)
    pub async fn get_pr_review_comments(&self, number: u64) -> Result<Vec<PrReviewComment>> {
        debug!("Getting PR review comments #{}", number);

        let url = format!(
            "https://api.github.com/repos/{}/{}/pulls/{}/comments",
            self.repo.owner, self.repo.repo, number
        );

        let response: Vec<serde_json::Value> = self
            .octocrab
            .get(&url, None::<&()>)
            .await
            .map_err(|e| GitHubError::Api(e.to_string()))?;

        let comments = response
            .into_iter()
            .map(|c| self.convert_review_comment(&c))
            .collect();

        Ok(comments)
    }

    /// Get issue comments (general discussion comments on PR)
    pub async fn get_pr_issue_comments(&self, number: u64) -> Result<Vec<PrIssueComment>> {
        debug!("Getting PR issue comments #{}", number);

        let url = format!(
            "https://api.github.com/repos/{}/{}/issues/{}/comments",
            self.repo.owner, self.repo.repo, number
        );

        let response: Vec<serde_json::Value> = self
            .octocrab
            .get(&url, None::<&()>)
            .await
            .map_err(|e| GitHubError::Api(e.to_string()))?;

        let comments = response
            .into_iter()
            .map(|c| self.convert_issue_comment(&c))
            .collect();

        Ok(comments)
    }

    /// Create a review comment on a specific line
    pub async fn create_review_comment(
        &self,
        number: u64,
        request: CreateReviewCommentRequest,
    ) -> Result<PrReviewComment> {
        info!(
            "Creating review comment on PR #{} at {}:{}",
            number, request.path, request.line
        );

        let url = format!(
            "https://api.github.com/repos/{}/{}/pulls/{}/comments",
            self.repo.owner, self.repo.repo, number
        );

        let body = serde_json::json!({
            "body": request.body,
            "commit_id": request.commit_id,
            "path": request.path,
            "line": request.line,
            "side": request.side.as_str(),
        });

        let response: serde_json::Value = self
            .octocrab
            .post(&url, Some(&body))
            .await
            .map_err(|e| GitHubError::Api(e.to_string()))?;

        Ok(self.convert_review_comment(&response))
    }

    /// Reply to a review comment thread
    pub async fn reply_to_review_comment(
        &self,
        number: u64,
        comment_id: u64,
        body: &str,
    ) -> Result<PrReviewComment> {
        info!(
            "Replying to review comment {} on PR #{}",
            comment_id, number
        );

        let url = format!(
            "https://api.github.com/repos/{}/{}/pulls/{}/comments/{}/replies",
            self.repo.owner, self.repo.repo, number, comment_id
        );

        let request_body = serde_json::json!({
            "body": body,
        });

        let response: serde_json::Value = self
            .octocrab
            .post(&url, Some(&request_body))
            .await
            .map_err(|e| GitHubError::Api(e.to_string()))?;

        Ok(self.convert_review_comment(&response))
    }

    fn convert_review_comment(&self, c: &serde_json::Value) -> PrReviewComment {
        let user = c["user"]
            .as_object()
            .map(|u| GitHubUser {
                login: u["login"].as_str().unwrap_or("").to_string(),
                avatar_url: u["avatar_url"].as_str().unwrap_or("").to_string(),
                html_url: u["html_url"].as_str().unwrap_or("").to_string(),
            })
            .unwrap_or_else(|| GitHubUser {
                login: "unknown".to_string(),
                avatar_url: String::new(),
                html_url: String::new(),
            });

        let side = match c["side"].as_str().unwrap_or("RIGHT") {
            "LEFT" => DiffSide::Left,
            _ => DiffSide::Right,
        };

        let reactions = c["reactions"].as_object().map(|r| Reactions {
            total_count: r["total_count"].as_u64().unwrap_or(0) as u32,
            plus_one: r["+1"].as_u64().unwrap_or(0) as u32,
            minus_one: r["-1"].as_u64().unwrap_or(0) as u32,
            laugh: r["laugh"].as_u64().unwrap_or(0) as u32,
            hooray: r["hooray"].as_u64().unwrap_or(0) as u32,
            confused: r["confused"].as_u64().unwrap_or(0) as u32,
            heart: r["heart"].as_u64().unwrap_or(0) as u32,
            rocket: r["rocket"].as_u64().unwrap_or(0) as u32,
            eyes: r["eyes"].as_u64().unwrap_or(0) as u32,
        });

        PrReviewComment {
            id: c["id"].as_u64().unwrap_or(0),
            body: c["body"].as_str().unwrap_or("").to_string(),
            path: c["path"].as_str().unwrap_or("").to_string(),
            line: c["line"].as_u64().map(|l| l as u32),
            original_line: c["original_line"].as_u64().map(|l| l as u32),
            side,
            commit_id: c["commit_id"].as_str().unwrap_or("").to_string(),
            user,
            created_at: c["created_at"]
                .as_str()
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(chrono::Utc::now),
            updated_at: c["updated_at"]
                .as_str()
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(chrono::Utc::now),
            html_url: c["html_url"].as_str().unwrap_or("").to_string(),
            in_reply_to_id: c["in_reply_to_id"].as_u64(),
            reactions,
        }
    }

    fn convert_issue_comment(&self, c: &serde_json::Value) -> PrIssueComment {
        let user = c["user"]
            .as_object()
            .map(|u| GitHubUser {
                login: u["login"].as_str().unwrap_or("").to_string(),
                avatar_url: u["avatar_url"].as_str().unwrap_or("").to_string(),
                html_url: u["html_url"].as_str().unwrap_or("").to_string(),
            })
            .unwrap_or_else(|| GitHubUser {
                login: "unknown".to_string(),
                avatar_url: String::new(),
                html_url: String::new(),
            });

        let reactions = c["reactions"].as_object().map(|r| Reactions {
            total_count: r["total_count"].as_u64().unwrap_or(0) as u32,
            plus_one: r["+1"].as_u64().unwrap_or(0) as u32,
            minus_one: r["-1"].as_u64().unwrap_or(0) as u32,
            laugh: r["laugh"].as_u64().unwrap_or(0) as u32,
            hooray: r["hooray"].as_u64().unwrap_or(0) as u32,
            confused: r["confused"].as_u64().unwrap_or(0) as u32,
            heart: r["heart"].as_u64().unwrap_or(0) as u32,
            rocket: r["rocket"].as_u64().unwrap_or(0) as u32,
            eyes: r["eyes"].as_u64().unwrap_or(0) as u32,
        });

        PrIssueComment {
            id: c["id"].as_u64().unwrap_or(0),
            body: c["body"].as_str().unwrap_or("").to_string(),
            user,
            created_at: c["created_at"]
                .as_str()
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(chrono::Utc::now),
            updated_at: c["updated_at"]
                .as_str()
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(chrono::Utc::now),
            html_url: c["html_url"].as_str().unwrap_or("").to_string(),
            reactions,
        }
    }
}

// =============================================================================
// Pull Request Reviews
// =============================================================================

impl GitHubClient {
    /// Get reviews for a PR
    pub async fn get_pr_reviews(&self, number: u64) -> Result<Vec<PrReview>> {
        debug!("Getting PR reviews #{}", number);

        let url = format!(
            "https://api.github.com/repos/{}/{}/pulls/{}/reviews",
            self.repo.owner, self.repo.repo, number
        );

        let response: Vec<serde_json::Value> = self
            .octocrab
            .get(&url, None::<&()>)
            .await
            .map_err(|e| GitHubError::Api(e.to_string()))?;

        let reviews = response
            .into_iter()
            .map(|r| {
                let user = r["user"]
                    .as_object()
                    .map(|u| GitHubUser {
                        login: u["login"].as_str().unwrap_or("").to_string(),
                        avatar_url: u["avatar_url"].as_str().unwrap_or("").to_string(),
                        html_url: u["html_url"].as_str().unwrap_or("").to_string(),
                    })
                    .unwrap_or_else(|| GitHubUser {
                        login: "unknown".to_string(),
                        avatar_url: String::new(),
                        html_url: String::new(),
                    });

                let state = match r["state"].as_str().unwrap_or("COMMENTED") {
                    "APPROVED" => ReviewState::Approved,
                    "CHANGES_REQUESTED" => ReviewState::ChangesRequested,
                    "COMMENTED" => ReviewState::Commented,
                    "PENDING" => ReviewState::Pending,
                    "DISMISSED" => ReviewState::Dismissed,
                    _ => ReviewState::Commented,
                };

                PrReview {
                    id: r["id"].as_u64().unwrap_or(0),
                    user,
                    state,
                    body: r["body"]
                        .as_str()
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string()),
                    submitted_at: r["submitted_at"]
                        .as_str()
                        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                        .map(|dt| dt.with_timezone(&chrono::Utc)),
                    html_url: r["html_url"].as_str().unwrap_or("").to_string(),
                }
            })
            .collect();

        Ok(reviews)
    }
}

// =============================================================================
// CI Status
// =============================================================================

impl GitHubClient {
    pub async fn get_ci_status(&self, ref_name: &str) -> Result<CiStatus> {
        debug!("Getting CI status for ref: {}", ref_name);

        let checks = self
            .octocrab
            .checks(&self.repo.owner, &self.repo.repo)
            .list_check_runs_for_git_ref(ref_name.to_string().into())
            .send()
            .await?;

        let check_runs: Vec<CheckRun> = checks
            .check_runs
            .into_iter()
            .map(|cr| CheckRun {
                name: cr.name,
                status: "unknown".to_string(),
                conclusion: cr.conclusion.map(|c| format!("{:?}", c)),
                html_url: cr.html_url.map(|u| u.to_string()),
            })
            .collect();

        let state = self.compute_aggregate_state(&check_runs);

        Ok(CiStatus {
            state,
            total_count: check_runs.len() as u32,
            checks: check_runs,
        })
    }

    pub async fn get_pr_ci_status(&self, pr_number: u64) -> Result<CiStatus> {
        let pr = self.get_pull_request(pr_number).await?;
        self.get_ci_status(&pr.head_branch).await
    }

    pub async fn wait_for_ci(
        &self,
        ref_name: &str,
        poll_interval_secs: u64,
        max_wait_secs: u64,
    ) -> Result<CiStatus> {
        info!("Waiting for CI on {} (max {}s)", ref_name, max_wait_secs);

        let start = std::time::Instant::now();
        let poll_duration = std::time::Duration::from_secs(poll_interval_secs);
        let max_duration = std::time::Duration::from_secs(max_wait_secs);

        loop {
            let status = self.get_ci_status(ref_name).await?;

            match status.state {
                CiState::Success | CiState::Failure | CiState::Error => {
                    info!("CI completed with state: {:?}", status.state);
                    return Ok(status);
                }
                CiState::Pending => {
                    if start.elapsed() > max_duration {
                        info!("CI wait timeout after {:?}", start.elapsed());
                        return Ok(status);
                    }
                    debug!("CI still pending, waiting {}s...", poll_interval_secs);
                    tokio::time::sleep(poll_duration).await;
                }
            }
        }
    }

    fn compute_aggregate_state(&self, checks: &[CheckRun]) -> CiState {
        if checks.is_empty() {
            return CiState::Pending;
        }

        let mut has_pending = false;
        let mut has_failure = false;

        for check in checks {
            match check.conclusion.as_deref() {
                None => has_pending = true,
                Some(c) if c.contains("Success") => {}
                Some(c) if c.contains("Skipped") || c.contains("Neutral") => {}
                Some(c)
                    if c.contains("Failure")
                        || c.contains("Cancelled")
                        || c.contains("TimedOut") =>
                {
                    has_failure = true
                }
                Some(c) if c.contains("ActionRequired") => has_pending = true,
                _ => {}
            }
        }

        if has_failure {
            CiState::Failure
        } else if has_pending {
            CiState::Pending
        } else {
            CiState::Success
        }
    }
}

impl GitHubClient {
    pub async fn get_issue(&self, number: u64) -> Result<Issue> {
        debug!("Getting issue #{}", number);

        let issue = self
            .octocrab
            .issues(&self.repo.owner, &self.repo.repo)
            .get(number)
            .await?;

        Ok(self.convert_issue(issue))
    }

    pub async fn list_issues(&self, state: Option<IssueState>) -> Result<Vec<Issue>> {
        debug!("Listing issues with state: {:?}", state);

        let issues_handler = self.octocrab.issues(&self.repo.owner, &self.repo.repo);

        let issues = match state {
            Some(IssueState::Open) => {
                issues_handler
                    .list()
                    .state(octocrab::params::State::Open)
                    .send()
                    .await?
            }
            Some(IssueState::Closed) => {
                issues_handler
                    .list()
                    .state(octocrab::params::State::Closed)
                    .send()
                    .await?
            }
            None => issues_handler.list().send().await?,
        };

        Ok(issues
            .items
            .into_iter()
            .filter(|i| i.pull_request.is_none())
            .map(|i| self.convert_issue(i))
            .collect())
    }

    pub async fn import_issue(&self, number: u64) -> Result<opencode_core::Task> {
        let issue = self.get_issue(number).await?;

        let task =
            opencode_core::Task::new(issue.title.clone(), issue.body.clone().unwrap_or_default());

        info!("Imported issue #{} as task {}", number, task.id);
        Ok(task)
    }

    fn convert_issue(&self, issue: octocrab::models::issues::Issue) -> Issue {
        let state = match issue.state {
            OctocrabIssueState::Closed => IssueState::Closed,
            OctocrabIssueState::Open => IssueState::Open,
            _ => IssueState::Open,
        };

        Issue {
            number: issue.number,
            title: issue.title,
            body: issue.body,
            state,
            labels: issue.labels.into_iter().map(|l| l.name).collect(),
            html_url: issue.html_url.to_string(),
            created_at: issue.created_at,
            updated_at: issue.updated_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repo_config_from_full_name() {
        let config = RepoConfig::from_full_name("owner/repo").unwrap();
        assert_eq!(config.owner, "owner");
        assert_eq!(config.repo, "repo");
    }

    #[test]
    fn test_repo_config_from_invalid_name() {
        assert!(RepoConfig::from_full_name("invalid").is_none());
        assert!(RepoConfig::from_full_name("a/b/c").is_none());
    }

    #[test]
    fn test_repo_config_from_git_url_ssh() {
        let config = RepoConfig::from_git_url("git@github.com:owner/repo.git").unwrap();
        assert_eq!(config.owner, "owner");
        assert_eq!(config.repo, "repo");
    }

    #[test]
    fn test_repo_config_from_git_url_https() {
        let config = RepoConfig::from_git_url("https://github.com/owner/repo.git").unwrap();
        assert_eq!(config.owner, "owner");
        assert_eq!(config.repo, "repo");
    }

    #[test]
    fn test_repo_config_from_git_url_https_no_suffix() {
        let config = RepoConfig::from_git_url("https://github.com/owner/repo").unwrap();
        assert_eq!(config.owner, "owner");
        assert_eq!(config.repo, "repo");
    }

    #[test]
    fn test_repo_config_from_git_url_invalid() {
        assert!(RepoConfig::from_git_url("https://gitlab.com/owner/repo").is_none());
        assert!(RepoConfig::from_git_url("invalid").is_none());
    }

    #[test]
    fn test_create_pr_request_builder() {
        let req = CreatePrRequest::new("Title", "feature", "main")
            .with_body("Description")
            .as_draft();

        assert_eq!(req.title, "Title");
        assert_eq!(req.head, "feature");
        assert_eq!(req.base, "main");
        assert_eq!(req.body, "Description");
        assert!(req.draft);
    }

    #[test]
    fn test_compute_aggregate_state_empty() {
        let state = compute_aggregate_state_helper(&[]);
        assert_eq!(state, CiState::Pending);
    }

    #[test]
    fn test_compute_aggregate_state_all_success() {
        let checks = vec![
            CheckRun {
                name: "test".to_string(),
                status: "Completed".to_string(),
                conclusion: Some("Success".to_string()),
                html_url: None,
            },
            CheckRun {
                name: "lint".to_string(),
                status: "Completed".to_string(),
                conclusion: Some("Success".to_string()),
                html_url: None,
            },
        ];
        let state = compute_aggregate_state_helper(&checks);
        assert_eq!(state, CiState::Success);
    }

    #[test]
    fn test_compute_aggregate_state_with_failure() {
        let checks = vec![
            CheckRun {
                name: "test".to_string(),
                status: "Completed".to_string(),
                conclusion: Some("Success".to_string()),
                html_url: None,
            },
            CheckRun {
                name: "lint".to_string(),
                status: "Completed".to_string(),
                conclusion: Some("Failure".to_string()),
                html_url: None,
            },
        ];
        let state = compute_aggregate_state_helper(&checks);
        assert_eq!(state, CiState::Failure);
    }

    #[test]
    fn test_compute_aggregate_state_with_pending() {
        let checks = vec![
            CheckRun {
                name: "test".to_string(),
                status: "Completed".to_string(),
                conclusion: Some("Success".to_string()),
                html_url: None,
            },
            CheckRun {
                name: "lint".to_string(),
                status: "InProgress".to_string(),
                conclusion: None,
                html_url: None,
            },
        ];
        let state = compute_aggregate_state_helper(&checks);
        assert_eq!(state, CiState::Pending);
    }

    fn compute_aggregate_state_helper(checks: &[CheckRun]) -> CiState {
        if checks.is_empty() {
            return CiState::Pending;
        }

        let mut has_pending = false;
        let mut has_failure = false;

        for check in checks {
            match check.conclusion.as_deref() {
                None => has_pending = true,
                Some(c) if c.contains("Success") => {}
                Some(c) if c.contains("Skipped") || c.contains("Neutral") => {}
                Some(c)
                    if c.contains("Failure")
                        || c.contains("Cancelled")
                        || c.contains("TimedOut") =>
                {
                    has_failure = true
                }
                Some(c) if c.contains("ActionRequired") => has_pending = true,
                _ => {}
            }
        }

        if has_failure {
            CiState::Failure
        } else if has_pending {
            CiState::Pending
        } else {
            CiState::Success
        }
    }
}
