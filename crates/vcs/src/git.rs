use async_trait::async_trait;
use std::path::PathBuf;
use tokio::process::Command;
use tracing::{debug, warn};

use crate::error::{Result, VcsError};
use crate::traits::{
    ConflictFile, ConflictType, DiffSummary, MergeResult, VersionControl, Workspace,
};

pub struct GitVcs {
    repo_path: PathBuf,
    workspace_base: PathBuf,
    main_branch: String,
}

impl GitVcs {
    pub fn new(repo_path: PathBuf, workspace_base: PathBuf) -> Self {
        Self {
            repo_path,
            workspace_base,
            main_branch: "main".to_string(),
        }
    }

    pub fn with_main_branch(mut self, branch: impl Into<String>) -> Self {
        self.main_branch = branch.into();
        self
    }

    async fn run_git(&self, args: &[&str], cwd: &PathBuf) -> Result<String> {
        debug!("Running git {:?} in {:?}", args, cwd);

        let output = Command::new("git")
            .args(args)
            .current_dir(cwd)
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(VcsError::CommandFailed(format!(
                "git {} failed: {}",
                args.join(" "),
                stderr
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }

    fn workspace_path(&self, task_id: &str) -> PathBuf {
        self.workspace_base.join(format!("task-{}", task_id))
    }

    fn branch_name(&self, task_id: &str) -> String {
        format!("task-{}", task_id)
    }

    async fn get_repo_conflicts(&self) -> Result<Vec<ConflictFile>> {
        let output = self
            .run_git(&["diff", "--name-only", "--diff-filter=U"], &self.repo_path)
            .await;

        match output {
            Ok(text) => {
                let conflicts: Vec<ConflictFile> = text
                    .lines()
                    .filter(|line| !line.is_empty())
                    .map(|path| ConflictFile {
                        path: PathBuf::from(path),
                        conflict_type: ConflictType::Content,
                    })
                    .collect();
                Ok(conflicts)
            }
            Err(_) => Ok(Vec::new()),
        }
    }
}

#[async_trait]
impl VersionControl for GitVcs {
    fn name(&self) -> &'static str {
        "git"
    }

    async fn is_available(&self) -> bool {
        Command::new("git")
            .arg("--version")
            .output()
            .await
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    async fn is_initialized(&self) -> Result<bool> {
        let git_dir = self.repo_path.join(".git");
        Ok(git_dir.exists())
    }

    async fn create_workspace(&self, task_id: &str) -> Result<Workspace> {
        let workspace_path = self.workspace_path(task_id);
        let branch = self.branch_name(task_id);

        if workspace_path.exists() {
            return Err(VcsError::WorkspaceAlreadyExists(task_id.to_string()));
        }

        self.run_git(
            &[
                "worktree",
                "add",
                "-b",
                &branch,
                workspace_path
                    .to_str()
                    .ok_or_else(|| VcsError::InvalidPath(workspace_path.display().to_string()))?,
                &self.main_branch,
            ],
            &self.repo_path,
        )
        .await?;

        Ok(Workspace::new(task_id, workspace_path, branch))
    }

    async fn get_diff(&self, workspace: &Workspace) -> Result<String> {
        if !workspace.path.exists() {
            return Err(VcsError::WorkspaceNotFound(workspace.task_id.clone()));
        }

        // Get all changes compared to main branch (committed changes)
        let committed = self
            .run_git(&["diff", &self.main_branch, "HEAD"], &workspace.path)
            .await?;

        // Get staged changes (not yet committed)
        let staged = self.run_git(&["diff", "--cached"], &workspace.path).await?;

        // Get unstaged changes (working directory)
        let unstaged = self.run_git(&["diff"], &workspace.path).await?;

        Ok(format!("{}{}{}", committed, staged, unstaged))
    }

    async fn get_status(&self, workspace: &Workspace) -> Result<String> {
        if !workspace.path.exists() {
            return Err(VcsError::WorkspaceNotFound(workspace.task_id.clone()));
        }

        self.run_git(&["status", "--porcelain"], &workspace.path)
            .await
    }

    async fn merge_workspace(&self, workspace: &Workspace, message: &str) -> Result<MergeResult> {
        if !workspace.path.exists() {
            return Err(VcsError::WorkspaceNotFound(workspace.task_id.clone()));
        }

        // Commit any uncommitted changes in the workspace
        let status = self.get_status(workspace).await?;
        if !status.is_empty() {
            self.run_git(&["add", "-A"], &workspace.path).await?;
            self.run_git(&["commit", "-m", message], &workspace.path)
                .await?;
        }

        // Check if main branch has uncommitted changes in the main repo
        // This would prevent checkout, so we use a different strategy
        let main_status = self
            .run_git(&["status", "--porcelain"], &self.repo_path)
            .await?;

        if !main_status.trim().is_empty() {
            // Main repo has uncommitted changes - use fetch + merge strategy
            // First, fetch the workspace branch into main repo
            // Then merge using git merge without checkout

            // Get the commit SHA from workspace
            let workspace_sha = self
                .run_git(&["rev-parse", "HEAD"], &workspace.path)
                .await?
                .trim()
                .to_string();

            // Update the branch ref in main repo to point to workspace's HEAD
            self.run_git(
                &[
                    "fetch",
                    workspace.path.to_str().unwrap_or("."),
                    &format!("{}:{}", workspace.branch_name, workspace.branch_name),
                ],
                &self.repo_path,
            )
            .await?;

            // Now try to merge using git merge-tree to check for conflicts first
            // If there are conflicts, we abort. Otherwise, we do the merge.

            // Check if fast-forward is possible
            let merge_base = self
                .run_git(
                    &["merge-base", &self.main_branch, &workspace.branch_name],
                    &self.repo_path,
                )
                .await?
                .trim()
                .to_string();

            let main_sha = self
                .run_git(&["rev-parse", &self.main_branch], &self.repo_path)
                .await?
                .trim()
                .to_string();

            if merge_base == main_sha {
                // Fast-forward is possible - update main branch ref directly
                self.run_git(
                    &[
                        "update-ref",
                        &format!("refs/heads/{}", self.main_branch),
                        &workspace_sha,
                    ],
                    &self.repo_path,
                )
                .await?;

                debug!(
                    "Fast-forwarded {} to {}",
                    self.main_branch, workspace.branch_name
                );
                return Ok(MergeResult::Success);
            }

            // Non-fast-forward merge needed - this is more complex
            // For safety, we'll return an error asking user to resolve manually
            // or stash their changes first
            return Err(VcsError::CommandFailed(
                "Cannot merge: main branch has diverged and your working directory has uncommitted changes. \
                 Please commit or stash your changes in the main repository first, then try again.".to_string()
            ));
        }

        // Main repo is clean - use standard checkout + merge approach
        self.run_git(&["checkout", &self.main_branch], &self.repo_path)
            .await?;

        let merge_result = self
            .run_git(
                &["merge", "--no-ff", &workspace.branch_name, "-m", message],
                &self.repo_path,
            )
            .await;

        match merge_result {
            Ok(_) => Ok(MergeResult::Success),
            Err(e) => {
                warn!("Merge failed: {}", e);
                let conflicts = self.get_repo_conflicts().await?;
                if conflicts.is_empty() {
                    let _ = self.run_git(&["merge", "--abort"], &self.repo_path).await;
                    Err(e)
                } else {
                    let _ = self.run_git(&["merge", "--abort"], &self.repo_path).await;
                    Ok(MergeResult::Conflicts { files: conflicts })
                }
            }
        }
    }

    async fn cleanup_workspace(&self, workspace: &Workspace) -> Result<()> {
        let _ = self
            .run_git(
                &[
                    "worktree",
                    "remove",
                    "--force",
                    workspace.path.to_str().unwrap_or(""),
                ],
                &self.repo_path,
            )
            .await;

        let _ = self
            .run_git(&["branch", "-D", &workspace.branch_name], &self.repo_path)
            .await;

        if workspace.path.exists() {
            tokio::fs::remove_dir_all(&workspace.path).await?;
        }

        Ok(())
    }

    async fn list_workspaces(&self) -> Result<Vec<Workspace>> {
        let output = self
            .run_git(&["worktree", "list", "--porcelain"], &self.repo_path)
            .await?;

        let mut workspaces = Vec::new();
        let mut current_path: Option<PathBuf> = None;
        let mut current_branch: Option<String> = None;

        for line in output.lines() {
            if let Some(path) = line.strip_prefix("worktree ") {
                current_path = Some(PathBuf::from(path));
            } else if let Some(branch) = line.strip_prefix("branch refs/heads/") {
                current_branch = Some(branch.to_string());
            } else if line.is_empty() {
                if let (Some(path), Some(branch)) = (current_path.take(), current_branch.take()) {
                    if let Some(task_id) = branch.strip_prefix("task-") {
                        let task_id = task_id.to_string();
                        workspaces.push(Workspace::new(task_id, path, branch));
                    }
                }
            }
        }

        if let (Some(path), Some(branch)) = (current_path, current_branch) {
            if let Some(task_id) = branch.strip_prefix("task-") {
                let task_id = task_id.to_string();
                workspaces.push(Workspace::new(task_id, path, branch));
            }
        }

        Ok(workspaces)
    }

    async fn get_conflicts(&self, workspace: &Workspace) -> Result<Vec<ConflictFile>> {
        if !workspace.path.exists() {
            return Err(VcsError::WorkspaceNotFound(workspace.task_id.clone()));
        }

        let output = self
            .run_git(&["diff", "--name-only", "--diff-filter=U"], &workspace.path)
            .await;

        match output {
            Ok(text) => {
                let conflicts: Vec<ConflictFile> = text
                    .lines()
                    .filter(|line| !line.is_empty())
                    .map(|path| ConflictFile {
                        path: PathBuf::from(path),
                        conflict_type: ConflictType::Content,
                    })
                    .collect();
                Ok(conflicts)
            }
            Err(_) => Ok(Vec::new()),
        }
    }

    async fn commit(&self, workspace: &Workspace, message: &str) -> Result<String> {
        if !workspace.path.exists() {
            return Err(VcsError::WorkspaceNotFound(workspace.task_id.clone()));
        }

        self.run_git(&["add", "-A"], &workspace.path).await?;
        self.run_git(&["commit", "-m", message], &workspace.path)
            .await?;

        let output = self
            .run_git(&["rev-parse", "HEAD"], &workspace.path)
            .await?;

        Ok(output.trim().to_string())
    }

    async fn push(&self, workspace: &Workspace, remote: &str) -> Result<()> {
        if !workspace.path.exists() {
            return Err(VcsError::WorkspaceNotFound(workspace.task_id.clone()));
        }

        self.run_git(
            &["push", "-u", remote, &workspace.branch_name],
            &workspace.path,
        )
        .await?;

        Ok(())
    }

    async fn get_diff_summary(&self, workspace: &Workspace) -> Result<DiffSummary> {
        if !workspace.path.exists() {
            return Err(VcsError::WorkspaceNotFound(workspace.task_id.clone()));
        }

        // Get diff stats comparing workspace branch to main branch
        // Use --numstat for machine-readable output: additions deletions filename
        let output = self
            .run_git(
                &["diff", "--numstat", &self.main_branch, "HEAD"],
                &workspace.path,
            )
            .await?;

        let mut files_changed: u32 = 0;
        let mut additions: u32 = 0;
        let mut deletions: u32 = 0;

        for line in output.lines() {
            if line.is_empty() {
                continue;
            }
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 2 {
                files_changed += 1;
                // Binary files show "-" for additions/deletions
                if let Ok(add) = parts[0].parse::<u32>() {
                    additions += add;
                }
                if let Ok(del) = parts[1].parse::<u32>() {
                    deletions += del;
                }
            }
        }

        Ok(DiffSummary {
            files_changed,
            additions,
            deletions,
        })
    }

    fn main_branch(&self) -> &str {
        &self.main_branch
    }

    async fn has_uncommitted_changes(&self, workspace: &Workspace) -> Result<bool> {
        if !workspace.path.exists() {
            return Err(VcsError::WorkspaceNotFound(workspace.task_id.clone()));
        }

        let status = self.get_status(workspace).await?;
        Ok(!status.trim().is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_path() {
        let vcs = GitVcs::new(PathBuf::from("/repo"), PathBuf::from("/workspaces"));

        let path = vcs.workspace_path("123");
        assert_eq!(path, PathBuf::from("/workspaces/task-123"));
    }

    #[test]
    fn test_branch_name() {
        let vcs = GitVcs::new(PathBuf::from("/repo"), PathBuf::from("/workspaces"));

        let name = vcs.branch_name("abc-456");
        assert_eq!(name, "task-abc-456");
    }

    #[test]
    fn test_with_main_branch() {
        let vcs = GitVcs::new(PathBuf::from("/repo"), PathBuf::from("/workspaces"))
            .with_main_branch("master");

        assert_eq!(vcs.main_branch, "master");
    }
}
