//! Code indexer for creating embeddings and storing chunks

pub mod reader;

use std::path::Path;
use std::sync::Arc;

use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};

use crate::domain::chunk::{ChunkType, CodeChunk};
use crate::domain::index_status::{IndexProgress, IndexState, IndexStatus};
use crate::error::{WikiError, WikiResult};
use crate::openrouter::OpenRouterClient;
use crate::vector_store::VectorStore;

use reader::{FileInfo, FileReader};

const EMBEDDING_BATCH_SIZE: usize = 20;

pub struct CodeIndexer {
    openrouter: Arc<OpenRouterClient>,
    vector_store: Arc<VectorStore>,
    embedding_model: String,
    max_chunk_tokens: usize,
    chunk_overlap: usize,
}

impl CodeIndexer {
    pub fn new(
        openrouter: Arc<OpenRouterClient>,
        vector_store: Arc<VectorStore>,
        embedding_model: String,
        max_chunk_tokens: usize,
        chunk_overlap: usize,
    ) -> Self {
        Self {
            openrouter,
            vector_store,
            embedding_model,
            max_chunk_tokens,
            chunk_overlap,
        }
    }

    pub async fn index_branch(
        &self,
        root_path: &Path,
        branch: &str,
        commit_sha: &str,
        progress_tx: Option<broadcast::Sender<IndexProgress>>,
    ) -> WikiResult<IndexStatus> {
        info!("Starting indexing for branch '{}' at {:?}", branch, root_path);

        let send_progress = |progress: IndexProgress| {
            if let Some(ref tx) = progress_tx {
                let _ = tx.send(progress);
            }
        };

        if let Some(existing) = self.vector_store.get_index_status(branch)? {
            if existing.last_commit_sha.as_deref() == Some(commit_sha)
                && existing.state == IndexState::Indexed
            {
                info!("Branch '{}' already indexed at commit {}", branch, commit_sha);
                return Ok(existing);
            }
        }

        self.vector_store.clear_branch(branch)?;

        let mut status = IndexStatus::new(branch.to_string());
        status.state = IndexState::Indexing;
        status.last_commit_sha = Some(commit_sha.to_string());
        self.vector_store.update_index_status(&status)?;

        let reader = FileReader::new(self.max_chunk_tokens, self.chunk_overlap);
        let files = match reader.read_directory(root_path) {
            Ok(f) => f,
            Err(e) => {
                let err_msg = format!("Failed to read directory: {}", e);
                error!("{}", err_msg);
                status.state = IndexState::Failed;
                status.error_message = Some(err_msg.clone());
                self.vector_store.update_index_status(&status)?;
                send_progress(IndexProgress::Failed {
                    branch: branch.to_string(),
                    error: err_msg.clone(),
                });
                return Err(WikiError::IndexingFailed(err_msg));
            }
        };

        let total_files = files.len() as u32;
        info!("Found {} files to index", total_files);

        send_progress(IndexProgress::Started {
            branch: branch.to_string(),
            total_files,
        });

        let mut all_chunks: Vec<CodeChunk> = Vec::new();

        for (i, file) in files.iter().enumerate() {
            send_progress(IndexProgress::ReadingFiles {
                current: i as u32 + 1,
                total: total_files,
                current_file: file.relative_path.clone(),
            });

            let chunks = self.create_chunks_from_file(file, branch, commit_sha, &reader);
            all_chunks.extend(chunks);
        }

        let total_chunks = all_chunks.len();
        info!("Created {} chunks from {} files", total_chunks, total_files);

        for chunk in &all_chunks {
            self.vector_store.insert_chunk(chunk)?;
        }

        let chunk_contents: Vec<String> = all_chunks.iter().map(|c| c.content.clone()).collect();
        let chunk_ids: Vec<_> = all_chunks.iter().map(|c| c.id).collect();

        let total_batches = chunk_contents.len().div_ceil(EMBEDDING_BATCH_SIZE);

        for (batch_idx, batch) in chunk_contents.chunks(EMBEDDING_BATCH_SIZE).enumerate() {
            let batch_start = batch_idx * EMBEDDING_BATCH_SIZE;

            send_progress(IndexProgress::CreatingEmbeddings {
                current: (batch_idx + 1) as u32,
                total: total_batches as u32,
            });

            debug!(
                "Creating embeddings for batch {}/{} ({} chunks)",
                batch_idx + 1,
                total_batches,
                batch.len()
            );

            let batch_vec: Vec<String> = batch.to_vec();

            let store_embeddings = |embeddings: Vec<Vec<f32>>, batch_len: usize| -> WikiResult<()> {
                if embeddings.len() != batch_len {
                    return Err(WikiError::IndexingFailed(format!(
                        "Embedding count mismatch: expected {}, got {}",
                        batch_len,
                        embeddings.len()
                    )));
                }
                for (j, embedding) in embeddings.into_iter().enumerate() {
                    let chunk_id = chunk_ids[batch_start + j];
                    self.vector_store.insert_embedding(&chunk_id, &embedding)?;
                }
                Ok(())
            };

            match self
                .openrouter
                .create_embeddings_batch(&batch_vec, &self.embedding_model)
                .await
            {
                Ok(embeddings) => {
                    if let Err(e) = store_embeddings(embeddings, batch.len()) {
                        error!("{}", e);
                        status.state = IndexState::Failed;
                        status.error_message = Some(e.to_string());
                        self.vector_store.update_index_status(&status)?;
                        send_progress(IndexProgress::Failed {
                            branch: branch.to_string(),
                            error: e.to_string(),
                        });
                        return Err(e);
                    }
                }
                Err(WikiError::RateLimited { retry_after }) => {
                    let wait_secs = retry_after.unwrap_or(60);
                    warn!("Rate limited, waiting {}s before retry", wait_secs);
                    tokio::time::sleep(tokio::time::Duration::from_secs(wait_secs)).await;

                    match self
                        .openrouter
                        .create_embeddings_batch(&batch_vec, &self.embedding_model)
                        .await
                    {
                        Ok(embeddings) => {
                            if let Err(e) = store_embeddings(embeddings, batch.len()) {
                                error!("{}", e);
                                status.state = IndexState::Failed;
                                status.error_message = Some(e.to_string());
                                self.vector_store.update_index_status(&status)?;
                                send_progress(IndexProgress::Failed {
                                    branch: branch.to_string(),
                                    error: e.to_string(),
                                });
                                return Err(e);
                            }
                        }
                        Err(e) => {
                            let err_msg = format!("Embedding creation failed after retry: {}", e);
                            error!("{}", err_msg);
                            status.state = IndexState::Failed;
                            status.error_message = Some(err_msg.clone());
                            self.vector_store.update_index_status(&status)?;
                            send_progress(IndexProgress::Failed {
                                branch: branch.to_string(),
                                error: err_msg.clone(),
                            });
                            return Err(WikiError::IndexingFailed(err_msg));
                        }
                    }
                }
                Err(e) => {
                    let err_msg = format!("Embedding creation failed: {}", e);
                    error!("{}", err_msg);
                    status.state = IndexState::Failed;
                    status.error_message = Some(err_msg.clone());
                    self.vector_store.update_index_status(&status)?;
                    send_progress(IndexProgress::Failed {
                        branch: branch.to_string(),
                        error: err_msg.clone(),
                    });
                    return Err(WikiError::IndexingFailed(err_msg));
                }
            }
        }

        status.state = IndexState::Indexed;
        status.file_count = total_files;
        status.chunk_count = total_chunks as u32;
        status.last_indexed_at = Some(chrono::Utc::now());
        status.progress_percent = 100;
        status.error_message = None;
        self.vector_store.update_index_status(&status)?;

        send_progress(IndexProgress::Completed {
            branch: branch.to_string(),
            file_count: total_files,
            chunk_count: total_chunks as u32,
            page_count: 0,
            duration_secs: 0.0,
        });

        info!(
            "Indexing complete for branch '{}': {} files, {} chunks",
            branch, total_files, total_chunks
        );

        Ok(status)
    }

    fn create_chunks_from_file(
        &self,
        file: &FileInfo,
        branch: &str,
        commit_sha: &str,
        reader: &FileReader,
    ) -> Vec<CodeChunk> {
        let text_splitter = reader.text_splitter();
        let split_chunks = text_splitter.split(&file.content);

        split_chunks
            .into_iter()
            .enumerate()
            .map(|(idx, (content, start_line, end_line))| {
                let token_count = text_splitter.count_tokens(&content);
                let chunk_type = Self::detect_chunk_type(&file.relative_path, &content);

                CodeChunk::new(
                    branch.to_string(),
                    file.relative_path.clone(),
                    start_line,
                    end_line,
                    content,
                    chunk_type,
                    file.language.clone(),
                    token_count as u32,
                    idx as u32,
                    commit_sha.to_string(),
                )
            })
            .collect()
    }

    fn detect_chunk_type(file_path: &str, content: &str) -> ChunkType {
        let path_lower = file_path.to_lowercase();

        if path_lower.contains("test") || path_lower.contains("spec") {
            return ChunkType::Test;
        }

        if path_lower.ends_with(".json")
            || path_lower.ends_with(".yaml")
            || path_lower.ends_with(".yml")
            || path_lower.ends_with(".toml")
            || path_lower.ends_with(".xml")
            || path_lower.contains("config")
        {
            return ChunkType::Config;
        }

        if path_lower.ends_with(".md") || path_lower.ends_with(".txt") {
            return ChunkType::Documentation;
        }

        let content_lower = content.to_lowercase();

        if content_lower.contains("fn ")
            || content_lower.contains("def ")
            || content_lower.contains("function ")
            || content_lower.contains("func ")
        {
            return ChunkType::Function;
        }

        if content_lower.contains("class ")
            || content_lower.contains("struct ")
            || content_lower.contains("interface ")
            || content_lower.contains("trait ")
        {
            return ChunkType::Class;
        }

        if content_lower.contains("mod ") || content_lower.contains("module ") {
            return ChunkType::Module;
        }

        ChunkType::Code
    }

    pub fn needs_reindex(&self, branch: &str, current_commit: &str) -> WikiResult<bool> {
        match self.vector_store.get_index_status(branch)? {
            Some(status) => {
                if status.state != IndexState::Indexed {
                    return Ok(true);
                }
                Ok(status.last_commit_sha.as_deref() != Some(current_commit))
            }
            None => Ok(true),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_chunk_type_test_file() {
        assert_eq!(
            CodeIndexer::detect_chunk_type("src/lib_test.rs", "fn test_foo() {}"),
            ChunkType::Test
        );
        assert_eq!(
            CodeIndexer::detect_chunk_type("tests/integration.rs", "fn main() {}"),
            ChunkType::Test
        );
    }

    #[test]
    fn test_detect_chunk_type_config() {
        assert_eq!(
            CodeIndexer::detect_chunk_type("config.json", "{}"),
            ChunkType::Config
        );
        assert_eq!(
            CodeIndexer::detect_chunk_type("Cargo.toml", "[package]"),
            ChunkType::Config
        );
    }

    #[test]
    fn test_detect_chunk_type_function() {
        assert_eq!(
            CodeIndexer::detect_chunk_type("lib.rs", "fn main() { }"),
            ChunkType::Function
        );
        assert_eq!(
            CodeIndexer::detect_chunk_type("main.py", "def hello(): pass"),
            ChunkType::Function
        );
    }

    #[test]
    fn test_detect_chunk_type_class() {
        assert_eq!(
            CodeIndexer::detect_chunk_type("model.rs", "struct User { }"),
            ChunkType::Class
        );
        assert_eq!(
            CodeIndexer::detect_chunk_type("app.py", "class App: pass"),
            ChunkType::Class
        );
    }

    #[test]
    fn test_detect_chunk_type_docs() {
        assert_eq!(
            CodeIndexer::detect_chunk_type("README.md", "# Hello"),
            ChunkType::Documentation
        );
    }
}
