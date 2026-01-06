//! Wiki page generator using AI

pub mod analyzer;
pub mod mermaid;
pub mod prompts;

use std::path::Path;
use std::sync::Arc;

use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};

use crate::domain::index_status::IndexProgress;
use crate::domain::wiki_page::{
    Importance, PageType, SourceCitation, WikiPage, WikiStructure, WikiTree,
};
use crate::domain::wiki_section::{GenerationMode, WikiSection};
use crate::error::{WikiError, WikiResult};
use crate::openrouter::{ChatMessage, OpenRouterClient};
use crate::vector_store::VectorStore;

use analyzer::{FileImportance, ProjectAnalyzer, ProjectStructure};

const MAX_CONTENT_TOKENS: usize = 4000;
const MAX_FILE_CONTENT_TOKENS: usize = 3000;
const MAX_STRUCTURE_RETRIES: u32 = 3;
const TEMPERATURE_STRUCTURE_LOW: f32 = 0.3;
const TEMPERATURE_CONTENT_CREATIVE: f32 = 0.7;

/// Structure definition from AI response for wiki planning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiPlan {
    pub title: String,
    pub description: String,
    pub sections: Vec<SectionPlan>,
    pub pages: Vec<PagePlan>,
}

/// Section definition from AI response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionPlan {
    pub id: String,
    pub title: String,
    pub description: String,
    pub page_ids: Vec<String>,
}

/// Page definition from AI response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PagePlan {
    pub id: String,
    pub title: String,
    pub section_id: String,
    pub importance: String,
    pub file_paths: Vec<String>,
    pub related_pages: Vec<String>,
    pub description: String,
}

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
        info!(
            "Generating wiki for project '{}' on branch '{}'",
            project_name, branch
        );

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
        let overview = self
            .generate_overview(&structure, branch, commit_sha)
            .await?;
        self.vector_store.insert_wiki_page(&overview)?;
        current_page += 1;

        let mut module_pages = Vec::new();
        for module in top_modules {
            send_progress(current_page, total_pages as u32, &module.name);
            match self
                .generate_module_page(root_path, module, branch, commit_sha)
                .await
            {
                Ok(page) => {
                    self.vector_store.insert_wiki_page(&page)?;
                    module_pages.push(page);
                }
                Err(e) => {
                    warn!(
                        "Failed to generate page for module '{}': {}",
                        module.name, e
                    );
                }
            }
            current_page += 1;
        }

        let mut file_pages = Vec::new();
        for key_file in critical_files.iter().take(10) {
            send_progress(current_page, total_pages as u32, &key_file.name);
            match self
                .generate_file_page(root_path, key_file, branch, commit_sha)
                .await
            {
                Ok(page) => {
                    self.vector_store.insert_wiki_page(&page)?;
                    file_pages.push(page);
                }
                Err(e) => {
                    warn!(
                        "Failed to generate page for file '{}': {}",
                        key_file.name, e
                    );
                }
            }
            current_page += 1;
        }

        let wiki_structure =
            self.build_wiki_structure(branch, &overview, &module_pages, &file_pages);
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
            ChatMessage::system(prompts::SYSTEM_PROMPT),
            ChatMessage::user(prompt),
        ];

        let content = self
            .openrouter
            .chat_completion(
                messages,
                &self.chat_model,
                Some(TEMPERATURE_CONTENT_CREATIVE),
                Some(4000),
            )
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
            ChatMessage::system(prompts::SYSTEM_PROMPT),
            ChatMessage::user(prompt),
        ];

        let content = self
            .openrouter
            .chat_completion(
                messages,
                &self.chat_model,
                Some(TEMPERATURE_CONTENT_CREATIVE),
                Some(3000),
            )
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
            ChatMessage::system(prompts::SYSTEM_PROMPT),
            ChatMessage::user(prompt),
        ];

        let generated = self
            .openrouter
            .chat_completion(
                messages,
                &self.chat_model,
                Some(TEMPERATURE_CONTENT_CREATIVE),
                Some(3000),
            )
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
        let fixed = mermaid::MermaidValidator::strip_invalid_diagrams(content);

        if fixed != content {
            warn!("Some Mermaid diagrams were fixed or removed");
        }

        fixed
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

    pub async fn generate_wiki_advanced(
        &self,
        root_path: &Path,
        project_name: &str,
        branch: &str,
        commit_sha: &str,
        mode: GenerationMode,
        progress_tx: Option<broadcast::Sender<IndexProgress>>,
    ) -> WikiResult<WikiStructure> {
        info!(
            branch = %branch,
            project = %project_name,
            mode = %mode.as_str(),
            "Starting wiki generation"
        );

        let send_progress = |current: u32, total: u32, page: &str| {
            if let Some(ref tx) = progress_tx {
                let _ = tx.send(IndexProgress::GeneratingWiki {
                    current,
                    total,
                    current_page: page.to_string(),
                });
            }
        };

        info!(branch = %branch, "Analyzing project structure...");
        let analyzer = ProjectAnalyzer::new(self.max_chunk_tokens, self.chunk_overlap);
        let structure = analyzer.analyze(root_path, project_name).map_err(|e| {
            WikiError::GenerationFailed(format!("Failed to analyze project: {}", e))
        })?;
        info!(
            branch = %branch,
            modules = structure.modules.len(),
            key_files = structure.key_files.len(),
            languages = structure.languages.len(),
            "Project analysis complete"
        );

        let file_tree = self.build_file_tree(&structure);
        let readme = self.read_readme(root_path);

        info!(branch = %branch, "Generating wiki structure with AI...");
        send_progress(0, 1, "planning");
        let wiki_plan_result = self
            .generate_wiki_structure(project_name, &file_tree, &readme, mode)
            .await;

        let wiki_plan = match wiki_plan_result {
            Ok(plan) => {
                info!(
                    branch = %branch,
                    sections = plan.sections.len(),
                    pages = plan.pages.len(),
                    "Wiki structure generated successfully"
                );
                plan
            }
            Err(e) => {
                warn!(branch = %branch, error = %e, "Advanced wiki structure generation failed, falling back to simple generation");
                return self
                    .generate_wiki(root_path, project_name, branch, commit_sha, progress_tx)
                    .await;
            }
        };

        let total_pages = wiki_plan.pages.len() as u32;
        let mut all_pages = Vec::new();
        let mut sections: Vec<WikiSection> = Vec::new();

        info!(branch = %branch, count = wiki_plan.sections.len(), "Creating wiki sections...");
        for section_plan in &wiki_plan.sections {
            let mut section = WikiSection::new(
                section_plan.id.clone(),
                branch.to_string(),
                section_plan.title.clone(),
                Some(section_plan.description.clone()),
                sections.len() as u32,
            );

            for page_id in &section_plan.page_ids {
                section.add_page(page_id.clone());
            }

            self.vector_store.insert_wiki_section(&section)?;
            sections.push(section);
        }

        info!(branch = %branch, total = total_pages, "Generating wiki pages...");
        for (idx, page_plan) in wiki_plan.pages.iter().enumerate() {
            send_progress(idx as u32, total_pages, &page_plan.title);
            info!(
                branch = %branch,
                page = idx + 1,
                total = total_pages,
                title = %page_plan.title,
                "Generating page"
            );

            match self
                .generate_page_from_plan(root_path, page_plan, branch, commit_sha, idx as u32)
                .await
            {
                Ok(page) => {
                    self.vector_store.insert_wiki_page(&page)?;
                    all_pages.push(page);
                    info!(
                        branch = %branch,
                        page = idx + 1,
                        total = total_pages,
                        title = %page_plan.title,
                        "Page generated successfully"
                    );
                }
                Err(e) => {
                    warn!(
                        branch = %branch,
                        page = idx + 1,
                        total = total_pages,
                        title = %page_plan.title,
                        error = %e,
                        "Failed to generate page"
                    );
                }
            }
        }

        let wiki_structure = self.build_wiki_structure_from_pages(branch, &all_pages, sections);
        self.vector_store.save_wiki_structure(&wiki_structure)?;

        info!(
            branch = %branch,
            pages = all_pages.len(),
            sections = wiki_structure.sections.len(),
            "Wiki generation completed"
        );

        Ok(wiki_structure)
    }

    async fn generate_wiki_structure(
        &self,
        project_name: &str,
        file_tree: &str,
        readme: &str,
        mode: GenerationMode,
    ) -> WikiResult<WikiPlan> {
        info!(project = %project_name, mode = %mode.as_str(), "Requesting wiki structure from AI...");

        let mut last_error = String::new();

        for attempt in 1..=MAX_STRUCTURE_RETRIES {
            info!(
                project = %project_name,
                attempt = attempt,
                max_attempts = MAX_STRUCTURE_RETRIES,
                "AI structure generation attempt"
            );

            let prompt = if attempt == 1 {
                prompts::structure_generation_prompt(project_name, file_tree, readme, mode)
            } else {
                prompts::structure_generation_prompt_strict(project_name, file_tree, readme, mode)
            };

            let messages = vec![
                ChatMessage::system(prompts::STRUCTURE_SYSTEM_PROMPT),
                ChatMessage::user(prompt),
            ];

            let response = self
                .openrouter
                .chat_completion(
                    messages,
                    &self.chat_model,
                    Some(TEMPERATURE_STRUCTURE_LOW),
                    Some(4000),
                )
                .await?;

            info!(project = %project_name, attempt = attempt, "AI response received, parsing...");

            match Self::parse_wiki_plan_robust(&response) {
                Ok(plan) => {
                    info!(
                        project = %project_name,
                        attempt = attempt,
                        sections = plan.sections.len(),
                        pages = plan.pages.len(),
                        "Wiki structure parsed successfully"
                    );
                    return Ok(plan);
                }
                Err(e) => {
                    warn!(
                        project = %project_name,
                        attempt = attempt,
                        max_attempts = MAX_STRUCTURE_RETRIES,
                        error = %e,
                        "Failed to parse wiki structure"
                    );
                    last_error = e;
                }
            }
        }

        error!(
            project = %project_name,
            attempts = MAX_STRUCTURE_RETRIES,
            error = %last_error,
            "Failed to generate wiki structure after all attempts"
        );
        Err(WikiError::GenerationFailed(format!(
            "Failed to generate valid wiki structure after {} attempts: {}",
            MAX_STRUCTURE_RETRIES, last_error
        )))
    }

    async fn generate_page_from_plan(
        &self,
        root_path: &Path,
        plan: &PagePlan,
        branch: &str,
        commit_sha: &str,
        order: u32,
    ) -> WikiResult<WikiPage> {
        debug!(
            title = %plan.title,
            files = plan.file_paths.len(),
            section = %plan.section_id,
            "Generating page content"
        );

        let file_contents = self.read_file_contents(root_path, &plan.file_paths);

        let prompt = prompts::page_content_prompt(
            &plan.title,
            &plan.description,
            &plan.file_paths,
            &file_contents,
            &plan.related_pages,
        );

        let messages = vec![
            ChatMessage::system(prompts::SYSTEM_PROMPT),
            ChatMessage::user(prompt),
        ];

        let content = self
            .openrouter
            .chat_completion(
                messages,
                &self.chat_model,
                Some(TEMPERATURE_CONTENT_CREATIVE),
                Some(4000),
            )
            .await?;

        let content = self.validate_and_fix_mermaid(&content).await;
        let source_citations = Self::extract_source_citations(&content);
        let importance = Importance::parse(&plan.importance).unwrap_or_default();
        let page_type = Self::infer_page_type(&plan.section_id);

        Ok(WikiPage::new_advanced(
            branch.to_string(),
            plan.id.clone(),
            plan.title.clone(),
            content,
            page_type,
            None,
            order,
            plan.file_paths.clone(),
            commit_sha.to_string(),
            importance,
            plan.related_pages.clone(),
            Some(plan.section_id.clone()),
            source_citations,
        ))
    }

    fn build_file_tree(&self, structure: &ProjectStructure) -> String {
        let mut tree = String::new();

        for module in &structure.modules {
            tree.push_str(&format!("{}/\n", module.path));
            for file in module.key_files.iter().take(5) {
                tree.push_str(&format!("  {}\n", file));
            }
            if module.key_files.len() > 5 {
                tree.push_str(&format!(
                    "  ... ({} more files)\n",
                    module.key_files.len() - 5
                ));
            }
        }

        for key_file in structure.key_files.iter().take(20) {
            if !structure
                .modules
                .iter()
                .any(|m| key_file.path.starts_with(&m.path))
            {
                tree.push_str(&format!("{}\n", key_file.path));
            }
        }

        tree
    }

    fn read_readme(&self, root_path: &Path) -> String {
        let readme_names = ["README.md", "readme.md", "README", "README.txt"];

        for name in readme_names {
            let path = root_path.join(name);
            if let Ok(content) = std::fs::read_to_string(&path) {
                return Self::truncate_content(&content, 2000);
            }
        }

        "No README found.".to_string()
    }

    fn read_file_contents(&self, root_path: &Path, file_paths: &[String]) -> String {
        let mut contents = String::new();
        let per_file_limit = MAX_FILE_CONTENT_TOKENS / file_paths.len().max(1);

        for path in file_paths.iter().take(8) {
            let full_path = root_path.join(path);
            if let Ok(content) = std::fs::read_to_string(&full_path) {
                let truncated = Self::truncate_content(&content, per_file_limit);
                let extension = full_path.extension().and_then(|e| e.to_str()).unwrap_or("");
                contents.push_str(&format!(
                    "### {}\n```{}\n{}\n```\n\n",
                    path, extension, truncated
                ));
            }
        }

        contents
    }

    fn parse_wiki_plan_robust(response: &str) -> Result<WikiPlan, String> {
        let cleaned = Self::clean_json_response(response);

        if let Ok(plan) = serde_json::from_str::<WikiPlan>(&cleaned) {
            return Ok(plan);
        }

        if let Some(plan) = Self::extract_wiki_plan_via_regex(&cleaned) {
            return Ok(plan);
        }

        if let Ok(value) = serde_json::from_str::<Value>(&cleaned) {
            if let Some(plan) = Self::reconstruct_wiki_plan_from_value(&value) {
                return Ok(plan);
            }
        }

        Err(format!(
            "Failed to parse wiki structure after all attempts. Response: {}",
            &cleaned[..cleaned.len().min(500)]
        ))
    }

    fn clean_json_response(response: &str) -> String {
        let trimmed = response.trim();

        let trimmed = trimmed
            .strip_prefix("```json")
            .unwrap_or(trimmed)
            .strip_prefix("```")
            .unwrap_or(trimmed)
            .strip_suffix("```")
            .unwrap_or(trimmed)
            .trim();

        if let Some(start) = trimmed.find('{') {
            if let Some(end) = trimmed.rfind('}') {
                let json_str = &trimmed[start..=end];
                return json_str.replace(['\n', '\r', '\t'], " ");
            }
        }

        trimmed.to_string()
    }

    fn extract_wiki_plan_via_regex(content: &str) -> Option<WikiPlan> {
        let title_re = Regex::new(r#""title"\s*:\s*"([^"]+)""#).ok()?;
        let title = title_re.captures(content)?.get(1)?.as_str().to_string();

        let desc_re = Regex::new(r#""description"\s*:\s*"([^"]+)""#).ok()?;
        let description = desc_re
            .captures(content)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().to_string())
            .unwrap_or_else(|| format!("Documentation for {}", title));

        let mut sections = Vec::new();
        let mut pages = Vec::new();

        let sections_re = Regex::new(r#""sections"\s*:\s*\[(.*?)\]"#).ok()?;
        if let Some(sections_match) = sections_re.captures(content).and_then(|c| c.get(1)) {
            let section_obj_re =
                Regex::new(r#"\{[^{}]*"id"\s*:\s*"([^"]+)"[^{}]*"title"\s*:\s*"([^"]+)"[^{}]*\}"#)
                    .ok()?;
            for cap in section_obj_re.captures_iter(sections_match.as_str()) {
                let id = cap
                    .get(1)
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_default();
                let section_title = cap
                    .get(2)
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_default();
                if !id.is_empty() {
                    sections.push(SectionPlan {
                        id: id.clone(),
                        title: section_title,
                        description: String::new(),
                        page_ids: vec![],
                    });
                }
            }
        }

        let pages_re = Regex::new(r#""pages"\s*:\s*\[(.*)\]"#).ok()?;
        if let Some(pages_match) = pages_re.captures(content).and_then(|c| c.get(1)) {
            let page_id_re = Regex::new(r#""id"\s*:\s*"([^"]+)""#).ok()?;
            let page_title_re = Regex::new(r#""title"\s*:\s*"([^"]+)""#).ok()?;
            let section_id_re = Regex::new(r#""section_id"\s*:\s*"([^"]+)""#).ok()?;

            for page_block in pages_match.as_str().split("},") {
                let id = page_id_re
                    .captures(page_block)
                    .and_then(|c| c.get(1))
                    .map(|m| m.as_str().to_string());
                let page_title = page_title_re
                    .captures(page_block)
                    .and_then(|c| c.get(1))
                    .map(|m| m.as_str().to_string());
                let section_id = section_id_re
                    .captures(page_block)
                    .and_then(|c| c.get(1))
                    .map(|m| m.as_str().to_string());

                if let (Some(id), Some(page_title)) = (id, page_title) {
                    pages.push(PagePlan {
                        id,
                        title: page_title,
                        section_id: section_id.unwrap_or_else(|| "overview".to_string()),
                        importance: "medium".to_string(),
                        file_paths: vec![],
                        related_pages: vec![],
                        description: String::new(),
                    });
                }
            }
        }

        if pages.is_empty() {
            return None;
        }

        if sections.is_empty() {
            sections.push(SectionPlan {
                id: "overview".to_string(),
                title: "Overview".to_string(),
                description: String::new(),
                page_ids: pages.iter().map(|p| p.id.clone()).collect(),
            });
        }

        Some(WikiPlan {
            title,
            description,
            sections,
            pages,
        })
    }

    fn reconstruct_wiki_plan_from_value(value: &Value) -> Option<WikiPlan> {
        let title = value.get("title")?.as_str()?.to_string();
        let description = value
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let mut sections = Vec::new();
        if let Some(sections_arr) = value.get("sections").and_then(|v| v.as_array()) {
            for section_val in sections_arr {
                if let (Some(id), Some(section_title)) = (
                    section_val.get("id").and_then(|v| v.as_str()),
                    section_val.get("title").and_then(|v| v.as_str()),
                ) {
                    let page_ids = section_val
                        .get("page_ids")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                .collect()
                        })
                        .unwrap_or_default();

                    sections.push(SectionPlan {
                        id: id.to_string(),
                        title: section_title.to_string(),
                        description: section_val
                            .get("description")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        page_ids,
                    });
                }
            }
        }

        let mut pages = Vec::new();
        if let Some(pages_arr) = value.get("pages").and_then(|v| v.as_array()) {
            for page_val in pages_arr {
                if let (Some(id), Some(page_title)) = (
                    page_val.get("id").and_then(|v| v.as_str()),
                    page_val.get("title").and_then(|v| v.as_str()),
                ) {
                    let file_paths = page_val
                        .get("file_paths")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                .collect()
                        })
                        .unwrap_or_default();

                    let related_pages = page_val
                        .get("related_pages")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                .collect()
                        })
                        .unwrap_or_default();

                    pages.push(PagePlan {
                        id: id.to_string(),
                        title: page_title.to_string(),
                        section_id: page_val
                            .get("section_id")
                            .and_then(|v| v.as_str())
                            .unwrap_or("overview")
                            .to_string(),
                        importance: page_val
                            .get("importance")
                            .and_then(|v| v.as_str())
                            .unwrap_or("medium")
                            .to_string(),
                        file_paths,
                        related_pages,
                        description: page_val
                            .get("description")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                    });
                }
            }
        }

        if pages.is_empty() {
            return None;
        }

        Some(WikiPlan {
            title,
            description,
            sections,
            pages,
        })
    }

    pub fn extract_source_citations(content: &str) -> Vec<SourceCitation> {
        let re = Regex::new(r"\[([^\]]+?):(\d+)(?:-(\d+))?\]\(\)").unwrap();
        let mut citations = Vec::new();

        for cap in re.captures_iter(content) {
            let file_path = cap
                .get(1)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
            let start_line: Option<u32> = cap.get(2).and_then(|m| m.as_str().parse().ok());
            let end_line: Option<u32> = cap.get(3).and_then(|m| m.as_str().parse().ok());

            if !file_path.is_empty() {
                citations.push(SourceCitation::new(
                    file_path,
                    start_line,
                    end_line.or(start_line),
                ));
            }
        }

        citations.dedup_by(|a, b| a.file_path == b.file_path && a.start_line == b.start_line);
        citations
    }

    fn infer_page_type(section_id: &str) -> PageType {
        match section_id {
            "overview" => PageType::Overview,
            "architecture" => PageType::Architecture,
            "backend" | "frontend" | "core-features" => PageType::Module,
            _ => PageType::Custom,
        }
    }

    fn build_wiki_structure_from_pages(
        &self,
        branch: &str,
        pages: &[WikiPage],
        sections: Vec<WikiSection>,
    ) -> WikiStructure {
        let overview = pages.iter().find(|p| p.page_type == PageType::Overview);

        let root = if let Some(overview_page) = overview {
            let mut root = WikiTree::new(
                overview_page.slug.clone(),
                overview_page.title.clone(),
                PageType::Overview,
                0,
            );

            for page in pages {
                if page.page_type != PageType::Overview {
                    let node = WikiTree::new(
                        page.slug.clone(),
                        page.title.clone(),
                        page.page_type,
                        page.order,
                    );
                    root.add_child(node);
                }
            }

            root
        } else {
            WikiTree::new(
                "wiki".to_string(),
                "Wiki".to_string(),
                PageType::Overview,
                0,
            )
        };

        WikiStructure::with_sections(branch.to_string(), root, sections)
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

    #[test]
    fn test_extract_source_citations() {
        let content = r#"
The authentication system uses JWT tokens.

Sources: [src/auth/jwt.rs:15-45](), [src/middleware/auth.rs:10-30]()

Single line: [config.rs:42]()

File only: [README.md]()
"#;

        let citations = WikiGenerator::extract_source_citations(content);

        assert_eq!(citations.len(), 3);

        assert_eq!(citations[0].file_path, "src/auth/jwt.rs");
        assert_eq!(citations[0].start_line, Some(15));
        assert_eq!(citations[0].end_line, Some(45));

        assert_eq!(citations[1].file_path, "src/middleware/auth.rs");
        assert_eq!(citations[1].start_line, Some(10));
        assert_eq!(citations[1].end_line, Some(30));

        assert_eq!(citations[2].file_path, "config.rs");
        assert_eq!(citations[2].start_line, Some(42));
        assert_eq!(citations[2].end_line, Some(42));
    }

    #[test]
    fn test_extract_source_citations_empty() {
        let content = "No citations here, just regular text.";
        let citations = WikiGenerator::extract_source_citations(content);
        assert!(citations.is_empty());
    }

    #[test]
    fn test_clean_json_response() {
        let with_markdown = r#"```json
{"title": "Test", "pages": []}
```"#;

        let cleaned = WikiGenerator::clean_json_response(with_markdown);
        assert!(cleaned.starts_with('{'));
        assert!(cleaned.ends_with('}'));
    }

    #[test]
    fn test_clean_json_response_pure_json() {
        let pure = r#"{"title": "Test"}"#;
        let cleaned = WikiGenerator::clean_json_response(pure);
        assert_eq!(cleaned, pure);
    }

    #[test]
    fn test_infer_page_type() {
        assert_eq!(
            WikiGenerator::infer_page_type("overview"),
            PageType::Overview
        );
        assert_eq!(
            WikiGenerator::infer_page_type("architecture"),
            PageType::Architecture
        );
        assert_eq!(WikiGenerator::infer_page_type("backend"), PageType::Module);
        assert_eq!(WikiGenerator::infer_page_type("frontend"), PageType::Module);
        assert_eq!(
            WikiGenerator::infer_page_type("deployment"),
            PageType::Custom
        );
    }

    #[test]
    fn test_wiki_plan_deserialize() {
        let json = r#"{
            "title": "Test Wiki",
            "description": "A test project",
            "sections": [
                {
                    "id": "overview",
                    "title": "Overview",
                    "description": "Project overview",
                    "page_ids": ["overview-intro"]
                }
            ],
            "pages": [
                {
                    "id": "overview-intro",
                    "title": "Introduction",
                    "section_id": "overview",
                    "importance": "high",
                    "file_paths": ["README.md", "src/lib.rs"],
                    "related_pages": [],
                    "description": "Main overview page"
                }
            ]
        }"#;

        let plan: WikiPlan = serde_json::from_str(json).unwrap();

        assert_eq!(plan.title, "Test Wiki");
        assert_eq!(plan.sections.len(), 1);
        assert_eq!(plan.pages.len(), 1);
        assert_eq!(plan.pages[0].importance, "high");
        assert_eq!(plan.pages[0].file_paths.len(), 2);
    }

    #[test]
    fn test_parse_wiki_plan_robust_valid_json() {
        let json = r#"{"title":"Test","description":"Desc","sections":[{"id":"overview","title":"Overview","description":"","page_ids":["p1"]}],"pages":[{"id":"p1","title":"Page 1","section_id":"overview","importance":"high","file_paths":["a.rs"],"related_pages":[],"description":""}]}"#;

        let result = WikiGenerator::parse_wiki_plan_robust(json);
        assert!(result.is_ok());
        let plan = result.unwrap();
        assert_eq!(plan.title, "Test");
        assert_eq!(plan.pages.len(), 1);
    }

    #[test]
    fn test_parse_wiki_plan_robust_with_markdown_fence() {
        let json = r#"```json
{"title":"Test","description":"","sections":[],"pages":[{"id":"p1","title":"Page","section_id":"overview","importance":"medium","file_paths":[],"related_pages":[],"description":""}]}
```"#;

        let result = WikiGenerator::parse_wiki_plan_robust(json);
        assert!(result.is_ok());
        let plan = result.unwrap();
        assert_eq!(plan.title, "Test");
    }

    #[test]
    fn test_parse_wiki_plan_robust_partial_json() {
        let partial = r#"Here is your JSON: {"title":"Partial","description":"test","sections":[],"pages":[{"id":"x","title":"X","section_id":"overview","importance":"low","file_paths":["main.rs"],"related_pages":[],"description":"d"}]}"#;

        let result = WikiGenerator::parse_wiki_plan_robust(partial);
        assert!(result.is_ok());
        let plan = result.unwrap();
        assert_eq!(plan.title, "Partial");
    }

    #[test]
    fn test_extract_wiki_plan_via_regex() {
        let messy = r#"{"title":"RegexTest","description":"d","pages":[{"id":"p1","title":"Title1","section_id":"s1"}]}"#;

        let result = WikiGenerator::extract_wiki_plan_via_regex(messy);
        assert!(result.is_some());
        let plan = result.unwrap();
        assert_eq!(plan.title, "RegexTest");
        assert_eq!(plan.pages.len(), 1);
        assert_eq!(plan.pages[0].id, "p1");
    }

    #[test]
    fn test_reconstruct_wiki_plan_from_value() {
        let value: Value = serde_json::json!({
            "title": "FromValue",
            "description": "Test description",
            "sections": [{"id": "s1", "title": "Section 1", "page_ids": ["p1"]}],
            "pages": [{"id": "p1", "title": "Page 1", "section_id": "s1", "file_paths": ["lib.rs"]}]
        });

        let result = WikiGenerator::reconstruct_wiki_plan_from_value(&value);
        assert!(result.is_some());
        let plan = result.unwrap();
        assert_eq!(plan.title, "FromValue");
        assert_eq!(plan.sections.len(), 1);
        assert_eq!(plan.pages.len(), 1);
        assert_eq!(plan.pages[0].file_paths, vec!["lib.rs"]);
    }
}
