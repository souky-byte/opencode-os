//! MCP (Model Context Protocol) configuration for different execution phases.
//!
//! This module defines which MCP servers are available for each session phase,
//! allowing different tools to be exposed based on the current task phase.
//! This helps optimize context window usage by only loading relevant tools.

use opencode_core::SessionPhase;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Specification for an MCP server that can be attached to a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerSpec {
    /// Unique name for this MCP server (e.g., "opencode-findings")
    pub name: String,

    /// Path to the MCP server binary, or "builtin:<name>" for built-in servers
    pub binary: McpBinarySource,

    /// Environment variables to pass to the MCP server.
    /// Supported placeholders: {task_id}, {session_id}, {workspace_path}
    pub env_vars: HashMap<String, String>,

    /// Timeout in milliseconds for MCP operations
    #[serde(default = "default_timeout")]
    pub timeout_ms: u32,
}

fn default_timeout() -> u32 {
    10000 // 10 seconds
}

/// Source of the MCP binary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum McpBinarySource {
    /// Built-in MCP server (e.g., "builtin:findings")
    Builtin(String),
    /// Path to external MCP binary
    Path(PathBuf),
}

impl McpBinarySource {
    /// Check if this is the built-in findings server
    pub fn is_findings_server(&self) -> bool {
        matches!(self, McpBinarySource::Builtin(name) if name == "builtin:findings")
    }

    /// Get the binary path, resolving built-in servers
    pub fn resolve_path(&self, mcp_binary_dir: Option<&PathBuf>) -> Option<PathBuf> {
        match self {
            McpBinarySource::Builtin(name) if name == "builtin:findings" => {
                // Try to find the opencode-mcp-findings binary
                if let Some(dir) = mcp_binary_dir {
                    Some(dir.join("opencode-mcp-findings"))
                } else {
                    // Try common locations
                    let candidates = [
                        PathBuf::from("./target/debug/opencode-mcp-findings"),
                        PathBuf::from("./target/release/opencode-mcp-findings"),
                        PathBuf::from("/usr/local/bin/opencode-mcp-findings"),
                    ];
                    candidates.into_iter().find(|p| p.exists())
                }
            }
            McpBinarySource::Builtin(_) => None, // Unknown built-in
            McpBinarySource::Path(path) => Some(path.clone()),
        }
    }
}

/// Configuration for MCP servers per session phase.
#[derive(Debug, Clone, Default)]
pub struct PhaseMcpConfig {
    /// Map of session phases to their MCP server specifications
    phase_servers: HashMap<SessionPhase, Vec<McpServerSpec>>,
}

impl PhaseMcpConfig {
    /// Create a new empty configuration
    pub fn new() -> Self {
        Self {
            phase_servers: HashMap::new(),
        }
    }

    /// Create the default configuration with findings server for Review and Fix phases
    pub fn default_config() -> Self {
        let mut config = Self::new();

        // Findings MCP server specification
        let findings_server = McpServerSpec {
            name: "opencode-findings".to_string(),
            binary: McpBinarySource::Builtin("builtin:findings".to_string()),
            env_vars: [
                ("OPENCODE_TASK_ID".to_string(), "{task_id}".to_string()),
                (
                    "OPENCODE_SESSION_ID".to_string(),
                    "{session_id}".to_string(),
                ),
                (
                    "OPENCODE_WORKSPACE_PATH".to_string(),
                    "{workspace_path}".to_string(),
                ),
            ]
            .into_iter()
            .collect(),
            timeout_ms: 10000,
        };

        // AI Review phase gets findings server (to create findings)
        config.add_server(SessionPhase::Review, findings_server.clone());

        // Fix phase gets findings server (to read and mark findings as fixed)
        config.add_server(SessionPhase::Fix, findings_server);

        config
    }

    /// Add an MCP server to a phase
    pub fn add_server(&mut self, phase: SessionPhase, server: McpServerSpec) {
        self.phase_servers.entry(phase).or_default().push(server);
    }

    /// Get MCP servers for a specific phase
    pub fn get_servers(&self, phase: &SessionPhase) -> Option<&Vec<McpServerSpec>> {
        self.phase_servers.get(phase)
    }

    /// Check if a phase has any MCP servers configured
    pub fn has_servers(&self, phase: &SessionPhase) -> bool {
        self.phase_servers
            .get(phase)
            .map(|servers| !servers.is_empty())
            .unwrap_or(false)
    }

    /// Get all configured phases
    pub fn configured_phases(&self) -> Vec<&SessionPhase> {
        self.phase_servers.keys().collect()
    }
}

/// Helper to expand environment variable placeholders
pub fn expand_env_vars(
    template: &str,
    task_id: &str,
    session_id: &str,
    workspace_path: &str,
) -> String {
    template
        .replace("{task_id}", task_id)
        .replace("{session_id}", session_id)
        .replace("{workspace_path}", workspace_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_has_review_phase() {
        let config = PhaseMcpConfig::default_config();
        assert!(config.has_servers(&SessionPhase::Review));
        assert!(config.has_servers(&SessionPhase::Fix));
        assert!(!config.has_servers(&SessionPhase::Planning));
        assert!(!config.has_servers(&SessionPhase::Implementation));
    }

    #[test]
    fn test_expand_env_vars() {
        let result = expand_env_vars(
            "{task_id}-{session_id}",
            "task-123",
            "session-456",
            "/workspace",
        );
        assert_eq!(result, "task-123-session-456");
    }
}
