//! Roadmap Service
//!
//! Orchestrates roadmap generation using OpenCode sessions.
//! Handles two phases:
//! 1. Discovery - Analyze project to understand purpose, audience, and current state
//! 2. Features - Generate prioritized feature roadmap based on discovery data

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use events::{Event, EventBus, EventEnvelope};
use opencode_client::apis::configuration::Configuration;
use opencode_core::{Roadmap, RoadmapGenerationStatus};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::error::{OrchestratorError, Result};
use crate::services::opencode_client::OpenCodeClient;
use crate::services::roadmap_prompts::{
    get_features_prompt_with_discovery, ROADMAP_DISCOVERY_PROMPT,
};
use crate::services::roadmap_store::RoadmapStore;

pub type SharedRoadmapStatus = Arc<RwLock<RoadmapGenerationStatus>>;
pub type SharedGenerationId = Arc<AtomicU64>;

pub struct RoadmapService {
    config: Arc<Configuration>,
    project_path: PathBuf,
    store: RoadmapStore,
    status: SharedRoadmapStatus,
    provider_id: String,
    model_id: String,
    event_bus: Option<EventBus>,
    /// Generation ID this service was created for
    my_generation_id: u64,
    /// Shared generation ID counter - if it doesn't match my_generation_id, this task is stale
    global_generation_id: Option<SharedGenerationId>,
}

impl RoadmapService {
    pub fn new(
        config: Arc<Configuration>,
        project_path: impl AsRef<Path>,
        status: SharedRoadmapStatus,
    ) -> Self {
        let project_path = project_path.as_ref().to_path_buf();
        Self {
            config,
            store: RoadmapStore::new(&project_path),
            project_path,
            status,
            provider_id: "anthropic".to_string(),
            model_id: "claude-sonnet-4-20250514".to_string(),
            event_bus: None,
            my_generation_id: 0,
            global_generation_id: None,
        }
    }

    /// Set generation ID for cancellation support
    pub fn with_generation_id(mut self, my_id: u64, global_id: SharedGenerationId) -> Self {
        self.my_generation_id = my_id;
        self.global_generation_id = Some(global_id);
        self
    }

    /// Check if this generation is still current (not cancelled by a newer generation)
    fn is_current_generation(&self) -> bool {
        match &self.global_generation_id {
            Some(global_id) => global_id.load(Ordering::SeqCst) == self.my_generation_id,
            None => false, // No generation tracking means old task - don't allow updates
        }
    }

    pub fn with_model(mut self, provider_id: &str, model_id: &str) -> Self {
        self.provider_id = provider_id.to_string();
        self.model_id = model_id.to_string();
        self
    }

    /// Set event bus for progress notifications via SSE
    pub fn with_event_bus(mut self, event_bus: EventBus) -> Self {
        self.event_bus = Some(event_bus);
        self
    }

    /// Get the current generation status
    pub async fn status(&self) -> RoadmapGenerationStatus {
        self.status.read().await.clone()
    }

    /// Check if a roadmap exists
    pub async fn exists(&self) -> bool {
        self.store.exists().await
    }

    /// Load the current roadmap
    pub async fn load(&self) -> Result<Option<Roadmap>> {
        self.store.load().await
    }

    /// Generate a new roadmap
    ///
    /// This runs two phases:
    /// 1. Discovery - Analyzes the project and produces discovery JSON
    /// 2. Features - Uses discovery to generate the roadmap
    ///
    /// If `force` is true, will regenerate even if a roadmap exists
    pub async fn generate(&self, force: bool) -> Result<Roadmap> {
        // Always start by resetting status to ensure clean state
        debug!("Resetting roadmap generation status to idle");
        self.set_status(RoadmapGenerationStatus::idle()).await;

        // Check for incomplete previous generation and clean up
        if self.store.is_incomplete().await {
            info!("Detected incomplete previous generation, cleaning up partial files");
            self.store.cleanup_all().await?;
        }

        // Check if roadmap already exists
        if !force && self.exists().await {
            info!("Roadmap already exists, skipping generation (use force=true to regenerate)");
            return self
                .load()
                .await?
                .ok_or_else(|| OrchestratorError::NotFound("Roadmap not found".to_string()));
        }

        // If force=true, clean up all existing files before regenerating
        if force {
            info!("Force regeneration requested, cleaning up existing files");
            self.store.cleanup_all().await?;
        }

        info!(
            project_path = %self.project_path.display(),
            provider = %self.provider_id,
            model = %self.model_id,
            "Starting roadmap generation with configured model"
        );
        debug!(
            "Using model {}/{} for roadmap generation",
            self.provider_id, self.model_id
        );

        // Update status: Analyzing
        self.set_status_with_event(RoadmapGenerationStatus::analyzing())
            .await;

        // Create OpenCode client
        let client = OpenCodeClient::new(Arc::clone(&self.config))
            .with_model(&self.provider_id, &self.model_id);

        // Phase 1: Discovery
        debug!("Starting discovery phase");
        let discovery = match self.run_discovery_phase(&client).await {
            Ok(d) => {
                debug!("Discovery phase completed successfully");
                d
            }
            Err(e) => {
                error!(error = %e, "Discovery phase failed");
                self.set_status_with_event(RoadmapGenerationStatus::error(e.to_string()))
                    .await;
                return Err(e);
            }
        };

        // Phase 2: Features (status updated inside run_features_phase after session creation)
        debug!("Starting features phase");
        let roadmap = match self.run_features_phase(&client, &discovery).await {
            Ok(r) => {
                debug!("Features phase completed successfully");
                r
            }
            Err(e) => {
                error!(error = %e, "Features phase failed");
                self.set_status_with_event(RoadmapGenerationStatus::error(e.to_string()))
                    .await;
                return Err(e);
            }
        };

        // Save roadmap
        debug!("Saving roadmap to disk");
        self.store.save(&roadmap).await?;

        // Update status: Complete
        self.set_status_with_event(RoadmapGenerationStatus::complete())
            .await;

        info!(
            features = roadmap.features.len(),
            phases = roadmap.phases.len(),
            "Roadmap generation complete"
        );

        Ok(roadmap)
    }

    /// Run the discovery phase
    async fn run_discovery_phase(&self, client: &OpenCodeClient) -> Result<serde_json::Value> {
        info!("Running discovery phase");

        // Create a new session for discovery (still in "analyzing" phase)
        let session = client.create_session(&self.project_path).await?;
        debug!(session_id = %session.id, "Created discovery session");

        // Now transition to "discovering" - about to send the actual prompt
        self.set_status_with_event(RoadmapGenerationStatus::discovering())
            .await;

        // Send discovery prompt
        let response = client
            .send_prompt(
                &session.id,
                ROADMAP_DISCOVERY_PROMPT,
                &self.project_path,
                None,
            )
            .await?;

        debug!(response_len = response.len(), "Got discovery response");

        // Parse discovery JSON from response
        let discovery = self.extract_json_from_response(&response)?;

        // Save discovery data
        self.store.save_discovery(&discovery).await?;

        info!("Discovery phase complete");
        Ok(discovery)
    }

    /// Run the features phase
    async fn run_features_phase(
        &self,
        client: &OpenCodeClient,
        discovery: &serde_json::Value,
    ) -> Result<Roadmap> {
        info!("Running features phase");

        // Create a new session for features (still in "discovering" phase during setup)
        let session = client.create_session(&self.project_path).await?;
        debug!(session_id = %session.id, "Created features session");

        // Build features prompt with discovery data
        let discovery_json = serde_json::to_string_pretty(discovery).map_err(|e| {
            OrchestratorError::Serialization(format!("Failed to serialize discovery: {}", e))
        })?;
        let prompt = get_features_prompt_with_discovery(&discovery_json);

        // Now transition to "generating" - about to send the actual prompt
        self.set_status_with_event(RoadmapGenerationStatus::generating())
            .await;

        // Send features prompt
        let response = client
            .send_prompt(&session.id, &prompt, &self.project_path, None)
            .await?;

        debug!(response_len = response.len(), "Got features response");

        // Parse roadmap JSON from response
        let roadmap_json = self.extract_json_from_response(&response)?;

        // Convert JSON to Roadmap struct
        let roadmap: Roadmap = serde_json::from_value(roadmap_json).map_err(|e| {
            error!(error = %e, "Failed to parse roadmap from response");
            OrchestratorError::Serialization(format!("Failed to parse roadmap JSON: {}", e))
        })?;

        info!(
            features = roadmap.features.len(),
            phases = roadmap.phases.len(),
            "Features phase complete"
        );
        Ok(roadmap)
    }

    /// Extract JSON from a response that may contain markdown
    fn extract_json_from_response(&self, response: &str) -> Result<serde_json::Value> {
        // Try to find JSON in code blocks first
        if let Some(json) = self.extract_json_from_codeblock(response) {
            return serde_json::from_str(&json).map_err(|e| {
                error!(error = %e, json = %json, "Failed to parse JSON from code block");
                OrchestratorError::Serialization(format!("Invalid JSON in code block: {}", e))
            });
        }

        // Try to find raw JSON (starts with { and ends with })
        if let Some(json) = self.extract_raw_json(response) {
            return serde_json::from_str(&json).map_err(|e| {
                error!(error = %e, "Failed to parse raw JSON");
                OrchestratorError::Serialization(format!("Invalid raw JSON: {}", e))
            });
        }

        // Last resort: try to parse the entire response
        serde_json::from_str(response).map_err(|e| {
            warn!(error = %e, response_len = response.len(), "Could not find valid JSON in response");
            OrchestratorError::Serialization(format!(
                "No valid JSON found in response. Error: {}",
                e
            ))
        })
    }

    /// Extract JSON from a markdown code block
    fn extract_json_from_codeblock(&self, text: &str) -> Option<String> {
        // Look for ```json ... ``` or ``` ... ```
        let patterns = ["```json\n", "```JSON\n", "```\n"];

        for pattern in patterns {
            if let Some(start_idx) = text.find(pattern) {
                let json_start = start_idx + pattern.len();
                if let Some(end_idx) = text[json_start..].find("```") {
                    let json = text[json_start..json_start + end_idx].trim();
                    return Some(json.to_string());
                }
            }
        }

        None
    }

    /// Extract raw JSON that starts with { and ends with }
    fn extract_raw_json(&self, text: &str) -> Option<String> {
        let start = text.find('{')?;
        let end = text.rfind('}')?;

        if start < end {
            Some(text[start..=end].to_string())
        } else {
            None
        }
    }

    /// Update the generation status (internal, without event)
    async fn set_status(&self, status: RoadmapGenerationStatus) {
        let is_current = self.is_current_generation();
        let has_global_id = self.global_generation_id.is_some();

        if !is_current {
            warn!(
                my_id = self.my_generation_id,
                has_global_id = has_global_id,
                phase = ?status.phase,
                progress = status.progress,
                "BLOCKED status update - generation cancelled or stale"
            );
            return;
        }
        debug!(
            my_id = self.my_generation_id,
            phase = ?status.phase,
            progress = status.progress,
            "Status update"
        );
        *self.status.write().await = status;
    }

    /// Update the generation status and emit SSE event
    async fn set_status_with_event(&self, status: RoadmapGenerationStatus) {
        let is_current = self.is_current_generation();
        let has_global_id = self.global_generation_id.is_some();

        if !is_current {
            warn!(
                my_id = self.my_generation_id,
                has_global_id = has_global_id,
                phase = ?status.phase,
                progress = status.progress,
                "BLOCKED status update with event - generation cancelled or stale"
            );
            return;
        }

        let phase_str = status.phase.as_str().to_string();
        let progress = status.progress;
        let message = status.message.clone();

        debug!(
            phase = %phase_str,
            progress = progress,
            message = %message,
            "Status update with event"
        );

        *self.status.write().await = status;

        // Emit SSE event if event bus is available
        if let Some(ref bus) = self.event_bus {
            bus.publish(EventEnvelope::new(Event::RoadmapGenerationProgress {
                phase: phase_str,
                progress,
                message,
            }));
        }
    }

    /// Delete the roadmap
    pub async fn delete(&self) -> Result<()> {
        self.store.delete().await
    }

    /// Get the roadmap store for direct access
    pub fn store(&self) -> &RoadmapStore {
        &self.store
    }
}

impl Clone for RoadmapService {
    fn clone(&self) -> Self {
        Self {
            config: Arc::clone(&self.config),
            project_path: self.project_path.clone(),
            store: RoadmapStore::new(&self.project_path),
            status: Arc::clone(&self.status),
            provider_id: self.provider_id.clone(),
            model_id: self.model_id.clone(),
            event_bus: self.event_bus.clone(),
            my_generation_id: self.my_generation_id,
            global_generation_id: self.global_generation_id.clone(),
        }
    }
}

#[cfg(test)]
fn create_test_status() -> SharedRoadmapStatus {
    Arc::new(RwLock::new(RoadmapGenerationStatus::idle()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json_from_codeblock() {
        let service = RoadmapService::new(
            Arc::new(Configuration::new()),
            std::path::Path::new("/tmp"),
            create_test_status(),
        );

        let text = r#"Here is the JSON:
```json
{"name": "test", "value": 123}
```
That's all!"#;

        let json = service.extract_json_from_codeblock(text);
        assert!(json.is_some());
        assert_eq!(json.unwrap(), r#"{"name": "test", "value": 123}"#);
    }

    #[test]
    fn test_extract_json_from_codeblock_no_lang() {
        let service = RoadmapService::new(
            Arc::new(Configuration::new()),
            std::path::Path::new("/tmp"),
            create_test_status(),
        );

        let text = r#"Here is the JSON:
```
{"name": "test"}
```"#;

        let json = service.extract_json_from_codeblock(text);
        assert!(json.is_some());
        assert_eq!(json.unwrap(), r#"{"name": "test"}"#);
    }

    #[test]
    fn test_extract_raw_json() {
        let service = RoadmapService::new(
            Arc::new(Configuration::new()),
            std::path::Path::new("/tmp"),
            create_test_status(),
        );

        let text =
            r#"Some preamble text {"name": "test", "nested": {"a": 1}} and some trailing text"#;

        let json = service.extract_raw_json(text);
        assert!(json.is_some());
        assert_eq!(json.unwrap(), r#"{"name": "test", "nested": {"a": 1}}"#);
    }

    #[test]
    fn test_extract_json_from_response() {
        let service = RoadmapService::new(
            Arc::new(Configuration::new()),
            std::path::Path::new("/tmp"),
            create_test_status(),
        );

        // Test with code block
        let text1 = r#"```json
{"name": "test"}
```"#;
        let result1 = service.extract_json_from_response(text1);
        assert!(result1.is_ok());
        assert_eq!(result1.unwrap()["name"], "test");

        // Test with raw JSON
        let text2 = r#"The result is {"name": "raw"} here"#;
        let result2 = service.extract_json_from_response(text2);
        assert!(result2.is_ok());
        assert_eq!(result2.unwrap()["name"], "raw");
    }
}
