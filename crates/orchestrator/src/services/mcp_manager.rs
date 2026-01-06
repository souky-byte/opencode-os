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

const MCP_WIKI_NAME: &str = "opencode-wiki";
const MCP_WIKI_BINARY: &str = "opencode-mcp-wiki";

#[derive(Clone)]
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
        project_path: &Path,
    ) -> Result<()> {
        let mcp_binary = self.get_binary_path();

        let mut environment = HashMap::new();
        environment.insert("OPENCODE_TASK_ID".to_string(), task_id.to_string());
        environment.insert("OPENCODE_SESSION_ID".to_string(), session_id.to_string());
        environment.insert(
            "OPENCODE_WORKSPACE_PATH".to_string(),
            workspace_path.to_string_lossy().to_string(),
        );
        // Project path is where findings are stored (main repo, not worktree)
        environment.insert(
            "OPENCODE_PROJECT_PATH".to_string(),
            project_path.to_string_lossy().to_string(),
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

    /// Setup Wiki MCP server for code search and RAG Q&A
    ///
    /// This server provides:
    /// - search_code: Semantic code search
    /// - get_documentation: Retrieve wiki pages
    /// - ask_codebase: RAG Q&A over codebase
    /// - list_wiki_pages: Browse wiki structure
    pub async fn setup_wiki_server(
        &self,
        workspace_path: &Path,
        wiki_config: &WikiMcpConfig,
    ) -> Result<()> {
        let mcp_binary = self.get_wiki_binary_path();

        let mut environment = HashMap::new();
        environment.insert(
            "OPENROUTER_API_KEY".to_string(),
            wiki_config.openrouter_api_key.clone(),
        );
        environment.insert(
            "OPENCODE_WIKI_DB_PATH".to_string(),
            wiki_config.db_path.to_string_lossy().to_string(),
        );
        if let Some(ref model) = wiki_config.embedding_model {
            environment.insert("OPENCODE_WIKI_EMBEDDING_MODEL".to_string(), model.clone());
        }
        if let Some(ref model) = wiki_config.chat_model {
            environment.insert("OPENCODE_WIKI_CHAT_MODEL".to_string(), model.clone());
        }
        if let Some(ref base_url) = wiki_config.api_base_url {
            environment.insert("OPENROUTER_API_BASE_URL".to_string(), base_url.clone());
        }

        let mut config = McpAddRequestConfig::local(vec![mcp_binary]);
        config.environment = Some(environment);
        config.enabled = Some(true);
        config.timeout = Some(30000); // 30 seconds for wiki operations

        let request = McpAddRequest::new(MCP_WIKI_NAME.to_string(), config);
        let directory = workspace_path.to_str();

        info!(
            db_path = %wiki_config.db_path.display(),
            "Adding MCP wiki server to OpenCode"
        );

        default_api::mcp_add(&self.opencode_config, directory, Some(request))
            .await
            .map_err(|e| {
                error!(error = %e, "Failed to add MCP wiki server");
                OrchestratorError::OpenCodeError(format!("Failed to add MCP wiki server: {}", e))
            })?;

        default_api::mcp_connect(&self.opencode_config, MCP_WIKI_NAME, directory)
            .await
            .map_err(|e| {
                error!(error = %e, "Failed to connect MCP wiki server");
                OrchestratorError::OpenCodeError(format!(
                    "Failed to connect MCP wiki server: {}",
                    e
                ))
            })?;

        info!("MCP wiki server connected");
        Ok(())
    }

    /// Cleanup Wiki MCP server
    pub async fn cleanup_wiki_server(&self, workspace_path: &Path) -> Result<()> {
        let directory = workspace_path.to_str();

        info!("Disconnecting MCP wiki server");

        if let Err(e) =
            default_api::mcp_disconnect(&self.opencode_config, MCP_WIKI_NAME, directory).await
        {
            warn!(error = %e, "Failed to disconnect MCP wiki server (may already be disconnected)");
        }

        Ok(())
    }

    fn get_binary_path(&self) -> String {
        self.find_binary(MCP_FINDINGS_BINARY)
    }

    fn get_wiki_binary_path(&self) -> String {
        self.find_binary(MCP_WIKI_BINARY)
    }

    fn find_binary(&self, binary_name: &str) -> String {
        if cfg!(debug_assertions) {
            if let Ok(exe_path) = std::env::current_exe() {
                if let Some(parent) = exe_path.parent() {
                    let mcp_path = parent.join(binary_name);
                    if mcp_path.exists() {
                        return mcp_path.to_string_lossy().to_string();
                    }
                }
            }
        }
        binary_name.to_string()
    }
}

/// Configuration for Wiki MCP server
#[derive(Debug, Clone)]
pub struct WikiMcpConfig {
    /// OpenRouter API key (required)
    pub openrouter_api_key: String,
    /// Path to wiki database
    pub db_path: std::path::PathBuf,
    /// Embedding model (optional, defaults to openai/text-embedding-3-small)
    pub embedding_model: Option<String>,
    /// Chat model (optional, defaults to anthropic/claude-3.5-sonnet)
    pub chat_model: Option<String>,
    /// OpenRouter API base URL (optional)
    pub api_base_url: Option<String>,
}

impl WikiMcpConfig {
    /// Create a new WikiMcpConfig with required fields
    pub fn new(
        openrouter_api_key: impl Into<String>,
        db_path: impl Into<std::path::PathBuf>,
    ) -> Self {
        Self {
            openrouter_api_key: openrouter_api_key.into(),
            db_path: db_path.into(),
            embedding_model: None,
            chat_model: None,
            api_base_url: None,
        }
    }

    /// Set the embedding model
    pub fn with_embedding_model(mut self, model: impl Into<String>) -> Self {
        self.embedding_model = Some(model.into());
        self
    }

    /// Set the chat model
    pub fn with_chat_model(mut self, model: impl Into<String>) -> Self {
        self.chat_model = Some(model.into());
        self
    }

    /// Set the API base URL
    pub fn with_api_base_url(mut self, url: impl Into<String>) -> Self {
        self.api_base_url = Some(url.into());
        self
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

    #[test]
    fn test_wiki_mcp_config() {
        let config = WikiMcpConfig::new("test-key", "/tmp/wiki.db")
            .with_embedding_model("openai/text-embedding-3-small")
            .with_chat_model("anthropic/claude-3.5-sonnet");

        assert_eq!(config.openrouter_api_key, "test-key");
        assert_eq!(
            config.embedding_model,
            Some("openai/text-embedding-3-small".to_string())
        );
        assert_eq!(
            config.chat_model,
            Some("anthropic/claude-3.5-sonnet".to_string())
        );
    }

    #[test]
    fn test_find_binary() {
        let config = Arc::new(Configuration::new());
        let manager = McpManager::new(config);
        assert!(!manager.get_wiki_binary_path().is_empty());
    }
}
