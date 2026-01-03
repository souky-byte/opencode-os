//! Plan parser for detecting phases in markdown implementation plans
//!
//! This module parses markdown plans to extract individual implementation phases.
//! It supports various formats for phase headers and is backward compatible with
//! single-phase (legacy) plans.

use regex::Regex;

use crate::files::{ParsedPlan, PlanPhase};

/// Parse a markdown plan to extract phases
///
/// Detects phase headers in various formats:
/// - `## Phase 1: Title` or `### Phase 1: Title`
/// - `## Phase 1 - Title` or `### Phase 1 - Title`
/// - `## Fáze 1: Title` (Czech)
/// - `## Step 1: Title` or `### Krok 1: Title`
///
/// If no phases are detected, returns the entire plan as a single phase.
pub fn parse_plan_phases(plan_content: &str) -> ParsedPlan {
    // Pattern to match phase headers
    // Captures: full match, phase number, separator, title
    let phase_pattern =
        Regex::new(r"(?m)^(##?#?)\s*(?:Phase|Fáze|Step|Krok)\s+(\d+)\s*[:\-–]\s*(.+)$")
            .expect("Invalid phase regex pattern");

    let mut phases = Vec::new();
    let mut preamble = String::new();
    let mut current_phase: Option<(u32, String, usize)> = None; // (number, title, start_pos)

    // Find all phase headers
    let matches: Vec<_> = phase_pattern.captures_iter(plan_content).collect();

    if matches.is_empty() {
        // No phases found - return entire plan as single phase
        return ParsedPlan {
            preamble: String::new(),
            phases: vec![PlanPhase {
                number: 1,
                title: "Implementation".to_string(),
                content: plan_content.trim().to_string(),
            }],
        };
    }

    // Get positions in the text
    let positions: Vec<_> = phase_pattern
        .find_iter(plan_content)
        .map(|m| m.start())
        .collect();

    // Extract preamble (content before first phase)
    if let Some(&first_pos) = positions.first() {
        preamble = plan_content[..first_pos].trim().to_string();
    }

    // Process each phase
    for (i, caps) in matches.iter().enumerate() {
        let phase_number: u32 = caps
            .get(2)
            .and_then(|m| m.as_str().parse().ok())
            .unwrap_or((i + 1) as u32);

        let title = caps
            .get(3)
            .map(|m| m.as_str().trim().to_string())
            .unwrap_or_else(|| format!("Phase {}", phase_number));

        let start_pos = positions[i];

        // If there was a previous phase, finalize its content
        if let Some((prev_number, prev_title, prev_start)) = current_phase.take() {
            let content = extract_phase_content(plan_content, prev_start, start_pos);
            phases.push(PlanPhase {
                number: prev_number,
                title: prev_title,
                content,
            });
        }

        current_phase = Some((phase_number, title, start_pos));
    }

    // Finalize the last phase
    if let Some((number, title, start)) = current_phase {
        let content = extract_phase_content(plan_content, start, plan_content.len());
        phases.push(PlanPhase {
            number,
            title,
            content,
        });
    }

    // Renumber phases sequentially if needed
    for (i, phase) in phases.iter_mut().enumerate() {
        phase.number = (i + 1) as u32;
    }

    ParsedPlan { preamble, phases }
}

/// Extract the content of a phase between start and end positions
fn extract_phase_content(content: &str, start: usize, end: usize) -> String {
    content[start..end].trim().to_string()
}

/// Extract a phase summary from AI response text
///
/// Looks for the structured summary block between PHASE_SUMMARY markers
pub fn extract_phase_summary(response: &str) -> Option<ExtractedSummary> {
    let summary_pattern =
        Regex::new(r"(?s)###\s*PHASE_SUMMARY\s*\n(.*?)###\s*END_PHASE_SUMMARY").ok()?;

    let caps = summary_pattern.captures(response)?;
    let block = caps.get(1)?.as_str();

    // Parse the structured content
    let summary = extract_field(block, "Shrnutí", "Summary")?;
    let files_changed = extract_file_list(block);
    let notes = extract_field(block, "Poznámky pro další fázi", "Notes for next phase");

    Some(ExtractedSummary {
        summary,
        files_changed,
        notes,
    })
}

/// Extracted summary from AI response
#[derive(Debug, Clone)]
pub struct ExtractedSummary {
    pub summary: String,
    pub files_changed: Vec<String>,
    pub notes: Option<String>,
}

fn extract_field(block: &str, czech_label: &str, english_label: &str) -> Option<String> {
    let patterns = [
        format!(r"(?m)^\*\*{}:\*\*\s*(.+?)(?:\n\*\*|$)", czech_label),
        format!(r"(?m)^\*\*{}:\*\*\s*(.+?)(?:\n\*\*|$)", english_label),
    ];

    for pattern in &patterns {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(caps) = re.captures(block) {
                if let Some(m) = caps.get(1) {
                    let value = m.as_str().trim();
                    if !value.is_empty() {
                        return Some(value.to_string());
                    }
                }
            }
        }
    }

    None
}

fn extract_file_list(block: &str) -> Vec<String> {
    let mut files = Vec::new();

    // Look for "Changed files:" or "Změněné soubory:" section
    let list_pattern =
        Regex::new(r"(?s)\*\*(?:Změněné soubory|Changed files):\*\*\s*\n((?:\s*-\s*.+\n?)+)");

    if let Ok(re) = list_pattern {
        if let Some(caps) = re.captures(block) {
            if let Some(list) = caps.get(1) {
                for line in list.as_str().lines() {
                    let line = line.trim();
                    if let Some(stripped) = line.strip_prefix('-') {
                        let file = stripped.trim();
                        if !file.is_empty() {
                            files.push(file.to_string());
                        }
                    }
                }
            }
        }
    }

    files
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_phase_plan() {
        let plan = r#"
# Implementation Plan

This is a simple plan without phases.

## Steps
1. Do something
2. Do something else
"#;

        let parsed = parse_plan_phases(plan);
        assert!(parsed.is_single_phase());
        assert_eq!(parsed.phases.len(), 1);
        assert!(parsed.phases[0].content.contains("Do something"));
    }

    #[test]
    fn test_parse_multi_phase_plan() {
        let plan = r#"
# Implementation Plan

This is the preamble with overview.

## Phase 1: Setup

Create the basic structure.
- Add files
- Configure settings

## Phase 2: Implementation

Implement the core logic.
- Write functions
- Add tests

## Phase 3: Integration

Connect everything together.
"#;

        let parsed = parse_plan_phases(plan);
        assert!(!parsed.is_single_phase());
        assert_eq!(parsed.phases.len(), 3);
        assert_eq!(parsed.total_phases(), 3);

        assert!(parsed.preamble.contains("preamble"));
        assert_eq!(parsed.phases[0].number, 1);
        assert_eq!(parsed.phases[0].title, "Setup");
        assert!(parsed.phases[0]
            .content
            .contains("Create the basic structure"));

        assert_eq!(parsed.phases[1].number, 2);
        assert_eq!(parsed.phases[1].title, "Implementation");

        assert_eq!(parsed.phases[2].number, 3);
        assert_eq!(parsed.phases[2].title, "Integration");
    }

    #[test]
    fn test_parse_czech_phases() {
        let plan = r#"
# Plán

## Fáze 1: Příprava

Připravit prostředí.

## Fáze 2: Vývoj

Implementovat funkce.
"#;

        let parsed = parse_plan_phases(plan);
        assert_eq!(parsed.phases.len(), 2);
        assert_eq!(parsed.phases[0].title, "Příprava");
        assert_eq!(parsed.phases[1].title, "Vývoj");
    }

    #[test]
    fn test_parse_step_format() {
        let plan = r#"
## Step 1 - Initial Setup

Do the setup.

## Step 2 - Core Work

Do the work.
"#;

        let parsed = parse_plan_phases(plan);
        assert_eq!(parsed.phases.len(), 2);
        assert_eq!(parsed.phases[0].title, "Initial Setup");
        assert_eq!(parsed.phases[1].title, "Core Work");
    }

    #[test]
    fn test_extract_phase_summary() {
        let response = r#"
I've completed the work.

### PHASE_SUMMARY
**Shrnutí:** Implementoval jsem základní strukturu a přidal testy.
**Změněné soubory:**
- src/lib.rs
- src/types.rs
- tests/integration.rs
**Poznámky pro další fázi:** Připravit databázové připojení.
### END_PHASE_SUMMARY

Done!
"#;

        let summary = extract_phase_summary(response).unwrap();
        assert!(summary.summary.contains("základní strukturu"));
        assert_eq!(summary.files_changed.len(), 3);
        assert!(summary.files_changed.contains(&"src/lib.rs".to_string()));
        assert!(summary.notes.as_ref().unwrap().contains("databázové"));
    }

    #[test]
    fn test_extract_english_summary() {
        let response = r#"
### PHASE_SUMMARY
**Summary:** Implemented the basic structure.
**Changed files:**
- src/main.rs
**Notes for next phase:** Add error handling.
### END_PHASE_SUMMARY
"#;

        let summary = extract_phase_summary(response).unwrap();
        assert!(summary.summary.contains("basic structure"));
        assert_eq!(summary.files_changed.len(), 1);
    }
}
