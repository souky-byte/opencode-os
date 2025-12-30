//! File management for plans and reviews
//!
//! Handles reading/writing of plan and review markdown files in the
//! `.opencode-studio/kanban/` directory structure.

use std::path::PathBuf;
use tokio::fs;
use tracing::{debug, info};
use uuid::Uuid;

use crate::error::{OrchestratorError, Result};

/// Base directory for OpenCode Studio files
const STUDIO_DIR: &str = ".opencode-studio";
/// Directory for kanban-related files
const KANBAN_DIR: &str = "kanban";
/// Directory for plan files
const PLANS_DIR: &str = "plans";
/// Directory for review files
const REVIEWS_DIR: &str = "reviews";

/// Manages plan and review files for tasks
#[derive(Debug, Clone)]
pub struct FileManager {
    /// Base path of the repository
    base_path: PathBuf,
}

impl FileManager {
    /// Create a new FileManager with the given repository base path
    pub fn new(base_path: impl Into<PathBuf>) -> Self {
        Self {
            base_path: base_path.into(),
        }
    }

    /// Get the path to the plans directory
    pub fn plans_dir(&self) -> PathBuf {
        self.base_path
            .join(STUDIO_DIR)
            .join(KANBAN_DIR)
            .join(PLANS_DIR)
    }

    /// Get the path to the reviews directory
    pub fn reviews_dir(&self) -> PathBuf {
        self.base_path
            .join(STUDIO_DIR)
            .join(KANBAN_DIR)
            .join(REVIEWS_DIR)
    }

    /// Get the path to a plan file for a task
    pub fn plan_path(&self, task_id: Uuid) -> PathBuf {
        self.plans_dir().join(format!("{}.md", task_id))
    }

    /// Get the path to a review file for a task
    pub fn review_path(&self, task_id: Uuid) -> PathBuf {
        self.reviews_dir().join(format!("{}.md", task_id))
    }

    /// Ensure all required directories exist
    pub async fn ensure_directories(&self) -> Result<()> {
        let plans_dir = self.plans_dir();
        let reviews_dir = self.reviews_dir();

        debug!("Ensuring directories exist: {:?}, {:?}", plans_dir, reviews_dir);

        fs::create_dir_all(&plans_dir).await.map_err(|e| {
            OrchestratorError::ExecutionFailed(format!(
                "Failed to create plans directory {:?}: {}",
                plans_dir, e
            ))
        })?;

        fs::create_dir_all(&reviews_dir).await.map_err(|e| {
            OrchestratorError::ExecutionFailed(format!(
                "Failed to create reviews directory {:?}: {}",
                reviews_dir, e
            ))
        })?;

        Ok(())
    }

    /// Write a plan file for a task (atomic write via temp file + rename)
    pub async fn write_plan(&self, task_id: Uuid, content: &str) -> Result<PathBuf> {
        self.ensure_directories().await?;
        let path = self.plan_path(task_id);
        let temp_path = self.plans_dir().join(format!(".{}.tmp", task_id));

        info!("Writing plan to {:?}", path);
        
        fs::write(&temp_path, content).await.map_err(|e| {
            OrchestratorError::ExecutionFailed(format!(
                "Failed to write temp plan file {:?}: {}",
                temp_path, e
            ))
        })?;

        fs::rename(&temp_path, &path).await.map_err(|e| {
            OrchestratorError::ExecutionFailed(format!(
                "Failed to rename plan file {:?} -> {:?}: {}",
                temp_path, path, e
            ))
        })?;

        Ok(path)
    }

    /// Read a plan file for a task
    pub async fn read_plan(&self, task_id: Uuid) -> Result<String> {
        let path = self.plan_path(task_id);

        debug!("Reading plan from {:?}", path);
        fs::read_to_string(&path).await.map_err(|e| {
            OrchestratorError::ExecutionFailed(format!(
                "Failed to read plan file {:?}: {}",
                path, e
            ))
        })
    }

    /// Check if a plan exists for a task
    pub async fn plan_exists(&self, task_id: Uuid) -> bool {
        fs::try_exists(self.plan_path(task_id)).await.unwrap_or(false)
    }

    /// Write a review file for a task (atomic write via temp file + rename)
    pub async fn write_review(&self, task_id: Uuid, content: &str) -> Result<PathBuf> {
        self.ensure_directories().await?;
        let path = self.review_path(task_id);
        let temp_path = self.reviews_dir().join(format!(".{}.tmp", task_id));

        info!("Writing review to {:?}", path);

        fs::write(&temp_path, content).await.map_err(|e| {
            OrchestratorError::ExecutionFailed(format!(
                "Failed to write temp review file {:?}: {}",
                temp_path, e
            ))
        })?;

        fs::rename(&temp_path, &path).await.map_err(|e| {
            OrchestratorError::ExecutionFailed(format!(
                "Failed to rename review file {:?} -> {:?}: {}",
                temp_path, path, e
            ))
        })?;

        Ok(path)
    }

    /// Read a review file for a task
    pub async fn read_review(&self, task_id: Uuid) -> Result<String> {
        let path = self.review_path(task_id);

        debug!("Reading review from {:?}", path);
        fs::read_to_string(&path).await.map_err(|e| {
            OrchestratorError::ExecutionFailed(format!(
                "Failed to read review file {:?}: {}",
                path, e
            ))
        })
    }

    /// Check if a review exists for a task
    pub async fn review_exists(&self, task_id: Uuid) -> bool {
        fs::try_exists(self.review_path(task_id)).await.unwrap_or(false)
    }

    /// Delete a plan file for a task
    pub async fn delete_plan(&self, task_id: Uuid) -> Result<()> {
        let path = self.plan_path(task_id);
        if fs::try_exists(&path).await.unwrap_or(false) {
            fs::remove_file(&path).await.map_err(|e| {
                OrchestratorError::ExecutionFailed(format!(
                    "Failed to delete plan file {:?}: {}",
                    path, e
                ))
            })?;
        }
        Ok(())
    }

    /// Delete a review file for a task
    pub async fn delete_review(&self, task_id: Uuid) -> Result<()> {
        let path = self.review_path(task_id);
        if fs::try_exists(&path).await.unwrap_or(false) {
            fs::remove_file(&path).await.map_err(|e| {
                OrchestratorError::ExecutionFailed(format!(
                    "Failed to delete review file {:?}: {}",
                    path, e
                ))
            })?;
        }
        Ok(())
    }

    /// Get the relative path for a plan (used in prompts)
    pub fn plan_relative_path(&self, task_id: Uuid) -> String {
        format!(
            "{}/{}/{}/{}.md",
            STUDIO_DIR, KANBAN_DIR, PLANS_DIR, task_id
        )
    }

    /// Get the relative path for a review (used in prompts)
    pub fn review_relative_path(&self, task_id: Uuid) -> String {
        format!(
            "{}/{}/{}/{}.md",
            STUDIO_DIR, KANBAN_DIR, REVIEWS_DIR, task_id
        )
    }
}

impl Default for FileManager {
    fn default() -> Self {
        Self::new(".")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn setup_test_file_manager() -> (FileManager, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let fm = FileManager::new(temp_dir.path());
        (fm, temp_dir)
    }

    #[tokio::test]
    async fn test_ensure_directories() {
        let (fm, _temp_dir) = setup_test_file_manager().await;

        fm.ensure_directories().await.unwrap();

        assert!(fm.plans_dir().exists());
        assert!(fm.reviews_dir().exists());
    }

    #[tokio::test]
    async fn test_write_and_read_plan() {
        let (fm, _temp_dir) = setup_test_file_manager().await;
        let task_id = Uuid::new_v4();
        let content = "# Plan\n\nThis is a test plan.";

        let path = fm.write_plan(task_id, content).await.unwrap();
        assert!(path.exists());

        let read_content = fm.read_plan(task_id).await.unwrap();
        assert_eq!(read_content, content);
    }

    #[tokio::test]
    async fn test_write_and_read_review() {
        let (fm, _temp_dir) = setup_test_file_manager().await;
        let task_id = Uuid::new_v4();
        let content = "# Review\n\nAPPROVED\n\nGreat work!";

        let path = fm.write_review(task_id, content).await.unwrap();
        assert!(path.exists());

        let read_content = fm.read_review(task_id).await.unwrap();
        assert_eq!(read_content, content);
    }

    #[tokio::test]
    async fn test_plan_exists() {
        let (fm, _temp_dir) = setup_test_file_manager().await;
        let task_id = Uuid::new_v4();

        assert!(!fm.plan_exists(task_id).await);

        fm.write_plan(task_id, "test").await.unwrap();

        assert!(fm.plan_exists(task_id).await);
    }

    #[tokio::test]
    async fn test_delete_plan() {
        let (fm, _temp_dir) = setup_test_file_manager().await;
        let task_id = Uuid::new_v4();

        fm.write_plan(task_id, "test").await.unwrap();
        assert!(fm.plan_exists(task_id).await);

        fm.delete_plan(task_id).await.unwrap();
        assert!(!fm.plan_exists(task_id).await);
    }

    #[tokio::test]
    async fn test_relative_paths() {
        let fm = FileManager::new("/repo");
        let task_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

        assert_eq!(
            fm.plan_relative_path(task_id),
            ".opencode-studio/kanban/plans/550e8400-e29b-41d4-a716-446655440000.md"
        );
        assert_eq!(
            fm.review_relative_path(task_id),
            ".opencode-studio/kanban/reviews/550e8400-e29b-41d4-a716-446655440000.md"
        );
    }
}
