use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::chunk::ChunkType;

/// A search result from semantic search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Chunk ID
    pub chunk_id: Uuid,

    /// File path
    pub file_path: String,

    /// Starting line number
    pub start_line: u32,

    /// Ending line number
    pub end_line: u32,

    /// Matched content
    pub content: String,

    /// Chunk type
    pub chunk_type: ChunkType,

    /// Programming language
    pub language: Option<String>,

    /// Similarity score (0.0 - 1.0)
    pub score: f32,

    /// Context before the match (previous chunk if available)
    pub context_before: Option<String>,

    /// Context after the match (next chunk if available)
    pub context_after: Option<String>,
}

impl SearchResult {
    /// Create a new SearchResult
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        chunk_id: Uuid,
        file_path: String,
        start_line: u32,
        end_line: u32,
        content: String,
        chunk_type: ChunkType,
        language: Option<String>,
        score: f32,
    ) -> Self {
        Self {
            chunk_id,
            file_path,
            start_line,
            end_line,
            content,
            chunk_type,
            language,
            score,
            context_before: None,
            context_after: None,
        }
    }

    /// Get a display-friendly location string
    pub fn location(&self) -> String {
        if self.start_line == self.end_line {
            format!("{}:{}", self.file_path, self.start_line)
        } else {
            format!("{}:{}-{}", self.file_path, self.start_line, self.end_line)
        }
    }

    /// Get score as percentage string
    pub fn score_percent(&self) -> String {
        format!("{:.1}%", self.score * 100.0)
    }

    /// Add context to the result
    pub fn with_context(mut self, before: Option<String>, after: Option<String>) -> Self {
        self.context_before = before;
        self.context_after = after;
        self
    }
}

/// Aggregated search results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    /// Query that was searched
    pub query: String,

    /// Search results
    pub results: Vec<SearchResult>,

    /// Total number of results (may be more than returned)
    pub total_count: u32,

    /// Time taken in milliseconds
    pub duration_ms: u64,
}

impl SearchResponse {
    /// Create a new SearchResponse
    pub fn new(
        query: String,
        results: Vec<SearchResult>,
        total_count: u32,
        duration_ms: u64,
    ) -> Self {
        Self {
            query,
            results,
            total_count,
            duration_ms,
        }
    }

    /// Check if there are any results
    pub fn is_empty(&self) -> bool {
        self.results.is_empty()
    }

    /// Get the top result if any
    pub fn top_result(&self) -> Option<&SearchResult> {
        self.results.first()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_result_location() {
        let result = SearchResult::new(
            Uuid::new_v4(),
            "src/lib.rs".to_string(),
            10,
            20,
            "fn test() {}".to_string(),
            ChunkType::Function,
            Some("rust".to_string()),
            0.95,
        );

        assert_eq!(result.location(), "src/lib.rs:10-20");
    }

    #[test]
    fn test_search_result_score_percent() {
        let result = SearchResult::new(
            Uuid::new_v4(),
            "src/lib.rs".to_string(),
            1,
            1,
            "test".to_string(),
            ChunkType::Code,
            None,
            0.875,
        );

        assert_eq!(result.score_percent(), "87.5%");
    }

    #[test]
    fn test_search_response() {
        let results = vec![
            SearchResult::new(
                Uuid::new_v4(),
                "a.rs".to_string(),
                1,
                10,
                "code".to_string(),
                ChunkType::Code,
                None,
                0.9,
            ),
            SearchResult::new(
                Uuid::new_v4(),
                "b.rs".to_string(),
                1,
                10,
                "more code".to_string(),
                ChunkType::Code,
                None,
                0.8,
            ),
        ];

        let response = SearchResponse::new("test query".to_string(), results, 2, 50);

        assert!(!response.is_empty());
        assert_eq!(response.total_count, 2);
        assert!(response.top_result().is_some());
        assert_eq!(response.top_result().unwrap().score, 0.9);
    }
}
