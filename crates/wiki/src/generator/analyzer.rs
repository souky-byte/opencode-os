//! Project analyzer for extracting structure and key files

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::indexer::reader::FileReader;

#[derive(Debug, Clone)]
pub struct ProjectStructure {
    pub name: String,
    pub root_path: PathBuf,
    pub modules: Vec<ModuleInfo>,
    pub key_files: Vec<KeyFile>,
    pub file_count: usize,
    pub languages: Vec<LanguageStats>,
}

#[derive(Debug, Clone)]
pub struct ModuleInfo {
    pub name: String,
    pub path: String,
    pub file_count: usize,
    pub description: Option<String>,
    pub submodules: Vec<String>,
    pub key_files: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct KeyFile {
    pub path: String,
    pub name: String,
    pub language: Option<String>,
    pub importance: FileImportance,
    pub token_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileImportance {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone)]
pub struct LanguageStats {
    pub language: String,
    pub file_count: usize,
    pub percentage: f32,
}

pub struct ProjectAnalyzer {
    max_chunk_tokens: usize,
    chunk_overlap: usize,
}

impl ProjectAnalyzer {
    pub fn new(max_chunk_tokens: usize, chunk_overlap: usize) -> Self {
        Self {
            max_chunk_tokens,
            chunk_overlap,
        }
    }

    pub fn analyze(&self, root_path: &Path, project_name: &str) -> std::io::Result<ProjectStructure> {
        let reader = FileReader::new(self.max_chunk_tokens, self.chunk_overlap);
        let files = reader.read_directory(root_path)?;

        let mut language_counts: HashMap<String, usize> = HashMap::new();
        let mut module_files: HashMap<String, Vec<String>> = HashMap::new();

        for file in &files {
            if let Some(lang) = &file.language {
                *language_counts.entry(lang.clone()).or_insert(0) += 1;
            }

            let module_path = self.get_module_path(&file.relative_path);
            module_files
                .entry(module_path)
                .or_default()
                .push(file.relative_path.clone());
        }

        let total_files = files.len();
        let mut languages: Vec<LanguageStats> = language_counts
            .into_iter()
            .map(|(lang, count)| LanguageStats {
                language: lang,
                file_count: count,
                percentage: (count as f32 / total_files as f32) * 100.0,
            })
            .collect();
        languages.sort_by(|a, b| b.file_count.cmp(&a.file_count));

        let modules: Vec<ModuleInfo> = module_files
            .iter()
            .filter(|(path, _)| !path.is_empty())
            .map(|(path, file_list)| {
                let submodules: Vec<String> = module_files
                    .keys()
                    .filter(|p| p.starts_with(path) && *p != path && !p[path.len()..].contains('/'))
                    .cloned()
                    .collect();

                let key_files: Vec<String> = file_list
                    .iter()
                    .filter(|f| self.is_key_file_in_module(f))
                    .cloned()
                    .collect();

                ModuleInfo {
                    name: path.split('/').next_back().unwrap_or(path).to_string(),
                    path: path.clone(),
                    file_count: file_list.len(),
                    description: None,
                    submodules,
                    key_files,
                }
            })
            .collect();

        let key_files: Vec<KeyFile> = files
            .iter()
            .filter_map(|f| {
                let importance = self.assess_importance(&f.relative_path);
                if importance == FileImportance::Low {
                    return None;
                }
                Some(KeyFile {
                    path: f.relative_path.clone(),
                    name: Path::new(&f.relative_path)
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default(),
                    language: f.language.clone(),
                    importance,
                    token_count: f.token_count,
                })
            })
            .collect();

        Ok(ProjectStructure {
            name: project_name.to_string(),
            root_path: root_path.to_path_buf(),
            modules,
            key_files,
            file_count: total_files,
            languages,
        })
    }

    fn get_module_path(&self, file_path: &str) -> String {
        Path::new(file_path)
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default()
    }

    fn is_key_file_in_module(&self, path: &str) -> bool {
        let name = Path::new(path)
            .file_name()
            .map(|n| n.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        matches!(
            name.as_str(),
            "mod.rs" | "lib.rs" | "main.rs" | "__init__.py" | "index.ts" | "index.js" | "index.tsx"
        )
    }

    fn assess_importance(&self, path: &str) -> FileImportance {
        let path_lower = path.to_lowercase();
        let name = Path::new(path)
            .file_name()
            .map(|n| n.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        if matches!(
            name.as_str(),
            "lib.rs" | "main.rs" | "mod.rs" | "app.rs" | "server.rs"
                | "main.py" | "app.py" | "__init__.py"
                | "index.ts" | "index.js" | "index.tsx" | "app.tsx" | "app.ts"
                | "main.go" | "app.go"
        ) {
            return FileImportance::Critical;
        }

        if path_lower.contains("config")
            || path_lower.contains("schema")
            || path_lower.contains("migration")
            || name.ends_with(".toml")
            || name.ends_with(".yaml")
            || name.ends_with(".yml")
        {
            return FileImportance::High;
        }

        if path_lower.contains("model")
            || path_lower.contains("service")
            || path_lower.contains("handler")
            || path_lower.contains("controller")
            || path_lower.contains("router")
            || path_lower.contains("api")
        {
            return FileImportance::High;
        }

        if path_lower.contains("util")
            || path_lower.contains("helper")
            || path_lower.contains("common")
            || path_lower.contains("types")
        {
            return FileImportance::Medium;
        }

        if path_lower.contains("test") || path_lower.contains("spec") {
            return FileImportance::Low;
        }

        FileImportance::Medium
    }

    pub fn get_critical_files<'a>(&self, structure: &'a ProjectStructure) -> Vec<&'a KeyFile> {
        structure
            .key_files
            .iter()
            .filter(|f| f.importance == FileImportance::Critical)
            .collect()
    }

    pub fn get_top_modules<'a>(&self, structure: &'a ProjectStructure, limit: usize) -> Vec<&'a ModuleInfo> {
        let mut modules: Vec<_> = structure.modules.iter().collect();
        modules.sort_by(|a, b| b.file_count.cmp(&a.file_count));
        modules.into_iter().take(limit).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_project_analyzer_creation() {
        let analyzer = ProjectAnalyzer::new(350, 100);
        assert_eq!(analyzer.max_chunk_tokens, 350);
    }

    #[test]
    fn test_assess_importance() {
        let analyzer = ProjectAnalyzer::new(350, 100);

        assert_eq!(analyzer.assess_importance("src/lib.rs"), FileImportance::Critical);
        assert_eq!(analyzer.assess_importance("src/main.rs"), FileImportance::Critical);
        assert_eq!(analyzer.assess_importance("config/settings.toml"), FileImportance::High);
        assert_eq!(analyzer.assess_importance("src/models/user.rs"), FileImportance::High);
        assert_eq!(analyzer.assess_importance("src/utils/helpers.rs"), FileImportance::Medium);
        assert_eq!(analyzer.assess_importance("tests/test_main.rs"), FileImportance::Low);
    }

    #[test]
    fn test_analyze_project() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("src");
        fs::create_dir(&src).unwrap();

        fs::write(src.join("lib.rs"), "pub mod api;").unwrap();
        fs::write(src.join("main.rs"), "fn main() {}").unwrap();

        let api = src.join("api");
        fs::create_dir(&api).unwrap();
        fs::write(api.join("mod.rs"), "pub mod routes;").unwrap();
        fs::write(api.join("routes.rs"), "fn get() {}").unwrap();

        let analyzer = ProjectAnalyzer::new(350, 100);
        let structure = analyzer.analyze(dir.path(), "test-project").unwrap();

        assert_eq!(structure.name, "test-project");
        assert_eq!(structure.file_count, 4);
        assert!(!structure.languages.is_empty());
    }

    #[test]
    fn test_get_module_path() {
        let analyzer = ProjectAnalyzer::new(350, 100);

        assert_eq!(analyzer.get_module_path("src/lib.rs"), "src");
        assert_eq!(analyzer.get_module_path("src/api/routes.rs"), "src/api");
        assert_eq!(analyzer.get_module_path("main.rs"), "");
    }
}
