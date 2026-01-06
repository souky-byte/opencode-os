use crate::project_manager::{GlobalConfigManager, ProjectContext, ProjectError, ProjectManager};
use crate::routes::sse::{EventBuffer, SharedEventBuffer, DEFAULT_EVENT_BUFFER_SIZE};
use events::EventBus;
use github::{GitHubClient, RepoConfig};
use opencode_core::RoadmapGenerationStatus;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, RwLock};
use tokio::sync::RwLock as TokioRwLock;

pub type SharedRoadmapStatus = Arc<TokioRwLock<RoadmapGenerationStatus>>;
pub type GenerationId = Arc<AtomicU64>;

#[derive(Clone)]
pub struct AppState {
    pub project_manager: Arc<ProjectManager>,
    pub global_config: GlobalConfigManager,
    pub event_bus: EventBus,
    pub event_buffer: SharedEventBuffer,
    pub opencode_url: String,
    pub app_dir: Option<PathBuf>,
    /// Cached GitHub client - token hash is stored to detect when token changes
    github_client: Arc<RwLock<Option<(String, GitHubClient)>>>,
    pub roadmap_status: SharedRoadmapStatus,
    /// Current roadmap generation ID - incremented on each new generation to invalidate old tasks
    pub roadmap_generation_id: GenerationId,
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
            github_client: Arc::new(RwLock::new(None)),
            roadmap_status: Arc::new(TokioRwLock::new(RoadmapGenerationStatus::default())),
            roadmap_generation_id: Arc::new(AtomicU64::new(0)),
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

    pub async fn github_client(&self) -> Result<GitHubClient, github::GitHubError> {
        let project = self
            .project()
            .await
            .map_err(|e| github::GitHubError::Config(format!("No project open: {}", e)))?;

        // Get repo config first - we need it for cache key
        let repo_config = RepoConfig::from_git_remote(&project.path)
            .await
            .ok_or_else(|| {
                github::GitHubError::Config(
                    "Could not detect GitHub repository from git remote".to_string(),
                )
            })?;

        // Get token from global config (Settings UI) or env
        let config_token = self.global_config.get_github_token();
        let env_token = std::env::var("GITHUB_TOKEN").ok();
        let current_token = config_token.clone().or(env_token).unwrap_or_default();

        // Cache key includes both token and repo to handle project switching
        let cache_key = format!(
            "{}::{}/{}",
            current_token, repo_config.owner, repo_config.repo
        );

        // Check if we have a cached client with the same token AND repo
        {
            let cache = self.github_client.read().unwrap();
            if let Some((cached_key, client)) = cache.as_ref() {
                if cached_key == &cache_key {
                    return Ok(client.clone());
                }
            }
        }

        tracing::info!(
            "Creating GitHub client for {}/{} (token source: {})",
            repo_config.owner,
            repo_config.repo,
            if config_token.is_some() {
                "settings"
            } else if std::env::var("GITHUB_TOKEN").is_ok() {
                "env"
            } else {
                "none"
            }
        );

        // Use token from config if available, otherwise fall back to GITHUB_TOKEN env var
        let client = GitHubClient::from_token_or_env(config_token, repo_config)?;

        // Cache the client with combined key
        {
            let mut cache = self.github_client.write().unwrap();
            *cache = Some((cache_key, client.clone()));
        }

        Ok(client)
    }
}
