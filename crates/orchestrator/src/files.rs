//! File management for plans, reviews, and findings
//!
//! Handles reading/writing of plan and review markdown files and
//! structured findings JSON in the `.opencode-studio/kanban/` directory structure.

use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
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
/// Directory for findings files
const FINDINGS_DIR: &str = "findings";

// ============================================================================
// Review Findings Types
// ============================================================================

/// Severity level of a finding
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum FindingSeverity {
    Error,
    Warning,
    Info,
}

impl FindingSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            FindingSeverity::Error => "error",
            FindingSeverity::Warning => "warning",
            FindingSeverity::Info => "info",
        }
    }
}

/// Status of a finding
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum FindingStatus {
    Pending,
    Fixed,
    Skipped,
}

/// A single review finding
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ReviewFinding {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_start: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_end: Option<i32>,
    pub title: String,
    pub description: String,
    pub severity: FindingSeverity,
    pub status: FindingStatus,
}

/// Collection of findings from an AI review
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct ReviewFindings {
    pub task_id: Uuid,
    pub session_id: Uuid,
    pub approved: bool,
    pub created_at: DateTime<Utc>,
    pub summary: String,
    pub findings: Vec<ReviewFinding>,
}

impl ReviewFindings {
    /// Create a new approved review with no findings
    pub fn approved(task_id: Uuid, session_id: Uuid, summary: String) -> Self {
        Self {
            task_id,
            session_id,
            approved: true,
            created_at: Utc::now(),
            summary,
            findings: Vec::new(),
        }
    }

    /// Create a new review with findings
    pub fn with_findings(
        task_id: Uuid,
        session_id: Uuid,
        summary: String,
        findings: Vec<ReviewFinding>,
    ) -> Self {
        Self {
            task_id,
            session_id,
            approved: findings.is_empty(),
            created_at: Utc::now(),
            summary,
            findings,
        }
    }

    /// Count pending findings
    pub fn pending_count(&self) -> usize {
        self.findings
            .iter()
            .filter(|f| f.status == FindingStatus::Pending)
            .count()
    }
}

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

    /// Get the path to the findings directory
    pub fn findings_dir(&self) -> PathBuf {
        self.base_path
            .join(STUDIO_DIR)
            .join(KANBAN_DIR)
            .join(FINDINGS_DIR)
    }

    /// Get the path to a plan file for a task
    pub fn plan_path(&self, task_id: Uuid) -> PathBuf {
        self.plans_dir().join(format!("{}.md", task_id))
    }

    /// Get the path to a review file for a task
    pub fn review_path(&self, task_id: Uuid) -> PathBuf {
        self.reviews_dir().join(format!("{}.md", task_id))
    }

    /// Get the path to a findings file for a task
    pub fn findings_path(&self, task_id: Uuid) -> PathBuf {
        self.findings_dir().join(format!("{}.json", task_id))
    }

    /// Ensure all required directories exist
    pub async fn ensure_directories(&self) -> Result<()> {
        let plans_dir = self.plans_dir();
        let reviews_dir = self.reviews_dir();
        let findings_dir = self.findings_dir();

        debug!(
            "Ensuring directories exist: {:?}, {:?}, {:?}",
            plans_dir, reviews_dir, findings_dir
        );

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

        fs::create_dir_all(&findings_dir).await.map_err(|e| {
            OrchestratorError::ExecutionFailed(format!(
                "Failed to create findings directory {:?}: {}",
                findings_dir, e
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

    // ========================================================================
    // Findings Methods
    // ========================================================================

    /// Write findings to a JSON file for a task (atomic write)
    pub async fn write_findings(&self, task_id: Uuid, findings: &ReviewFindings) -> Result<PathBuf> {
        self.ensure_directories().await?;
        let path = self.findings_path(task_id);
        let temp_path = self.findings_dir().join(format!(".{}.tmp", task_id));

        info!("Writing findings to {:?}", path);

        let json = serde_json::to_string_pretty(findings).map_err(|e| {
            OrchestratorError::ExecutionFailed(format!("Failed to serialize findings: {}", e))
        })?;

        fs::write(&temp_path, &json).await.map_err(|e| {
            OrchestratorError::ExecutionFailed(format!(
                "Failed to write temp findings file {:?}: {}",
                temp_path, e
            ))
        })?;

        fs::rename(&temp_path, &path).await.map_err(|e| {
            OrchestratorError::ExecutionFailed(format!(
                "Failed to rename findings file {:?} -> {:?}: {}",
                temp_path, path, e
            ))
        })?;

        Ok(path)
    }

    /// Read findings from a JSON file for a task
    pub async fn read_findings(&self, task_id: Uuid) -> Result<Option<ReviewFindings>> {
        let path = self.findings_path(task_id);

        if !fs::try_exists(&path).await.unwrap_or(false) {
            return Ok(None);
        }

        debug!("Reading findings from {:?}", path);
        let content = fs::read_to_string(&path).await.map_err(|e| {
            OrchestratorError::ExecutionFailed(format!(
                "Failed to read findings file {:?}: {}",
                path, e
            ))
        })?;

        let findings: ReviewFindings = serde_json::from_str(&content).map_err(|e| {
            OrchestratorError::ExecutionFailed(format!(
                "Failed to parse findings file {:?}: {}",
                path, e
            ))
        })?;

        Ok(Some(findings))
    }

    /// Check if findings exist for a task
    pub async fn findings_exists(&self, task_id: Uuid) -> bool {
        fs::try_exists(self.findings_path(task_id))
            .await
            .unwrap_or(false)
    }

    /// Delete findings file for a task
    pub async fn delete_findings(&self, task_id: Uuid) -> Result<()> {
        let path = self.findings_path(task_id);
        if fs::try_exists(&path).await.unwrap_or(false) {
            fs::remove_file(&path).await.map_err(|e| {
                OrchestratorError::ExecutionFailed(format!(
                    "Failed to delete findings file {:?}: {}",
                    path, e
                ))
            })?;
        }
        Ok(())
    }

    /// Update status of specific findings in the file
    pub async fn update_findings_status(
        &self,
        task_id: Uuid,
        finding_ids: &[String],
        status: FindingStatus,
    ) -> Result<()> {
        let mut findings = self
            .read_findings(task_id)
            .await?
            .ok_or_else(|| OrchestratorError::ExecutionFailed("Findings file not found".into()))?;

        for finding in &mut findings.findings {
            if finding_ids.contains(&finding.id) {
                finding.status = status;
            }
        }

        self.write_findings(task_id, &findings).await?;
        Ok(())
    }

    /// Mark all pending findings as skipped
    pub async fn skip_all_findings(&self, task_id: Uuid) -> Result<()> {
        let mut findings = self
            .read_findings(task_id)
            .await?
            .ok_or_else(|| OrchestratorError::ExecutionFailed("Findings file not found".into()))?;

        for finding in &mut findings.findings {
            if finding.status == FindingStatus::Pending {
                finding.status = FindingStatus::Skipped;
            }
        }

        self.write_findings(task_id, &findings).await?;
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
