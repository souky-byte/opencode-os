use std::path::PathBuf;
use std::sync::Arc;
use tokio::process::Command;
use tracing::{debug, info, warn};

use crate::error::{Result, VcsError};
use crate::traits::{MergeResult, VersionControl, Workspace};

#[derive(Debug, Clone)]
pub struct WorkspaceConfig {
    pub workspace_base: PathBuf,
    pub init_scripts: Vec<PathBuf>,
    pub cleanup_scripts: Vec<PathBuf>,
    pub copy_files: Vec<String>,
    pub symlink_dirs: Vec<String>,
}

impl Default for WorkspaceConfig {
    fn default() -> Self {
        Self {
            workspace_base: PathBuf::from("../.workspaces"),
            init_scripts: Vec::new(),
            cleanup_scripts: Vec::new(),
            copy_files: vec![".env".to_string(), ".env.local".to_string()],
            symlink_dirs: vec![
                "node_modules".to_string(),
                "target".to_string(),
                ".venv".to_string(),
            ],
        }
    }
}

impl WorkspaceConfig {
    pub fn new(workspace_base: PathBuf) -> Self {
        Self {
            workspace_base,
            ..Default::default()
        }
    }

    pub fn with_init_scripts(mut self, scripts: Vec<PathBuf>) -> Self {
        self.init_scripts = scripts;
        self
    }

    pub fn with_cleanup_scripts(mut self, scripts: Vec<PathBuf>) -> Self {
        self.cleanup_scripts = scripts;
        self
    }
}

pub struct WorkspaceManager {
    vcs: Arc<dyn VersionControl>,
    config: WorkspaceConfig,
    repo_path: PathBuf,
}

impl WorkspaceManager {
    pub fn new(vcs: Arc<dyn VersionControl>, config: WorkspaceConfig, repo_path: PathBuf) -> Self {
        Self {
            vcs,
            config,
            repo_path,
        }
    }

    pub async fn setup_workspace(&self, task_id: &str) -> Result<Workspace> {
        info!("Setting up workspace for task {}", task_id);

        let workspace = self.vcs.create_workspace(task_id).await?;

        if let Err(e) = self.run_init_scripts(&workspace).await {
            warn!("Init scripts failed: {}, cleaning up workspace", e);
            let _ = self.cleanup_workspace(&workspace).await;
            return Err(e);
        }

        if let Err(e) = self.setup_files(&workspace).await {
            warn!("File setup failed: {}, cleaning up workspace", e);
            let _ = self.cleanup_workspace(&workspace).await;
            return Err(e);
        }

        info!("Workspace created at {:?}", workspace.path);
        Ok(workspace)
    }

    async fn run_init_scripts(&self, workspace: &Workspace) -> Result<()> {
        for script in &self.config.init_scripts {
            if !script.exists() {
                warn!("Init script not found: {:?}", script);
                continue;
            }

            debug!("Running init script: {:?}", script);

            let output = Command::new("bash")
                .arg(script)
                .arg(&workspace.path)
                .arg(&workspace.task_id)
                .arg(&self.repo_path)
                .current_dir(&self.repo_path)
                .output()
                .await?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(VcsError::CommandFailed(format!(
                    "Init script {:?} failed: {}",
                    script, stderr
                )));
            }
        }

        Ok(())
    }

    async fn setup_files(&self, workspace: &Workspace) -> Result<()> {
        for file in &self.config.copy_files {
            let src = self.repo_path.join(file);
            let dst = workspace.path.join(file);

            if src.exists() {
                debug!("Copying {} to workspace", file);
                if let Some(parent) = dst.parent() {
                    tokio::fs::create_dir_all(parent).await?;
                }
                tokio::fs::copy(&src, &dst).await?;
            }
        }

        for dir in &self.config.symlink_dirs {
            let src = self.repo_path.join(dir);
            let dst = workspace.path.join(dir);

            if src.exists() && !dst.exists() {
                debug!("Symlinking {} to workspace", dir);
                #[cfg(unix)]
                tokio::fs::symlink(&src, &dst).await?;
                #[cfg(windows)]
                tokio::fs::symlink_dir(&src, &dst).await?;
            }
        }

        Ok(())
    }

    pub async fn cleanup_workspace(&self, workspace: &Workspace) -> Result<()> {
        info!("Cleaning up workspace for task {}", workspace.task_id);

        for script in &self.config.cleanup_scripts {
            if !script.exists() {
                warn!("Cleanup script not found: {:?}", script);
                continue;
            }

            debug!("Running cleanup script: {:?}", script);

            match Command::new("bash")
                .arg(script)
                .arg(&workspace.path)
                .arg(&workspace.task_id)
                .current_dir(&self.repo_path)
                .output()
                .await
            {
                Ok(output) => {
                    if !output.status.success() {
                        warn!(
                            "Cleanup script {:?} failed with status {:?}: {}",
                            script,
                            output.status.code(),
                            String::from_utf8_lossy(&output.stderr)
                        );
                    }
                }
                Err(e) => {
                    warn!("Failed to execute cleanup script {:?}: {}", script, e);
                }
            }
        }

        self.vcs.cleanup_workspace(workspace).await?;

        info!("Workspace cleaned up: {}", workspace.task_id);
        Ok(())
    }

    pub async fn get_diff(&self, workspace: &Workspace) -> Result<String> {
        self.vcs.get_diff(workspace).await
    }

    pub async fn get_status(&self, workspace: &Workspace) -> Result<String> {
        self.vcs.get_status(workspace).await
    }

    pub async fn merge_workspace(
        &self,
        workspace: &Workspace,
        message: &str,
    ) -> Result<MergeResult> {
        self.vcs.merge_workspace(workspace, message).await
    }

    pub async fn list_workspaces(&self) -> Result<Vec<Workspace>> {
        self.vcs.list_workspaces().await
    }

    pub async fn commit(&self, workspace: &Workspace, message: &str) -> Result<String> {
        self.vcs.commit(workspace, message).await
    }

    pub async fn push(&self, workspace: &Workspace, remote: &str) -> Result<()> {
        self.vcs.push(workspace, remote).await
    }

    /// Get a reference to the underlying VCS implementation
    pub fn vcs(&self) -> &dyn VersionControl {
        self.vcs.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_config_default() {
        let config = WorkspaceConfig::default();
        assert_eq!(config.workspace_base, PathBuf::from("../.workspaces"));
        assert!(config.init_scripts.is_empty());
        assert!(config.cleanup_scripts.is_empty());
    }

    #[test]
    fn test_workspace_config_builder() {
        let config = WorkspaceConfig::new(PathBuf::from("/custom/path"))
            .with_init_scripts(vec![PathBuf::from("init.sh")])
            .with_cleanup_scripts(vec![PathBuf::from("cleanup.sh")]);

        assert_eq!(config.workspace_base, PathBuf::from("/custom/path"));
        assert_eq!(config.init_scripts.len(), 1);
        assert_eq!(config.cleanup_scripts.len(), 1);
    }
}
