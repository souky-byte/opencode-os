use opencode_client::apis::configuration::Configuration;
use opencode_client::apis::default_api;
use opencode_client::models::{McpAddRequest, McpAddRequestConfig};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::error::{OrchestratorError, Result};

const MCP_FINDINGS_NAME: &str = "opencode-findings";
const MCP_FINDINGS_BINARY: &str = "opencode-mcp-findings";

pub struct McpManager {
    opencode_config: Arc<Configuration>,
}

impl McpManager {
    pub fn new(opencode_config: Arc<Configuration>) -> Self {
        Self { opencode_config }
    }

    pub async fn setup_findings_server(
        &self,
        task_id: Uuid,
        session_id: Uuid,
        workspace_path: &Path,
    ) -> Result<()> {
        let mcp_binary = self.get_binary_path();

        let mut environment = HashMap::new();
        environment.insert("OPENCODE_TASK_ID".to_string(), task_id.to_string());
        environment.insert("OPENCODE_SESSION_ID".to_string(), session_id.to_string());
        environment.insert(
            "OPENCODE_WORKSPACE_PATH".to_string(),
            workspace_path.to_string_lossy().to_string(),
        );

        let mut config = McpAddRequestConfig::local(vec![mcp_binary]);
        config.environment = Some(environment);
        config.enabled = Some(true);
        config.timeout = Some(10000);

        let request = McpAddRequest::new(MCP_FINDINGS_NAME.to_string(), config);
        let directory = workspace_path.to_str();

        info!(
            task_id = %task_id,
            session_id = %session_id,
            "Adding MCP findings server to OpenCode"
        );

        default_api::mcp_add(&self.opencode_config, directory, Some(request))
            .await
            .map_err(|e| {
                error!(error = %e, "Failed to add MCP findings server");
                OrchestratorError::OpenCodeError(format!("Failed to add MCP server: {}", e))
            })?;

        default_api::mcp_connect(&self.opencode_config, MCP_FINDINGS_NAME, directory)
            .await
            .map_err(|e| {
                error!(error = %e, "Failed to connect MCP findings server");
                OrchestratorError::OpenCodeError(format!("Failed to connect MCP server: {}", e))
            })?;

        info!("MCP findings server connected");
        Ok(())
    }

    pub async fn cleanup_findings_server(&self, workspace_path: &Path) -> Result<()> {
        let directory = workspace_path.to_str();

        info!("Disconnecting MCP findings server");

        if let Err(e) =
            default_api::mcp_disconnect(&self.opencode_config, MCP_FINDINGS_NAME, directory).await
        {
            warn!(error = %e, "Failed to disconnect MCP findings server (may already be disconnected)");
        }

        Ok(())
    }

    fn get_binary_path(&self) -> String {
        if cfg!(debug_assertions) {
            if let Ok(exe_path) = std::env::current_exe() {
                if let Some(parent) = exe_path.parent() {
                    let mcp_path = parent.join(MCP_FINDINGS_BINARY);
                    if mcp_path.exists() {
                        return mcp_path.to_string_lossy().to_string();
                    }
                }
            }
        }
        MCP_FINDINGS_BINARY.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_manager_creation() {
        let config = Arc::new(Configuration::new());
        let manager = McpManager::new(config);
        assert!(!manager.get_binary_path().is_empty());
    }
}
