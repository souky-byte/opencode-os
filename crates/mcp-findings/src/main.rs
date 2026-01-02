//! MCP Findings Server Binary
//!
//! This binary runs the MCP server for AI code review findings.
//! It communicates via stdio and is configured through environment variables.
//!
//! Environment variables:
//! - OPENCODE_TASK_ID: UUID of the task being reviewed
//! - OPENCODE_SESSION_ID: UUID of the review session
//! - OPENCODE_WORKSPACE_PATH: Path to the workspace directory

use anyhow::{Context, Result};
use mcp_findings::FindingsService;
use rmcp::{transport::stdio, ServiceExt};
use std::path::PathBuf;
use tracing::info;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing (log to stderr to not interfere with stdio)
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    // Read configuration from environment
    let task_id = std::env::var("OPENCODE_TASK_ID")
        .context("OPENCODE_TASK_ID environment variable not set")?;
    let task_id: Uuid = task_id
        .parse()
        .context("OPENCODE_TASK_ID is not a valid UUID")?;

    let session_id = std::env::var("OPENCODE_SESSION_ID")
        .context("OPENCODE_SESSION_ID environment variable not set")?;
    let session_id: Uuid = session_id
        .parse()
        .context("OPENCODE_SESSION_ID is not a valid UUID")?;

    let workspace_path = std::env::var("OPENCODE_WORKSPACE_PATH")
        .context("OPENCODE_WORKSPACE_PATH environment variable not set")?;
    let workspace_path = PathBuf::from(workspace_path);

    info!(
        task_id = %task_id,
        session_id = %session_id,
        workspace_path = %workspace_path.display(),
        "Starting MCP Findings Server"
    );

    // Create the service and start serving
    let service = FindingsService::new(task_id, session_id, workspace_path);
    let server = service.serve(stdio()).await?;

    info!("MCP Findings Server running");

    // Wait for the server to finish (client disconnects)
    server.waiting().await?;

    info!("MCP Findings Server shutting down");

    Ok(())
}
