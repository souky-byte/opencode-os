//! Wiki crate for OpenCode Studio
//!
//! Provides AI-powered code documentation generation, semantic search,
//! and RAG (Retrieval-Augmented Generation) capabilities.
//!
//! # Architecture
//!
//! - **OpenRouter Client**: Embeddings and chat completions via OpenRouter API
//! - **Vector Store**: SQLite + sqlite-vec for vector similarity search
//! - **Chunker**: Intelligent code splitting with overlap
//! - **Indexer**: File traversal, chunking, and embedding creation
//! - **Generator**: Wiki page generation with Mermaid diagrams
//! - **RAG Engine**: Question answering over codebase

pub mod chunker;
pub mod domain;
pub mod error;
pub mod generator;
pub mod git;
pub mod indexer;
pub mod openrouter;
pub mod rag;
pub mod sync;
pub mod vector_store;

pub use chunker::TextSplitter;
pub use domain::{
    chunk::{ChunkType, CodeChunk},
    index_status::{IndexProgress, IndexState, IndexStatus},
    search_result::SearchResult,
    wiki_page::{Importance, PageType, SourceCitation, WikiPage, WikiStructure, WikiTree},
    wiki_section::{GenerationMode, WikiSection},
};
pub use error::{WikiError, WikiResult};
pub use generator::{analyzer::ProjectAnalyzer, WikiGenerator};
pub use indexer::{reader::FileReader, CodeIndexer};
pub use openrouter::client::OpenRouterClient;
pub use openrouter::types::ChatMessage;
pub use rag::{Conversation, Message, MessageRole, RagEngine, RagResponse, RagSource};
pub use sync::WikiSyncService;
pub use vector_store::VectorStore;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration for the Wiki engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiConfig {
    /// Branches to index and generate wiki for
    pub branches: Vec<String>,

    /// OpenRouter API key
    pub openrouter_api_key: String,

    /// Embedding model: openai/text-embedding-3-small
    pub embedding_model: String,

    /// Chat model for generation: google/gemini-3-flash-preview
    pub chat_model: String,

    /// Path to wiki database
    pub db_path: PathBuf,

    /// Auto-sync on git push
    pub auto_sync: bool,

    /// Maximum chunk size in tokens
    pub max_chunk_tokens: usize,

    /// Chunk overlap in tokens
    pub chunk_overlap: usize,

    /// OpenRouter API base URL
    pub api_base_url: String,

    /// Remote repository URL (e.g., "https://github.com/owner/repo")
    /// If set, branches will be cloned from this URL instead of using local project
    #[serde(default)]
    pub repo_url: Option<String>,

    /// Access token for private repositories (GitHub PAT, GitLab token, etc.)
    #[serde(default)]
    pub access_token: Option<String>,
}

impl Default for WikiConfig {
    fn default() -> Self {
        Self {
            branches: vec!["main".to_string()],
            openrouter_api_key: String::new(),
            embedding_model: "openai/text-embedding-3-small".to_string(),
            chat_model: "google/gemini-3-flash-preview".to_string(),
            db_path: PathBuf::from(".opencode-studio/wiki.db"),
            auto_sync: true,
            max_chunk_tokens: 350,
            chunk_overlap: 100,
            api_base_url: "https://openrouter.ai/api/v1".to_string(),
            repo_url: None,
            access_token: None,
        }
    }
}

/// Main Wiki engine that orchestrates indexing, generation, and search
pub struct WikiEngine {
    config: WikiConfig,
    openrouter: OpenRouterClient,
    vector_store: VectorStore,
    text_splitter: TextSplitter,
}

impl WikiEngine {
    /// Create a new WikiEngine with the given configuration
    pub fn new(config: WikiConfig) -> WikiResult<Self> {
        let openrouter = OpenRouterClient::new(
            config.openrouter_api_key.clone(),
            config.api_base_url.clone(),
        );

        let vector_store = VectorStore::new(&config.db_path)?;
        let text_splitter = TextSplitter::new(config.max_chunk_tokens, config.chunk_overlap);

        Ok(Self {
            config,
            openrouter,
            vector_store,
            text_splitter,
        })
    }

    /// Get the configuration
    pub fn config(&self) -> &WikiConfig {
        &self.config
    }

    /// Get a reference to the OpenRouter client
    pub fn openrouter(&self) -> &OpenRouterClient {
        &self.openrouter
    }

    /// Get a reference to the vector store
    pub fn vector_store(&self) -> &VectorStore {
        &self.vector_store
    }

    /// Get a reference to the text splitter
    pub fn text_splitter(&self) -> &TextSplitter {
        &self.text_splitter
    }

    /// Get the index status for a branch
    pub fn get_index_status(&self, branch: &str) -> WikiResult<Option<IndexStatus>> {
        self.vector_store.get_index_status(branch)
    }

    /// Search for similar code chunks
    pub async fn search(&self, query: &str, limit: usize) -> WikiResult<Vec<SearchResult>> {
        // Create embedding for query
        let embedding = self
            .openrouter
            .create_embedding(query, &self.config.embedding_model)
            .await?;

        // Search vector store
        self.vector_store.search_similar(&embedding, limit)
    }

    /// Get wiki page by slug
    pub fn get_page(&self, slug: &str) -> WikiResult<Option<WikiPage>> {
        self.vector_store.get_wiki_page(slug)
    }

    /// Get wiki structure (tree of pages)
    pub fn get_structure(&self, branch: &str) -> WikiResult<Option<WikiStructure>> {
        self.vector_store.get_wiki_structure(branch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_wiki_config_default() {
        let config = WikiConfig::default();
        assert_eq!(config.branches, vec!["main".to_string()]);
        assert_eq!(config.max_chunk_tokens, 350);
        assert_eq!(config.chunk_overlap, 100);
        assert!(config.auto_sync);
    }

    #[test]
    fn test_wiki_engine_creation() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("wiki.db");

        let config = WikiConfig {
            db_path,
            openrouter_api_key: "test-key".to_string(),
            ..Default::default()
        };

        let engine = WikiEngine::new(config);
        assert!(engine.is_ok());
    }
}
