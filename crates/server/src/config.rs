use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;
use tracing::{debug, warn};
use utoipa::ToSchema;

const CONFIG_FILE: &str = ".opencode-studio/config.json";

/// Model selection for a specific phase
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct ModelSelection {
    /// Provider ID (e.g., "anthropic", "openai")
    pub provider_id: String,
    /// Model ID (e.g., "claude-sonnet-4-20250514", "gpt-4o")
    pub model_id: String,
}

/// Per-phase model configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct PhaseModels {
    /// Model for planning phase
    #[serde(skip_serializing_if = "Option::is_none")]
    pub planning: Option<ModelSelection>,
    /// Model for implementation phase
    #[serde(skip_serializing_if = "Option::is_none")]
    pub implementation: Option<ModelSelection>,
    /// Model for review phase
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review: Option<ModelSelection>,
    /// Model for fix phase
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix: Option<ModelSelection>,
}

/// Project-level configuration stored in .opencode-studio/config.json
#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct ProjectConfig {
    /// Per-phase model settings
    #[serde(default)]
    pub phase_models: PhaseModels,
}

impl ProjectConfig {
    /// Read config from project directory
    pub async fn read(project_path: &Path) -> Self {
        let config_path = project_path.join(CONFIG_FILE);

        if !config_path.exists() {
            debug!(path = %config_path.display(), "Config file does not exist, using defaults");
            return Self::default();
        }

        match fs::read_to_string(&config_path).await {
            Ok(content) => match serde_json::from_str(&content) {
                Ok(config) => {
                    debug!(path = %config_path.display(), "Config loaded successfully");
                    config
                }
                Err(e) => {
                    warn!(path = %config_path.display(), error = %e, "Failed to parse config, using defaults");
                    Self::default()
                }
            },
            Err(e) => {
                warn!(path = %config_path.display(), error = %e, "Failed to read config file, using defaults");
                Self::default()
            }
        }
    }

    /// Write config to project directory
    pub async fn write(&self, project_path: &Path) -> std::io::Result<()> {
        let config_dir = project_path.join(".opencode-studio");
        let config_path = config_dir.join("config.json");

        if !config_dir.exists() {
            fs::create_dir_all(&config_dir).await?;
        }

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        fs::write(&config_path, content).await?;
        debug!(path = %config_path.display(), "Config saved successfully");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_config_default() {
        let config = ProjectConfig::default();
        assert!(config.phase_models.planning.is_none());
        assert!(config.phase_models.implementation.is_none());
        assert!(config.phase_models.review.is_none());
        assert!(config.phase_models.fix.is_none());
    }

    #[tokio::test]
    async fn test_config_read_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let config = ProjectConfig::read(temp_dir.path()).await;
        assert!(config.phase_models.planning.is_none());
    }

    #[tokio::test]
    async fn test_config_write_and_read() {
        let temp_dir = TempDir::new().unwrap();

        let config = ProjectConfig {
            phase_models: PhaseModels {
                planning: Some(ModelSelection {
                    provider_id: "anthropic".to_string(),
                    model_id: "claude-sonnet-4-20250514".to_string(),
                }),
                implementation: None,
                review: Some(ModelSelection {
                    provider_id: "openai".to_string(),
                    model_id: "gpt-4o".to_string(),
                }),
                fix: None,
            },
        };

        config.write(temp_dir.path()).await.unwrap();

        let loaded = ProjectConfig::read(temp_dir.path()).await;
        assert!(loaded.phase_models.planning.is_some());
        assert_eq!(
            loaded.phase_models.planning.as_ref().unwrap().provider_id,
            "anthropic"
        );
        assert!(loaded.phase_models.review.is_some());
        assert_eq!(
            loaded.phase_models.review.as_ref().unwrap().provider_id,
            "openai"
        );
    }
}
