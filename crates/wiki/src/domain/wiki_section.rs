//! Wiki section types for hierarchical organization

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Predefined main sections for wiki structure
pub const MAIN_SECTIONS: &[(&str, &str, &str)] = &[
    ("overview", "Overview", "General project information and introduction"),
    ("architecture", "System Architecture", "Design patterns, component relationships, and system design"),
    ("core-features", "Core Features", "Key functionality and main features"),
    ("data-flow", "Data Management", "Data storage, pipelines, and state management"),
    ("backend", "Backend Systems", "Server-side components, APIs, and services"),
    ("frontend", "Frontend Components", "UI components, pages, and user interactions"),
    ("deployment", "Deployment & Infrastructure", "Build, deployment, and infrastructure setup"),
];

/// A section in the wiki that groups related pages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiSection {
    /// Unique section identifier (e.g., "overview", "architecture")
    pub id: String,

    /// Branch this section belongs to
    pub branch: String,

    /// Display title
    pub title: String,

    /// Optional description of what this section covers
    pub description: Option<String>,

    /// Slugs of pages in this section (ordered)
    pub page_slugs: Vec<String>,

    /// IDs of nested subsections
    pub subsection_ids: Vec<String>,

    /// Display order (lower = first)
    pub order: u32,

    /// When the section was created
    pub created_at: DateTime<Utc>,

    /// When the section was last updated
    pub updated_at: DateTime<Utc>,
}

impl WikiSection {
    /// Create a new WikiSection
    pub fn new(
        id: String,
        branch: String,
        title: String,
        description: Option<String>,
        order: u32,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            branch,
            title,
            description,
            page_slugs: Vec::new(),
            subsection_ids: Vec::new(),
            order,
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a main section from predefined constants
    pub fn from_main_section(id: &str, branch: &str) -> Option<Self> {
        MAIN_SECTIONS
            .iter()
            .enumerate()
            .find(|(_, (section_id, _, _))| *section_id == id)
            .map(|(idx, (section_id, title, description))| {
                Self::new(
                    section_id.to_string(),
                    branch.to_string(),
                    title.to_string(),
                    Some(description.to_string()),
                    idx as u32,
                )
            })
    }

    /// Add a page slug to this section
    pub fn add_page(&mut self, slug: String) {
        if !self.page_slugs.contains(&slug) {
            self.page_slugs.push(slug);
            self.updated_at = Utc::now();
        }
    }

    /// Add a subsection ID
    pub fn add_subsection(&mut self, subsection_id: String) {
        if !self.subsection_ids.contains(&subsection_id) {
            self.subsection_ids.push(subsection_id);
            self.updated_at = Utc::now();
        }
    }

    /// Check if this is a main/root section
    pub fn is_main_section(&self) -> bool {
        MAIN_SECTIONS.iter().any(|(id, _, _)| *id == self.id)
    }
}

/// Generation mode for wiki - determines depth and detail level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum GenerationMode {
    /// Comprehensive wiki with 8-12 pages, detailed content
    #[default]
    Comprehensive,
    /// Concise wiki with 4-6 pages, essential content only
    Concise,
}

impl GenerationMode {
    pub fn page_count_range(&self) -> (usize, usize) {
        match self {
            GenerationMode::Comprehensive => (6, 8),
            GenerationMode::Concise => (3, 5),
        }
    }

    /// Get string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            GenerationMode::Comprehensive => "comprehensive",
            GenerationMode::Concise => "concise",
        }
    }

    /// Parse from string
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "comprehensive" => Some(GenerationMode::Comprehensive),
            "concise" => Some(GenerationMode::Concise),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main_sections_exist() {
        assert!(!MAIN_SECTIONS.is_empty());
        assert_eq!(MAIN_SECTIONS.len(), 7);
    }

    #[test]
    fn test_section_from_main() {
        let section = WikiSection::from_main_section("overview", "main").unwrap();
        assert_eq!(section.id, "overview");
        assert_eq!(section.title, "Overview");
        assert!(section.description.is_some());
        assert_eq!(section.order, 0);
    }

    #[test]
    fn test_section_from_main_invalid() {
        assert!(WikiSection::from_main_section("nonexistent", "main").is_none());
    }

    #[test]
    fn test_add_page() {
        let mut section = WikiSection::new(
            "test".to_string(),
            "main".to_string(),
            "Test".to_string(),
            None,
            0,
        );

        section.add_page("page-1".to_string());
        section.add_page("page-2".to_string());
        section.add_page("page-1".to_string()); // Duplicate

        assert_eq!(section.page_slugs.len(), 2);
    }

    #[test]
    fn test_generation_mode() {
        assert_eq!(GenerationMode::Comprehensive.page_count_range(), (6, 8));
        assert_eq!(GenerationMode::Concise.page_count_range(), (3, 5));

        assert_eq!(GenerationMode::parse("comprehensive"), Some(GenerationMode::Comprehensive));
        assert_eq!(GenerationMode::parse("concise"), Some(GenerationMode::Concise));
        assert_eq!(GenerationMode::parse("invalid"), None);
    }
}
