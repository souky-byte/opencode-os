use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::wiki_section::WikiSection;

/// Importance level of a wiki page
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Importance {
    /// Critical pages - entry points, core logic, main APIs
    High,
    /// Important pages - supporting modules, secondary features
    #[default]
    Medium,
    /// Supporting pages - utilities, helpers, configuration
    Low,
}

impl Importance {
    /// Get string representation for database storage
    pub fn as_str(&self) -> &'static str {
        match self {
            Importance::High => "high",
            Importance::Medium => "medium",
            Importance::Low => "low",
        }
    }

    /// Parse from database string
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "high" => Some(Importance::High),
            "medium" => Some(Importance::Medium),
            "low" => Some(Importance::Low),
            _ => None,
        }
    }
}

/// A source code citation with file path and optional line numbers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceCitation {
    /// Path to the source file
    pub file_path: String,

    /// Starting line number (1-based, optional)
    pub start_line: Option<u32>,

    /// Ending line number (1-based, optional)
    pub end_line: Option<u32>,
}

impl SourceCitation {
    /// Create a new source citation
    pub fn new(file_path: String, start_line: Option<u32>, end_line: Option<u32>) -> Self {
        Self {
            file_path,
            start_line,
            end_line,
        }
    }

    /// Create a citation for a whole file (no line numbers)
    pub fn file(file_path: String) -> Self {
        Self::new(file_path, None, None)
    }

    /// Create a citation for a specific line range
    pub fn lines(file_path: String, start: u32, end: u32) -> Self {
        Self::new(file_path, Some(start), Some(end))
    }

    /// Create a citation for a single line
    pub fn line(file_path: String, line: u32) -> Self {
        Self::new(file_path, Some(line), Some(line))
    }

    /// Format as a markdown citation string: `[file.rs:10-25]()`
    pub fn to_markdown(&self) -> String {
        match (self.start_line, self.end_line) {
            (Some(start), Some(end)) if start == end => {
                format!("[{}:{}]()", self.file_path, start)
            }
            (Some(start), Some(end)) => {
                format!("[{}:{}-{}]()", self.file_path, start, end)
            }
            (Some(start), None) => {
                format!("[{}:{}]()", self.file_path, start)
            }
            _ => {
                format!("[{}]()", self.file_path)
            }
        }
    }

    /// Parse from markdown citation format: `[file.rs:10-25]()`
    pub fn from_markdown(s: &str) -> Option<Self> {
        // Pattern: [file_path:start-end]() or [file_path:line]() or [file_path]()
        let s = s.trim();
        if !s.starts_with('[') || !s.ends_with("()") {
            return None;
        }

        let inner = &s[1..s.len() - 3]; // Remove [ and ]()
        if let Some(colon_idx) = inner.rfind(':') {
            let file_path = inner[..colon_idx].to_string();
            let line_part = &inner[colon_idx + 1..];

            if let Some(dash_idx) = line_part.find('-') {
                let start: u32 = line_part[..dash_idx].parse().ok()?;
                let end: u32 = line_part[dash_idx + 1..].parse().ok()?;
                Some(Self::lines(file_path, start, end))
            } else {
                let line: u32 = line_part.parse().ok()?;
                Some(Self::line(file_path, line))
            }
        } else {
            Some(Self::file(inner.to_string()))
        }
    }
}

/// Type of wiki page
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PageType {
    /// Project overview page
    Overview,
    /// Module/directory documentation
    Module,
    /// Important file documentation
    File,
    /// API documentation
    Api,
    /// Architecture documentation
    Architecture,
    /// Custom/user-created page
    Custom,
}

impl PageType {
    /// Get string representation for database storage
    pub fn as_str(&self) -> &'static str {
        match self {
            PageType::Overview => "overview",
            PageType::Module => "module",
            PageType::File => "file",
            PageType::Api => "api",
            PageType::Architecture => "architecture",
            PageType::Custom => "custom",
        }
    }

    /// Parse from database string
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "overview" => Some(PageType::Overview),
            "module" => Some(PageType::Module),
            "file" => Some(PageType::File),
            "api" => Some(PageType::Api),
            "architecture" => Some(PageType::Architecture),
            "custom" => Some(PageType::Custom),
            _ => None,
        }
    }
}

/// A wiki documentation page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiPage {
    /// Unique identifier
    pub id: Uuid,

    /// Branch this page belongs to
    pub branch: String,

    /// URL-friendly slug
    pub slug: String,

    /// Page title
    pub title: String,

    /// Markdown content
    pub content: String,

    /// Type of page
    pub page_type: PageType,

    /// Parent page slug (for hierarchy)
    pub parent_slug: Option<String>,

    /// Order within parent (for sorting)
    pub order: u32,

    /// Related file paths (if applicable)
    pub file_paths: Vec<String>,

    /// Mermaid diagrams in the content
    pub has_diagrams: bool,

    /// Commit SHA when generated
    pub commit_sha: String,

    /// When the page was created
    pub created_at: DateTime<Utc>,

    /// When the page was last updated
    pub updated_at: DateTime<Utc>,

    /// Importance level (high/medium/low)
    #[serde(default)]
    pub importance: Importance,

    /// Slugs of related wiki pages
    #[serde(default)]
    pub related_pages: Vec<String>,

    /// Section this page belongs to
    #[serde(default)]
    pub section_id: Option<String>,

    /// Source code citations with line numbers
    #[serde(default)]
    pub source_citations: Vec<SourceCitation>,
}

impl WikiPage {
    /// Create a new WikiPage
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        branch: String,
        slug: String,
        title: String,
        content: String,
        page_type: PageType,
        parent_slug: Option<String>,
        order: u32,
        file_paths: Vec<String>,
        commit_sha: String,
    ) -> Self {
        let now = Utc::now();
        let has_diagrams = content.contains("```mermaid");

        Self {
            id: Uuid::new_v4(),
            branch,
            slug,
            title,
            content,
            page_type,
            parent_slug,
            order,
            file_paths,
            has_diagrams,
            commit_sha,
            created_at: now,
            updated_at: now,
            importance: Importance::default(),
            related_pages: Vec::new(),
            section_id: None,
            source_citations: Vec::new(),
        }
    }

    /// Create a new WikiPage with all advanced fields
    #[allow(clippy::too_many_arguments)]
    pub fn new_advanced(
        branch: String,
        slug: String,
        title: String,
        content: String,
        page_type: PageType,
        parent_slug: Option<String>,
        order: u32,
        file_paths: Vec<String>,
        commit_sha: String,
        importance: Importance,
        related_pages: Vec<String>,
        section_id: Option<String>,
        source_citations: Vec<SourceCitation>,
    ) -> Self {
        let now = Utc::now();
        let has_diagrams = content.contains("```mermaid");

        Self {
            id: Uuid::new_v4(),
            branch,
            slug,
            title,
            content,
            page_type,
            parent_slug,
            order,
            file_paths,
            has_diagrams,
            commit_sha,
            created_at: now,
            updated_at: now,
            importance,
            related_pages,
            section_id,
            source_citations,
        }
    }

    /// Get the full path for this page in the wiki hierarchy
    pub fn full_path(&self) -> String {
        match &self.parent_slug {
            Some(parent) => format!("{}/{}", parent, self.slug),
            None => self.slug.clone(),
        }
    }
}

/// A node in the wiki structure tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiTree {
    /// Page slug
    pub slug: String,

    /// Page title
    pub title: String,

    /// Page type
    pub page_type: PageType,

    /// Order for sorting
    pub order: u32,

    /// Child pages
    pub children: Vec<WikiTree>,
}

impl WikiTree {
    /// Create a new tree node
    pub fn new(slug: String, title: String, page_type: PageType, order: u32) -> Self {
        Self {
            slug,
            title,
            page_type,
            order,
            children: Vec::new(),
        }
    }

    /// Add a child node
    pub fn add_child(&mut self, child: WikiTree) {
        self.children.push(child);
        self.children.sort_by_key(|c| c.order);
    }

    /// Find a node by slug
    pub fn find(&self, slug: &str) -> Option<&WikiTree> {
        if self.slug == slug {
            return Some(self);
        }

        for child in &self.children {
            if let Some(found) = child.find(slug) {
                return Some(found);
            }
        }

        None
    }

    /// Count total nodes in tree
    pub fn count(&self) -> usize {
        1 + self.children.iter().map(|c| c.count()).sum::<usize>()
    }
}

/// Complete wiki structure for a branch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiStructure {
    /// Branch name
    pub branch: String,

    /// Root of the wiki tree
    pub root: WikiTree,

    /// Total page count
    pub page_count: u32,

    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,

    /// Organized sections (Overview, Architecture, etc.)
    #[serde(default)]
    pub sections: Vec<WikiSection>,

    /// IDs of root-level sections
    #[serde(default)]
    pub root_section_ids: Vec<String>,
}

impl WikiStructure {
    /// Create a new WikiStructure
    pub fn new(branch: String, root: WikiTree) -> Self {
        let page_count = root.count() as u32;
        Self {
            branch,
            root,
            page_count,
            updated_at: Utc::now(),
            sections: Vec::new(),
            root_section_ids: Vec::new(),
        }
    }

    /// Create a WikiStructure with sections
    pub fn with_sections(branch: String, root: WikiTree, sections: Vec<WikiSection>) -> Self {
        let page_count = root.count() as u32;
        let root_section_ids: Vec<String> = sections
            .iter()
            .filter(|s| s.is_main_section())
            .map(|s| s.id.clone())
            .collect();

        Self {
            branch,
            root,
            page_count,
            updated_at: Utc::now(),
            sections,
            root_section_ids,
        }
    }

    /// Find a page by slug
    pub fn find_page(&self, slug: &str) -> Option<&WikiTree> {
        self.root.find(slug)
    }

    /// Find a section by ID
    pub fn find_section(&self, section_id: &str) -> Option<&WikiSection> {
        self.sections.iter().find(|s| s.id == section_id)
    }

    /// Get all pages in a section
    pub fn pages_in_section(&self, section_id: &str) -> Vec<&WikiTree> {
        self.find_section(section_id)
            .map(|section| {
                section
                    .page_slugs
                    .iter()
                    .filter_map(|slug| self.find_page(slug))
                    .collect()
            })
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_type_roundtrip() {
        let types = [
            PageType::Overview,
            PageType::Module,
            PageType::File,
            PageType::Api,
            PageType::Architecture,
            PageType::Custom,
        ];

        for t in types {
            let s = t.as_str();
            let parsed = PageType::parse(s);
            assert_eq!(parsed, Some(t));
        }
    }

    #[test]
    fn test_importance_roundtrip() {
        let levels = [Importance::High, Importance::Medium, Importance::Low];

        for level in levels {
            let s = level.as_str();
            let parsed = Importance::parse(s);
            assert_eq!(parsed, Some(level));
        }
    }

    #[test]
    fn test_source_citation_markdown() {
        let citation = SourceCitation::lines("src/lib.rs".to_string(), 10, 25);
        assert_eq!(citation.to_markdown(), "[src/lib.rs:10-25]()");

        let single = SourceCitation::line("main.rs".to_string(), 42);
        assert_eq!(single.to_markdown(), "[main.rs:42]()");

        let file_only = SourceCitation::file("config.toml".to_string());
        assert_eq!(file_only.to_markdown(), "[config.toml]()");
    }

    #[test]
    fn test_source_citation_parse() {
        let parsed = SourceCitation::from_markdown("[src/lib.rs:10-25]()").unwrap();
        assert_eq!(parsed.file_path, "src/lib.rs");
        assert_eq!(parsed.start_line, Some(10));
        assert_eq!(parsed.end_line, Some(25));

        let single = SourceCitation::from_markdown("[main.rs:42]()").unwrap();
        assert_eq!(single.file_path, "main.rs");
        assert_eq!(single.start_line, Some(42));
        assert_eq!(single.end_line, Some(42));

        let file_only = SourceCitation::from_markdown("[config.toml]()").unwrap();
        assert_eq!(file_only.file_path, "config.toml");
        assert_eq!(file_only.start_line, None);
    }

    #[test]
    fn test_wiki_page_full_path() {
        let page = WikiPage::new(
            "main".to_string(),
            "lib-rs".to_string(),
            "lib.rs".to_string(),
            "# Library\n\nMain entry point.".to_string(),
            PageType::File,
            Some("src".to_string()),
            0,
            vec!["src/lib.rs".to_string()],
            "abc123".to_string(),
        );

        assert_eq!(page.full_path(), "src/lib-rs");
        assert_eq!(page.importance, Importance::Medium);
        assert!(page.related_pages.is_empty());
    }

    #[test]
    fn test_wiki_page_has_diagrams() {
        let page_with = WikiPage::new(
            "main".to_string(),
            "overview".to_string(),
            "Overview".to_string(),
            "# Overview\n\n```mermaid\ngraph TD\nA-->B\n```".to_string(),
            PageType::Overview,
            None,
            0,
            vec![],
            "abc123".to_string(),
        );
        assert!(page_with.has_diagrams);

        let page_without = WikiPage::new(
            "main".to_string(),
            "readme".to_string(),
            "README".to_string(),
            "# README\n\nNo diagrams here.".to_string(),
            PageType::Overview,
            None,
            0,
            vec![],
            "abc123".to_string(),
        );
        assert!(!page_without.has_diagrams);
    }

    #[test]
    fn test_wiki_tree() {
        let mut root = WikiTree::new(
            "root".to_string(),
            "Root".to_string(),
            PageType::Overview,
            0,
        );

        let child1 = WikiTree::new("child1".to_string(), "Child 1".to_string(), PageType::Module, 1);
        let child2 = WikiTree::new("child2".to_string(), "Child 2".to_string(), PageType::Module, 0);

        root.add_child(child1);
        root.add_child(child2);

        assert_eq!(root.children[0].slug, "child2");
        assert_eq!(root.children[1].slug, "child1");
        assert_eq!(root.count(), 3);
        assert!(root.find("child1").is_some());
        assert!(root.find("nonexistent").is_none());
    }

    #[test]
    fn test_wiki_structure_with_sections() {
        let root = WikiTree::new("overview".to_string(), "Overview".to_string(), PageType::Overview, 0);

        let mut section = WikiSection::from_main_section("overview", "main").unwrap();
        section.add_page("overview".to_string());

        let structure = WikiStructure::with_sections("main".to_string(), root, vec![section]);

        assert_eq!(structure.sections.len(), 1);
        assert!(!structure.root_section_ids.is_empty());
        assert!(structure.find_section("overview").is_some());
    }
}
