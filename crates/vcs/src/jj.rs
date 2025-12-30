use async_trait::async_trait;
use std::path::PathBuf;
use tokio::process::Command;
use tracing::{debug, warn};

use crate::error::{Result, VcsError};
use crate::traits::{ConflictFile, ConflictType, MergeResult, VersionControl, Workspace};

pub struct JujutsuVcs {
    repo_path: PathBuf,
    workspace_base: PathBuf,
}

impl JujutsuVcs {
    pub fn new(repo_path: PathBuf, workspace_base: PathBuf) -> Self {
        Self {
            repo_path,
            workspace_base,
        }
    }

    async fn run_jj(&self, args: &[&str], cwd: &PathBuf) -> Result<String> {
        debug!("Running jj {:?} in {:?}", args, cwd);

        let output = Command::new("jj")
            .args(args)
            .current_dir(cwd)
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(VcsError::CommandFailed(format!(
                "jj {} failed: {}",
                args.join(" "),
                stderr
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }

    fn workspace_path(&self, task_id: &str) -> PathBuf {
        self.workspace_base.join(format!("task-{}", task_id))
    }

    fn workspace_name(&self, task_id: &str) -> String {
        format!("task-{}", task_id)
    }
}

#[async_trait]
impl VersionControl for JujutsuVcs {
    fn name(&self) -> &'static str {
        "jujutsu"
    }

    async fn is_available(&self) -> bool {
        Command::new("jj")
            .arg("--version")
            .output()
            .await
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    async fn is_initialized(&self) -> Result<bool> {
        let jj_dir = self.repo_path.join(".jj");
        Ok(jj_dir.exists())
    }

    async fn create_workspace(&self, task_id: &str) -> Result<Workspace> {
        let workspace_path = self.workspace_path(task_id);
        let workspace_name = self.workspace_name(task_id);

        if workspace_path.exists() {
            return Err(VcsError::WorkspaceAlreadyExists(task_id.to_string()));
        }

        self.run_jj(
            &[
                "new",
                "main",
                "-m",
                &format!("task-{}: Start implementation", task_id),
            ],
            &self.repo_path,
        )
        .await?;

        self.run_jj(
            &[
                "workspace",
                "add",
                workspace_path
                    .to_str()
                    .ok_or_else(|| VcsError::InvalidPath(workspace_path.display().to_string()))?,
                "--name",
                &workspace_name,
            ],
            &self.repo_path,
        )
        .await?;

        Ok(Workspace::new(task_id, workspace_path, workspace_name))
    }

    async fn get_diff(&self, workspace: &Workspace) -> Result<String> {
        if !workspace.path.exists() {
            return Err(VcsError::WorkspaceNotFound(workspace.task_id.clone()));
        }

        self.run_jj(&["diff"], &workspace.path).await
    }

    async fn get_status(&self, workspace: &Workspace) -> Result<String> {
        if !workspace.path.exists() {
            return Err(VcsError::WorkspaceNotFound(workspace.task_id.clone()));
        }

        self.run_jj(&["status"], &workspace.path).await
    }

    async fn merge_workspace(&self, workspace: &Workspace, message: &str) -> Result<MergeResult> {
        if !workspace.path.exists() {
            return Err(VcsError::WorkspaceNotFound(workspace.task_id.clone()));
        }

        self.run_jj(&["describe", "-m", message], &workspace.path)
            .await?;

        let result = self
            .run_jj(&["rebase", "-d", "main"], &workspace.path)
            .await;

        match result {
            Ok(_) => {
                let conflicts = self.get_conflicts(workspace).await?;
                if conflicts.is_empty() {
                    Ok(MergeResult::Success)
                } else {
                    Ok(MergeResult::Conflicts { files: conflicts })
                }
            }
            Err(e) => {
                warn!("Rebase failed: {}", e);
                let conflicts = self.get_conflicts(workspace).await.unwrap_or_default();
                if conflicts.is_empty() {
                    Err(e)
                } else {
                    Ok(MergeResult::Conflicts { files: conflicts })
                }
            }
        }
    }

    async fn cleanup_workspace(&self, workspace: &Workspace) -> Result<()> {
        let workspace_name = self.workspace_name(&workspace.task_id);

        let _ = self
            .run_jj(&["workspace", "forget", &workspace_name], &self.repo_path)
            .await;

        if workspace.path.exists() {
            tokio::fs::remove_dir_all(&workspace.path).await?;
        }

        Ok(())
    }

    async fn list_workspaces(&self) -> Result<Vec<Workspace>> {
        let output = self.run_jj(&["workspace", "list"], &self.repo_path).await?;

        let mut workspaces = Vec::new();

        for line in output.lines() {
            if let Some(name) = line.split_whitespace().next() {
                // jj workspace list outputs "task-123: <commit_id> <description>"
                // We need to strip the trailing colon from the workspace name
                let name = name.trim_end_matches(':');
                if name.starts_with("task-") {
                    let task_id = name.strip_prefix("task-").unwrap_or(name);
                    let path = self.workspace_path(task_id);

                    if path.exists() {
                        workspaces.push(Workspace::new(task_id, path, name));
                    }
                }
            }
        }

        Ok(workspaces)
    }

    async fn get_conflicts(&self, workspace: &Workspace) -> Result<Vec<ConflictFile>> {
        if !workspace.path.exists() {
            return Err(VcsError::WorkspaceNotFound(workspace.task_id.clone()));
        }

        let output = self.run_jj(&["resolve", "--list"], &workspace.path).await;

        match output {
            Ok(text) => {
                let conflicts: Vec<ConflictFile> = text
                    .lines()
                    .filter(|line| !line.is_empty())
                    .map(|line| {
                        let path = line.split_whitespace().next().unwrap_or(line);
                        ConflictFile {
                            path: PathBuf::from(path),
                            conflict_type: ConflictType::Content,
                        }
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

        self.run_jj(&["describe", "-m", message], &workspace.path)
            .await?;

        let output = self
            .run_jj(
                &["log", "-r", "@", "--no-graph", "-T", "change_id"],
                &workspace.path,
            )
            .await?;

        Ok(output.trim().to_string())
    }

    async fn push(&self, workspace: &Workspace, remote: &str) -> Result<()> {
        if !workspace.path.exists() {
            return Err(VcsError::WorkspaceNotFound(workspace.task_id.clone()));
        }

        self.run_jj(
            &[
                "bookmark",
                "create",
                &workspace.branch_name,
                "-r",
                "@",
                "--allow-backwards",
            ],
            &workspace.path,
        )
        .await?;

        self.run_jj(
            &[
                "git",
                "push",
                "--remote",
                remote,
                "--bookmark",
                &workspace.branch_name,
            ],
            &workspace.path,
        )
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_path() {
        let vcs = JujutsuVcs::new(PathBuf::from("/repo"), PathBuf::from("/workspaces"));

        let path = vcs.workspace_path("123");
        assert_eq!(path, PathBuf::from("/workspaces/task-123"));
    }

    #[test]
    fn test_workspace_name() {
        let vcs = JujutsuVcs::new(PathBuf::from("/repo"), PathBuf::from("/workspaces"));

        let name = vcs.workspace_name("abc-456");
        assert_eq!(name, "task-abc-456");
    }
}
