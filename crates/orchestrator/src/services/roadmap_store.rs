//! Roadmap Store
//!
//! Handles persistence of roadmap data to JSON files in the `.opencode-studio/roadmap/` directory.
//!
//! File structure:
//! ```
//! .opencode-studio/
//! └── roadmap/
//!     ├── roadmap.json           # Main roadmap data
//!     └── roadmap_discovery.json # Discovery phase output
//! ```

use opencode_core::{Roadmap, RoadmapFeature, RoadmapFeatureStatus, UpdateFeatureRequest};
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{debug, error, info};

use crate::error::{OrchestratorError, Result};

const STUDIO_DIR: &str = ".opencode-studio";
const ROADMAP_DIR: &str = "roadmap";
const ROADMAP_FILE: &str = "roadmap.json";
const DISCOVERY_FILE: &str = "roadmap_discovery.json";

/// Store for roadmap data using JSON files
#[derive(Debug, Clone)]
pub struct RoadmapStore {
    project_path: PathBuf,
}

impl RoadmapStore {
    /// Create a new RoadmapStore for a project
    pub fn new(project_path: impl AsRef<Path>) -> Self {
        Self {
            project_path: project_path.as_ref().to_path_buf(),
        }
    }

    /// Get the roadmap directory path
    fn roadmap_dir(&self) -> PathBuf {
        self.project_path.join(STUDIO_DIR).join(ROADMAP_DIR)
    }

    /// Get the roadmap file path
    fn roadmap_path(&self) -> PathBuf {
        self.roadmap_dir().join(ROADMAP_FILE)
    }

    /// Get the discovery file path
    fn discovery_path(&self) -> PathBuf {
        self.roadmap_dir().join(DISCOVERY_FILE)
    }

    /// Ensure the roadmap directory exists
    pub async fn ensure_dir(&self) -> Result<()> {
        let dir = self.roadmap_dir();
        if !dir.exists() {
            fs::create_dir_all(&dir).await?;
            debug!("Created roadmap directory: {}", dir.display());
        }
        Ok(())
    }

    /// Check if a roadmap exists
    pub async fn exists(&self) -> bool {
        self.roadmap_path().exists()
    }

    /// Load the roadmap from disk
    pub async fn load(&self) -> Result<Option<Roadmap>> {
        let path = self.roadmap_path();

        if !path.exists() {
            debug!("No roadmap file found at {}", path.display());
            return Ok(None);
        }

        let content = fs::read_to_string(&path).await?;

        let roadmap: Roadmap = serde_json::from_str(&content).map_err(|e| {
            error!("Failed to parse roadmap JSON: {}", e);
            OrchestratorError::Serialization(format!("Failed to parse roadmap JSON: {}", e))
        })?;

        info!("Loaded roadmap with {} features", roadmap.features.len());
        Ok(Some(roadmap))
    }

    /// Save the roadmap to disk
    pub async fn save(&self, roadmap: &Roadmap) -> Result<()> {
        self.ensure_dir().await?;

        let path = self.roadmap_path();
        let content = serde_json::to_string_pretty(roadmap).map_err(|e| {
            OrchestratorError::Serialization(format!("Failed to serialize roadmap: {}", e))
        })?;

        fs::write(&path, content).await?;

        info!(
            "Saved roadmap to {} ({} features, {} phases)",
            path.display(),
            roadmap.features.len(),
            roadmap.phases.len()
        );
        Ok(())
    }

    /// Delete the roadmap
    pub async fn delete(&self) -> Result<()> {
        let path = self.roadmap_path();

        if path.exists() {
            fs::remove_file(&path).await?;
            info!("Deleted roadmap at {}", path.display());
        }

        Ok(())
    }

    /// Delete all roadmap-related files (cleanup for regeneration)
    /// This removes both roadmap.json and roadmap_discovery.json
    pub async fn cleanup_all(&self) -> Result<()> {
        let roadmap_path = self.roadmap_path();
        let discovery_path = self.discovery_path();

        if roadmap_path.exists() {
            fs::remove_file(&roadmap_path).await?;
            info!("Deleted roadmap file: {}", roadmap_path.display());
        }

        if discovery_path.exists() {
            fs::remove_file(&discovery_path).await?;
            info!("Deleted discovery file: {}", discovery_path.display());
        }

        debug!("Cleanup complete for roadmap directory");
        Ok(())
    }

    /// Check if generation appears to be incomplete
    /// (discovery exists but no roadmap, indicating interrupted generation)
    pub async fn is_incomplete(&self) -> bool {
        let has_discovery = self.discovery_path().exists();
        let has_roadmap = self.roadmap_path().exists();

        // Incomplete if we have discovery but no roadmap
        has_discovery && !has_roadmap
    }

    /// Save discovery data
    pub async fn save_discovery(&self, discovery: &serde_json::Value) -> Result<()> {
        self.ensure_dir().await?;

        let path = self.discovery_path();
        let content = serde_json::to_string_pretty(discovery).map_err(|e| {
            OrchestratorError::Serialization(format!("Failed to serialize discovery: {}", e))
        })?;

        fs::write(&path, content).await?;

        debug!("Saved discovery to {}", path.display());
        Ok(())
    }

    /// Load discovery data
    pub async fn load_discovery(&self) -> Result<Option<serde_json::Value>> {
        let path = self.discovery_path();

        if !path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&path).await?;

        let discovery: serde_json::Value = serde_json::from_str(&content).map_err(|e| {
            OrchestratorError::Serialization(format!("Failed to parse discovery JSON: {}", e))
        })?;

        Ok(Some(discovery))
    }

    /// Update a feature in the roadmap
    pub async fn update_feature(
        &self,
        feature_id: &str,
        update: &UpdateFeatureRequest,
    ) -> Result<RoadmapFeature> {
        let mut roadmap = self
            .load()
            .await?
            .ok_or_else(|| OrchestratorError::NotFound("Roadmap not found".to_string()))?;

        let feature_idx = roadmap
            .features
            .iter()
            .position(|f| f.id == feature_id)
            .ok_or_else(|| {
                OrchestratorError::NotFound(format!("Feature {} not found", feature_id))
            })?;

        if let Some(status) = update.status {
            roadmap.features[feature_idx].status = status;
        }
        if let Some(ref phase_id) = update.phase_id {
            roadmap.features[feature_idx].phase_id = phase_id.clone();
        }
        if let Some(priority) = update.priority {
            roadmap.features[feature_idx].priority = priority;
        }
        if let Some(ref task_id) = update.linked_task_id {
            roadmap.features[feature_idx].linked_task_id = Some(task_id.clone());
        }

        let updated_feature = roadmap.features[feature_idx].clone();
        roadmap.updated_at = chrono::Utc::now();

        self.save(&roadmap).await?;

        info!("Updated feature {}", feature_id);
        Ok(updated_feature)
    }

    /// Delete a feature from the roadmap
    pub async fn delete_feature(&self, feature_id: &str) -> Result<()> {
        let mut roadmap = self
            .load()
            .await?
            .ok_or_else(|| OrchestratorError::NotFound("Roadmap not found".to_string()))?;

        let initial_len = roadmap.features.len();
        roadmap.features.retain(|f| f.id != feature_id);

        if roadmap.features.len() == initial_len {
            return Err(OrchestratorError::NotFound(format!(
                "Feature {} not found",
                feature_id
            )));
        }

        // Also remove from phase feature lists
        for phase in &mut roadmap.phases {
            phase.features.retain(|id| id != feature_id);
        }

        // Update timestamp
        roadmap.updated_at = chrono::Utc::now();

        self.save(&roadmap).await?;

        info!("Deleted feature {}", feature_id);
        Ok(())
    }

    /// Link a feature to a task
    pub async fn link_feature_to_task(
        &self,
        feature_id: &str,
        task_id: &str,
    ) -> Result<RoadmapFeature> {
        self.update_feature(
            feature_id,
            &UpdateFeatureRequest {
                status: Some(RoadmapFeatureStatus::InProgress),
                linked_task_id: Some(task_id.to_string()),
                ..Default::default()
            },
        )
        .await
    }

    /// Mark a feature as done (called when linked task is completed)
    pub async fn mark_feature_done(&self, feature_id: &str) -> Result<RoadmapFeature> {
        self.update_feature(
            feature_id,
            &UpdateFeatureRequest {
                status: Some(RoadmapFeatureStatus::Done),
                ..Default::default()
            },
        )
        .await
    }

    /// Get features by status
    pub async fn get_features_by_status(
        &self,
        status: RoadmapFeatureStatus,
    ) -> Result<Vec<RoadmapFeature>> {
        let roadmap = self.load().await?;

        Ok(roadmap
            .map(|r| {
                r.features
                    .into_iter()
                    .filter(|f| f.status == status)
                    .collect()
            })
            .unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opencode_core::{RoadmapComplexity, RoadmapImpact, RoadmapPriority};
    use tempfile::TempDir;

    async fn create_test_store() -> (RoadmapStore, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let store = RoadmapStore::new(temp_dir.path());
        (store, temp_dir)
    }

    fn create_test_roadmap() -> Roadmap {
        let mut roadmap = Roadmap::new("Test Project", "Build amazing things");
        roadmap.features.push(RoadmapFeature {
            id: "feature-1".to_string(),
            title: "Feature 1".to_string(),
            description: "Description".to_string(),
            rationale: "Rationale".to_string(),
            priority: RoadmapPriority::Must,
            complexity: RoadmapComplexity::Medium,
            impact: RoadmapImpact::High,
            phase_id: "phase-1".to_string(),
            dependencies: Vec::new(),
            status: RoadmapFeatureStatus::UnderReview,
            acceptance_criteria: vec!["Criteria 1".to_string()],
            user_stories: vec!["As a user...".to_string()],
            linked_task_id: None,
        });
        roadmap
    }

    #[tokio::test]
    async fn test_save_and_load() {
        let (store, _temp) = create_test_store().await;
        let roadmap = create_test_roadmap();

        // Save
        store.save(&roadmap).await.unwrap();

        // Load
        let loaded = store.load().await.unwrap();
        assert!(loaded.is_some());

        let loaded = loaded.unwrap();
        assert_eq!(loaded.project_name, "Test Project");
        assert_eq!(loaded.features.len(), 1);
    }

    #[tokio::test]
    async fn test_exists() {
        let (store, _temp) = create_test_store().await;

        // Initially doesn't exist
        assert!(!store.exists().await);

        // After save, exists
        let roadmap = create_test_roadmap();
        store.save(&roadmap).await.unwrap();
        assert!(store.exists().await);
    }

    #[tokio::test]
    async fn test_update_feature() {
        let (store, _temp) = create_test_store().await;
        let roadmap = create_test_roadmap();
        store.save(&roadmap).await.unwrap();

        // Update feature status
        let updated = store
            .update_feature(
                "feature-1",
                &UpdateFeatureRequest {
                    status: Some(RoadmapFeatureStatus::Planned),
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        assert_eq!(updated.status, RoadmapFeatureStatus::Planned);

        // Verify persistence
        let loaded = store.load().await.unwrap().unwrap();
        let feature = loaded.feature_by_id("feature-1").unwrap();
        assert_eq!(feature.status, RoadmapFeatureStatus::Planned);
    }

    #[tokio::test]
    async fn test_delete_feature() {
        let (store, _temp) = create_test_store().await;
        let roadmap = create_test_roadmap();
        store.save(&roadmap).await.unwrap();

        // Delete feature
        store.delete_feature("feature-1").await.unwrap();

        // Verify deletion
        let loaded = store.load().await.unwrap().unwrap();
        assert!(loaded.features.is_empty());
    }

    #[tokio::test]
    async fn test_link_feature_to_task() {
        let (store, _temp) = create_test_store().await;
        let roadmap = create_test_roadmap();
        store.save(&roadmap).await.unwrap();

        // Link feature to task
        let updated = store
            .link_feature_to_task("feature-1", "task-123")
            .await
            .unwrap();

        assert_eq!(updated.status, RoadmapFeatureStatus::InProgress);
        assert_eq!(updated.linked_task_id, Some("task-123".to_string()));
    }
}
