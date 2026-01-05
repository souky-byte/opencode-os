//! AI prompts for wiki generation - DeepWiki-style comprehensive documentation

use crate::domain::wiki_section::GenerationMode;

pub const SYSTEM_PROMPT: &str = r#"You are an expert technical writer and software architect.
Your task is to generate comprehensive, accurate technical documentation for software projects.

CRITICAL RULES:
1. Ground every claim in the provided source files - no speculation
2. Include source citations with line numbers: [filename.ext:10-25]()
3. Use Mermaid diagrams for architecture visualization (graph TD, sequenceDiagram)
4. Use tables for structured data (parameters, configs, APIs)
5. Write in professional but accessible technical language
6. Prioritize accuracy over verbosity"#;

pub const STRUCTURE_SYSTEM_PROMPT: &str = r#"You are a JSON generator for wiki documentation structures.
You MUST output ONLY valid JSON. No markdown, no explanations, no code fences.
Your response must start with { and end with }."#;

pub fn structure_generation_prompt(
    project_name: &str,
    file_tree: &str,
    readme: &str,
    mode: GenerationMode,
) -> String {
    let (page_count, detail_level) = match mode {
        GenerationMode::Comprehensive => ("6-8", "detailed"),
        GenerationMode::Concise => ("3-5", "focused"),
    };

    let truncated_tree = if file_tree.len() > 3000 {
        format!("{}...\n(truncated)", &file_tree[..3000])
    } else {
        file_tree.to_string()
    };

    let truncated_readme = if readme.len() > 1500 {
        format!("{}...\n(truncated)", &readme[..1500])
    } else {
        readme.to_string()
    };

    format!(
        r#"Create a wiki structure for "{project_name}".

## File Tree:
{truncated_tree}

## README (excerpt):
{truncated_readme}

## Task
Create {page_count} wiki pages. Group into 2-4 sections (only relevant ones):
- overview, architecture, core-features, backend, frontend, deployment

## JSON Output (NO markdown, NO code blocks, ONLY valid JSON):
{{"title":"Wiki for {project_name}","description":"...","sections":[{{"id":"overview","title":"Overview","description":"...","page_ids":["overview-main"]}}],"pages":[{{"id":"overview-main","title":"Project Overview","section_id":"overview","importance":"high","file_paths":["README.md","src/main.rs"],"related_pages":[],"description":"..."}}]}}

RULES:
- Output ONLY the JSON object, nothing else
- Keep descriptions SHORT (under 50 chars)
- Use 3-5 file_paths per page (real files from tree)
- importance: "high", "medium", or "low"
- {detail_level} content with {page_count} pages total"#
    )
}

pub fn structure_generation_prompt_strict(
    project_name: &str,
    file_tree: &str,
    readme: &str,
    mode: GenerationMode,
) -> String {
    let page_count = match mode {
        GenerationMode::Comprehensive => "5",
        GenerationMode::Concise => "3",
    };

    let truncated_tree = if file_tree.len() > 2000 {
        format!("{}...", &file_tree[..2000])
    } else {
        file_tree.to_string()
    };

    let truncated_readme = if readme.len() > 1000 {
        format!("{}...", &readme[..1000])
    } else {
        readme.to_string()
    };

    format!(
        r#"OUTPUT ONLY VALID JSON. NO TEXT BEFORE OR AFTER.

Project: {project_name}
Files: {truncated_tree}
README: {truncated_readme}

Generate exactly this JSON structure with {page_count} pages:

{{"title":"Wiki for {project_name}","description":"Project documentation","sections":[{{"id":"overview","title":"Overview","description":"Main overview","page_ids":["overview"]}}],"pages":[{{"id":"overview","title":"Overview","section_id":"overview","importance":"high","file_paths":["README.md"],"related_pages":[],"description":"Overview"}}]}}"#
    )
}

pub fn page_content_prompt(
    page_title: &str,
    page_description: &str,
    file_paths: &[String],
    file_contents: &str,
    related_pages: &[String],
) -> String {
    let file_list = file_paths
        .iter()
        .map(|f| format!("- [{}]()", f))
        .collect::<Vec<_>>()
        .join("\n");

    let related_list = if related_pages.is_empty() {
        "None specified".to_string()
    } else {
        related_pages
            .iter()
            .map(|p| format!("- {}", p))
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(
        r#"Generate a comprehensive wiki page for "{page_title}".

## Page Description
{page_description}

## Related Pages
{related_list}

## Source Files (MUST be cited throughout the page)
{file_list}

## Source File Contents
{file_contents}

## CRITICAL OUTPUT REQUIREMENTS

1. **START with a <details> block** listing ALL source files:
```markdown
<details>
<summary>Relevant source files</summary>

The following files were used as context for generating this wiki page:

{file_list}
</details>
```

2. **Title**: Use `# {page_title}` as the main heading

3. **Introduction**: 1-2 paragraphs explaining:
   - What this component/feature does
   - Its role in the larger project
   - Why it exists

4. **Detailed Sections** (use ## and ### headings):
   - Break down into logical sections
   - Explain architecture, components, data flow
   - Identify key functions, classes, types

5. **Mermaid Diagrams** (REQUIRED for architecture pages):
   - Use `graph TD` for flow diagrams (NEVER `graph LR`)
   - Use `sequenceDiagram` for interactions
   - Keep node names short (3-4 words max)
   - Provide brief explanation before/after each diagram

6. **Tables** for structured data:
   - API endpoints, parameters, types
   - Configuration options
   - Data model fields

7. **Source Citations** (CRITICAL - minimum 5 citations per page):
   - Format: `Sources: [filename.ext:start_line-end_line]()`
   - Place after each major section or explanation
   - Multiple files: `Sources: [file1.ext:10-25](), [file2.ext:5]()`

8. **Related Pages Section** (at the end):
   - Link to related pages: `See also: [Page Title](#page-id)`

## Example Citation Usage

The authentication system uses JWT tokens for session management.

Sources: [src/auth/jwt.rs:15-45](), [src/middleware/auth.rs:10-30]()

## Output

Generate the complete markdown content following ALL requirements above.
Do NOT wrap in markdown code fences. Start directly with the <details> block."#
    )
}

pub fn overview_prompt(
    project_name: &str,
    languages: &str,
    modules: &str,
    key_files: &str,
) -> String {
    format!(
        r#"Generate a comprehensive project overview for "{project_name}".

## Project Information

**Languages:** {languages}

**Main Modules:**
{modules}

**Key Files:**
{key_files}

## Required Output

<details>
<summary>Relevant source files</summary>

Key files analyzed for this overview:
{key_files}
</details>

# {project_name} - Overview

Create a markdown document with:

1. **Project Overview** (2-3 paragraphs)
   - What the project does
   - Main purpose and goals
   - Target users/use cases

2. **Architecture Diagram** (Mermaid - REQUIRED)
   ```mermaid
   graph TD
       ... show main components and their relationships
   ```

3. **Technology Stack** (use a table)
   | Category | Technology | Purpose |
   |----------|------------|---------|
   | ... | ... | ... |

4. **Project Structure**
   - Explain the directory organization
   - Describe the purpose of main directories

5. **Getting Started**
   - Brief setup instructions
   - Key entry points

Sources: List the key files with line numbers where architecture is defined.

Keep the documentation concise but informative. Use actual module and file names."#
    )
}

pub fn module_prompt(
    module_name: &str,
    module_path: &str,
    files: &str,
    code_samples: &str,
) -> String {
    format!(
        r#"Generate documentation for the "{module_name}" module located at `{module_path}`.

## Module Files
{files}

## Code Samples
{code_samples}

## Required Output

<details>
<summary>Relevant source files</summary>

Files in this module:
{files}
</details>

# {module_name}

Create a markdown document with:

1. **Module Overview** (1-2 paragraphs)
   - What this module does
   - Its role in the larger project
   
   Sources: [primary files with line numbers]()

2. **Architecture Diagram** (Mermaid)
   ```mermaid
   graph TD
       ... show module structure and dependencies
   ```

3. **Key Components** (use a table)
   | Component | Type | Description |
   |-----------|------|-------------|
   | ... | Function/Class/Type | ... |
   
   Sources: [file:lines]() for each component

4. **Usage Examples**
   - How to use this module
   - Common patterns
   
   Sources: [example file locations]()

5. **Dependencies**
   - What this module depends on
   - What depends on this module

All sections MUST include source citations with line numbers."#
    )
}

pub fn file_prompt(file_name: &str, file_path: &str, content: &str, language: &str) -> String {
    format!(
        r#"Generate documentation for the file "{file_name}" at `{file_path}`.

## File Content ({language})
```{language}
{content}
```

## Required Output

<details>
<summary>Relevant source files</summary>

- [{file_path}]()
</details>

# {file_name}

Create a markdown document with:

1. **File Overview** (1 paragraph)
   - What this file does
   - Its role in the module/project
   
   Sources: [{file_path}:1-10]()

2. **Key Components** (use a table)
   | Name | Type | Lines | Description |
   |------|------|-------|-------------|
   | ... | Function/Struct/etc | 10-25 | ... |

3. **Code Flow Diagram** (Mermaid, if complex logic exists)
   ```mermaid
   graph TD
       ... show how main functions interact
   ```

4. **Implementation Details**
   - Key algorithms or patterns used
   - Configuration or constants defined
   - Error handling approaches
   
   Sources: [{file_path}:relevant-lines]()

Keep documentation proportional to file complexity. Include line numbers in all citations."#
    )
}

pub fn validate_mermaid(content: &str) -> bool {
    if !content.contains("```mermaid") {
        return true;
    }

    let mermaid_blocks: Vec<&str> = content
        .split("```mermaid")
        .skip(1)
        .filter_map(|block| block.split("```").next())
        .collect();

    for block in mermaid_blocks {
        let trimmed = block.trim();
        if trimmed.is_empty() {
            return false;
        }

        let valid_starts = [
            "graph",
            "flowchart",
            "sequenceDiagram",
            "classDiagram",
            "stateDiagram",
            "erDiagram",
            "journey",
            "gantt",
            "pie",
            "gitGraph",
            "mindmap",
            "timeline",
        ];

        let first_line = trimmed.lines().next().unwrap_or("");
        if !valid_starts.iter().any(|s| first_line.starts_with(s)) {
            return false;
        }
    }

    true
}

pub fn fix_mermaid_prompt(broken_diagram: &str) -> String {
    format!(
        r#"The following Mermaid diagram has syntax errors. Fix it and return ONLY the corrected diagram code (no markdown fences, no explanation):

{broken_diagram}

Common issues to check:
- Use graph TD (top-down), NEVER graph LR
- Use simple alphanumeric node IDs
- Arrow syntax: --> for solid, -.-> for dotted
- Subgraph syntax: subgraph Title ... end
- Keep node labels short

Return only the fixed Mermaid code."#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_structure_generation_prompt_comprehensive() {
        let prompt = structure_generation_prompt(
            "my-project",
            "src/\n  lib.rs\n  main.rs",
            "# My Project",
            GenerationMode::Comprehensive,
        );

        assert!(prompt.contains("my-project"));
        assert!(prompt.contains("6-8"));
        assert!(prompt.contains("detailed"));
    }

    #[test]
    fn test_structure_generation_prompt_concise() {
        let prompt = structure_generation_prompt(
            "my-project",
            "src/\n  lib.rs",
            "# My Project",
            GenerationMode::Concise,
        );

        assert!(prompt.contains("3-5"));
        assert!(prompt.contains("focused"));
    }

    #[test]
    fn test_page_content_prompt() {
        let prompt = page_content_prompt(
            "Authentication",
            "User authentication system",
            &["src/auth.rs".to_string(), "src/jwt.rs".to_string()],
            "// auth code here",
            &["authorization".to_string()],
        );

        assert!(prompt.contains("Authentication"));
        assert!(prompt.contains("<details>"));
        assert!(prompt.contains("Sources:"));
        assert!(prompt.contains("authorization"));
    }

    #[test]
    fn test_validate_mermaid_valid() {
        let content = r#"
# Overview

```mermaid
graph TD
    A --> B
    B --> C
```

Some text.
"#;
        assert!(validate_mermaid(content));
    }

    #[test]
    fn test_validate_mermaid_invalid_empty() {
        let content = r#"
```mermaid
```
"#;
        assert!(!validate_mermaid(content));
    }

    #[test]
    fn test_validate_mermaid_invalid_start() {
        let content = r#"
```mermaid
invalid diagram content
```
"#;
        assert!(!validate_mermaid(content));
    }

    #[test]
    fn test_validate_mermaid_no_diagram() {
        let content = "# Just text\n\nNo diagrams here.";
        assert!(validate_mermaid(content));
    }

    #[test]
    fn test_overview_prompt() {
        let prompt = overview_prompt(
            "my-project",
            "Rust, TypeScript",
            "- src/api\n- src/db",
            "- src/lib.rs\n- src/main.rs",
        );

        assert!(prompt.contains("my-project"));
        assert!(prompt.contains("Rust, TypeScript"));
        assert!(prompt.contains("<details>"));
    }

    #[test]
    fn test_module_prompt() {
        let prompt = module_prompt("api", "src/api", "- mod.rs\n- routes.rs", "fn handle() {}");

        assert!(prompt.contains("api"));
        assert!(prompt.contains("src/api"));
        assert!(prompt.contains("Sources:"));
    }

    #[test]
    fn test_file_prompt() {
        let prompt = file_prompt("lib.rs", "src/lib.rs", "pub mod api;", "rust");

        assert!(prompt.contains("lib.rs"));
        assert!(prompt.contains("pub mod api"));
        assert!(prompt.contains("<details>"));
    }
}
