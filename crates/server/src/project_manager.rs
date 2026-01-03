//! Project lifecycle management for multi-project support.
//!
//! Handles opening, initializing, and switching between projects at runtime.

use db::{SessionActivityRepository, SessionRepository, TaskRepository};
use events::EventBus;
use opencode_client::apis::configuration::Configuration as OpenCodeConfig;
use orchestrator::{ExecutorConfig, ModelSelection, PhaseModels, SessionActivityRegistry, TaskExecutor};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use vcs::{GitVcs, JujutsuVcs, VersionControl, WorkspaceConfig, WorkspaceManager};

use sha2::{Digest, Sha256};

use crate::config::ProjectConfig as JsonProjectConfig;

const STUDIO_DIR: &str = ".opencode-studio";
const GLOBAL_STUDIO_DIR: &str = ".opencode-studio";
const PROJECT_CONFIG_FILE: &str = "config.toml";
const KANBAN_DIR: &str = "kanban";
const PLANS_DIR: &str = "plans";
const REVIEWS_DIR: &str = "reviews";

/// Get the global data directory for a project based on path hash.
/// This allows storing DB files outside the project directory.
pub fn get_project_data_dir(project_path: &Path) -> Result<PathBuf, ProjectError> {
    let canonical = project_path.canonicalize()?;
    let hash = {
        let mut hasher = Sha256::new();
        hasher.update(canonical.to_string_lossy().as_bytes());
        let result = hasher.finalize();
        hex::encode(&result[..8]) // 16 hex chars for uniqueness
    };

    let global_dir = dirs::home_dir()
        .ok_or_else(|| ProjectError::Config("Could not determine home directory".into()))?
        .join(GLOBAL_STUDIO_DIR)
        .join("data")
        .join(&hash);

    Ok(global_dir)
}

/// Get the database path for a project (in global data directory).
pub fn get_db_path(project_path: &Path) -> Result<PathBuf, ProjectError> {
    let data_dir = get_project_data_dir(project_path)?;
    Ok(data_dir.join("studio.db"))
}

/// Migrate database from old per-project location to new global location.
pub async fn migrate_db_if_needed(project_path: &Path) -> Result<(), ProjectError> {
    let old_db = project_path.join(STUDIO_DIR).join("studio.db");
    let new_db = get_db_path(project_path)?;

    if old_db.exists() && !new_db.exists() {
        // Create target directory
        if let Some(parent) = new_db.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // Move the database file
        tokio::fs::rename(&old_db, &new_db).await?;

        // Move WAL files if they exist
        let old_wal = old_db.with_extension("db-wal");
        let old_shm = old_db.with_extension("db-shm");
        if old_wal.exists() {
            let _ = tokio::fs::rename(&old_wal, new_db.with_extension("db-wal")).await;
        }
        if old_shm.exists() {
            let _ = tokio::fs::rename(&old_shm, new_db.with_extension("db-shm")).await;
        }

        tracing::info!(
            old = %old_db.display(),
            new = %new_db.display(),
            "Migrated database to global location"
        );
    }

    Ok(())
}

/// Errors that can occur during project operations.
#[derive(Debug, thiserror::Error)]
pub enum ProjectError {
    #[error("Path not found: {0}")]
    PathNotFound(PathBuf),

    #[error("Path is not a directory: {0}")]
    NotDirectory(PathBuf),

    #[error("Not a git or jujutsu repository: {0}")]
    NotVcsRepo(PathBuf),

    #[error("Failed to initialize project: {0}")]
    InitFailed(String),

    #[error("Database connection failed: {0}")]
    DbConnectFailed(#[from] sqlx::Error),

    #[error("Database migration failed: {0}")]
    MigrationFailed(#[from] sqlx::migrate::MigrateError),

    #[error("No project is currently open")]
    NoProjectOpen,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Config error: {0}")]
    Config(String),
}

/// Error code for API responses.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProjectErrorCode {
    PathNotFound,
    NotDirectory,
    NotVcsRepo,
    InitFailed,
    DbConnectFailed,
    NoProjectOpen,
    IoError,
    ConfigError,
}

impl From<&ProjectError> for ProjectErrorCode {
    fn from(err: &ProjectError) -> Self {
        match err {
            ProjectError::PathNotFound(_) => Self::PathNotFound,
            ProjectError::NotDirectory(_) => Self::NotDirectory,
            ProjectError::NotVcsRepo(_) => Self::NotVcsRepo,
            ProjectError::InitFailed(_) => Self::InitFailed,
            ProjectError::DbConnectFailed(_) => Self::DbConnectFailed,
            ProjectError::MigrationFailed(_) => Self::DbConnectFailed,
            ProjectError::NoProjectOpen => Self::NoProjectOpen,
            ProjectError::Io(_) => Self::IoError,
            ProjectError::Config(_) => Self::ConfigError,
        }
    }
}

/// Project-specific configuration stored in .opencode-studio/config.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    #[serde(default)]
    pub name: Option<String>,

    #[serde(default = "default_plan_approval")]
    pub require_plan_approval: bool,

    #[serde(default = "default_human_review")]
    pub require_human_review: bool,

    #[serde(default = "default_max_iterations")]
    pub max_iterations: u32,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            name: None,
            require_plan_approval: true,
            require_human_review: true,
            max_iterations: 3,
        }
    }
}

fn default_plan_approval() -> bool {
    true
}

fn default_human_review() -> bool {
    true
}

fn default_max_iterations() -> u32 {
    3
}

/// Information about a project for API responses.
#[derive(Debug, Clone, Serialize)]
pub struct ProjectInfo {
    pub name: String,
    pub path: String,
    pub vcs: String,
    pub tasks_count: i64,
    pub initialized: bool,
}

/// Result of opening a project.
#[derive(Debug, Clone, Serialize)]
pub struct OpenProjectResult {
    pub project: ProjectInfo,
    /// True if we just created .opencode-studio/ structure
    pub was_initialized: bool,
}

/// Result of initializing a project.
#[derive(Debug, Clone, Serialize)]
pub struct InitProjectResult {
    pub project: ProjectInfo,
    pub already_initialized: bool,
}

/// Per-project context holding all project-specific resources.
///
/// This is swapped out when switching between projects.
#[derive(Clone)]
pub struct ProjectContext {
    pub path: PathBuf,
    pub project_path: PathBuf,
    pub pool: SqlitePool,
    pub task_repository: TaskRepository,
    pub session_repository: SessionRepository,
    pub task_executor: Arc<TaskExecutor>,
    pub workspace_manager: Arc<WorkspaceManager>,
    pub activity_registry: SessionActivityRegistry,
    pub config: ProjectConfig,
}

impl ProjectContext {
    /// Create a new project context from a path.
    ///
    /// This connects to the project's database and sets up all resources.
    pub async fn new(
        path: PathBuf,
        opencode_url: &str,
        event_bus: EventBus,
    ) -> Result<Self, ProjectError> {
        if !path.exists() {
            return Err(ProjectError::PathNotFound(path));
        }
        if !path.is_dir() {
            return Err(ProjectError::NotDirectory(path));
        }

        let vcs_type = detect_vcs(&path);
        if vcs_type == "none" {
            return Err(ProjectError::NotVcsRepo(path));
        }

        let studio_dir = path.join(STUDIO_DIR);
        let config = load_project_config(&studio_dir);

        // Migrate database from old per-project location if needed
        migrate_db_if_needed(&path).await?;

        // Database is now stored in global location (~/.opencode-studio/data/{hash}/)
        let db_path = get_db_path(&path)?;
        if let Some(parent) = db_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        let database_url = format!("sqlite:{}", db_path.display());
        let pool = db::create_pool(&database_url).await?;
        db::run_migrations(&pool).await?;

        let workspace_base = path
            .parent()
            .map(|p| p.join(".workspaces"))
            .unwrap_or_else(|| PathBuf::from("../.workspaces"));

        let vcs = detect_vcs_impl(&path, &workspace_base);
        let ws_config = WorkspaceConfig::new(workspace_base.clone());
        let workspace_manager = Arc::new(WorkspaceManager::new(vcs, ws_config, path.clone()));

        let session_repository = SessionRepository::new(pool.clone());
        let task_repository = TaskRepository::new(pool.clone());
        let activity_repository = SessionActivityRepository::new(pool.clone());

        let activity_registry = SessionActivityRegistry::new().with_repository(activity_repository);

        let mut opencode_config = OpenCodeConfig::new();
        opencode_config.base_path = opencode_url.to_string();
        let opencode_config = Arc::new(opencode_config);

        let executor_config = ExecutorConfig::new(&path)
            .with_plan_approval(config.require_plan_approval)
            .with_human_review(config.require_human_review)
            .with_max_iterations(config.max_iterations)
            .with_phase_models(convert_phase_models(&path).await);

        let task_executor = TaskExecutor::new(opencode_config, executor_config)
            .with_workspace_manager(workspace_manager.clone())
            .with_session_repo(Arc::new(session_repository.clone()))
            .with_task_repo(Arc::new(task_repository.clone()))
            .with_event_bus(event_bus)
            .with_activity_registry(activity_registry.clone());

        Ok(Self {
            path: path.clone(),
            project_path: path,
            pool,
            task_repository,
            session_repository,
            task_executor: Arc::new(task_executor),
            workspace_manager,
            activity_registry,
            config,
        })
    }

    /// Get project info for API responses.
    pub async fn info(&self) -> ProjectInfo {
        let name = self
            .config
            .name
            .clone()
            .or_else(|| {
                self.path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(String::from)
            })
            .unwrap_or_else(|| "unknown".to_string());

        let tasks_count = self
            .task_repository
            .find_all()
            .await
            .map(|t| t.len() as i64)
            .unwrap_or(0);

        ProjectInfo {
            name,
            path: self.path.display().to_string(),
            vcs: detect_vcs(&self.path).to_string(),
            tasks_count,
            initialized: true,
        }
    }
}

/// Manages project lifecycle - opening, switching, closing projects.
pub struct ProjectManager {
    context: Arc<RwLock<Option<ProjectContext>>>,
    opencode_url: String,
    event_bus: EventBus,
}

impl ProjectManager {
    /// Create a new project manager.
    pub fn new(opencode_url: String, event_bus: EventBus) -> Self {
        Self {
            context: Arc::new(RwLock::new(None)),
            opencode_url,
            event_bus,
        }
    }

    pub async fn open(&self, path: &Path) -> Result<OpenProjectResult, ProjectError> {
        if !path.exists() {
            return Err(ProjectError::PathNotFound(path.to_path_buf()));
        }
        if !path.is_dir() {
            return Err(ProjectError::NotDirectory(path.to_path_buf()));
        }

        let vcs_type = detect_vcs(path);
        if vcs_type == "none" {
            return Err(ProjectError::NotVcsRepo(path.to_path_buf()));
        }

        let studio_dir = path.join(STUDIO_DIR);
        let was_initialized = if !studio_dir.exists() {
            init_project_structure(path)?;
            true
        } else {
            false
        };

        self.close().await?;

        let ctx = ProjectContext::new(
            path.to_path_buf(),
            &self.opencode_url,
            self.event_bus.clone(),
        )
        .await?;

        let project_info = ctx.info().await;

        let mut guard = self.context.write().await;
        *guard = Some(ctx);

        self.event_bus
            .publish(events::EventEnvelope::new(events::Event::ProjectOpened {
                path: project_info.path.clone(),
                name: project_info.name.clone(),
                was_initialized,
            }));

        Ok(OpenProjectResult {
            project: project_info,
            was_initialized,
        })
    }

    /// Initialize a project without switching to it.
    pub async fn init(&self, path: &Path, force: bool) -> Result<InitProjectResult, ProjectError> {
        if !path.exists() {
            return Err(ProjectError::PathNotFound(path.to_path_buf()));
        }
        if !path.is_dir() {
            return Err(ProjectError::NotDirectory(path.to_path_buf()));
        }

        let vcs_type = detect_vcs(path);
        if vcs_type == "none" {
            return Err(ProjectError::NotVcsRepo(path.to_path_buf()));
        }

        let studio_dir = path.join(STUDIO_DIR);
        let already_initialized = studio_dir.exists();

        if !already_initialized || force {
            init_project_structure(path)?;
        }

        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        Ok(InitProjectResult {
            project: ProjectInfo {
                name,
                path: path.display().to_string(),
                vcs: vcs_type.to_string(),
                tasks_count: 0,
                initialized: true,
            },
            already_initialized,
        })
    }

    /// Get the current project context.
    pub async fn current(&self) -> Option<ProjectContext> {
        let guard = self.context.read().await;
        guard.clone()
    }

    pub async fn close(&self) -> Result<(), ProjectError> {
        let mut guard = self.context.write().await;
        if let Some(ctx) = guard.take() {
            let path = ctx.path.display().to_string();
            ctx.pool.close().await;
            self.event_bus
                .publish(events::EventEnvelope::new(events::Event::ProjectClosed {
                    path,
                }));
        }
        Ok(())
    }

    /// Check if a project is currently open.
    pub async fn is_open(&self) -> bool {
        let guard = self.context.read().await;
        guard.is_some()
    }
}

/// Initialize the .opencode-studio/ directory structure.
fn init_project_structure(path: &Path) -> Result<(), ProjectError> {
    let studio_dir = path.join(STUDIO_DIR);

    std::fs::create_dir_all(&studio_dir)?;
    std::fs::create_dir_all(studio_dir.join(KANBAN_DIR).join(PLANS_DIR))?;
    std::fs::create_dir_all(studio_dir.join(KANBAN_DIR).join(REVIEWS_DIR))?;

    let config_path = studio_dir.join(PROJECT_CONFIG_FILE);
    if !config_path.exists() {
        let config = ProjectConfig::default();
        let toml_str =
            toml::to_string_pretty(&config).map_err(|e| ProjectError::Config(e.to_string()))?;
        std::fs::write(&config_path, toml_str)?;
    }

    Ok(())
}

/// Load project configuration from .opencode-studio/config.toml
fn load_project_config(studio_dir: &Path) -> ProjectConfig {
    let config_path = studio_dir.join(PROJECT_CONFIG_FILE);

    if !config_path.exists() {
        return ProjectConfig::default();
    }

    std::fs::read_to_string(&config_path)
        .ok()
        .and_then(|content| toml::from_str(&content).ok())
        .unwrap_or_default()
}

/// Detect VCS type from a path.
pub fn detect_vcs(path: &Path) -> &'static str {
    if path.join(".jj").exists() {
        "jujutsu"
    } else if path.join(".git").exists() {
        "git"
    } else {
        "none"
    }
}

async fn convert_phase_models(project_path: &Path) -> PhaseModels {
    let json_config = JsonProjectConfig::read(project_path).await;

    let convert_model = |m: Option<crate::config::ModelSelection>| -> Option<ModelSelection> {
        m.map(|s| ModelSelection::new(s.provider_id, s.model_id))
    };

    PhaseModels {
        planning: convert_model(json_config.phase_models.planning),
        implementation: convert_model(json_config.phase_models.implementation),
        review: convert_model(json_config.phase_models.review),
        fix: convert_model(json_config.phase_models.fix),
    }
}

fn detect_vcs_impl(repo_path: &Path, workspace_base: &Path) -> Arc<dyn VersionControl> {
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

const GLOBAL_CONFIG_FILE: &str = "global.toml";
const MAX_RECENT_PROJECTS: usize = 10;

/// Global configuration stored in ~/.opencode-studio/global.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    #[serde(default)]
    pub recent_projects: Vec<String>,

    #[serde(default)]
    pub last_project: Option<String>,

    #[serde(default = "default_auto_open_last")]
    pub auto_open_last: bool,

    #[serde(default = "default_max_recent")]
    pub max_recent: usize,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            recent_projects: Vec::new(),
            last_project: None,
            auto_open_last: true,
            max_recent: MAX_RECENT_PROJECTS,
        }
    }
}

fn default_auto_open_last() -> bool {
    true
}

fn default_max_recent() -> usize {
    MAX_RECENT_PROJECTS
}

/// Manages the global ~/.opencode-studio/global.toml configuration.
#[derive(Clone)]
pub struct GlobalConfigManager {
    config_dir: PathBuf,
}

impl GlobalConfigManager {
    pub fn new() -> Self {
        let config_dir = dirs::home_dir()
            .map(|h| h.join(GLOBAL_STUDIO_DIR))
            .unwrap_or_else(|| PathBuf::from(GLOBAL_STUDIO_DIR));

        Self { config_dir }
    }

    #[cfg(test)]
    pub fn with_config_dir(config_dir: PathBuf) -> Self {
        Self { config_dir }
    }

    fn config_path(&self) -> PathBuf {
        self.config_dir.join(GLOBAL_CONFIG_FILE)
    }

    fn ensure_config_dir(&self) -> Result<(), ProjectError> {
        if !self.config_dir.exists() {
            std::fs::create_dir_all(&self.config_dir)?;
        }
        Ok(())
    }

    pub fn load(&self) -> GlobalConfig {
        let path = self.config_path();
        if !path.exists() {
            return GlobalConfig::default();
        }

        std::fs::read_to_string(&path)
            .ok()
            .and_then(|content| toml::from_str(&content).ok())
            .unwrap_or_default()
    }

    fn save(&self, config: &GlobalConfig) -> Result<(), ProjectError> {
        self.ensure_config_dir()?;

        let toml_str =
            toml::to_string_pretty(config).map_err(|e| ProjectError::Config(e.to_string()))?;

        let config_path = self.config_path();
        let temp_path = config_path.with_extension("toml.tmp");

        std::fs::write(&temp_path, toml_str)?;
        std::fs::rename(&temp_path, &config_path)?;

        Ok(())
    }

    pub fn add_recent(&self, path: &Path) -> Result<(), ProjectError> {
        let mut config = self.load();
        let path_str = path.display().to_string();

        config.recent_projects.retain(|p| p != &path_str);
        config.recent_projects.insert(0, path_str);

        let max = config.max_recent.max(1);
        config.recent_projects.truncate(max);

        self.save(&config)
    }

    pub fn get_recent(&self) -> Vec<PathBuf> {
        self.load()
            .recent_projects
            .into_iter()
            .map(PathBuf::from)
            .collect()
    }

    pub fn remove_recent(&self, path: &Path) -> Result<(), ProjectError> {
        let mut config = self.load();
        let path_str = path.display().to_string();
        config.recent_projects.retain(|p| p != &path_str);
        self.save(&config)
    }

    pub fn clear_recent(&self) -> Result<(), ProjectError> {
        let mut config = self.load();
        config.recent_projects.clear();
        config.last_project = None;
        self.save(&config)
    }

    pub fn set_last(&self, path: &Path) -> Result<(), ProjectError> {
        let mut config = self.load();
        config.last_project = Some(path.display().to_string());
        self.save(&config)
    }

    pub fn get_last(&self) -> Option<PathBuf> {
        self.load().last_project.map(PathBuf::from)
    }

    pub fn should_auto_open_last(&self) -> bool {
        self.load().auto_open_last
    }
}

impl Default for GlobalConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_git_repo(dir: &Path) {
        std::fs::create_dir_all(dir.join(".git")).unwrap();
    }

    #[test]
    fn test_detect_vcs_git() {
        let tmp = TempDir::new().unwrap();
        create_git_repo(tmp.path());
        assert_eq!(detect_vcs(tmp.path()), "git");
    }

    #[test]
    fn test_detect_vcs_jujutsu() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join(".jj")).unwrap();
        assert_eq!(detect_vcs(tmp.path()), "jujutsu");
    }

    #[test]
    fn test_detect_vcs_none() {
        let tmp = TempDir::new().unwrap();
        assert_eq!(detect_vcs(tmp.path()), "none");
    }

    #[test]
    fn test_init_project_structure() {
        let tmp = TempDir::new().unwrap();
        create_git_repo(tmp.path());

        init_project_structure(tmp.path()).unwrap();

        let studio_dir = tmp.path().join(STUDIO_DIR);
        assert!(studio_dir.exists());
        assert!(studio_dir.join(PROJECT_CONFIG_FILE).exists());
        assert!(studio_dir.join(KANBAN_DIR).join(PLANS_DIR).exists());
        assert!(studio_dir.join(KANBAN_DIR).join(REVIEWS_DIR).exists());
    }

    #[test]
    fn test_load_project_config_default() {
        let tmp = TempDir::new().unwrap();
        let config = load_project_config(tmp.path());

        assert!(config.require_plan_approval);
        assert!(config.require_human_review);
        assert_eq!(config.max_iterations, 3);
    }

    #[test]
    fn test_project_error_codes() {
        let err = ProjectError::PathNotFound(PathBuf::from("/test"));
        assert!(matches!(
            ProjectErrorCode::from(&err),
            ProjectErrorCode::PathNotFound
        ));

        let err = ProjectError::NotVcsRepo(PathBuf::from("/test"));
        assert!(matches!(
            ProjectErrorCode::from(&err),
            ProjectErrorCode::NotVcsRepo
        ));

        let err = ProjectError::NoProjectOpen;
        assert!(matches!(
            ProjectErrorCode::from(&err),
            ProjectErrorCode::NoProjectOpen
        ));
    }

    #[test]
    fn test_global_config_manager_add_recent() {
        let tmp = TempDir::new().unwrap();
        let manager = GlobalConfigManager::with_config_dir(tmp.path().to_path_buf());

        manager.add_recent(Path::new("/project/a")).unwrap();
        manager.add_recent(Path::new("/project/b")).unwrap();
        manager.add_recent(Path::new("/project/c")).unwrap();

        let recent = manager.get_recent();
        assert_eq!(recent.len(), 3);
        assert_eq!(recent[0], PathBuf::from("/project/c"));
        assert_eq!(recent[1], PathBuf::from("/project/b"));
        assert_eq!(recent[2], PathBuf::from("/project/a"));
    }

    #[test]
    fn test_global_config_manager_deduplicates() {
        let tmp = TempDir::new().unwrap();
        let manager = GlobalConfigManager::with_config_dir(tmp.path().to_path_buf());

        manager.add_recent(Path::new("/project/a")).unwrap();
        manager.add_recent(Path::new("/project/b")).unwrap();
        manager.add_recent(Path::new("/project/a")).unwrap();

        let recent = manager.get_recent();
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0], PathBuf::from("/project/a"));
        assert_eq!(recent[1], PathBuf::from("/project/b"));
    }

    #[test]
    fn test_global_config_manager_last_project() {
        let tmp = TempDir::new().unwrap();
        let manager = GlobalConfigManager::with_config_dir(tmp.path().to_path_buf());

        assert!(manager.get_last().is_none());

        manager.set_last(Path::new("/project/x")).unwrap();
        assert_eq!(manager.get_last(), Some(PathBuf::from("/project/x")));
    }

    #[test]
    fn test_global_config_manager_truncates_recent() {
        let tmp = TempDir::new().unwrap();
        let manager = GlobalConfigManager::with_config_dir(tmp.path().to_path_buf());

        for i in 0..15 {
            manager
                .add_recent(Path::new(&format!("/project/{}", i)))
                .unwrap();
        }

        let recent = manager.get_recent();
        assert_eq!(recent.len(), MAX_RECENT_PROJECTS);
        assert_eq!(recent[0], PathBuf::from("/project/14"));
    }
}
