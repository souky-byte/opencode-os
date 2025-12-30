use octocrab::models::IssueState as OctocrabIssueState;
use octocrab::Octocrab;
use tracing::{debug, info};

use crate::error::{GitHubError, Result};
use crate::types::{
    CheckRun, CiState, CiStatus, CreatePrRequest, Issue, IssueState, PrState, PullRequest,
    RepoConfig,
};

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
            Some(PrState::Open) => pulls_handler.list().state(octocrab::params::State::Open).send().await?,
            Some(PrState::Closed) | Some(PrState::Merged) => {
                pulls_handler.list().state(octocrab::params::State::Closed).send().await?
            }
            None => pulls_handler.list().send().await?,
        };

        Ok(prs.items.into_iter().map(|pr| self.convert_pr(pr)).collect())
    }

    pub async fn merge_pull_request(&self, number: u64, commit_message: Option<&str>) -> Result<()> {
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
            html_url: pr
                .html_url
                .map(|u| u.to_string())
                .unwrap_or_default(),
            created_at: pr.created_at.unwrap_or_default(),
            updated_at: pr.updated_at.unwrap_or_default(),
            merged_at: pr.merged_at,
            ci_status: None,
        }
    }
}

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
        info!(
            "Waiting for CI on {} (max {}s)",
            ref_name, max_wait_secs
        );

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
                Some(c) if c.contains("Failure") || c.contains("Cancelled") || c.contains("TimedOut") => {
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
            Some(IssueState::Open) => issues_handler.list().state(octocrab::params::State::Open).send().await?,
            Some(IssueState::Closed) => issues_handler.list().state(octocrab::params::State::Closed).send().await?,
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

        let task = opencode_core::Task::new(
            issue.title.clone(),
            issue.body.clone().unwrap_or_default(),
        );

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
                Some(c) if c.contains("Failure") || c.contains("Cancelled") || c.contains("TimedOut") => {
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
