use opencode_client::apis::configuration::Configuration;
use opencode_client::apis::default_api;
use opencode_client::models::{
    Part, Session as OpenCodeSession, SessionCreateRequest, SessionPromptRequest,
    SessionPromptRequestPartsInner,
};
use std::path::Path;
use std::sync::Arc;
use tracing::{error, info};

use crate::activity_store::SessionActivityStore;
use crate::error::{OrchestratorError, Result};
use crate::services::MessageParser;

const DEFAULT_PROVIDER_ID: &str = "anthropic";
const DEFAULT_MODEL_ID: &str = "claude-sonnet-4-20250514";

pub struct OpenCodeClient {
    config: Arc<Configuration>,
    provider_id: String,
    model_id: String,
}

impl OpenCodeClient {
    pub fn new(config: Arc<Configuration>) -> Self {
        Self {
            config,
            provider_id: DEFAULT_PROVIDER_ID.to_string(),
            model_id: DEFAULT_MODEL_ID.to_string(),
        }
    }

    pub fn with_model(mut self, provider_id: &str, model_id: &str) -> Self {
        self.provider_id = provider_id.to_string();
        self.model_id = model_id.to_string();
        self
    }

    pub fn provider_id(&self) -> &str {
        &self.provider_id
    }

    pub fn model_id(&self) -> &str {
        &self.model_id
    }

    pub fn config(&self) -> &Arc<Configuration> {
        &self.config
    }

    pub async fn create_session(&self, working_dir: &Path) -> Result<OpenCodeSession> {
        let request = SessionCreateRequest {
            title: None,
            parent_id: None,
        };

        let directory = working_dir.to_str();
        info!(
            directory = ?directory,
            "Creating OpenCode session in directory"
        );

        default_api::session_create(&self.config, directory, Some(request))
            .await
            .map_err(|e| {
                error!(error = %e, directory = ?directory, "Failed to create OpenCode session");
                OrchestratorError::OpenCodeError(format!("Failed to create session: {}", e))
            })
    }

    pub async fn send_prompt(
        &self,
        session_id: &str,
        prompt: &str,
        working_dir: &Path,
        activity_store: Option<&SessionActivityStore>,
    ) -> Result<String> {
        let model = opencode_client::models::SessionPromptRequestModel {
            provider_id: self.provider_id.clone(),
            model_id: self.model_id.clone(),
        };

        let request = SessionPromptRequest {
            parts: vec![Self::create_text_part(prompt)],
            model: Some(Box::new(model)),
            message_id: None,
            agent: None,
            no_reply: None,
            tools: None,
            system: None,
            variant: None,
        };

        let directory = working_dir.to_str();
        let response = default_api::session_prompt(
            &self.config,
            session_id,
            directory,
            Some(request),
        )
        .await
        .map_err(|e| {
            error!(error = %e, directory = ?directory, "Failed to send message to OpenCode");
            OrchestratorError::OpenCodeError(format!("Failed to send message: {}", e))
        })?;

        if let Some(store) = activity_store {
            Self::push_activities_to_store(store, &response.parts);
        }

        Ok(MessageParser::extract_text_from_parts(&response.parts))
    }

    pub async fn send_prompt_sync(
        config: &Configuration,
        session_id: &str,
        prompt: &str,
        directory: Option<&str>,
    ) -> Result<String> {
        let request = SessionPromptRequest {
            parts: vec![Self::create_text_part(prompt)],
            model: None,
            message_id: None,
            agent: None,
            no_reply: None,
            tools: None,
            system: None,
            variant: None,
        };

        let response = default_api::session_prompt(config, session_id, directory, Some(request))
            .await
            .map_err(|e| {
                error!(error = %e, directory = ?directory, "Failed to send message to OpenCode");
                OrchestratorError::OpenCodeError(format!("Failed to send message: {}", e))
            })?;

        Ok(MessageParser::extract_text_from_parts(&response.parts))
    }

    pub async fn create_session_static(
        config: &Configuration,
        directory: Option<&str>,
    ) -> Result<OpenCodeSession> {
        let request = SessionCreateRequest {
            title: None,
            parent_id: None,
        };

        default_api::session_create(config, directory, Some(request))
            .await
            .map_err(|e| {
                OrchestratorError::OpenCodeError(format!("Failed to create session: {}", e))
            })
    }

    fn create_text_part(text: &str) -> SessionPromptRequestPartsInner {
        SessionPromptRequestPartsInner {
            r#type: opencode_client::models::session_prompt_request_parts_inner::Type::Text,
            text: text.to_string(),
            id: None,
            synthetic: None,
            ignored: None,
            time: None,
            metadata: None,
            mime: String::new(),
            filename: None,
            url: String::new(),
            source: None,
            name: String::new(),
            prompt: String::new(),
            description: String::new(),
            agent: String::new(),
            command: None,
        }
    }

    fn push_activities_to_store(store: &SessionActivityStore, parts: &[Part]) {
        for activity in MessageParser::parse_message_parts(parts) {
            store.push(activity);
        }
    }
}

impl Clone for OpenCodeClient {
    fn clone(&self) -> Self {
        Self {
            config: Arc::clone(&self.config),
            provider_id: self.provider_id.clone(),
            model_id: self.model_id.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opencode_client_creation() {
        let config = Arc::new(Configuration::new());
        let client = OpenCodeClient::new(config);
        assert_eq!(client.provider_id(), DEFAULT_PROVIDER_ID);
        assert_eq!(client.model_id(), DEFAULT_MODEL_ID);
    }

    #[test]
    fn test_opencode_client_with_model() {
        let config = Arc::new(Configuration::new());
        let client = OpenCodeClient::new(config).with_model("openai", "gpt-4");
        assert_eq!(client.provider_id(), "openai");
        assert_eq!(client.model_id(), "gpt-4");
    }
}
