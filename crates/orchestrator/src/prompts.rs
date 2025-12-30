use opencode_core::Task;

pub struct PhasePrompts;

impl PhasePrompts {
    pub fn planning(task: &Task) -> String {
        format!(
            r#"You are analyzing a development task. Create a detailed implementation plan.

## Task
**Title:** {title}
**Description:** {description}

## Required Output
Save your analysis to: `.opencode-studio/kanban/plans/{id}.md`

The plan should include:
1. Technical analysis
2. Files to modify/create
3. Step-by-step implementation steps
4. Potential risks
5. Estimated complexity (S/M/L/XL)

Do NOT implement anything yet. Only create the plan."#,
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

    pub fn review(task: &Task, diff: &str) -> String {
        format!(
            r#"Review the following code changes for task: {title}

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

## Output
Save your review to: `.opencode-studio/kanban/reviews/{id}.md`

If approved, respond with: APPROVED
If changes needed, respond with: CHANGES_REQUESTED and explain what needs fixing."#,
            title = task.title,
            diff = diff,
            id = task.id
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
3. Commit your changes

Fix the issues now."#,
            title = task.title,
            feedback = feedback
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
    fn test_review_prompt_contains_diff() {
        let task = sample_task();
        let diff = "+ added line\n- removed line";
        let prompt = PhasePrompts::review(&task, diff);

        assert!(prompt.contains(diff));
        assert!(prompt.contains("APPROVED"));
        assert!(prompt.contains("CHANGES_REQUESTED"));
    }
}
