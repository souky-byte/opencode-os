//! Wiki synchronization service
//!
//! Provides automatic synchronization of wiki content when code changes.

use std::path::Path;
use std::sync::Arc;

use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};

use crate::domain::index_status::{IndexProgress, IndexState, IndexStatus};
use crate::error::WikiResult;
use crate::generator::WikiGenerator;
use crate::indexer::CodeIndexer;
use crate::openrouter::OpenRouterClient;
use crate::vector_store::VectorStore;
use crate::WikiConfig;

pub struct WikiSyncService {
    config: WikiConfig,
    openrouter: Arc<OpenRouterClient>,
    #[allow(clippy::arc_with_non_send_sync)]
    vector_store: Arc<VectorStore>,
}

impl WikiSyncService {
    #[allow(clippy::arc_with_non_send_sync)]
    pub fn new(config: WikiConfig) -> WikiResult<Self> {
        let openrouter = Arc::new(OpenRouterClient::new(
            config.openrouter_api_key.clone(),
            config.api_base_url.clone(),
        ));

        let vector_store = Arc::new(VectorStore::new(&config.db_path)?);

        Ok(Self {
            config,
            openrouter,
            vector_store,
        })
    }

    pub fn from_parts(
        config: WikiConfig,
        openrouter: Arc<OpenRouterClient>,
        #[allow(clippy::arc_with_non_send_sync)] vector_store: Arc<VectorStore>,
    ) -> Self {
        Self {
            config,
            openrouter,
            vector_store,
        }
    }

    pub fn needs_reindex(&self, branch: &str, current_commit: &str) -> WikiResult<bool> {
        match self.vector_store.get_index_status(branch)? {
            Some(status) => {
                if status.state != IndexState::Indexed {
                    debug!(
                        "Branch '{}' needs reindex: state is {:?}",
                        branch, status.state
                    );
                    return Ok(true);
                }
                let needs = status.last_commit_sha.as_deref() != Some(current_commit);
                if needs {
                    debug!(
                        "Branch '{}' needs reindex: commit changed from {:?} to {}",
                        branch, status.last_commit_sha, current_commit
                    );
                }
                Ok(needs)
            }
            None => {
                debug!("Branch '{}' needs reindex: no index status found", branch);
                Ok(true)
            }
        }
    }

    pub async fn sync_if_needed(
        &self,
        root_path: &Path,
        branch: &str,
        current_commit: &str,
        progress_tx: Option<broadcast::Sender<IndexProgress>>,
    ) -> WikiResult<Option<IndexStatus>> {
        if !self.needs_reindex(branch, current_commit)? {
            info!("Branch '{}' is up to date, skipping sync", branch);
            return Ok(None);
        }

        info!(
            "Branch '{}' needs sync, starting full indexing and generation",
            branch
        );

        self.sync(root_path, branch, current_commit, progress_tx)
            .await
            .map(Some)
    }

    pub async fn sync(
        &self,
        root_path: &Path,
        branch: &str,
        current_commit: &str,
        progress_tx: Option<broadcast::Sender<IndexProgress>>,
    ) -> WikiResult<IndexStatus> {
        let start_time = std::time::Instant::now();

        let indexer = CodeIndexer::new(
            self.openrouter.clone(),
            self.vector_store.clone(),
            self.config.embedding_model.clone(),
            self.config.max_chunk_tokens,
            self.config.chunk_overlap,
        );

        let index_status = indexer
            .index_branch(root_path, branch, current_commit, progress_tx.clone())
            .await?;

        if index_status.state != IndexState::Indexed {
            warn!(
                "Indexing did not complete successfully for branch '{}': {:?}",
                branch, index_status.state
            );
            return Ok(index_status);
        }

        info!(
            "Indexing complete for branch '{}', starting wiki generation",
            branch
        );

        let generator = WikiGenerator::new(
            self.openrouter.clone(),
            self.vector_store.clone(),
            self.config.chat_model.clone(),
            self.config.max_chunk_tokens,
            self.config.chunk_overlap,
        );

        let project_name = root_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("project");

        match generator
            .generate_wiki(
                root_path,
                project_name,
                branch,
                current_commit,
                progress_tx.clone(),
            )
            .await
        {
            Ok(structure) => {
                let duration = start_time.elapsed();
                info!(
                    "Wiki generation complete for branch '{}': {} pages in {:.1}s",
                    branch,
                    structure.page_count,
                    duration.as_secs_f64()
                );

                if let Some(tx) = &progress_tx {
                    let _ = tx.send(IndexProgress::Completed {
                        branch: branch.to_string(),
                        file_count: index_status.file_count,
                        chunk_count: index_status.chunk_count,
                        page_count: structure.page_count,
                        duration_secs: duration.as_secs_f64(),
                    });
                }
            }
            Err(e) => {
                error!("Wiki generation failed for branch '{}': {}", branch, e);
                if let Some(tx) = &progress_tx {
                    let _ = tx.send(IndexProgress::Failed {
                        branch: branch.to_string(),
                        error: format!("Wiki generation failed: {}", e),
                    });
                }
            }
        }

        Ok(index_status)
    }

    pub async fn sync_all_branches(
        &self,
        root_path: &Path,
        get_commit: impl Fn(&str) -> WikiResult<String>,
        progress_tx: Option<broadcast::Sender<IndexProgress>>,
    ) -> WikiResult<Vec<IndexStatus>> {
        let mut results = Vec::new();

        for branch in &self.config.branches {
            match get_commit(branch) {
                Ok(commit) => {
                    match self
                        .sync_if_needed(root_path, branch, &commit, progress_tx.clone())
                        .await
                    {
                        Ok(Some(status)) => results.push(status),
                        Ok(None) => {
                            if let Ok(Some(status)) = self.vector_store.get_index_status(branch) {
                                results.push(status);
                            }
                        }
                        Err(e) => {
                            error!("Failed to sync branch '{}': {}", branch, e);
                        }
                    }
                }
                Err(e) => {
                    warn!("Could not get commit for branch '{}': {}", branch, e);
                }
            }
        }

        Ok(results)
    }

    pub fn get_status(&self, branch: &str) -> WikiResult<Option<IndexStatus>> {
        self.vector_store.get_index_status(branch)
    }

    pub fn get_all_statuses(&self) -> WikiResult<Vec<IndexStatus>> {
        let mut statuses = Vec::new();
        for branch in &self.config.branches {
            if let Ok(Some(status)) = self.vector_store.get_index_status(branch) {
                statuses.push(status);
            }
        }
        Ok(statuses)
    }

    pub fn clear_branch(&self, branch: &str) -> WikiResult<()> {
        info!("Clearing wiki data for branch '{}'", branch);
        self.vector_store.clear_branch(branch)
    }

    pub fn clear_all(&self) -> WikiResult<()> {
        for branch in &self.config.branches {
            self.clear_branch(branch)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_sync_service_creation() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("wiki.db");

        let config = WikiConfig {
            db_path,
            openrouter_api_key: "test-key".to_string(),
            ..Default::default()
        };

        let service = WikiSyncService::new(config);
        assert!(service.is_ok());
    }

    #[test]
    fn test_needs_reindex_no_status() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("wiki.db");

        let config = WikiConfig {
            db_path,
            openrouter_api_key: "test-key".to_string(),
            ..Default::default()
        };

        let service = WikiSyncService::new(config).unwrap();
        assert!(service.needs_reindex("main", "abc123").unwrap());
    }
}
