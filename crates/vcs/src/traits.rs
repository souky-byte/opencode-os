use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use utoipa::ToSchema;

use crate::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct Workspace {
    pub task_id: String,
    #[schema(value_type = String)]
    #[cfg_attr(feature = "typescript", ts(type = "string"))]
    pub path: PathBuf,
    pub branch_name: String,
    pub status: WorkspaceStatus,
    pub created_at: DateTime<Utc>,
}

impl Workspace {
    pub fn new(task_id: impl Into<String>, path: PathBuf, branch_name: impl Into<String>) -> Self {
        Self {
            task_id: task_id.into(),
            path,
            branch_name: branch_name.into(),
            status: WorkspaceStatus::Active,
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceStatus {
    Active,
    Merged,
    Abandoned,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum MergeResult {
    Success,
    Conflicts { files: Vec<ConflictFile> },
}

impl MergeResult {
    pub fn is_success(&self) -> bool {
        matches!(self, MergeResult::Success)
    }

    pub fn conflicts(&self) -> Option<&[ConflictFile]> {
        match self {
            MergeResult::Conflicts { files } => Some(files),
            MergeResult::Success => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct ConflictFile {
    #[schema(value_type = String)]
    #[cfg_attr(feature = "typescript", ts(type = "string"))]
    pub path: PathBuf,
    pub conflict_type: ConflictType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[serde(rename_all = "snake_case")]
pub enum ConflictType {
    Content,
    AddAdd,
    ModifyDelete,
    DeleteModify,
    Rename,
}

/// Summary of changes in a workspace
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct DiffSummary {
    /// Number of files changed
    pub files_changed: u32,
    /// Number of lines added
    pub additions: u32,
    /// Number of lines deleted
    pub deletions: u32,
}

/// Trait for version control system operations
#[async_trait]
pub trait VersionControl: Send + Sync {
    /// Get the name of the VCS backend
    fn name(&self) -> &'static str;

    /// Check if the VCS is available (command exists)
    async fn is_available(&self) -> bool;

    /// Check if the repository is initialized with this VCS
    async fn is_initialized(&self) -> Result<bool>;

    /// Create an isolated workspace for a task
    async fn create_workspace(&self, task_id: &str) -> Result<Workspace>;

    /// Get diff of changes in a workspace
    async fn get_diff(&self, workspace: &Workspace) -> Result<String>;

    /// Get the status of changes in a workspace
    async fn get_status(&self, workspace: &Workspace) -> Result<String>;

    /// Merge workspace changes back to main branch
    async fn merge_workspace(&self, workspace: &Workspace, message: &str) -> Result<MergeResult>;

    /// Clean up and remove a workspace
    async fn cleanup_workspace(&self, workspace: &Workspace) -> Result<()>;

    /// List all active workspaces
    async fn list_workspaces(&self) -> Result<Vec<Workspace>>;

    /// Get conflicts in a workspace (if any)
    async fn get_conflicts(&self, workspace: &Workspace) -> Result<Vec<ConflictFile>>;

    /// Commit changes in a workspace
    async fn commit(&self, workspace: &Workspace, message: &str) -> Result<String>;

    /// Push changes to remote (if applicable)
    async fn push(&self, workspace: &Workspace, remote: &str) -> Result<()>;

    /// Get a summary of changes in a workspace (files changed, additions, deletions)
    async fn get_diff_summary(&self, workspace: &Workspace) -> Result<DiffSummary>;

    /// Get the main/default branch name
    fn main_branch(&self) -> &str;

    /// Check if there are uncommitted changes in a workspace
    async fn has_uncommitted_changes(&self, workspace: &Workspace) -> Result<bool>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_new() {
        let ws = Workspace::new("task-123", PathBuf::from("/tmp/ws"), "branch-123");

        assert_eq!(ws.task_id, "task-123");
        assert_eq!(ws.path, PathBuf::from("/tmp/ws"));
        assert_eq!(ws.branch_name, "branch-123");
        assert_eq!(ws.status, WorkspaceStatus::Active);
    }

    #[test]
    fn test_merge_result_success() {
        let result = MergeResult::Success;

        assert!(result.is_success());
        assert!(result.conflicts().is_none());
    }

    #[test]
    fn test_merge_result_conflicts() {
        let conflict = ConflictFile {
            path: PathBuf::from("src/main.rs"),
            conflict_type: ConflictType::Content,
        };
        let result = MergeResult::Conflicts {
            files: vec![conflict],
        };

        assert!(!result.is_success());
        assert_eq!(result.conflicts().unwrap().len(), 1);
    }

    #[test]
    fn test_workspace_status_serialization() {
        let status = WorkspaceStatus::Active;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"active\"");

        let status = WorkspaceStatus::Merged;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"merged\"");
    }

    #[test]
    fn test_conflict_type_serialization() {
        let ct = ConflictType::ModifyDelete;
        let json = serde_json::to_string(&ct).unwrap();
        assert_eq!(json, "\"modify_delete\"");
    }
}
