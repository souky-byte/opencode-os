//! MCP Wiki Server Binary
//!
//! This binary runs the MCP server for wiki search and RAG Q&A.
//! It communicates via stdio and is configured through environment variables.
//!
//! Environment variables:
//! - OPENROUTER_API_KEY: API key for OpenRouter (required)
//! - OPENCODE_WIKI_DB_PATH: Path to wiki database (default: .opencode-studio/wiki.db)
//! - OPENCODE_WIKI_EMBEDDING_MODEL: Embedding model (default: openai/text-embedding-3-small)
//! - OPENCODE_WIKI_CHAT_MODEL: Chat model (default: anthropic/claude-3.5-sonnet)
//! - OPENROUTER_API_BASE_URL: OpenRouter API base URL (default: https://openrouter.ai/api/v1)

use anyhow::Result;
use mcp_wiki::{WikiService, WikiServiceConfig};
use rmcp::{transport::stdio, ServiceExt};
use tracing::info;

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

    // Load configuration from environment
    let service_config = WikiServiceConfig::from_env()?;

    info!(
        db_path = %service_config.db_path.display(),
        embedding_model = %service_config.embedding_model,
        chat_model = %service_config.chat_model,
        "Starting MCP Wiki Server"
    );

    // Create wiki config and service
    let wiki_config = service_config.to_wiki_config();
    let service = WikiService::new(wiki_config)?;

    // Start serving
    let server = service.serve(stdio()).await?;

    info!("MCP Wiki Server running");

    // Wait for the server to finish (client disconnects)
    server.waiting().await?;

    info!("MCP Wiki Server shutting down");

    Ok(())
}
