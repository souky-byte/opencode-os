use serde::{Deserialize, Serialize};

/// Role in a chat message
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
}

/// A chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
}

impl ChatMessage {
    /// Create a system message
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: Role::System,
            content: content.into(),
        }
    }

    /// Create a user message
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: content.into(),
        }
    }

    /// Create an assistant message
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: content.into(),
        }
    }
}

/// Request for embeddings
#[derive(Debug, Serialize)]
pub struct EmbeddingRequest {
    pub model: String,
    pub input: EmbeddingInput,
}

/// Input for embeddings (single or batch)
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum EmbeddingInput {
    Single(String),
    Batch(Vec<String>),
}

/// Response from embeddings endpoint
#[derive(Debug, Deserialize)]
pub struct EmbeddingResponse {
    pub data: Vec<EmbeddingData>,
    pub model: String,
    pub usage: EmbeddingUsage,
}

/// Single embedding in response
#[derive(Debug, Deserialize)]
pub struct EmbeddingData {
    pub embedding: Vec<f32>,
    pub index: usize,
}

/// Usage statistics for embeddings
#[derive(Debug, Deserialize)]
pub struct EmbeddingUsage {
    pub prompt_tokens: u32,
    pub total_tokens: u32,
}

/// Request for chat completions
#[derive(Debug, Serialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}

/// Response from chat completions endpoint
#[derive(Debug, Deserialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub choices: Vec<ChatChoice>,
    pub model: String,
    pub usage: Option<ChatUsage>,
}

/// A choice in chat completion response
#[derive(Debug, Deserialize)]
pub struct ChatChoice {
    pub index: usize,
    pub message: ChatMessage,
    pub finish_reason: Option<String>,
}

/// Usage statistics for chat
#[derive(Debug, Deserialize)]
pub struct ChatUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Streaming chat completion chunk
#[derive(Debug, Deserialize)]
pub struct ChatCompletionChunk {
    pub id: String,
    pub choices: Vec<ChatChunkChoice>,
    pub model: String,
}

/// A choice in streaming chunk
#[derive(Debug, Deserialize)]
pub struct ChatChunkChoice {
    pub index: usize,
    pub delta: ChatDelta,
    pub finish_reason: Option<String>,
}

/// Delta content in streaming
#[derive(Debug, Deserialize)]
pub struct ChatDelta {
    #[serde(default)]
    pub role: Option<Role>,
    #[serde(default)]
    pub content: Option<String>,
}

/// Error response from OpenRouter
#[derive(Debug, Deserialize)]
pub struct OpenRouterError {
    pub error: OpenRouterErrorDetail,
}

/// Error detail
#[derive(Debug, Deserialize)]
pub struct OpenRouterErrorDetail {
    pub message: String,
    #[serde(rename = "type")]
    pub error_type: Option<String>,
    pub code: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_message_constructors() {
        let sys = ChatMessage::system("You are helpful");
        assert_eq!(sys.role, Role::System);

        let user = ChatMessage::user("Hello");
        assert_eq!(user.role, Role::User);

        let asst = ChatMessage::assistant("Hi there");
        assert_eq!(asst.role, Role::Assistant);
    }

    #[test]
    fn test_embedding_request_serialization() {
        let req = EmbeddingRequest {
            model: "text-embedding-3-small".to_string(),
            input: EmbeddingInput::Single("test".to_string()),
        };

        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("text-embedding-3-small"));
        assert!(json.contains("test"));
    }

    #[test]
    fn test_embedding_response_deserialization() {
        let json = r#"{
            "data": [{"embedding": [0.1, 0.2, 0.3], "index": 0}],
            "model": "text-embedding-3-small",
            "usage": {"prompt_tokens": 10, "total_tokens": 10}
        }"#;

        let resp: EmbeddingResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.data.len(), 1);
        assert_eq!(resp.data[0].embedding.len(), 3);
    }
}
