//! AI prompts for wiki generation

pub const OVERVIEW_SYSTEM_PROMPT: &str = r#"You are a technical documentation expert. Generate clear, comprehensive documentation for software projects.

Guidelines:
- Write in a professional but accessible tone
- Include practical examples where helpful
- Structure content with clear headings
- Use Mermaid diagrams for architecture visualization
- Focus on the "why" and "how", not just the "what""#;

pub fn overview_prompt(project_name: &str, languages: &str, modules: &str, key_files: &str) -> String {
    format!(
        r#"Generate a comprehensive project overview for "{project_name}".

## Project Information

**Languages:** {languages}

**Main Modules:**
{modules}

**Key Files:**
{key_files}

## Required Output

Create a markdown document with:

1. **Project Overview** (2-3 paragraphs)
   - What the project does
   - Main purpose and goals
   - Target users/use cases

2. **Architecture Diagram** (Mermaid)
   - Create a flowchart or graph showing main components
   - Show relationships between modules
   - Use ```mermaid code block

3. **Technology Stack**
   - List main languages and frameworks
   - Brief explanation of technology choices

4. **Project Structure**
   - Explain the directory organization
   - Describe the purpose of main directories

5. **Getting Started**
   - Brief setup instructions (if inferable)
   - Key entry points

Keep the documentation concise but informative. Use the actual module and file names provided."#
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

Create a markdown document with:

1. **Module Overview** (1-2 paragraphs)
   - What this module does
   - Its role in the larger project

2. **Module Structure Diagram** (Mermaid)
   - Show relationships between files in this module
   - Use ```mermaid code block

3. **Key Components**
   - List and describe main functions/classes/types
   - Explain their purposes

4. **Usage Examples**
   - How to use this module
   - Common patterns

5. **Dependencies**
   - What this module depends on
   - What depends on this module (if known)

Keep explanations clear and focused on practical understanding."#
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

Create a markdown document with:

1. **File Overview** (1 paragraph)
   - What this file does
   - Its role in the module/project

2. **Key Components**
   - List main functions/classes/types with brief descriptions
   - Include function signatures where helpful

3. **Code Flow Diagram** (Mermaid, if applicable)
   - Show how main functions interact
   - Use ```mermaid code block
   - Only include if the file has complex logic worth diagramming

4. **Important Details**
   - Key algorithms or patterns used
   - Configuration or constants defined
   - Error handling approaches

Keep the documentation proportional to the file's complexity."#
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
            "graph", "flowchart", "sequenceDiagram", "classDiagram", "stateDiagram",
            "erDiagram", "journey", "gantt", "pie", "gitGraph", "mindmap", "timeline",
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
- Missing arrow syntax (use --> or ---)
- Invalid node IDs (use simple alphanumeric)
- Missing semicolons where required
- Incorrect subgraph syntax

Return only the fixed Mermaid code."#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(prompt.contains("src/api"));
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
    fn test_module_prompt() {
        let prompt = module_prompt(
            "api",
            "src/api",
            "- mod.rs\n- routes.rs",
            "fn handle() {}",
        );

        assert!(prompt.contains("api"));
        assert!(prompt.contains("src/api"));
    }

    #[test]
    fn test_file_prompt() {
        let prompt = file_prompt("lib.rs", "src/lib.rs", "pub mod api;", "rust");

        assert!(prompt.contains("lib.rs"));
        assert!(prompt.contains("pub mod api"));
    }
}
