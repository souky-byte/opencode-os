//! MCP Server for Wiki Search and RAG Q&A
//!
//! This crate provides an MCP (Model Context Protocol) server that enables
//! AI models to search code, retrieve documentation, and ask questions about
//! a codebase using semantic search and RAG (Retrieval-Augmented Generation).
//!
//! The server exposes tools like:
//! - `search_code` - Semantic search for code chunks
//! - `get_documentation` - Retrieve wiki pages by slug
//! - `ask_codebase` - RAG Q&A over the codebase
//! - `list_wiki_pages` - List all wiki pages and structure

use rmcp::{
    handler::server::{router::tool::ToolRouter, tool::Parameters},
    model::{ErrorData as McpError, *},
    schemars, tool, tool_handler, tool_router, ServerHandler,
};
use serde::Deserialize;
use std::borrow::Cow;
use std::future::Future;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info};
use wiki::{
    ChatMessage, Conversation, OpenRouterClient, RagSource, SearchResult, VectorStore, WikiConfig,
    WikiPage, WikiStructure,
};

/// Request to search for code
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SearchCodeRequest {
    /// The search query - describe what you're looking for
    #[schemars(description = "Natural language query describing what code you're looking for")]
    pub query: String,

    /// Maximum number of results to return (default: 10)
    #[schemars(description = "Maximum number of results to return (1-50, default: 10)")]
    pub limit: Option<usize>,
}

/// Request to get documentation page
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetDocumentationRequest {
    /// The slug of the wiki page to retrieve
    #[schemars(description = "The slug/path of the wiki page (e.g., 'overview', 'modules/auth')")]
    pub slug: String,
}

/// Request to ask a question about the codebase
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AskCodebaseRequest {
    /// The question to ask about the codebase
    #[schemars(description = "Your question about the codebase")]
    pub question: String,

    /// Conversation ID for multi-turn Q&A (optional)
    #[schemars(description = "Conversation ID to continue a previous conversation")]
    pub conversation_id: Option<String>,
}

/// Request to list wiki pages
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListWikiPagesRequest {
    /// Branch to list pages for (default: main)
    #[schemars(description = "Git branch to list pages for (default: main)")]
    pub branch: Option<String>,
}

/// Wiki MCP Service
#[derive(Clone)]
pub struct WikiService {
    openrouter: Arc<OpenRouterClient>,
    conversations: Arc<Mutex<std::collections::HashMap<String, Conversation>>>,
    config: WikiConfig,
    tool_router: ToolRouter<WikiService>,
}

impl WikiService {
    /// Create a new WikiService with the given configuration
    pub fn new(config: WikiConfig) -> Result<Self, wiki::WikiError> {
        // Verify database can be opened (creates if needed)
        let _ = VectorStore::new(&config.db_path)?;

        let openrouter = OpenRouterClient::new(
            config.openrouter_api_key.clone(),
            config.api_base_url.clone(),
        );

        Ok(Self {
            openrouter: Arc::new(openrouter),
            conversations: Arc::new(Mutex::new(std::collections::HashMap::new())),
            config,
            tool_router: Self::tool_router(),
        })
    }

    /// Format search results as text
    fn format_search_results(results: &[SearchResult]) -> String {
        if results.is_empty() {
            return "No matching code found.".to_string();
        }

        let mut output = format!("Found {} relevant code snippets:\n\n", results.len());

        for (i, result) in results.iter().enumerate() {
            output.push_str(&format!(
                "--- Result {} ({}) ---\n",
                i + 1,
                result.score_percent()
            ));
            output.push_str(&format!(
                "Location: {}:{}-{}\n",
                result.file_path, result.start_line, result.end_line
            ));
            if let Some(lang) = &result.language {
                output.push_str(&format!("Language: {}\n", lang));
            }
            output.push_str(&format!("Type: {:?}\n\n", result.chunk_type));

            // Add code with language hint
            if let Some(lang) = &result.language {
                output.push_str(&format!("```{}\n{}\n```\n\n", lang, result.content));
            } else {
                output.push_str(&format!("```\n{}\n```\n\n", result.content));
            }
        }

        output
    }

    /// Format RAG sources as text
    fn format_sources(sources: &[RagSource]) -> String {
        if sources.is_empty() {
            return String::new();
        }

        let mut output = "\n\n**Sources:**\n".to_string();
        for (i, source) in sources.iter().take(5).enumerate() {
            output.push_str(&format!(
                "{}. {}:{}-{} ({:.0}% relevance)\n",
                i + 1,
                source.file_path,
                source.start_line,
                source.end_line,
                source.score * 100.0
            ));
        }
        output
    }

    /// Format wiki page as text
    fn format_wiki_page(page: &WikiPage) -> String {
        let mut output = format!("# {}\n\n", page.title);
        output.push_str(&page.content);
        output.push_str(&format!("\n\n---\nPage type: {:?}", page.page_type));
        if !page.file_paths.is_empty() {
            output.push_str(&format!("\nRelated files: {}", page.file_paths.join(", ")));
        }
        output
    }

    /// Format wiki structure as text
    fn format_wiki_structure(structure: &WikiStructure, branch: &str) -> String {
        let mut output = format!(
            "Wiki Structure for branch '{}' ({} pages)\n\n",
            branch, structure.page_count
        );

        // Format tree structure
        fn format_tree(tree: &wiki::WikiTree, output: &mut String, indent: usize) {
            let prefix = "  ".repeat(indent);
            output.push_str(&format!("{}- {} ({})\n", prefix, tree.title, tree.slug));
            for child in &tree.children {
                format_tree(child, output, indent + 1);
            }
        }

        format_tree(&structure.root, &mut output, 0);

        output.push_str(&format!(
            "\nLast updated: {}",
            structure.updated_at.format("%Y-%m-%d %H:%M:%S")
        ));

        output
    }

    /// Format index status as text
    fn format_index_status(status: &wiki::IndexStatus, branch: &str) -> String {
        let mut output = format!("Index Status for branch '{}'\n\n", branch);
        output.push_str(&format!("State: {:?}\n", status.state));
        output.push_str(&format!("Files indexed: {}\n", status.file_count));
        output.push_str(&format!("Chunks created: {}\n", status.chunk_count));
        if let Some(sha) = &status.last_commit_sha {
            output.push_str(&format!("Last commit: {}\n", sha));
        }
        if let Some(indexed_at) = status.last_indexed_at {
            output.push_str(&format!(
                "Last indexed: {}\n",
                indexed_at.format("%Y-%m-%d %H:%M:%S")
            ));
        }
        if let Some(error) = &status.error_message {
            output.push_str(&format!("Error: {}\n", error));
        }
        output.push_str(&format!("Progress: {}%", status.progress_percent));
        output
    }
}

#[tool_router]
impl WikiService {
    #[tool(
        description = "Search for code in the indexed codebase using semantic search. Returns relevant code snippets with file locations."
    )]
    async fn search_code(
        &self,
        Parameters(request): Parameters<SearchCodeRequest>,
    ) -> Result<CallToolResult, McpError> {
        let limit = request.limit.unwrap_or(10).min(50);
        let query = request.query.clone();

        info!(query = %query, limit = limit, "Searching code");

        // Get embedding from OpenRouter
        let embedding = self
            .openrouter
            .create_embedding(&query, &self.config.embedding_model)
            .await
            .map_err(|e| McpError {
                code: ErrorCode(-32603),
                message: Cow::from(format!("Failed to create embedding: {}", e)),
                data: None,
            })?;

        // Search vector store in blocking task
        let db_path = self.config.db_path.clone();
        let results =
            tokio::task::spawn_blocking(move || -> Result<Vec<SearchResult>, wiki::WikiError> {
                let store = VectorStore::new(&db_path)?;
                store.search_similar(&embedding, limit)
            })
            .await
            .map_err(|e| McpError {
                code: ErrorCode(-32603),
                message: Cow::from(format!("Task join error: {}", e)),
                data: None,
            })?
            .map_err(|e| McpError {
                code: ErrorCode(-32603),
                message: Cow::from(format!("Search failed: {}", e)),
                data: None,
            })?;

        debug!("Found {} results", results.len());
        let output = Self::format_search_results(&results);
        Ok(CallToolResult::success(vec![Content::text(output)]))
    }

    #[tool(
        description = "Get a documentation page from the wiki by its slug. Returns the full page content with diagrams."
    )]
    async fn get_documentation(
        &self,
        Parameters(request): Parameters<GetDocumentationRequest>,
    ) -> Result<CallToolResult, McpError> {
        let slug = request.slug.clone();
        info!(slug = %slug, "Getting documentation");

        let db_path = self.config.db_path.clone();
        let page_result = tokio::task::spawn_blocking(move || {
            let store = VectorStore::new(&db_path)?;
            store.get_wiki_page(&slug)
        })
        .await
        .map_err(|e| McpError {
            code: ErrorCode(-32603),
            message: Cow::from(format!("Task join error: {}", e)),
            data: None,
        })?
        .map_err(|e| McpError {
            code: ErrorCode(-32603),
            message: Cow::from(format!("Failed to get page: {}", e)),
            data: None,
        })?;

        match page_result {
            Some(page) => {
                let output = Self::format_wiki_page(&page);
                Ok(CallToolResult::success(vec![Content::text(output)]))
            }
            None => Ok(CallToolResult::success(vec![Content::text(format!(
                "Page '{}' not found. Use list_wiki_pages to see available pages.",
                request.slug
            ))])),
        }
    }

    #[tool(
        description = "Ask a question about the codebase. Uses semantic search to find relevant code and generates an answer using AI."
    )]
    async fn ask_codebase(
        &self,
        Parameters(request): Parameters<AskCodebaseRequest>,
    ) -> Result<CallToolResult, McpError> {
        let question = request.question.clone();
        info!(question = %question, "Asking codebase");

        // Get embedding for the question
        let query_embedding = self
            .openrouter
            .create_embedding(&question, &self.config.embedding_model)
            .await
            .map_err(|e| McpError {
                code: ErrorCode(-32603),
                message: Cow::from(format!("Failed to create embedding: {}", e)),
                data: None,
            })?;

        // Search for similar chunks in blocking task
        let db_path = self.config.db_path.clone();
        let search_results = tokio::task::spawn_blocking(move || {
            let store = VectorStore::new(&db_path)?;
            store.search_similar(&query_embedding, 10)
        })
        .await
        .map_err(|e| McpError {
            code: ErrorCode(-32603),
            message: Cow::from(format!("Task join error: {}", e)),
            data: None,
        })?
        .map_err(|e| McpError {
            code: ErrorCode(-32603),
            message: Cow::from(format!("Search failed: {}", e)),
            data: None,
        })?;

        if search_results.is_empty() {
            return Ok(CallToolResult::success(vec![Content::text(
                "I couldn't find any relevant code in the indexed codebase to answer your question."
                    .to_string(),
            )]));
        }

        // Build context from search results
        let context = build_context(&search_results);
        let sources: Vec<RagSource> = search_results.iter().map(RagSource::from).collect();

        // Build messages for chat completion
        let mut messages = vec![ChatMessage::system(RAG_SYSTEM_PROMPT)];

        // Add conversation history if provided
        if let Some(conv_id) = &request.conversation_id {
            let conversations = self.conversations.lock().await;
            if let Some(conversation) = conversations.get(conv_id) {
                for msg in &conversation.messages {
                    match msg.role {
                        wiki::MessageRole::User => {
                            messages.push(ChatMessage::user(&msg.content))
                        }
                        wiki::MessageRole::Assistant => {
                            messages.push(ChatMessage::assistant(&msg.content))
                        }
                    }
                }
            }
        }

        // Add current question with context
        messages.push(ChatMessage::user(format_user_prompt(&question, &context)));

        // Get chat completion
        let answer = self
            .openrouter
            .chat_completion(messages, &self.config.chat_model, Some(0.3), Some(2048))
            .await
            .map_err(|e| McpError {
                code: ErrorCode(-32603),
                message: Cow::from(format!("Chat completion failed: {}", e)),
                data: None,
            })?;

        // Update conversation history if provided
        if let Some(conv_id) = request.conversation_id {
            let mut conversations = self.conversations.lock().await;
            let conversation = conversations
                .entry(conv_id.clone())
                .or_insert_with(|| Conversation::with_id(conv_id));
            conversation.add_user_message(&question);
            conversation.add_assistant_message(&answer);
        }

        let mut output = answer;
        output.push_str(&Self::format_sources(&sources));

        Ok(CallToolResult::success(vec![Content::text(output)]))
    }

    #[tool(description = "List all wiki pages and their structure for a given branch.")]
    async fn list_wiki_pages(
        &self,
        Parameters(request): Parameters<ListWikiPagesRequest>,
    ) -> Result<CallToolResult, McpError> {
        let branch = request.branch.clone().unwrap_or_else(|| "main".to_string());
        info!(branch = %branch, "Listing wiki pages");

        let db_path = self.config.db_path.clone();
        let branch_clone = branch.clone();
        let structure_result = tokio::task::spawn_blocking(move || {
            let store = VectorStore::new(&db_path)?;
            store.get_wiki_structure(&branch_clone)
        })
        .await
        .map_err(|e| McpError {
            code: ErrorCode(-32603),
            message: Cow::from(format!("Task join error: {}", e)),
            data: None,
        })?
        .map_err(|e| McpError {
            code: ErrorCode(-32603),
            message: Cow::from(format!("Failed to get wiki structure: {}", e)),
            data: None,
        })?;

        match structure_result {
            Some(structure) => {
                let output = Self::format_wiki_structure(&structure, &branch);
                Ok(CallToolResult::success(vec![Content::text(output)]))
            }
            None => Ok(CallToolResult::success(vec![Content::text(format!(
                "No wiki structure found for branch '{}'. The wiki may not be indexed yet.",
                branch
            ))])),
        }
    }

    #[tool(description = "Get the indexing status for the wiki.")]
    async fn get_index_status(
        &self,
        Parameters(request): Parameters<ListWikiPagesRequest>,
    ) -> Result<CallToolResult, McpError> {
        let branch = request.branch.clone().unwrap_or_else(|| "main".to_string());
        info!(branch = %branch, "Getting index status");

        let db_path = self.config.db_path.clone();
        let branch_clone = branch.clone();
        let status_result = tokio::task::spawn_blocking(move || {
            let store = VectorStore::new(&db_path)?;
            store.get_index_status(&branch_clone)
        })
        .await
        .map_err(|e| McpError {
            code: ErrorCode(-32603),
            message: Cow::from(format!("Task join error: {}", e)),
            data: None,
        })?
        .map_err(|e| McpError {
            code: ErrorCode(-32603),
            message: Cow::from(format!("Failed to get index status: {}", e)),
            data: None,
        })?;

        match status_result {
            Some(status) => {
                let output = Self::format_index_status(&status, &branch);
                Ok(CallToolResult::success(vec![Content::text(output)]))
            }
            None => Ok(CallToolResult::success(vec![Content::text(format!(
                "Branch '{}' has not been indexed yet.",
                branch
            ))])),
        }
    }
}

/// System prompt for code Q&A
const RAG_SYSTEM_PROMPT: &str = r#"You are a knowledgeable code assistant helping developers understand a codebase.

You have access to relevant code snippets from the codebase to answer questions.
When answering:
- Reference specific files and line numbers when relevant (format: `file_path:line_number`)
- Provide concise but complete explanations
- Include code examples when helpful
- If the context doesn't contain enough information, say so clearly
- Don't make up information that's not in the provided context

Always cite the relevant code locations to support your answers."#;

/// Build context string from search results
fn build_context(results: &[SearchResult]) -> String {
    const MAX_CONTEXT_LENGTH: usize = 32000;

    let mut context = String::new();
    let mut total_length = 0;

    for (i, result) in results.iter().enumerate() {
        let chunk_header = format!(
            "\n--- Source {}: {} (lines {}-{}) ---\n",
            i + 1,
            result.file_path,
            result.start_line,
            result.end_line
        );

        let chunk_content = if let Some(lang) = &result.language {
            format!("```{}\n{}\n```\n", lang, result.content)
        } else {
            format!("```\n{}\n```\n", result.content)
        };

        let chunk_total = chunk_header.len() + chunk_content.len();

        if total_length + chunk_total > MAX_CONTEXT_LENGTH {
            debug!("Context truncated at {} chunks due to length limit", i);
            break;
        }

        context.push_str(&chunk_header);
        context.push_str(&chunk_content);
        total_length += chunk_total;
    }

    context
}

/// Format the user prompt with query and context
fn format_user_prompt(query: &str, context: &str) -> String {
    format!(
        r#"Based on the following code snippets from the codebase, please answer this question:

**Question:** {}

**Relevant Code:**
{}

Please provide a clear and helpful answer based on the code context above."#,
        query, context
    )
}

#[tool_handler]
impl ServerHandler for WikiService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "opencode-wiki".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
            instructions: Some(
                "Use this server to search code and ask questions about the codebase.\n\n\
                 Available tools:\n\
                 - search_code: Find relevant code using semantic search\n\
                 - get_documentation: Retrieve wiki documentation pages\n\
                 - ask_codebase: Ask questions and get AI-generated answers\n\
                 - list_wiki_pages: Browse available documentation\n\
                 - get_index_status: Check wiki indexing status"
                    .to_string(),
            ),
        }
    }
}

/// Configuration from environment variables
pub struct WikiServiceConfig {
    pub db_path: PathBuf,
    pub openrouter_api_key: String,
    pub embedding_model: String,
    pub chat_model: String,
    pub api_base_url: String,
}

impl WikiServiceConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> anyhow::Result<Self> {
        use anyhow::Context;

        let db_path = std::env::var("OPENCODE_WIKI_DB_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(".opencode-studio/wiki.db"));

        let openrouter_api_key = std::env::var("OPENROUTER_API_KEY")
            .context("OPENROUTER_API_KEY environment variable not set")?;

        let embedding_model = std::env::var("OPENCODE_WIKI_EMBEDDING_MODEL")
            .unwrap_or_else(|_| "openai/text-embedding-3-small".to_string());

        let chat_model = std::env::var("OPENCODE_WIKI_CHAT_MODEL")
            .unwrap_or_else(|_| "anthropic/claude-3.5-sonnet".to_string());

        let api_base_url = std::env::var("OPENROUTER_API_BASE_URL")
            .unwrap_or_else(|_| "https://openrouter.ai/api/v1".to_string());

        Ok(Self {
            db_path,
            openrouter_api_key,
            embedding_model,
            chat_model,
            api_base_url,
        })
    }

    /// Convert to WikiConfig
    pub fn to_wiki_config(&self) -> WikiConfig {
        WikiConfig {
            db_path: self.db_path.clone(),
            openrouter_api_key: self.openrouter_api_key.clone(),
            embedding_model: self.embedding_model.clone(),
            chat_model: self.chat_model.clone(),
            api_base_url: self.api_base_url.clone(),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn create_test_config() -> WikiConfig {
        let dir = tempdir().unwrap();
        #[allow(deprecated)]
        let path = dir.into_path();
        WikiConfig {
            db_path: path.join("wiki.db"),
            openrouter_api_key: "test-key".to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn test_wiki_service_creation() {
        let config = create_test_config();
        let service = WikiService::new(config);
        assert!(service.is_ok());
    }

    #[test]
    fn test_format_search_results_empty() {
        let output = WikiService::format_search_results(&[]);
        assert_eq!(output, "No matching code found.");
    }

    #[test]
    fn test_format_sources_empty() {
        let output = WikiService::format_sources(&[]);
        assert!(output.is_empty());
    }

    #[test]
    fn test_format_sources() {
        let sources = vec![RagSource {
            file_path: "src/main.rs".to_string(),
            start_line: 1,
            end_line: 10,
            score: 0.95,
            snippet: "fn main()".to_string(),
        }];

        let output = WikiService::format_sources(&sources);
        assert!(output.contains("src/main.rs"));
        assert!(output.contains("95%"));
    }

    #[test]
    fn test_wiki_service_config_to_wiki_config() {
        let config = WikiServiceConfig {
            db_path: PathBuf::from("/tmp/wiki.db"),
            openrouter_api_key: "test-key".to_string(),
            embedding_model: "test-embed".to_string(),
            chat_model: "test-chat".to_string(),
            api_base_url: "https://test.api".to_string(),
        };

        let wiki_config = config.to_wiki_config();
        assert_eq!(wiki_config.db_path, PathBuf::from("/tmp/wiki.db"));
        assert_eq!(wiki_config.openrouter_api_key, "test-key");
        assert_eq!(wiki_config.embedding_model, "test-embed");
        assert_eq!(wiki_config.chat_model, "test-chat");
    }

    #[test]
    fn test_build_context() {
        use wiki::ChunkType;
        use uuid::Uuid;

        let results = vec![SearchResult::new(
            Uuid::new_v4(),
            "src/lib.rs".to_string(),
            1,
            10,
            "fn main() {}".to_string(),
            ChunkType::Function,
            Some("rust".to_string()),
            0.95,
        )];

        let context = build_context(&results);
        assert!(context.contains("src/lib.rs"));
        assert!(context.contains("fn main()"));
    }

    #[test]
    fn test_format_user_prompt() {
        let prompt = format_user_prompt("What does this do?", "fn test() {}");
        assert!(prompt.contains("What does this do?"));
        assert!(prompt.contains("fn test() {}"));
    }
}
