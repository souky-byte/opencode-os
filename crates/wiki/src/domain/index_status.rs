use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IndexState {
    NotIndexed,
    Indexing,
    Generating,
    Indexed,
    Failed,
    Stale,
}

impl IndexState {
    pub fn as_str(&self) -> &'static str {
        match self {
            IndexState::NotIndexed => "not_indexed",
            IndexState::Indexing => "indexing",
            IndexState::Generating => "generating",
            IndexState::Indexed => "indexed",
            IndexState::Failed => "failed",
            IndexState::Stale => "stale",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "not_indexed" => Some(IndexState::NotIndexed),
            "indexing" => Some(IndexState::Indexing),
            "generating" => Some(IndexState::Generating),
            "indexed" => Some(IndexState::Indexed),
            "failed" => Some(IndexState::Failed),
            "stale" => Some(IndexState::Stale),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStatus {
    pub branch: String,
    pub state: IndexState,
    pub last_commit_sha: Option<String>,
    pub file_count: u32,
    pub chunk_count: u32,
    pub page_count: u32,
    pub last_indexed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub progress_percent: u8,
    pub current_phase: Option<String>,
    pub current_item: Option<String>,
}

impl IndexStatus {
    pub fn new(branch: String) -> Self {
        Self {
            branch,
            state: IndexState::NotIndexed,
            last_commit_sha: None,
            file_count: 0,
            chunk_count: 0,
            page_count: 0,
            last_indexed_at: None,
            error_message: None,
            progress_percent: 0,
            current_phase: None,
            current_item: None,
        }
    }

    pub fn needs_indexing(&self) -> bool {
        matches!(
            self.state,
            IndexState::NotIndexed | IndexState::Stale | IndexState::Failed
        )
    }

    pub fn is_indexing(&self) -> bool {
        matches!(self.state, IndexState::Indexing | IndexState::Generating)
    }

    pub fn is_indexed(&self) -> bool {
        self.state == IndexState::Indexed
    }
}

/// Progress update during indexing
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum IndexProgress {
    /// Started indexing
    Started { branch: String, total_files: u32 },

    /// Reading files
    ReadingFiles {
        current: u32,
        total: u32,
        current_file: String,
    },

    /// Creating embeddings
    CreatingEmbeddings { current: u32, total: u32 },

    /// Generating wiki pages
    GeneratingWiki {
        current: u32,
        total: u32,
        current_page: String,
    },

    /// Completed successfully
    Completed {
        branch: String,
        file_count: u32,
        chunk_count: u32,
        page_count: u32,
        duration_secs: f64,
    },

    /// Failed with error
    Failed { branch: String, error: String },
}

impl IndexProgress {
    /// Get progress percentage (0-100)
    pub fn percent(&self) -> u8 {
        match self {
            IndexProgress::Started { .. } => 0,
            IndexProgress::ReadingFiles { current, total, .. } => {
                if *total == 0 {
                    0
                } else {
                    (((*current as f64 / *total as f64) * 30.0) as u8).min(30)
                }
            }
            IndexProgress::CreatingEmbeddings { current, total, .. } => {
                if *total == 0 {
                    30
                } else {
                    30 + (((*current as f64 / *total as f64) * 40.0) as u8).min(40)
                }
            }
            IndexProgress::GeneratingWiki { current, total, .. } => {
                if *total == 0 {
                    70
                } else {
                    70 + (((*current as f64 / *total as f64) * 30.0) as u8).min(30)
                }
            }
            IndexProgress::Completed { .. } => 100,
            IndexProgress::Failed { .. } => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_state_roundtrip() {
        let states = [
            IndexState::NotIndexed,
            IndexState::Indexing,
            IndexState::Indexed,
            IndexState::Failed,
            IndexState::Stale,
        ];

        for s in states {
            let str_val = s.as_str();
            let parsed = IndexState::parse(str_val);
            assert_eq!(parsed, Some(s));
        }
    }

    #[test]
    fn test_index_status_needs_indexing() {
        let mut status = IndexStatus::new("main".to_string());
        assert!(status.needs_indexing());

        status.state = IndexState::Indexing;
        assert!(!status.needs_indexing());

        status.state = IndexState::Indexed;
        assert!(!status.needs_indexing());

        status.state = IndexState::Stale;
        assert!(status.needs_indexing());
    }

    #[test]
    fn test_index_progress_percent() {
        let progress = IndexProgress::Started {
            branch: "main".to_string(),
            total_files: 100,
        };
        assert_eq!(progress.percent(), 0);

        let progress = IndexProgress::ReadingFiles {
            current: 50,
            total: 100,
            current_file: "test.rs".to_string(),
        };
        assert_eq!(progress.percent(), 15);

        let progress = IndexProgress::Completed {
            branch: "main".to_string(),
            file_count: 100,
            chunk_count: 500,
            page_count: 50,
            duration_secs: 10.0,
        };
        assert_eq!(progress.percent(), 100);
    }
}
