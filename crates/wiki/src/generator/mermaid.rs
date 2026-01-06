pub struct MermaidValidator;

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub fixed_diagram: Option<String>,
}

impl MermaidValidator {
    pub fn validate(diagram: &str) -> ValidationResult {
        let trimmed = diagram.trim();
        let mut errors = Vec::new();

        if trimmed.is_empty() {
            return ValidationResult {
                is_valid: false,
                errors: vec!["Empty diagram".to_string()],
                fixed_diagram: None,
            };
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
            "quadrantChart",
            "xychart",
        ];

        let first_line = trimmed.lines().next().unwrap_or("");
        if !valid_starts.iter().any(|s| first_line.starts_with(s)) {
            errors.push(format!("Invalid diagram type: '{}'", first_line));
        }

        if first_line.contains("graph LR") {
            errors.push("graph LR often causes rendering issues, prefer graph TD".to_string());
        }

        let open_brackets = trimmed.matches('[').count();
        let close_brackets = trimmed.matches(']').count();
        if open_brackets != close_brackets {
            errors.push(format!(
                "Unbalanced brackets: {} '[' vs {} ']'",
                open_brackets, close_brackets
            ));
        }

        let open_parens = trimmed.matches('(').count();
        let close_parens = trimmed.matches(')').count();
        if open_parens != close_parens {
            errors.push(format!(
                "Unbalanced parentheses: {} '(' vs {} ')'",
                open_parens, close_parens
            ));
        }

        let open_braces = trimmed.matches('{').count();
        let close_braces = trimmed.matches('}').count();
        if open_braces != close_braces {
            errors.push(format!(
                "Unbalanced braces: {} '{{' vs {} '}}'",
                open_braces, close_braces
            ));
        }

        if trimmed.contains("subgraph") {
            let subgraph_count = trimmed.matches("subgraph").count();
            let end_count = trimmed
                .lines()
                .filter(|l| l.trim() == "end" || l.trim().starts_with("end "))
                .count();
            if subgraph_count > end_count {
                errors.push("Missing 'end' for subgraph".to_string());
            }
        }

        for line in trimmed.lines().skip(1) {
            let line = line.trim();
            if line.is_empty() || line.starts_with("%%") {
                continue;
            }
            if line.contains('[')
                && line.contains(']')
                && (line.contains('<') || line.contains('>') || line.contains('&'))
            {
                errors.push(format!("Special chars in label may cause issues: {}", line));
            }
        }

        if trimmed.contains("```") {
            errors.push("Diagram contains nested code fences".to_string());
        }

        ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            fixed_diagram: None,
        }
    }

    pub fn fix(diagram: &str) -> String {
        let mut fixed = diagram.to_string();

        fixed = fixed.replace("graph LR", "graph TD");

        fixed = Self::fix_special_chars_in_labels(&fixed);

        fixed = Self::fix_node_ids(&fixed);

        fixed = fixed
            .lines()
            .filter(|l| !l.contains("```"))
            .collect::<Vec<_>>()
            .join("\n");

        fixed = fixed.trim().to_string();

        fixed
    }

    fn fix_special_chars_in_labels(diagram: &str) -> String {
        let mut result = diagram.to_string();

        let replacements = [("&", "&amp;"), ("<", "&lt;"), (">", "&gt;")];

        for line in diagram.lines() {
            if let Some(start) = line.find('[') {
                if let Some(end) = line.rfind(']') {
                    if start < end {
                        let label = &line[start + 1..end];
                        let mut fixed_label = label.to_string();

                        for (from, to) in &replacements {
                            if *from != "&" || !label.contains("&amp;") {
                                fixed_label = fixed_label.replace(from, to);
                            }
                        }

                        if fixed_label != label {
                            let old_segment = format!("[{}]", label);
                            let new_segment = format!("[{}]", fixed_label);
                            result = result.replace(&old_segment, &new_segment);
                        }
                    }
                }
            }
        }

        result
    }

    fn fix_node_ids(diagram: &str) -> String {
        let mut result = String::new();
        let mut node_map: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        let mut counter = 0;

        for line in diagram.lines() {
            let mut fixed_line = line.to_string();

            let parts: Vec<&str> = line.split(['[', '(', '{']).collect();
            if let Some(first_part) = parts.first() {
                let tokens: Vec<&str> = first_part.split_whitespace().collect();
                for token in tokens {
                    if token.contains('-') && !token.contains("->") && !token.contains("--") {
                        if !node_map.contains_key(token) {
                            let safe_id = format!("node{}", counter);
                            node_map.insert(token.to_string(), safe_id);
                            counter += 1;
                        }
                        if let Some(safe_id) = node_map.get(token) {
                            fixed_line = fixed_line.replace(token, safe_id);
                        }
                    }
                }
            }

            result.push_str(&fixed_line);
            result.push('\n');
        }

        result.trim_end().to_string()
    }

    pub fn validate_and_fix(diagram: &str) -> (bool, String) {
        let validation = Self::validate(diagram);

        if validation.is_valid {
            return (true, diagram.to_string());
        }

        let fixed = Self::fix(diagram);
        let revalidation = Self::validate(&fixed);

        if revalidation.is_valid {
            return (true, fixed);
        }

        (false, fixed)
    }

    pub fn strip_invalid_diagrams(content: &str) -> String {
        let mut result = String::new();
        let mut in_mermaid = false;
        let mut current_diagram = String::new();
        let mut before_diagram = String::new();

        for line in content.lines() {
            if line.trim().starts_with("```mermaid") {
                in_mermaid = true;
                before_diagram = result.clone();
                current_diagram.clear();
                continue;
            }

            if in_mermaid {
                if line.trim() == "```" {
                    in_mermaid = false;
                    let (is_valid, fixed) = Self::validate_and_fix(&current_diagram);

                    if is_valid {
                        result.push_str("```mermaid\n");
                        result.push_str(&fixed);
                        result.push_str("\n```\n");
                    } else {
                        result = before_diagram.clone();
                        result.push_str("\n<!-- Diagram removed due to syntax errors -->\n");
                    }
                    current_diagram.clear();
                } else {
                    current_diagram.push_str(line);
                    current_diagram.push('\n');
                }
            } else {
                result.push_str(line);
                result.push('\n');
            }
        }

        result.trim_end().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_valid_graph() {
        let diagram = "graph TD\n    A --> B\n    B --> C";
        let result = MermaidValidator::validate(diagram);
        assert!(result.is_valid);
    }

    #[test]
    fn test_validate_empty() {
        let result = MermaidValidator::validate("");
        assert!(!result.is_valid);
    }

    #[test]
    fn test_validate_invalid_type() {
        let diagram = "invalid\n    A --> B";
        let result = MermaidValidator::validate(diagram);
        assert!(!result.is_valid);
    }

    #[test]
    fn test_validate_unbalanced_brackets() {
        let diagram = "graph TD\n    A[Label --> B";
        let result = MermaidValidator::validate(diagram);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.contains("Unbalanced")));
    }

    #[test]
    fn test_fix_graph_lr() {
        let diagram = "graph LR\n    A --> B";
        let fixed = MermaidValidator::fix(diagram);
        assert!(fixed.contains("graph TD"));
    }

    #[test]
    fn test_fix_special_chars() {
        let diagram = "graph TD\n    A[Label <with> special]";
        let fixed = MermaidValidator::fix(diagram);
        assert!(fixed.contains("&lt;"));
        assert!(fixed.contains("&gt;"));
    }

    #[test]
    fn test_strip_invalid_diagrams() {
        let content = r#"# Title

Some text.

```mermaid
invalid diagram
```

More text."#;

        let result = MermaidValidator::strip_invalid_diagrams(content);
        assert!(!result.contains("```mermaid"));
        assert!(result.contains("Diagram removed"));
        assert!(result.contains("More text"));
    }

    #[test]
    fn test_strip_keeps_valid_diagrams() {
        let content = r#"# Title

```mermaid
graph TD
    A --> B
```

Text."#;

        let result = MermaidValidator::strip_invalid_diagrams(content);
        assert!(result.contains("```mermaid"));
        assert!(result.contains("graph TD"));
    }
}
