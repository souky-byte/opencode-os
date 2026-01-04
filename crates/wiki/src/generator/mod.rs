//! Wiki page generator using AI

pub mod analyzer;
pub mod prompts;

use std::path::Path;
use std::sync::Arc;

use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};

use crate::domain::index_status::IndexProgress;
use crate::domain::wiki_page::{PageType, WikiPage, WikiStructure, WikiTree};
use crate::error::{WikiError, WikiResult};
use crate::openrouter::{ChatMessage, OpenRouterClient};
use crate::vector_store::VectorStore;

use analyzer::{FileImportance, ProjectAnalyzer, ProjectStructure};

const MAX_CONTENT_TOKENS: usize = 4000;

pub struct WikiGenerator {
    openrouter: Arc<OpenRouterClient>,
    vector_store: Arc<VectorStore>,
    chat_model: String,
    max_chunk_tokens: usize,
    chunk_overlap: usize,
}

impl WikiGenerator {
    pub fn new(
        openrouter: Arc<OpenRouterClient>,
        vector_store: Arc<VectorStore>,
        chat_model: String,
        max_chunk_tokens: usize,
        chunk_overlap: usize,
    ) -> Self {
        Self {
            openrouter,
            vector_store,
            chat_model,
            max_chunk_tokens,
            chunk_overlap,
        }
    }

    pub async fn generate_wiki(
        &self,
        root_path: &Path,
        project_name: &str,
        branch: &str,
        commit_sha: &str,
        progress_tx: Option<broadcast::Sender<IndexProgress>>,
    ) -> WikiResult<WikiStructure> {
        info!("Generating wiki for project '{}' on branch '{}'", project_name, branch);

        let send_progress = |current: u32, total: u32, page: &str| {
            if let Some(ref tx) = progress_tx {
                let _ = tx.send(IndexProgress::GeneratingWiki {
                    current,
                    total,
                    current_page: page.to_string(),
                });
            }
        };

        let analyzer = ProjectAnalyzer::new(self.max_chunk_tokens, self.chunk_overlap);
        let structure = analyzer.analyze(root_path, project_name).map_err(|e| {
            WikiError::GenerationFailed(format!("Failed to analyze project: {}", e))
        })?;

        let critical_files = analyzer.get_critical_files(&structure);
        let top_modules = analyzer.get_top_modules(&structure, 10);

        let total_pages = 1 + top_modules.len() + critical_files.len().min(10);
        let mut current_page = 0u32;

        send_progress(current_page, total_pages as u32, "overview");
        let overview = self.generate_overview(&structure, branch, commit_sha).await?;
        self.vector_store.insert_wiki_page(&overview)?;
        current_page += 1;

        let mut module_pages = Vec::new();
        for module in top_modules {
            send_progress(current_page, total_pages as u32, &module.name);
            match self.generate_module_page(root_path, module, branch, commit_sha).await {
                Ok(page) => {
                    self.vector_store.insert_wiki_page(&page)?;
                    module_pages.push(page);
                }
                Err(e) => {
                    warn!("Failed to generate page for module '{}': {}", module.name, e);
                }
            }
            current_page += 1;
        }

        let mut file_pages = Vec::new();
        for key_file in critical_files.iter().take(10) {
            send_progress(current_page, total_pages as u32, &key_file.name);
            match self.generate_file_page(root_path, key_file, branch, commit_sha).await {
                Ok(page) => {
                    self.vector_store.insert_wiki_page(&page)?;
                    file_pages.push(page);
                }
                Err(e) => {
                    warn!("Failed to generate page for file '{}': {}", key_file.name, e);
                }
            }
            current_page += 1;
        }

        let wiki_structure = self.build_wiki_structure(
            branch,
            &overview,
            &module_pages,
            &file_pages,
        );
        self.vector_store.save_wiki_structure(&wiki_structure)?;

        info!(
            "Wiki generation complete: {} pages created",
            1 + module_pages.len() + file_pages.len()
        );

        Ok(wiki_structure)
    }

    async fn generate_overview(
        &self,
        structure: &ProjectStructure,
        branch: &str,
        commit_sha: &str,
    ) -> WikiResult<WikiPage> {
        debug!("Generating overview for '{}'", structure.name);

        let languages = structure
            .languages
            .iter()
            .take(5)
            .map(|l| format!("{} ({:.0}%)", l.language, l.percentage))
            .collect::<Vec<_>>()
            .join(", ");

        let modules = structure
            .modules
            .iter()
            .take(10)
            .map(|m| format!("- **{}** (`{}`): {} files", m.name, m.path, m.file_count))
            .collect::<Vec<_>>()
            .join("\n");

        let key_files = structure
            .key_files
            .iter()
            .filter(|f| f.importance == FileImportance::Critical)
            .take(10)
            .map(|f| format!("- `{}`", f.path))
            .collect::<Vec<_>>()
            .join("\n");

        let prompt = prompts::overview_prompt(&structure.name, &languages, &modules, &key_files);

        let messages = vec![
            ChatMessage::system(prompts::OVERVIEW_SYSTEM_PROMPT),
            ChatMessage::user(prompt),
        ];

        let content = self
            .openrouter
            .chat_completion(messages, &self.chat_model, Some(0.7), Some(2000))
            .await?;

        let content = self.validate_and_fix_mermaid(&content).await;

        Ok(WikiPage::new(
            branch.to_string(),
            "overview".to_string(),
            format!("{} - Overview", structure.name),
            content,
            PageType::Overview,
            None,
            0,
            vec![],
            commit_sha.to_string(),
        ))
    }

    async fn generate_module_page(
        &self,
        root_path: &Path,
        module: &analyzer::ModuleInfo,
        branch: &str,
        commit_sha: &str,
    ) -> WikiResult<WikiPage> {
        debug!("Generating page for module '{}'", module.name);

        let files_list = module
            .key_files
            .iter()
            .take(10)
            .map(|f| format!("- `{}`", f))
            .collect::<Vec<_>>()
            .join("\n");

        let mut code_samples = String::new();
        for file_path in module.key_files.iter().take(3) {
            let full_path = root_path.join(file_path);
            if let Ok(content) = std::fs::read_to_string(&full_path) {
                let truncated = Self::truncate_content(&content, MAX_CONTENT_TOKENS / 3);
                code_samples.push_str(&format!("### {}\n```\n{}\n```\n\n", file_path, truncated));
            }
        }

        let prompt = prompts::module_prompt(&module.name, &module.path, &files_list, &code_samples);

        let messages = vec![
            ChatMessage::system(prompts::OVERVIEW_SYSTEM_PROMPT),
            ChatMessage::user(prompt),
        ];

        let content = self
            .openrouter
            .chat_completion(messages, &self.chat_model, Some(0.7), Some(1500))
            .await?;

        let content = self.validate_and_fix_mermaid(&content).await;

        let slug = Self::slugify(&module.name);

        Ok(WikiPage::new(
            branch.to_string(),
            slug,
            module.name.clone(),
            content,
            PageType::Module,
            Some("overview".to_string()),
            1,
            module.key_files.clone(),
            commit_sha.to_string(),
        ))
    }

    async fn generate_file_page(
        &self,
        root_path: &Path,
        key_file: &analyzer::KeyFile,
        branch: &str,
        commit_sha: &str,
    ) -> WikiResult<WikiPage> {
        debug!("Generating page for file '{}'", key_file.name);

        let full_path = root_path.join(&key_file.path);
        let content = std::fs::read_to_string(&full_path).map_err(|e| {
            WikiError::GenerationFailed(format!("Failed to read file {}: {}", key_file.path, e))
        })?;

        let truncated = Self::truncate_content(&content, MAX_CONTENT_TOKENS);
        let language = key_file.language.as_deref().unwrap_or("text");

        let prompt = prompts::file_prompt(&key_file.name, &key_file.path, &truncated, language);

        let messages = vec![
            ChatMessage::system(prompts::OVERVIEW_SYSTEM_PROMPT),
            ChatMessage::user(prompt),
        ];

        let generated = self
            .openrouter
            .chat_completion(messages, &self.chat_model, Some(0.7), Some(1500))
            .await?;

        let generated = self.validate_and_fix_mermaid(&generated).await;

        let slug = Self::slugify(&key_file.name);
        let parent_slug = Self::get_parent_slug(&key_file.path);

        Ok(WikiPage::new(
            branch.to_string(),
            slug,
            key_file.name.clone(),
            generated,
            PageType::File,
            parent_slug,
            2,
            vec![key_file.path.clone()],
            commit_sha.to_string(),
        ))
    }

    async fn validate_and_fix_mermaid(&self, content: &str) -> String {
        if prompts::validate_mermaid(content) {
            return content.to_string();
        }

        warn!("Invalid Mermaid diagram detected, attempting to fix");

        let mermaid_blocks: Vec<&str> = content
            .split("```mermaid")
            .skip(1)
            .filter_map(|block| block.split("```").next())
            .collect();

        let mut fixed_content = content.to_string();

        for block in mermaid_blocks {
            let fix_prompt = prompts::fix_mermaid_prompt(block);
            let messages = vec![ChatMessage::user(fix_prompt)];

            match self
                .openrouter
                .chat_completion(messages, &self.chat_model, Some(0.3), Some(500))
                .await
            {
                Ok(fixed) => {
                    let fixed_trimmed = fixed.trim();
                    fixed_content = fixed_content.replace(block, fixed_trimmed);
                }
                Err(e) => {
                    error!("Failed to fix Mermaid diagram: {}", e);
                }
            }
        }

        fixed_content
    }

    fn build_wiki_structure(
        &self,
        branch: &str,
        overview: &WikiPage,
        module_pages: &[WikiPage],
        file_pages: &[WikiPage],
    ) -> WikiStructure {
        let mut root = WikiTree::new(
            overview.slug.clone(),
            overview.title.clone(),
            PageType::Overview,
            0,
        );

        for page in module_pages {
            let node = WikiTree::new(
                page.slug.clone(),
                page.title.clone(),
                PageType::Module,
                page.order,
            );
            root.add_child(node);
        }

        for page in file_pages {
            let node = WikiTree::new(
                page.slug.clone(),
                page.title.clone(),
                PageType::File,
                page.order,
            );
            root.add_child(node);
        }

        WikiStructure::new(branch.to_string(), root)
    }

    fn slugify(name: &str) -> String {
        name.to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>()
            .trim_matches('-')
            .to_string()
    }

    fn get_parent_slug(path: &str) -> Option<String> {
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() > 1 {
            Some(Self::slugify(parts[parts.len() - 2]))
        } else {
            None
        }
    }

    fn truncate_content(content: &str, max_chars: usize) -> String {
        let approx_chars = max_chars * 4;
        if content.len() <= approx_chars {
            content.to_string()
        } else {
            let truncated: String = content.chars().take(approx_chars).collect();
            format!("{}\n\n... (truncated)", truncated)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify() {
        assert_eq!(WikiGenerator::slugify("lib.rs"), "lib-rs");
        assert_eq!(WikiGenerator::slugify("My Module"), "my-module");
        assert_eq!(WikiGenerator::slugify("src/api"), "src-api");
    }

    #[test]
    fn test_get_parent_slug() {
        assert_eq!(
            WikiGenerator::get_parent_slug("src/api/routes.rs"),
            Some("api".to_string())
        );
        assert_eq!(WikiGenerator::get_parent_slug("lib.rs"), None);
    }

    #[test]
    fn test_truncate_content() {
        let short = "Hello";
        assert_eq!(WikiGenerator::truncate_content(short, 100), "Hello");

        let long = "x".repeat(10000);
        let truncated = WikiGenerator::truncate_content(&long, 100);
        assert!(truncated.contains("... (truncated)"));
        assert!(truncated.len() < long.len());
    }
}
