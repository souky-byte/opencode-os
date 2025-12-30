use reqwest::Client;

use crate::error::{OpenCodeError, Result};
use crate::types::{
    CreateSessionRequest, Message, MessageResponse, SendMessageRequest, Session,
    SessionListResponse, SessionMessagesResponse,
};

pub struct OpenCodeClient {
    base_url: String,
    client: Client,
}

impl OpenCodeClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            client: Client::new(),
        }
    }

    pub fn with_client(base_url: impl Into<String>, client: Client) -> Self {
        Self {
            base_url: base_url.into(),
            client,
        }
    }

    pub async fn create_session(&self, title: Option<String>) -> Result<Session> {
        let request = CreateSessionRequest {
            title,
            parent_id: None,
        };

        let response = self
            .client
            .post(format!("{}/session", self.base_url))
            .json(&request)
            .send()
            .await?;

        self.handle_response(response).await
    }

    pub async fn get_session(&self, session_id: &str) -> Result<Session> {
        let response = self
            .client
            .get(format!("{}/session/{}", self.base_url, session_id))
            .send()
            .await?;

        self.handle_response(response).await
    }

    pub async fn list_sessions(&self) -> Result<Vec<Session>> {
        let response = self
            .client
            .get(format!("{}/sessions", self.base_url))
            .send()
            .await?;

        let list: SessionListResponse = self.handle_response(response).await?;
        Ok(list.sessions)
    }

    pub async fn send_message(
        &self,
        session_id: &str,
        prompt: &str,
        model: Option<&str>,
    ) -> Result<MessageResponse> {
        let mut request = SendMessageRequest::new(prompt);
        if let Some(m) = model {
            request = request.with_model(m);
        }

        let response = self
            .client
            .post(format!("{}/session/{}/message", self.base_url, session_id))
            .json(&request)
            .send()
            .await?;

        self.handle_response(response).await
    }

    pub async fn send_message_async(&self, session_id: &str, prompt: &str) -> Result<()> {
        let request = SendMessageRequest::new(prompt);

        let response = self
            .client
            .post(format!(
                "{}/session/{}/prompt_async",
                self.base_url, session_id
            ))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(OpenCodeError::InvalidResponse(format!(
                "Status {}: {}",
                status, body
            )));
        }

        Ok(())
    }

    pub async fn abort_session(&self, session_id: &str) -> Result<()> {
        let response = self
            .client
            .post(format!("{}/session/{}/abort", self.base_url, session_id))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(OpenCodeError::InvalidResponse(format!(
                "Status {}: {}",
                status, body
            )));
        }

        Ok(())
    }

    pub async fn get_messages(&self, session_id: &str) -> Result<Vec<Message>> {
        let response = self
            .client
            .get(format!("{}/session/{}/messages", self.base_url, session_id))
            .send()
            .await?;

        let messages: SessionMessagesResponse = self.handle_response(response).await?;
        Ok(messages.messages)
    }

    async fn handle_response<T: serde::de::DeserializeOwned>(
        &self,
        response: reqwest::Response,
    ) -> Result<T> {
        let status = response.status();

        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(OpenCodeError::SessionNotFound(
                "Resource not found".to_string(),
            ));
        }

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(OpenCodeError::InvalidResponse(format!(
                "Status {}: {}",
                status, body
            )));
        }

        let body = response.json().await?;
        Ok(body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = OpenCodeClient::new("http://localhost:4096");
        assert_eq!(client.base_url, "http://localhost:4096");
    }

    #[test]
    fn test_send_message_request() {
        let request = SendMessageRequest::new("Hello world");
        assert_eq!(request.parts.len(), 1);
        assert!(request.model.is_none());

        let request_with_model = SendMessageRequest::new("Hello").with_model("gpt-4");
        assert_eq!(request_with_model.model, Some("gpt-4".to_string()));
    }
}
