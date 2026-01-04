use std::time::Duration;

use reqwest::Client;
use tracing::{debug, error, info, warn};

use super::types::*;
use crate::error::{WikiError, WikiResult};

const DEFAULT_MAX_RETRIES: u32 = 3;
const INITIAL_BACKOFF_MS: u64 = 1000;
const MAX_BACKOFF_MS: u64 = 60000;

/// Client for OpenRouter API
#[derive(Clone)]
pub struct OpenRouterClient {
    client: Client,
    api_key: String,
    base_url: String,
}

impl OpenRouterClient {
    pub fn new(api_key: String, base_url: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url,
        }
    }

    async fn with_retry<T, F, Fut>(&self, operation: F, operation_name: &str) -> WikiResult<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = WikiResult<T>>,
    {
        let mut retries = 0;
        let mut backoff_ms = INITIAL_BACKOFF_MS;

        loop {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(WikiError::RateLimited { retry_after }) => {
                    if retries >= DEFAULT_MAX_RETRIES {
                        error!(
                            "{} failed after {} retries due to rate limiting",
                            operation_name, retries
                        );
                        return Err(WikiError::RateLimited { retry_after });
                    }

                    let wait_ms = retry_after
                        .map(|s| s * 1000)
                        .unwrap_or(backoff_ms)
                        .min(MAX_BACKOFF_MS);

                    warn!(
                        "{} rate limited, retrying in {}ms (attempt {}/{})",
                        operation_name,
                        wait_ms,
                        retries + 1,
                        DEFAULT_MAX_RETRIES
                    );

                    tokio::time::sleep(Duration::from_millis(wait_ms)).await;
                    retries += 1;
                    backoff_ms = (backoff_ms * 2).min(MAX_BACKOFF_MS);
                }
                Err(WikiError::OpenRouterApi {
                    ref message,
                    status_code: Some(code),
                }) if code >= 500 => {
                    if retries >= DEFAULT_MAX_RETRIES {
                        error!(
                            "{} failed after {} retries due to server error: {}",
                            operation_name, retries, message
                        );
                        return Err(WikiError::OpenRouterApi {
                            message: message.clone(),
                            status_code: Some(code),
                        });
                    }

                    warn!(
                        "{} server error ({}), retrying in {}ms (attempt {}/{})",
                        operation_name,
                        code,
                        backoff_ms,
                        retries + 1,
                        DEFAULT_MAX_RETRIES
                    );

                    tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
                    retries += 1;
                    backoff_ms = (backoff_ms * 2).min(MAX_BACKOFF_MS);
                }
                Err(e) => {
                    if retries > 0 {
                        info!(
                            "{} failed after {} retries: {}",
                            operation_name, retries, e
                        );
                    }
                    return Err(e);
                }
            }
        }
    }

    pub async fn create_embedding(&self, text: &str, model: &str) -> WikiResult<Vec<f32>> {
        let text = text.to_string();
        let model = model.to_string();

        self.with_retry(
            || async {
                let texts = vec![text.clone()];
                let embeddings = self.create_embeddings_batch_inner(&texts, &model).await?;
                embeddings.into_iter().next().ok_or_else(|| WikiError::OpenRouterApi {
                    message: "No embedding returned".to_string(),
                    status_code: None,
                })
            },
            "create_embedding",
        )
        .await
    }

    pub async fn create_embeddings_batch(
        &self,
        texts: &[String],
        model: &str,
    ) -> WikiResult<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let texts = texts.to_vec();
        let model = model.to_string();

        self.with_retry(
            || async { self.create_embeddings_batch_inner(&texts, &model).await },
            "create_embeddings_batch",
        )
        .await
    }

    async fn create_embeddings_batch_inner(
        &self,
        texts: &[String],
        model: &str,
    ) -> WikiResult<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        debug!(
            "Creating embeddings for {} texts with model {}",
            texts.len(),
            model
        );

        let request = EmbeddingRequest {
            model: model.to_string(),
            input: if texts.len() == 1 {
                EmbeddingInput::Single(texts[0].clone())
            } else {
                EmbeddingInput::Batch(texts.to_vec())
            },
        };

        let response = self
            .client
            .post(format!("{}/embeddings", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();

            // Check for rate limiting
            if status.as_u16() == 429 {
                warn!("Rate limited by OpenRouter");
                return Err(WikiError::RateLimited { retry_after: None });
            }

            // Try to parse error response
            if let Ok(error_resp) = serde_json::from_str::<OpenRouterError>(&error_text) {
                error!(
                    "OpenRouter API error: {} (type: {:?})",
                    error_resp.error.message, error_resp.error.error_type
                );
                return Err(WikiError::OpenRouterApi {
                    message: error_resp.error.message,
                    status_code: Some(status.as_u16()),
                });
            }

            return Err(WikiError::OpenRouterApi {
                message: error_text,
                status_code: Some(status.as_u16()),
            });
        }

        let embedding_response: EmbeddingResponse = response.json().await?;

        // Sort by index and extract embeddings
        let mut data = embedding_response.data;
        data.sort_by_key(|d| d.index);

        Ok(data.into_iter().map(|d| d.embedding).collect())
    }

    pub async fn chat_completion(
        &self,
        messages: Vec<ChatMessage>,
        model: &str,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> WikiResult<String> {
        let model = model.to_string();

        self.with_retry(
            || async {
                self.chat_completion_inner(messages.clone(), &model, temperature, max_tokens)
                    .await
            },
            "chat_completion",
        )
        .await
    }

    async fn chat_completion_inner(
        &self,
        messages: Vec<ChatMessage>,
        model: &str,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> WikiResult<String> {
        debug!(
            "Creating chat completion with {} messages, model {}",
            messages.len(),
            model
        );

        let request = ChatCompletionRequest {
            model: model.to_string(),
            messages,
            temperature,
            max_tokens,
            stream: Some(false),
        };

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();

            if status.as_u16() == 429 {
                warn!("Rate limited by OpenRouter");
                return Err(WikiError::RateLimited { retry_after: None });
            }

            if let Ok(error_resp) = serde_json::from_str::<OpenRouterError>(&error_text) {
                error!(
                    "OpenRouter API error: {} (type: {:?})",
                    error_resp.error.message, error_resp.error.error_type
                );
                return Err(WikiError::OpenRouterApi {
                    message: error_resp.error.message,
                    status_code: Some(status.as_u16()),
                });
            }

            return Err(WikiError::OpenRouterApi {
                message: error_text,
                status_code: Some(status.as_u16()),
            });
        }

        let chat_response: ChatCompletionResponse = response.json().await?;

        chat_response
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .ok_or_else(|| WikiError::OpenRouterApi {
                message: "No completion returned".to_string(),
                status_code: None,
            })
    }

    /// Create a streaming chat completion
    ///
    /// Returns a stream of content chunks. Use `eventsource-stream` to parse SSE.
    pub async fn chat_completion_stream(
        &self,
        messages: Vec<ChatMessage>,
        model: &str,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> WikiResult<impl futures::Stream<Item = WikiResult<String>>> {
        use eventsource_stream::Eventsource;
        use futures::StreamExt;

        debug!(
            "Creating streaming chat completion with {} messages, model {}",
            messages.len(),
            model
        );

        let request = ChatCompletionRequest {
            model: model.to_string(),
            messages,
            temperature,
            max_tokens,
            stream: Some(true),
        };

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();

            if status.as_u16() == 429 {
                return Err(WikiError::RateLimited { retry_after: None });
            }

            if let Ok(error_resp) = serde_json::from_str::<OpenRouterError>(&error_text) {
                return Err(WikiError::OpenRouterApi {
                    message: error_resp.error.message,
                    status_code: Some(status.as_u16()),
                });
            }

            return Err(WikiError::OpenRouterApi {
                message: error_text,
                status_code: Some(status.as_u16()),
            });
        }

        let byte_stream = response.bytes_stream().map(|r| {
            r.map_err(std::io::Error::other)
        });

        let event_stream = byte_stream.eventsource();

        // Map SSE events to content strings
        let content_stream = event_stream.filter_map(|event_result| async move {
            match event_result {
                Ok(event) => {
                    if event.data == "[DONE]" {
                        return None;
                    }

                    match serde_json::from_str::<ChatCompletionChunk>(&event.data) {
                        Ok(chunk) => {
                            if let Some(choice) = chunk.choices.first() {
                                if let Some(content) = &choice.delta.content {
                                    return Some(Ok(content.clone()));
                                }
                            }
                            None
                        }
                        Err(e) => {
                            warn!("Failed to parse SSE chunk: {}", e);
                            None
                        }
                    }
                }
                Err(e) => Some(Err(WikiError::OpenRouterApi {
                    message: format!("SSE error: {}", e),
                    status_code: None,
                })),
            }
        });

        Ok(content_stream)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = OpenRouterClient::new(
            "test-key".to_string(),
            "https://openrouter.ai/api/v1".to_string(),
        );
        assert_eq!(client.api_key, "test-key");
        assert_eq!(client.base_url, "https://openrouter.ai/api/v1");
    }
}
