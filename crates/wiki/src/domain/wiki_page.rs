use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
        }
    }

    /// Find a page by slug
    pub fn find_page(&self, slug: &str) -> Option<&WikiTree> {
        self.root.find(slug)
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

        // Should be sorted by order
        assert_eq!(root.children[0].slug, "child2");
        assert_eq!(root.children[1].slug, "child1");

        // Count should include all nodes
        assert_eq!(root.count(), 3);

        // Find should work
        assert!(root.find("child1").is_some());
        assert!(root.find("nonexistent").is_none());
    }
}
