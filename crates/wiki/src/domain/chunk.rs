use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Type of code chunk
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChunkType {
    /// Full file content
    File,
    /// Function or method definition
    Function,
    /// Class or struct definition
    Class,
    /// Module or namespace
    Module,
    /// Documentation comment
    Documentation,
    /// Configuration file
    Config,
    /// Test code
    Test,
    /// Generic code block
    Code,
}

impl ChunkType {
    /// Get string representation for database storage
    pub fn as_str(&self) -> &'static str {
        match self {
            ChunkType::File => "file",
            ChunkType::Function => "function",
            ChunkType::Class => "class",
            ChunkType::Module => "module",
            ChunkType::Documentation => "documentation",
            ChunkType::Config => "config",
            ChunkType::Test => "test",
            ChunkType::Code => "code",
        }
    }

    /// Parse from database string
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "file" => Some(ChunkType::File),
            "function" => Some(ChunkType::Function),
            "class" => Some(ChunkType::Class),
            "module" => Some(ChunkType::Module),
            "documentation" => Some(ChunkType::Documentation),
            "config" => Some(ChunkType::Config),
            "test" => Some(ChunkType::Test),
            "code" => Some(ChunkType::Code),
            _ => None,
        }
    }
}

/// A chunk of code with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::too_many_arguments)]
pub struct CodeChunk {
    /// Unique identifier
    pub id: Uuid,

    /// Branch this chunk belongs to
    pub branch: String,

    /// Relative file path
    pub file_path: String,

    /// Starting line number (1-indexed)
    pub start_line: u32,

    /// Ending line number (1-indexed)
    pub end_line: u32,

    /// The actual content
    pub content: String,

    /// Type of chunk
    pub chunk_type: ChunkType,

    /// Programming language (detected from extension)
    pub language: Option<String>,

    /// Token count
    pub token_count: u32,

    /// Chunk index within the file (for ordering)
    pub chunk_index: u32,

    /// Git commit SHA when indexed
    pub commit_sha: String,

    /// Timestamp when created
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl CodeChunk {
    /// Create a new CodeChunk
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        branch: String,
        file_path: String,
        start_line: u32,
        end_line: u32,
        content: String,
        chunk_type: ChunkType,
        language: Option<String>,
        token_count: u32,
        chunk_index: u32,
        commit_sha: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            branch,
            file_path,
            start_line,
            end_line,
            content,
            chunk_type,
            language,
            token_count,
            chunk_index,
            commit_sha,
            created_at: chrono::Utc::now(),
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_type_roundtrip() {
        let types = [
            ChunkType::File,
            ChunkType::Function,
            ChunkType::Class,
            ChunkType::Module,
            ChunkType::Documentation,
            ChunkType::Config,
            ChunkType::Test,
            ChunkType::Code,
        ];

        for t in types {
            let s = t.as_str();
            let parsed = ChunkType::parse(s);
            assert_eq!(parsed, Some(t));
        }
    }

    #[test]
    fn test_code_chunk_location() {
        let chunk = CodeChunk::new(
            "main".to_string(),
            "src/lib.rs".to_string(),
            10,
            20,
            "fn test() {}".to_string(),
            ChunkType::Function,
            Some("rust".to_string()),
            5,
            0,
            "abc123".to_string(),
        );

        assert_eq!(chunk.location(), "src/lib.rs:10-20");
    }

    #[test]
    fn test_code_chunk_single_line_location() {
        let chunk = CodeChunk::new(
            "main".to_string(),
            "src/lib.rs".to_string(),
            10,
            10,
            "let x = 1;".to_string(),
            ChunkType::Code,
            Some("rust".to_string()),
            3,
            0,
            "abc123".to_string(),
        );

        assert_eq!(chunk.location(), "src/lib.rs:10");
    }
}
