use db::{SessionRepository, TaskRepository};
use events::EventBus;
use github::{GitHubClient, RepoConfig};
use opencode::OpenCodeClient;
use orchestrator::{ExecutorConfig, TaskExecutor};
use sqlx::SqlitePool;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::OnceCell;
use vcs::{GitVcs, JujutsuVcs, VersionControl, WorkspaceConfig, WorkspaceManager};

#[derive(Clone)]
pub struct AppState {
    pub task_repository: TaskRepository,
    pub session_repository: SessionRepository,
    pub task_executor: Arc<TaskExecutor>,
    pub workspace_manager: Arc<WorkspaceManager>,
    pub event_bus: EventBus,
    #[allow(dead_code)]
    pub repo_path: PathBuf,
    #[allow(dead_code)]
    github_client: Arc<OnceCell<GitHubClient>>,
}

impl AppState {
    pub fn new(pool: SqlitePool, opencode_url: &str) -> Self {
        let opencode_client = Arc::new(OpenCodeClient::new(opencode_url));

        let repo_path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let workspace_base = repo_path
            .parent()
            .map(|p| p.join(".workspaces"))
            .unwrap_or_else(|| PathBuf::from("../.workspaces"));

        let vcs = Self::detect_vcs(&repo_path, &workspace_base);
        let config = WorkspaceConfig::new(workspace_base.clone());
        let workspace_manager = Arc::new(WorkspaceManager::new(vcs, config, repo_path.clone()));

        let session_repository = SessionRepository::new(pool.clone());
        let event_bus = EventBus::new();

        let config = ExecutorConfig::new(&repo_path)
            .with_plan_approval(true)
            .with_human_review(true)
            .with_max_iterations(3);

        let task_executor = TaskExecutor::new(opencode_client, config)
            .with_workspace_manager(workspace_manager.clone())
            .with_session_repo(Arc::new(session_repository.clone()))
            .with_event_bus(event_bus.clone());

        Self {
            task_repository: TaskRepository::new(pool),
            session_repository,
            task_executor: Arc::new(task_executor),
            workspace_manager,
            event_bus,
            repo_path,
            github_client: Arc::new(OnceCell::new()),
        }
    }

    #[allow(dead_code)]
    pub async fn github_client(&self) -> Result<&GitHubClient, github::GitHubError> {
        self.github_client
            .get_or_try_init(|| async {
                let repo_config = RepoConfig::from_git_remote(&self.repo_path)
                    .await
                    .ok_or_else(|| {
                        github::GitHubError::Config(
                            "Could not detect GitHub repository from git remote".to_string(),
                        )
                    })?;

                tracing::info!(
                    "Detected GitHub repository: {}/{}",
                    repo_config.owner,
                    repo_config.repo
                );

                GitHubClient::from_env(repo_config)
            })
            .await
    }

    fn detect_vcs(repo_path: &Path, workspace_base: &Path) -> Arc<dyn VersionControl> {
        if repo_path.join(".jj").exists() {
            tracing::info!("Detected Jujutsu repository");
            Arc::new(JujutsuVcs::new(
                repo_path.to_path_buf(),
                workspace_base.to_path_buf(),
            ))
        } else {
            tracing::info!("Using Git as VCS backend");
            Arc::new(GitVcs::new(
                repo_path.to_path_buf(),
                workspace_base.to_path_buf(),
            ))
        }
    }
}
