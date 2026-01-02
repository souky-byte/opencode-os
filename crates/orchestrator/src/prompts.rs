use opencode_core::Task;

/// User review comment for fix prompts
#[derive(Debug, Clone)]
pub struct UserReviewComment {
    pub file_path: String,
    pub line_start: i64,
    pub line_end: i64,
    pub side: String,
    pub content: String,
}

pub struct PhasePrompts;

impl PhasePrompts {
    pub fn planning(task: &Task) -> String {
        format!(
            r#"You are analyzing a development task. Create a detailed implementation plan.

## Task
**Title:** {title}
**Description:** {description}

## CRITICAL: Output the ENTIRE plan in your response

Do NOT use any tools to write files. Simply output the complete plan as your response.
Your response text will be automatically saved to the plan file.

## Required Plan Structure

# Implementation Plan: {title}

**Task ID:** {id}
**Complexity:** [S/M/L/XL]

## Overview
[Brief technical analysis - what needs to be done and why]

## Files to Modify
- path/to/file1.rs - [what changes]
- path/to/file2.ts - [what changes]

## Phase 1: [Short Phase Title]

[Detailed description of what this phase accomplishes]

### Steps
- Step 1.1: [Specific action]
- Step 1.2: [Specific action]

### Files
- path/to/file.rs

## Phase 2: [Short Phase Title]

[Detailed description]

### Steps
- Step 2.1: [Specific action]

[Continue for all phases...]

## Risks and Mitigations
- Risk 1: [description] - Mitigation: [how to handle]

## Testing
- [What needs to be tested]

## Phase Guidelines
- Use EXACTLY the format shown above for phases (two hashes, space, Phase, number, colon, title)
- S complexity: 1-2 phases
- M complexity: 2-4 phases
- L/XL complexity: 4-8 phases
- Each phase = one independent unit of work

Output the complete plan now. Do NOT implement anything."#,
            title = task.title,
            description = task.description,
            id = task.id
        )
    }

    pub fn implementation(task: &Task) -> String {
        let plan_path = format!(".opencode-studio/kanban/plans/{}.md", task.id);

        format!(
            r#"Implement the following task according to the plan.

## Task
**Title:** {title}
**Plan:** Read from `{plan_path}`

## Instructions
1. Read the plan carefully
2. Implement each step
3. Write tests if applicable
4. Commit your changes

Start implementation now."#,
            title = task.title,
            plan_path = plan_path
        )
    }

    pub fn implementation_with_plan(task: &Task, plan: Option<&str>) -> String {
        if let Some(plan_content) = plan {
            format!(
                r#"Implement the following task according to the plan.

## Task
**Title:** {title}
**Description:** {description}

## Plan
{plan_content}

## Instructions
1. Follow the plan step by step
2. Implement each item thoroughly
3. Write tests if applicable
4. Ensure code quality and consistency

Start implementation now."#,
                title = task.title,
                description = task.description,
                plan_content = plan_content
            )
        } else {
            format!(
                r#"Implement the following task.

## Task
**Title:** {title}
**Description:** {description}

## Instructions
1. Analyze the task requirements
2. Implement the feature/fix
3. Write tests if applicable
4. Ensure code quality and consistency

Start implementation now."#,
                title = task.title,
                description = task.description
            )
        }
    }

    pub fn review(task: &Task, diff: &str) -> String {
        format!(
            r#"Review the following code changes for task: {title}

## Task Description
{description}

## Diff
```
{diff}
```

## Review Criteria
1. Code quality and style
2. Correctness - does it solve the task?
3. Tests - are they adequate?
4. Security concerns
5. Breaking changes

## Output Format
You MUST respond with a JSON object in this exact format:

```json
{{
  "approved": true,
  "summary": "Overall assessment of the changes...",
  "findings": []
}}
```

If there are issues, include them in the findings array:

```json
{{
  "approved": false,
  "summary": "Overall assessment...",
  "findings": [
    {{
      "file_path": "src/main.rs",
      "line_start": 42,
      "line_end": 45,
      "title": "Missing error handling",
      "description": "The function does not handle the case when the input is invalid. This could lead to a panic at runtime.",
      "severity": "error"
    }},
    {{
      "file_path": "src/utils.rs",
      "line_start": 10,
      "title": "Consider using const",
      "description": "This value could be a const instead of a let binding for better optimization.",
      "severity": "info"
    }}
  ]
}}
```

Severity levels:
- "error" - Must be fixed before merge
- "warning" - Should be fixed but not blocking
- "info" - Suggestion for improvement

Respond ONLY with the JSON object, no additional text."#,
            title = task.title,
            description = task.description,
            diff = diff
        )
    }

    /// Generate prompt for AI review using MCP tools
    pub fn review_with_mcp(task: &Task, diff: &str) -> String {
        format!(
            r#"Review the following code changes for task: {title}

## Task Description
{description}

## Diff
```
{diff}
```

## Review Criteria
1. Code quality and style
2. Correctness - does it solve the task?
3. Tests - are they adequate?
4. Security concerns
5. Breaking changes

## How to Report Findings

You have access to the "opencode-findings" MCP server with the following tools:

1. **create_finding** - Use this to report each issue you find:
   - `file_path`: The file where the issue is located (optional)
   - `line_start`: Starting line number (optional)
   - `line_end`: Ending line number (optional)
   - `title`: Short description of the issue (max 100 chars)
   - `description`: Detailed explanation of the issue
   - `severity`: "error" (must fix), "warning" (should fix), or "info" (suggestion)

2. **list_findings** - Use this to see all findings you've created

3. **approve_review** - Use this when the code has NO issues or only info-level suggestions
   - `summary`: Overall assessment of the changes
   - `approved`: true

4. **complete_review** - Use this when there ARE issues that need to be fixed
   - `summary`: Overall assessment of the changes
   - `approved`: false (if there are error-level issues)

## Instructions

1. Analyze the diff carefully
2. For each issue found, call `create_finding` with the appropriate details
3. After reviewing all changes:
   - If no issues or only info-level issues: call `approve_review`
   - If there are error/warning issues: call `complete_review` with approved=false

Start reviewing now."#,
            title = task.title,
            description = task.description,
            diff = diff
        )
    }

    /// Generate prompt for fixing specific findings
    pub fn fix_findings(task: &Task, findings: &[crate::files::ReviewFinding]) -> String {
        let findings_text = findings
            .iter()
            .enumerate()
            .map(|(i, f)| {
                let location = match (&f.file_path, f.line_start) {
                    (Some(path), Some(line)) => format!("{path}:{line}"),
                    (Some(path), None) => path.clone(),
                    _ => "Unknown location".to_string(),
                };
                format!(
                    "{}. [{:?}] {} ({})\n   {}\n",
                    i + 1,
                    f.severity,
                    f.title,
                    location,
                    f.description
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            r#"Fix the following issues identified in the code review for task: {title}

## Issues to Fix
{findings_text}

## Instructions
1. Address each issue mentioned above
2. Make minimal changes - only fix what's needed
3. Ensure the fix is complete and correct
4. Update tests if the fix requires it

Fix the issues now."#,
            title = task.title,
            findings_text = findings_text
        )
    }

    /// Generate prompt for fix phase using MCP tools
    pub fn fix_with_mcp(task: &Task) -> String {
        format!(
            r#"Fix the issues identified in the code review for task: {title}

## Task Description
{description}

## How to Use MCP Tools

You have access to the "opencode-findings" MCP server with the following tools:

1. **list_findings** - First, use this to see all findings that need to be fixed
   - Returns a list of findings with their IDs, locations, and descriptions

2. **get_finding** - Get details about a specific finding
   - `finding_id`: The ID of the finding

3. **mark_fixed** - After fixing an issue, mark it as fixed
   - `finding_id`: The ID of the finding you fixed

## Instructions

1. Call `list_findings` to see all issues that need fixing
2. For each finding:
   - Read the finding details
   - Navigate to the file and line mentioned
   - Fix the issue
   - Call `mark_fixed` with the finding ID
3. After fixing all issues, the review will be re-run automatically

Start by listing the findings and fixing them one by one."#,
            title = task.title,
            description = task.description
        )
    }

    pub fn fix_issues(task: &Task, feedback: &str) -> String {
        format!(
            r#"Fix the issues identified in the code review for task: {title}

## Review Feedback
{feedback}

## Instructions
1. Address each issue mentioned
2. Update tests if needed
3. Ensure the fix is complete

Fix the issues now."#,
            title = task.title,
            feedback = feedback
        )
    }

    /// Generate prompt for fixing user-provided review comments
    pub fn fix_user_comments(task: &Task, comments: &[UserReviewComment]) -> String {
        let comments_text = comments
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let line_info = if c.line_start == c.line_end {
                    format!("line {}", c.line_start)
                } else {
                    format!("lines {}-{}", c.line_start, c.line_end)
                };
                format!(
                    "{}. **{}** ({}, {} side)\n   {}\n",
                    i + 1,
                    c.file_path,
                    line_info,
                    c.side,
                    c.content
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            r#"Fix the issues identified by the user during code review for task: {title}

## Task Description
{description}

## User Review Comments
The user has reviewed your changes and provided the following feedback:

{comments_text}

## Instructions
1. Carefully read each comment and understand what the user is asking for
2. Navigate to each file and line mentioned
3. Make the necessary changes to address the feedback
4. Keep changes minimal - only fix what the user requested
5. Ensure the fixes are complete and correct

Start fixing the issues now."#,
            title = task.title,
            description = task.description,
            comments_text = comments_text
        )
    }

    /// Generate prompt for a single implementation phase
    pub fn implementation_phase(
        task: &Task,
        phase: &crate::files::PlanPhase,
        context: &crate::files::PhaseContext,
    ) -> String {
        let prev_context = context
            .previous_summary
            .as_ref()
            .map(|s| {
                let files_list = s
                    .files_changed
                    .iter()
                    .map(|f| format!("- {}", f))
                    .collect::<Vec<_>>()
                    .join("\n");

                format!(
                    r#"## Summary of Previous Phase (Phase {})

{}

### Changed Files
{}

### Notes for This Phase
{}"#,
                    s.phase_number,
                    s.summary,
                    if files_list.is_empty() {
                        "(none)".to_string()
                    } else {
                        files_list
                    },
                    s.notes.as_deref().unwrap_or("(none)")
                )
            })
            .unwrap_or_default();

        format!(
            r#"You are implementing Phase {phase_num} of {total_phases} for this task.

## Task
**Title:** {title}
**Description:** {description}

{prev_context}

## Current Phase: {phase_title}

{phase_content}

## Instructions

1. Implement ONLY this phase - do not work ahead to future phases
2. Complete each step thoroughly before moving on
3. Write tests where applicable
4. Commit your changes when done
5. **MANDATORY: Write the PHASE_SUMMARY block** (see format below)

## PHASE_SUMMARY Format (REQUIRED)

You MUST end your response with this exact format:

```
### PHASE_SUMMARY
**Summary:** [1-2 paragraphs describing what was done]
**Changed files:**
- path/to/file1.rs
- path/to/file2.rs
**Notes for next phase:** [Important context the next phase needs to know]
### END_PHASE_SUMMARY
```

⚠️ **CRITICAL**: The phase is NOT complete until you write the PHASE_SUMMARY block. This is required for the next phase to receive context about your work.

Start implementing now."#,
            phase_num = context.phase_number,
            total_phases = context.total_phases,
            title = task.title,
            description = task.description,
            prev_context = prev_context,
            phase_title = phase.title,
            phase_content = phase.content
        )
    }

    /// Generate follow-up prompt to request phase summary when AI forgot to include it
    pub fn request_phase_summary(phase_number: u32, phase_title: &str) -> String {
        format!(
            r#"You just completed Phase {phase_number}: {phase_title}, but you forgot to include the PHASE_SUMMARY block.

Please provide ONLY the summary now, in this exact format:

### PHASE_SUMMARY
**Summary:** [1-2 paragraphs describing what was done]
**Changed files:**
- path/to/file1.rs
- path/to/file2.rs
**Notes for next phase:** [Important context the next phase needs to know]
### END_PHASE_SUMMARY

Do not repeat any implementation work. Just provide the summary of what you did."#,
            phase_number = phase_number,
            phase_title = phase_title
        )
    }

    pub fn replan(task: &Task, feedback: &str) -> String {
        format!(
            r#"Revise the implementation plan based on feedback.

## Task
**Title:** {title}
**Description:** {description}

## Feedback on Previous Plan
{feedback}

## CRITICAL: Output the ENTIRE revised plan in your response

Do NOT use any tools to write files. Simply output the complete plan as your response.
Your response text will be automatically saved to the plan file.

## Required Plan Structure

# Implementation Plan: {title} (Revised)

**Task ID:** {id}
**Complexity:** [S/M/L/XL]

## Changes from Previous Plan
[What was changed based on feedback]

## Overview
[Brief technical analysis]

## Files to Modify
- path/to/file.rs - [what changes]

## Phase 1: [Short Phase Title]

[Detailed description]

### Steps
- Step 1.1: [Specific action]

### Files
- path/to/file.rs

## Phase 2: [Short Phase Title]

[Continue for all phases...]

## Risks and Mitigations
- Risk 1: [description] - Mitigation: [how to handle]

## Phase Guidelines
- Use EXACTLY the format shown above for phases (two hashes, space, Phase, number, colon, title)
- Each phase = one independent unit of work

Output the complete revised plan now. Do NOT implement anything."#,
            title = task.title,
            description = task.description,
            feedback = feedback,
            id = task.id
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn sample_task() -> Task {
        Task {
            id: Uuid::new_v4(),
            title: "Test Task".to_string(),
            description: "A test description".to_string(),
            status: opencode_core::TaskStatus::Todo,
            roadmap_item_id: None,
            workspace_path: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn test_planning_prompt_contains_task_info() {
        let task = sample_task();
        let prompt = PhasePrompts::planning(&task);

        assert!(prompt.contains(&task.title));
        assert!(prompt.contains(&task.description));
        assert!(prompt.contains(&task.id.to_string()));
    }

    #[test]
    fn test_implementation_prompt_references_plan() {
        let task = sample_task();
        let prompt = PhasePrompts::implementation(&task);

        assert!(prompt.contains(".opencode-studio/kanban/plans/"));
        assert!(prompt.contains(&task.id.to_string()));
    }

    #[test]
    fn test_implementation_with_plan_includes_content() {
        let task = sample_task();
        let plan = "## Steps\n1. Do something\n2. Do something else";
        let prompt = PhasePrompts::implementation_with_plan(&task, Some(plan));

        assert!(prompt.contains(&task.title));
        assert!(prompt.contains("Do something"));
        assert!(prompt.contains("Do something else"));
    }

    #[test]
    fn test_implementation_without_plan() {
        let task = sample_task();
        let prompt = PhasePrompts::implementation_with_plan(&task, None);

        assert!(prompt.contains(&task.title));
        assert!(prompt.contains(&task.description));
        assert!(prompt.contains("Analyze the task requirements"));
    }

    #[test]
    fn test_review_prompt_contains_diff() {
        let task = sample_task();
        let diff = "+ added line\n- removed line";
        let prompt = PhasePrompts::review(&task, diff);

        assert!(prompt.contains(diff));
        assert!(prompt.contains("approved"));
        assert!(prompt.contains("findings"));
    }

    #[test]
    fn test_fix_issues_contains_feedback() {
        let task = sample_task();
        let feedback = "Error handling is missing";
        let prompt = PhasePrompts::fix_issues(&task, feedback);

        assert!(prompt.contains(&task.title));
        assert!(prompt.contains(feedback));
    }

    #[test]
    fn test_replan_contains_feedback() {
        let task = sample_task();
        let feedback = "Plan is too vague";
        let prompt = PhasePrompts::replan(&task, feedback);

        assert!(prompt.contains(&task.title));
        assert!(prompt.contains(feedback));
        assert!(prompt.contains("revised plan"));
    }
}
