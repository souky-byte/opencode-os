//! RAII guard for MCP server connections.
//!
//! This module provides automatic cleanup of MCP server connections
//! when the guard goes out of scope, ensuring no resource leaks.

use std::path::PathBuf;
use tracing::{debug, warn};
use uuid::Uuid;

use crate::core::McpServerSpec;
use crate::error::Result;
use crate::services::McpManager;

/// RAII guard for MCP server connections.
///
/// When this guard is dropped, it automatically disconnects all
/// connected MCP servers. This ensures cleanup even in error paths.
///
/// # Example
///
/// ```ignore
/// let guard = McpGuard::connect(manager.clone(), path, &servers, task_id, session_id).await?;
/// // ... use MCP servers ...
/// // guard is automatically cleaned up when it goes out of scope
/// ```
pub struct McpGuard {
    manager: McpManager,
    workspace_path: PathBuf,
    servers: Vec<String>,
    connected: bool,
}

impl McpGuard {
    /// Connect to MCP servers and create a guard for automatic cleanup.
    ///
    /// # Arguments
    ///
    /// * `manager` - The MCP manager to use for connections (cloned)
    /// * `workspace_path` - Path to the workspace
    /// * `servers` - List of MCP server specifications to connect
    /// * `task_id` - Task ID for the session
    /// * `session_id` - Session ID for the connection
    ///
    /// # Returns
    ///
    /// A guard that will automatically disconnect servers when dropped.
    pub async fn connect(
        manager: McpManager,
        workspace_path: PathBuf,
        servers: &[McpServerSpec],
        task_id: Uuid,
        session_id: Uuid,
    ) -> Result<Self> {
        let mut guard = Self {
            manager,
            workspace_path: workspace_path.clone(),
            servers: Vec::new(),
            connected: false,
        };

        for server in servers {
            debug!(
                server = %server.name,
                task_id = %task_id,
                "Connecting MCP server"
            );

            guard
                .manager
                .setup_findings_server(task_id, session_id, &workspace_path)
                .await?;

            guard.servers.push(server.name.clone());
        }

        guard.connected = true;
        debug!(server_count = guard.servers.len(), "MCP servers connected");

        Ok(guard)
    }

    /// Check if any servers are connected.
    pub fn is_connected(&self) -> bool {
        self.connected && !self.servers.is_empty()
    }

    /// Get the list of connected server names.
    pub fn servers(&self) -> &[String] {
        &self.servers
    }

    /// Manually disconnect all servers.
    ///
    /// This is called automatically in `Drop`, but can be called
    /// explicitly if needed.
    pub async fn disconnect(&mut self) -> Result<()> {
        if !self.connected {
            return Ok(());
        }

        for server in &self.servers {
            debug!(server = %server, "Disconnecting MCP server");

            if let Err(e) = self
                .manager
                .cleanup_findings_server(&self.workspace_path)
                .await
            {
                warn!(
                    server = %server,
                    error = %e,
                    "Failed to disconnect MCP server"
                );
            }
        }

        self.servers.clear();
        self.connected = false;
        Ok(())
    }
}

impl Drop for McpGuard {
    fn drop(&mut self) {
        if self.connected && !self.servers.is_empty() {
            // Spawn cleanup task - cannot await in Drop
            let manager = self.manager.clone();
            let path = self.workspace_path.clone();
            let servers = std::mem::take(&mut self.servers);

            debug!(server_count = servers.len(), "Spawning MCP cleanup task");

            tokio::spawn(async move {
                for server in servers {
                    debug!(server = %server, "Cleaning up MCP server in Drop");

                    if let Err(e) = manager.cleanup_findings_server(&path).await {
                        warn!(
                            server = %server,
                            error = %e,
                            "MCP cleanup failed in Drop"
                        );
                    }
                }
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opencode_client::apis::configuration::Configuration;
    use std::sync::Arc;

    #[test]
    fn test_guard_initial_state() {
        let config = Arc::new(Configuration::new());
        let manager = McpManager::new(config);

        let guard = McpGuard {
            manager,
            workspace_path: PathBuf::from("/tmp/test"),
            servers: vec![],
            connected: false,
        };

        assert!(!guard.is_connected());
        assert!(guard.servers().is_empty());
    }

    #[test]
    fn test_guard_with_servers() {
        let config = Arc::new(Configuration::new());
        let manager = McpManager::new(config);

        // Create guard but mark as not connected to avoid Drop cleanup
        // (which would require a Tokio runtime)
        let mut guard = McpGuard {
            manager,
            workspace_path: PathBuf::from("/tmp/test"),
            servers: vec!["test-server".to_string()],
            connected: true,
        };

        assert!(guard.is_connected());
        assert_eq!(guard.servers(), &["test-server"]);

        // Mark as disconnected before drop to prevent tokio::spawn in Drop
        guard.connected = false;
    }
}
