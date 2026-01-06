//! Text chunking for code files

use std::sync::OnceLock;
use tiktoken_rs::{cl100k_base, CoreBPE};
use tracing::debug;

// Cached tokenizer - initialization is expensive (~10-50ms)
static BPE_TOKENIZER: OnceLock<Option<CoreBPE>> = OnceLock::new();

fn get_tokenizer() -> Option<&'static CoreBPE> {
    BPE_TOKENIZER.get_or_init(|| cl100k_base().ok()).as_ref()
}

/// Text splitter that chunks content with overlap
pub struct TextSplitter {
    /// Maximum tokens per chunk
    max_tokens: usize,
    /// Overlap between chunks in tokens
    overlap: usize,
}

impl TextSplitter {
    /// Create a new TextSplitter
    pub fn new(max_tokens: usize, overlap: usize) -> Self {
        Self {
            max_tokens,
            overlap,
        }
    }

    pub fn split(&self, content: &str) -> Vec<(String, u32, u32)> {
        let lines: Vec<&str> = content.lines().collect();
        if lines.is_empty() {
            return Vec::new();
        }

        let bpe = match get_tokenizer() {
            Some(b) => b,
            None => return self.split_by_lines(content, &lines),
        };

        let mut chunks = Vec::new();
        let mut current_chunk: Vec<String> = Vec::new();
        let mut current_tokens = 0;
        let mut chunk_start_line = 1u32;
        let mut overlap_lines: Vec<(usize, String)> = Vec::new();

        for (i, line) in lines.iter().enumerate() {
            let line_tokens = bpe.encode_ordinary(line).len();

            // If single line exceeds max, we still include it (will be a single chunk)
            if current_tokens + line_tokens > self.max_tokens && !current_chunk.is_empty() {
                // Save current chunk
                let chunk_content = current_chunk.join("\n");
                let chunk_end_line = chunk_start_line + current_chunk.len() as u32 - 1;
                chunks.push((chunk_content, chunk_start_line, chunk_end_line));

                debug!(
                    "Created chunk: lines {}-{}, {} tokens",
                    chunk_start_line, chunk_end_line, current_tokens
                );

                // Calculate overlap for next chunk
                overlap_lines.clear();
                let mut overlap_tokens = 0;
                for (j, prev_line) in current_chunk.iter().enumerate().rev() {
                    let prev_tokens = bpe.encode_ordinary(prev_line).len();
                    if overlap_tokens + prev_tokens > self.overlap {
                        break;
                    }
                    overlap_tokens += prev_tokens;
                    let original_line_num = chunk_start_line as usize + j;
                    overlap_lines.push((original_line_num, prev_line.clone()));
                }
                overlap_lines.reverse();

                // Start new chunk with overlap
                current_chunk.clear();
                current_tokens = 0;

                if !overlap_lines.is_empty() {
                    chunk_start_line = overlap_lines[0].0 as u32;
                    for (_, line_content) in &overlap_lines {
                        current_chunk.push(line_content.to_string());
                        current_tokens += bpe.encode_ordinary(line_content).len();
                    }
                } else {
                    chunk_start_line = i as u32 + 1;
                }
            }

            current_chunk.push(line.to_string());
            current_tokens += line_tokens;
        }

        // Don't forget the last chunk
        if !current_chunk.is_empty() {
            let chunk_content = current_chunk.join("\n");
            let chunk_end_line = chunk_start_line + current_chunk.len() as u32 - 1;
            chunks.push((chunk_content, chunk_start_line, chunk_end_line));

            debug!(
                "Created final chunk: lines {}-{}, {} tokens",
                chunk_start_line, chunk_end_line, current_tokens
            );
        }

        chunks
    }

    fn split_by_lines(&self, _content: &str, lines: &[&str]) -> Vec<(String, u32, u32)> {
        let max_lines = ((self.max_tokens * 4) / 80).max(1);
        let overlap_lines = ((self.overlap * 4) / 80).min(max_lines.saturating_sub(1));

        let mut chunks = Vec::new();
        let mut i = 0;

        while i < lines.len() {
            let end = (i + max_lines).min(lines.len());
            let chunk_lines: Vec<&str> = lines[i..end].to_vec();
            let chunk_content = chunk_lines.join("\n");

            chunks.push((chunk_content, i as u32 + 1, end as u32));

            if end >= lines.len() {
                break;
            }

            let next_i = end.saturating_sub(overlap_lines);
            if next_i <= i {
                i = end;
            } else {
                i = next_i;
            }
        }

        chunks
    }

    pub fn count_tokens(&self, text: &str) -> usize {
        match get_tokenizer() {
            Some(bpe) => bpe.encode_ordinary(text).len(),
            None => text.len() / 4,
        }
    }

    /// Detect programming language from file extension
    pub fn detect_language(file_path: &str) -> Option<String> {
        let ext = file_path.rsplit('.').next()?;

        let lang = match ext.to_lowercase().as_str() {
            "rs" => "rust",
            "py" => "python",
            "js" => "javascript",
            "ts" => "typescript",
            "tsx" => "typescript",
            "jsx" => "javascript",
            "go" => "go",
            "java" => "java",
            "c" => "c",
            "cpp" | "cc" | "cxx" => "cpp",
            "h" | "hpp" => "cpp",
            "cs" => "csharp",
            "rb" => "ruby",
            "php" => "php",
            "swift" => "swift",
            "kt" | "kts" => "kotlin",
            "scala" => "scala",
            "sh" | "bash" => "bash",
            "sql" => "sql",
            "html" | "htm" => "html",
            "css" => "css",
            "scss" | "sass" => "scss",
            "json" => "json",
            "yaml" | "yml" => "yaml",
            "toml" => "toml",
            "xml" => "xml",
            "md" | "markdown" => "markdown",
            _ => return None,
        };

        Some(lang.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_splitter_simple() {
        let splitter = TextSplitter::new(50, 10);

        let content = "line1\nline2\nline3\nline4\nline5";
        let chunks = splitter.split(content);

        assert!(!chunks.is_empty());
        assert_eq!(chunks[0].1, 1); // Start line
    }

    #[test]
    fn test_text_splitter_empty() {
        let splitter = TextSplitter::new(350, 100);
        let chunks = splitter.split("");
        assert!(chunks.is_empty());
    }

    #[test]
    fn test_text_splitter_single_line() {
        let splitter = TextSplitter::new(350, 100);
        let chunks = splitter.split("single line");
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].1, 1);
        assert_eq!(chunks[0].2, 1);
    }

    #[test]
    fn test_count_tokens() {
        let splitter = TextSplitter::new(350, 100);
        let tokens = splitter.count_tokens("Hello, world!");
        assert!(tokens > 0);
        assert!(tokens < 10);
    }

    #[test]
    fn test_detect_language() {
        assert_eq!(
            TextSplitter::detect_language("src/lib.rs"),
            Some("rust".to_string())
        );
        assert_eq!(
            TextSplitter::detect_language("main.py"),
            Some("python".to_string())
        );
        assert_eq!(
            TextSplitter::detect_language("index.tsx"),
            Some("typescript".to_string())
        );
        assert_eq!(TextSplitter::detect_language("Makefile"), None);
    }
}
