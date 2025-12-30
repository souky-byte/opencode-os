use db::{SessionRepository, TaskRepository};
use events::EventBus;
use opencode::OpenCodeClient;
use orchestrator::TaskExecutor;
use sqlx::SqlitePool;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use vcs::{GitVcs, JujutsuVcs, VersionControl, WorkspaceConfig, WorkspaceManager};

#[derive(Clone)]
pub struct AppState {
    pub task_repository: TaskRepository,
    pub session_repository: SessionRepository,
    pub task_executor: Arc<TaskExecutor>,
    pub workspace_manager: Arc<WorkspaceManager>,
    pub event_bus: EventBus,
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
        let workspace_manager = Arc::new(WorkspaceManager::new(vcs, config, repo_path));

        Self {
            task_repository: TaskRepository::new(pool.clone()),
            session_repository: SessionRepository::new(pool),
            task_executor: Arc::new(TaskExecutor::new(opencode_client)),
            workspace_manager,
            event_bus: EventBus::new(),
        }
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
