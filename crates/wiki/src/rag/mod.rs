//! RAG (Retrieval-Augmented Generation) engine for Q&A over codebase

use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tracing::{debug, info};

use crate::domain::search_result::SearchResult;
use crate::error::WikiResult;
use crate::openrouter::client::OpenRouterClient;
use crate::openrouter::types::ChatMessage;
use crate::vector_store::VectorStore;

/// Default number of chunks to retrieve for context
const DEFAULT_TOP_K: usize = 10;

/// Maximum context length in characters
const MAX_CONTEXT_LENGTH: usize = 32000;

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

/// A message in the conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Role of the message sender
    pub role: MessageRole,
    /// Content of the message
    pub content: String,
}

/// Role of a message sender
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Assistant,
}

impl Message {
    /// Create a new user message
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::User,
            content: content.into(),
        }
    }

    /// Create a new assistant message
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: content.into(),
        }
    }
}

/// Conversation state for multi-turn Q&A
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Conversation {
    /// Unique identifier for the conversation
    pub id: String,
    /// Messages in the conversation
    pub messages: Vec<Message>,
}

impl Conversation {
    /// Create a new conversation with a generated ID
    pub fn new() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            messages: Vec::new(),
        }
    }

    /// Create a conversation with a specific ID
    pub fn with_id(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            messages: Vec::new(),
        }
    }

    /// Add a user message to the conversation
    pub fn add_user_message(&mut self, content: impl Into<String>) {
        self.messages.push(Message::user(content));
    }

    /// Add an assistant message to the conversation
    pub fn add_assistant_message(&mut self, content: impl Into<String>) {
        self.messages.push(Message::assistant(content));
    }

    /// Get the last user message
    pub fn last_user_message(&self) -> Option<&str> {
        self.messages
            .iter()
            .rev()
            .find(|m| m.role == MessageRole::User)
            .map(|m| m.content.as_str())
    }

    /// Clear the conversation history
    pub fn clear(&mut self) {
        self.messages.clear();
    }

    /// Get message count
    pub fn len(&self) -> usize {
        self.messages.len()
    }

    /// Check if conversation is empty
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }
}

/// Response from RAG Q&A
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagResponse {
    /// The answer to the question
    pub answer: String,
    /// Sources used to generate the answer
    pub sources: Vec<RagSource>,
    /// The query that was asked
    pub query: String,
}

/// A source reference in a RAG response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagSource {
    /// File path
    pub file_path: String,
    /// Start line
    pub start_line: u32,
    /// End line
    pub end_line: u32,
    /// Relevance score
    pub score: f32,
    /// Content snippet
    pub snippet: String,
}

impl From<&SearchResult> for RagSource {
    fn from(result: &SearchResult) -> Self {
        Self {
            file_path: result.file_path.clone(),
            start_line: result.start_line,
            end_line: result.end_line,
            score: result.score,
            snippet: truncate_snippet(&result.content, 200),
        }
    }
}

/// RAG engine for question answering over codebase
pub struct RagEngine<'a> {
    openrouter: &'a OpenRouterClient,
    vector_store: &'a VectorStore,
    embedding_model: String,
    chat_model: String,
    top_k: usize,
}

impl<'a> RagEngine<'a> {
    /// Create a new RAG engine
    pub fn new(
        openrouter: &'a OpenRouterClient,
        vector_store: &'a VectorStore,
        embedding_model: impl Into<String>,
        chat_model: impl Into<String>,
    ) -> Self {
        Self {
            openrouter,
            vector_store,
            embedding_model: embedding_model.into(),
            chat_model: chat_model.into(),
            top_k: DEFAULT_TOP_K,
        }
    }

    /// Set the number of chunks to retrieve
    pub fn with_top_k(mut self, top_k: usize) -> Self {
        self.top_k = top_k;
        self
    }

    /// Ask a question about the codebase (non-streaming)
    pub async fn ask(&self, query: &str) -> WikiResult<RagResponse> {
        info!("RAG query: {}", query);

        // 1. Create embedding for the query
        let query_embedding = self
            .openrouter
            .create_embedding(query, &self.embedding_model)
            .await?;

        // 2. Search for similar chunks
        let search_results = self
            .vector_store
            .search_similar(&query_embedding, self.top_k)?;

        if search_results.is_empty() {
            return Ok(RagResponse {
                answer: "I couldn't find any relevant code in the indexed codebase to answer your question.".to_string(),
                sources: Vec::new(),
                query: query.to_string(),
            });
        }

        debug!("Found {} relevant chunks for query", search_results.len());

        // 3. Build context from search results
        let context = build_context(&search_results);
        let sources: Vec<RagSource> = search_results.iter().map(RagSource::from).collect();

        // 4. Create chat messages
        let messages = vec![
            ChatMessage::system(RAG_SYSTEM_PROMPT),
            ChatMessage::user(format_user_prompt(query, &context)),
        ];

        // 5. Get completion
        let answer = self
            .openrouter
            .chat_completion(messages, &self.chat_model, Some(0.3), Some(2048))
            .await?;

        Ok(RagResponse {
            answer,
            sources,
            query: query.to_string(),
        })
    }

    /// Ask a question with conversation history (non-streaming)
    pub async fn ask_with_history(
        &self,
        query: &str,
        conversation: &mut Conversation,
    ) -> WikiResult<RagResponse> {
        info!(
            "RAG query with history (conversation {}): {}",
            conversation.id, query
        );

        // Add user message to history
        conversation.add_user_message(query);

        // 1. Create embedding for the query
        let query_embedding = self
            .openrouter
            .create_embedding(query, &self.embedding_model)
            .await?;

        // 2. Search for similar chunks
        let search_results = self
            .vector_store
            .search_similar(&query_embedding, self.top_k)?;

        if search_results.is_empty() {
            let answer = "I couldn't find any relevant code in the indexed codebase to answer your question.".to_string();
            conversation.add_assistant_message(&answer);
            return Ok(RagResponse {
                answer,
                sources: Vec::new(),
                query: query.to_string(),
            });
        }

        // 3. Build context from search results
        let context = build_context(&search_results);
        let sources: Vec<RagSource> = search_results.iter().map(RagSource::from).collect();

        // 4. Create chat messages with history
        let mut messages = vec![ChatMessage::system(RAG_SYSTEM_PROMPT)];

        // Add conversation history (skip the last user message, we'll add it with context)
        for msg in conversation
            .messages
            .iter()
            .take(conversation.messages.len() - 1)
        {
            match msg.role {
                MessageRole::User => messages.push(ChatMessage::user(&msg.content)),
                MessageRole::Assistant => messages.push(ChatMessage::assistant(&msg.content)),
            }
        }

        // Add current query with context
        messages.push(ChatMessage::user(format_user_prompt(query, &context)));

        // 5. Get completion
        let answer = self
            .openrouter
            .chat_completion(messages, &self.chat_model, Some(0.3), Some(2048))
            .await?;

        // Add assistant response to history
        conversation.add_assistant_message(&answer);

        Ok(RagResponse {
            answer,
            sources,
            query: query.to_string(),
        })
    }

    /// Ask a question with streaming response
    pub async fn ask_stream(
        &self,
        query: &str,
    ) -> WikiResult<(mpsc::Receiver<WikiResult<String>>, Vec<RagSource>)> {
        info!("RAG streaming query: {}", query);

        // 1. Create embedding for the query
        let query_embedding = self
            .openrouter
            .create_embedding(query, &self.embedding_model)
            .await?;

        // 2. Search for similar chunks
        let search_results = self
            .vector_store
            .search_similar(&query_embedding, self.top_k)?;

        let sources: Vec<RagSource> = search_results.iter().map(RagSource::from).collect();

        if search_results.is_empty() {
            let (tx, rx) = mpsc::channel(1);
            tx.send(Ok("I couldn't find any relevant code in the indexed codebase to answer your question.".to_string()))
                .await
                .ok();
            return Ok((rx, sources));
        }

        debug!(
            "Found {} relevant chunks for streaming query",
            search_results.len()
        );

        // 3. Build context from search results
        let context = build_context(&search_results);

        // 4. Create chat messages
        let messages = vec![
            ChatMessage::system(RAG_SYSTEM_PROMPT),
            ChatMessage::user(format_user_prompt(query, &context)),
        ];

        // 5. Get streaming completion
        let stream = self
            .openrouter
            .chat_completion_stream(messages, &self.chat_model, Some(0.3), Some(2048))
            .await?;

        // Create channel for forwarding chunks
        let (tx, rx) = mpsc::channel(32);

        // Spawn task to forward stream chunks
        tokio::spawn(async move {
            tokio::pin!(stream);
            while let Some(result) = stream.next().await {
                if tx.send(result).await.is_err() {
                    break; // Receiver dropped
                }
            }
        });

        Ok((rx, sources))
    }

    /// Ask a question with conversation history and streaming response
    pub async fn ask_stream_with_history(
        &self,
        query: &str,
        conversation: &Conversation,
    ) -> WikiResult<(mpsc::Receiver<WikiResult<String>>, Vec<RagSource>)> {
        info!(
            "RAG streaming query with history (conversation {}): {}",
            conversation.id, query
        );

        // 1. Create embedding for the query
        let query_embedding = self
            .openrouter
            .create_embedding(query, &self.embedding_model)
            .await?;

        // 2. Search for similar chunks
        let search_results = self
            .vector_store
            .search_similar(&query_embedding, self.top_k)?;

        let sources: Vec<RagSource> = search_results.iter().map(RagSource::from).collect();

        if search_results.is_empty() {
            let (tx, rx) = mpsc::channel(1);
            tx.send(Ok("I couldn't find any relevant code in the indexed codebase to answer your question.".to_string()))
                .await
                .ok();
            return Ok((rx, sources));
        }

        // 3. Build context from search results
        let context = build_context(&search_results);

        // 4. Create chat messages with history
        let mut messages = vec![ChatMessage::system(RAG_SYSTEM_PROMPT)];

        // Add conversation history
        for msg in &conversation.messages {
            match msg.role {
                MessageRole::User => messages.push(ChatMessage::user(&msg.content)),
                MessageRole::Assistant => messages.push(ChatMessage::assistant(&msg.content)),
            }
        }

        // Add current query with context
        messages.push(ChatMessage::user(format_user_prompt(query, &context)));

        // 5. Get streaming completion
        let stream = self
            .openrouter
            .chat_completion_stream(messages, &self.chat_model, Some(0.3), Some(2048))
            .await?;

        // Create channel for forwarding chunks
        let (tx, rx) = mpsc::channel(32);

        // Spawn task to forward stream chunks
        tokio::spawn(async move {
            tokio::pin!(stream);
            while let Some(result) = stream.next().await {
                if tx.send(result).await.is_err() {
                    break;
                }
            }
        });

        Ok((rx, sources))
    }
}

/// Build context string from search results
fn build_context(results: &[SearchResult]) -> String {
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

        // Check if adding this chunk would exceed max length
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

/// Truncate a snippet to a maximum length
fn truncate_snippet(content: &str, max_len: usize) -> String {
    if content.len() <= max_len {
        content.to_string()
    } else {
        format!("{}...", &content[..max_len])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversation_new() {
        let conv = Conversation::new();
        assert!(!conv.id.is_empty());
        assert!(conv.is_empty());
    }

    #[test]
    fn test_conversation_messages() {
        let mut conv = Conversation::new();

        conv.add_user_message("What does this function do?");
        conv.add_assistant_message("This function processes data...");

        assert_eq!(conv.len(), 2);
        assert_eq!(conv.messages[0].role, MessageRole::User);
        assert_eq!(conv.messages[1].role, MessageRole::Assistant);
    }

    #[test]
    fn test_conversation_last_user_message() {
        let mut conv = Conversation::new();

        conv.add_user_message("First question");
        conv.add_assistant_message("First answer");
        conv.add_user_message("Second question");

        assert_eq!(conv.last_user_message(), Some("Second question"));
    }

    #[test]
    fn test_conversation_clear() {
        let mut conv = Conversation::new();
        conv.add_user_message("test");
        conv.clear();
        assert!(conv.is_empty());
    }

    #[test]
    fn test_message_constructors() {
        let user = Message::user("Hello");
        assert_eq!(user.role, MessageRole::User);
        assert_eq!(user.content, "Hello");

        let assistant = Message::assistant("Hi there");
        assert_eq!(assistant.role, MessageRole::Assistant);
        assert_eq!(assistant.content, "Hi there");
    }

    #[test]
    fn test_truncate_snippet() {
        let short = "short text";
        assert_eq!(truncate_snippet(short, 100), "short text");

        let long = "a".repeat(300);
        let truncated = truncate_snippet(&long, 100);
        assert!(truncated.ends_with("..."));
        assert_eq!(truncated.len(), 103); // 100 + "..."
    }

    #[test]
    fn test_build_context() {
        use crate::domain::chunk::ChunkType;
        use uuid::Uuid;

        let results = vec![
            SearchResult::new(
                Uuid::new_v4(),
                "src/lib.rs".to_string(),
                1,
                10,
                "fn main() {}".to_string(),
                ChunkType::Function,
                Some("rust".to_string()),
                0.95,
            ),
            SearchResult::new(
                Uuid::new_v4(),
                "src/utils.rs".to_string(),
                20,
                30,
                "fn helper() {}".to_string(),
                ChunkType::Function,
                Some("rust".to_string()),
                0.85,
            ),
        ];

        let context = build_context(&results);

        assert!(context.contains("src/lib.rs"));
        assert!(context.contains("lines 1-10"));
        assert!(context.contains("fn main() {}"));
        assert!(context.contains("src/utils.rs"));
        assert!(context.contains("```rust"));
    }

    #[test]
    fn test_format_user_prompt() {
        let query = "What does this do?";
        let context = "fn test() {}";

        let prompt = format_user_prompt(query, context);

        assert!(prompt.contains(query));
        assert!(prompt.contains(context));
        assert!(prompt.contains("Question:"));
        assert!(prompt.contains("Relevant Code:"));
    }

    #[test]
    fn test_rag_source_from_search_result() {
        use crate::domain::chunk::ChunkType;
        use uuid::Uuid;

        let result = SearchResult::new(
            Uuid::new_v4(),
            "src/main.rs".to_string(),
            5,
            15,
            "fn process_data() { /* implementation */ }".to_string(),
            ChunkType::Function,
            Some("rust".to_string()),
            0.92,
        );

        let source = RagSource::from(&result);

        assert_eq!(source.file_path, "src/main.rs");
        assert_eq!(source.start_line, 5);
        assert_eq!(source.end_line, 15);
        assert_eq!(source.score, 0.92);
        assert!(!source.snippet.is_empty());
    }

    #[test]
    fn test_rag_response_serialization() {
        let response = RagResponse {
            answer: "The function processes data...".to_string(),
            sources: vec![RagSource {
                file_path: "src/lib.rs".to_string(),
                start_line: 1,
                end_line: 10,
                score: 0.9,
                snippet: "fn test()".to_string(),
            }],
            query: "What does test do?".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("answer"));
        assert!(json.contains("sources"));
        assert!(json.contains("query"));
    }
}
