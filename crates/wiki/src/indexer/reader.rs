//! File reader with .gitignore support and token counting

use std::path::{Path, PathBuf};

use ignore::WalkBuilder;
use tracing::debug;

use crate::chunker::TextSplitter;

const DEFAULT_EXTENSIONS: &[&str] = &[
    "rs", "py", "js", "ts", "tsx", "jsx", "go", "java", "c", "cpp", "cc", "cxx", "h", "hpp", "cs",
    "rb", "php", "swift", "kt", "kts", "scala", "sh", "bash", "sql", "html", "htm", "css", "scss",
    "sass", "json", "yaml", "yml", "toml", "xml", "md", "markdown", "txt",
];

pub struct FileReader {
    extensions: Vec<String>,
    max_file_size: usize,
    text_splitter: TextSplitter,
}

pub struct FileInfo {
    pub path: PathBuf,
    pub relative_path: String,
    pub content: String,
    pub token_count: usize,
    pub language: Option<String>,
}

impl FileReader {
    pub fn new(max_chunk_tokens: usize, chunk_overlap: usize) -> Self {
        Self {
            extensions: DEFAULT_EXTENSIONS.iter().map(|s| s.to_string()).collect(),
            max_file_size: 1024 * 1024, // 1MB
            text_splitter: TextSplitter::new(max_chunk_tokens, chunk_overlap),
        }
    }

    pub fn with_extensions(mut self, extensions: Vec<String>) -> Self {
        self.extensions = extensions;
        self
    }

    pub fn with_max_file_size(mut self, size: usize) -> Self {
        self.max_file_size = size;
        self
    }

    pub fn read_directory(&self, root: &Path) -> std::io::Result<Vec<FileInfo>> {
        let mut files = Vec::new();

        let walker = WalkBuilder::new(root)
            .hidden(false)
            .git_ignore(true)
            .git_global(true)
            .git_exclude(true)
            .require_git(false)
            .build();

        for entry in walker {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    debug!("Error walking directory: {}", e);
                    continue;
                }
            };

            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            if !self.should_include(path) {
                continue;
            }

            if let Some(file_info) = self.read_file(root, path)? {
                files.push(file_info);
            }
        }

        Ok(files)
    }

    fn should_include(&self, path: &Path) -> bool {
        let ext = path
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        self.extensions.iter().any(|e| e == &ext)
    }

    fn read_file(&self, root: &Path, path: &Path) -> std::io::Result<Option<FileInfo>> {
        let metadata = std::fs::metadata(path)?;

        if metadata.len() as usize > self.max_file_size {
            return Ok(None);
        }

        let content = std::fs::read_to_string(path)?;

        if content.trim().is_empty() {
            return Ok(None);
        }

        let relative_path = path
            .strip_prefix(root)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();

        let token_count = self.text_splitter.count_tokens(&content);
        let language = TextSplitter::detect_language(&relative_path);

        Ok(Some(FileInfo {
            path: path.to_path_buf(),
            relative_path,
            content,
            token_count,
            language,
        }))
    }

    pub fn text_splitter(&self) -> &TextSplitter {
        &self.text_splitter
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_file_reader_creation() {
        let reader = FileReader::new(350, 100);
        assert!(!reader.extensions.is_empty());
    }

    #[test]
    fn test_should_include() {
        let reader = FileReader::new(350, 100);
        assert!(reader.should_include(Path::new("src/lib.rs")));
        assert!(reader.should_include(Path::new("main.py")));
        assert!(!reader.should_include(Path::new("image.png")));
        assert!(!reader.should_include(Path::new("binary.exe")));
    }

    #[test]
    fn test_read_directory() {
        let dir = tempdir().unwrap();
        let src_dir = dir.path().join("src");
        fs::create_dir(&src_dir).unwrap();

        fs::write(src_dir.join("lib.rs"), "fn main() {}").unwrap();
        fs::write(src_dir.join("test.py"), "def main(): pass").unwrap();
        fs::write(dir.path().join("README.md"), "# README").unwrap();
        fs::write(dir.path().join("image.png"), &[0u8; 100]).unwrap();

        let reader = FileReader::new(350, 100);
        let files = reader.read_directory(dir.path()).unwrap();

        assert_eq!(files.len(), 3);

        let paths: Vec<_> = files.iter().map(|f| f.relative_path.as_str()).collect();
        assert!(paths.iter().any(|p| p.contains("lib.rs")));
        assert!(paths.iter().any(|p| p.contains("test.py")));
        assert!(paths.iter().any(|p| p.contains("README.md")));
    }

    #[test]
    fn test_gitignore_respected() {
        let dir = tempdir().unwrap();

        fs::write(dir.path().join(".gitignore"), "ignored/\n*.log\n").unwrap();

        let ignored_dir = dir.path().join("ignored");
        fs::create_dir(&ignored_dir).unwrap();
        fs::write(ignored_dir.join("secret.rs"), "fn secret() {}").unwrap();

        fs::write(dir.path().join("debug.log"), "log content").unwrap();
        fs::write(dir.path().join("app.rs"), "fn main() {}").unwrap();

        let reader = FileReader::new(350, 100);
        let files = reader.read_directory(dir.path()).unwrap();

        let paths: Vec<_> = files.iter().map(|f| f.relative_path.as_str()).collect();

        assert!(paths.iter().any(|p| p.contains("app.rs")));
        assert!(!paths.iter().any(|p| p.contains("secret.rs")));
        assert!(!paths.iter().any(|p| p.contains("debug.log")));
    }

    #[test]
    fn test_node_modules_excluded_by_gitignore() {
        let dir = tempdir().unwrap();

        fs::write(dir.path().join(".gitignore"), "node_modules/\n").unwrap();

        let node_modules = dir.path().join("node_modules");
        fs::create_dir(&node_modules).unwrap();
        fs::write(node_modules.join("index.js"), "module.exports = {}").unwrap();

        fs::write(dir.path().join("app.js"), "console.log('hi')").unwrap();

        let reader = FileReader::new(350, 100);
        let files = reader.read_directory(dir.path()).unwrap();

        assert_eq!(files.len(), 1);
        assert!(files[0].relative_path.contains("app.js"));
    }
}
