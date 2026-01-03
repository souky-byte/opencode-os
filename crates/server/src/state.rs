use crate::project_manager::{GlobalConfigManager, ProjectContext, ProjectError, ProjectManager};
use crate::routes::sse::{EventBuffer, SharedEventBuffer, DEFAULT_EVENT_BUFFER_SIZE};
use events::EventBus;
use github::{GitHubClient, RepoConfig};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use tokio::sync::OnceCell;

#[derive(Clone)]
pub struct AppState {
    pub project_manager: Arc<ProjectManager>,
    pub global_config: GlobalConfigManager,
    pub event_bus: EventBus,
    pub event_buffer: SharedEventBuffer,
    pub opencode_url: String,
    pub app_dir: Option<PathBuf>,
    github_client: Arc<OnceCell<GitHubClient>>,
}

impl AppState {
    pub fn new(opencode_url: &str) -> Self {
        let event_bus = EventBus::new();
        let event_buffer = Arc::new(RwLock::new(EventBuffer::new(DEFAULT_EVENT_BUFFER_SIZE)));
        let global_config = GlobalConfigManager::new();
        let project_manager = Arc::new(ProjectManager::new(
            opencode_url.to_string(),
            event_bus.clone(),
        ));

        Self {
            project_manager,
            global_config,
            event_bus,
            event_buffer,
            opencode_url: opencode_url.to_string(),
            app_dir: None,
            github_client: Arc::new(OnceCell::new()),
        }
    }

    pub fn with_app_dir(mut self, app_dir: PathBuf) -> Self {
        self.app_dir = Some(app_dir);
        self
    }

    pub async fn project(&self) -> Result<ProjectContext, ProjectError> {
        self.project_manager
            .current()
            .await
            .ok_or(ProjectError::NoProjectOpen)
    }

    pub async fn open_project(&self, path: &Path) -> Result<(), ProjectError> {
        let result = self.project_manager.open(path).await?;

        self.global_config.add_recent(path)?;
        self.global_config.set_last(path)?;

        tracing::info!(
            "Opened project: {} (initialized: {})",
            result.project.name,
            result.was_initialized
        );

        Ok(())
    }

    pub async fn auto_open_last_project(&self) -> Result<bool, ProjectError> {
        if !self.global_config.should_auto_open_last() {
            return Ok(false);
        }

        if let Some(last_path) = self.global_config.get_last() {
            if last_path.exists() && last_path.join(".git").exists()
                || last_path.join(".jj").exists()
            {
                self.open_project(&last_path).await?;
                return Ok(true);
            }
        }

        Ok(false)
    }

    #[allow(dead_code)]
    pub async fn github_client(&self) -> Result<&GitHubClient, github::GitHubError> {
        let project = self
            .project()
            .await
            .map_err(|e| github::GitHubError::Config(format!("No project open: {}", e)))?;

        self.github_client
            .get_or_try_init(|| async {
                let repo_config = RepoConfig::from_git_remote(&project.path)
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
}
