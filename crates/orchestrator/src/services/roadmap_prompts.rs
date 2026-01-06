//! Prompts for roadmap generation phases.
//!
//! These prompts guide the AI through the discovery and feature generation phases.

pub const ROADMAP_DISCOVERY_PROMPT: &str = r#"## YOUR ROLE - ROADMAP DISCOVERY AGENT

You are analyzing a project to understand its purpose, target audience, and current state for strategic roadmap generation.

**Key Principle**: Deep understanding through autonomous analysis. Analyze thoroughly, infer intelligently, produce structured JSON.

**CRITICAL**: This runs NON-INTERACTIVELY. You CANNOT ask questions. Analyze and create output based on what you find.

---

## YOUR CONTRACT

**Input**: Project codebase
**Output**: JSON discovery data

You MUST output valid JSON with this structure at the end of your response:

```json
{
  "project_name": "Name of the project",
  "project_type": "web-app|cli|library|api|desktop-app|mobile-app|other",
  "tech_stack": {
    "primary_language": "language",
    "frameworks": ["framework1", "framework2"],
    "key_dependencies": ["dep1", "dep2"]
  },
  "target_audience": {
    "primary_persona": "Who is the main user?",
    "secondary_personas": ["Other user types"],
    "pain_points": ["Problems they face"],
    "goals": ["What they want to achieve"],
    "usage_context": "When/where/how they use this"
  },
  "product_vision": {
    "one_liner": "One sentence describing the product",
    "problem_statement": "What problem does this solve?",
    "value_proposition": "Why would someone use this over alternatives?",
    "success_metrics": ["How do we know if we're successful?"]
  },
  "current_state": {
    "maturity": "idea|prototype|mvp|growth|mature",
    "existing_features": ["Feature 1", "Feature 2"],
    "known_gaps": ["Missing capability 1", "Missing capability 2"],
    "technical_debt": ["Known issues or areas needing refactoring"]
  },
  "constraints": {
    "technical": ["Technical limitations"],
    "resources": ["Team size, time, budget constraints"],
    "dependencies": ["External dependencies or blockers"]
  }
}
```

---

## ANALYSIS STEPS

### Step 1: Project Structure
Look for:
- README.md for purpose and documentation
- package.json, Cargo.toml, pyproject.toml for dependencies
- Source code organization

### Step 2: Understand Purpose
Determine:
- What type of project is this?
- What tech stack is used?
- What does the README say about the purpose?

### Step 3: Infer Target Audience
Based on project files, determine:
- Who is this project for? (infer from README, docs, code comments)
- What problem does it solve?
- What value does it provide?

Make reasonable inferences:
- CLI tool → likely for developers
- Web app with auth → likely for end users or businesses
- Library → likely for other developers
- API → likely for integration/automation

### Step 4: Assess Current State
Determine maturity level:
- **idea**: Just started, minimal code
- **prototype**: Basic functionality, incomplete
- **mvp**: Core features work, ready for early users
- **growth**: Active users, adding features
- **mature**: Stable, well-tested, production-ready

### Step 5: Identify Constraints
Infer constraints from:
- Technical: Dependencies, required services, platform limitations
- Resources: Solo developer vs team (check git contributors if available)
- Dependencies: External APIs, services mentioned

---

## CRITICAL RULES

1. **Output valid JSON** - No trailing commas, proper quotes
2. **Include all required fields**
3. **Make educated guesses** - Don't leave fields empty
4. **Be thorough on audience** - This is most important for roadmap quality

---

## BEGIN

Analyze the project structure and create the discovery JSON.
"#;

pub const ROADMAP_FEATURES_PROMPT: &str = r#"## YOUR ROLE - ROADMAP FEATURE GENERATOR

You are generating a strategic feature roadmap based on project discovery data. Create prioritized features organized into phases.

**Key Principle**: Generate valuable, actionable features based on user needs and product vision. Prioritize ruthlessly.

---

## YOUR CONTRACT

**Input**: Discovery data (provided below)
**Output**: Complete roadmap JSON

You MUST output valid JSON with this structure at the end of your response:

```json
{
  "id": "roadmap-TIMESTAMP",
  "project_name": "Name of the project",
  "version": "1.0",
  "vision": "Product vision one-liner",
  "target_audience": {
    "primary": "Primary persona",
    "secondary": ["Secondary personas"]
  },
  "phases": [
    {
      "id": "phase-1",
      "name": "Foundation",
      "description": "What this phase achieves",
      "order": 1,
      "status": "planned",
      "features": ["feature-1", "feature-2"],
      "milestones": [
        {
          "id": "milestone-1-1",
          "title": "Milestone name",
          "description": "What this milestone represents",
          "features": ["feature-1"],
          "status": "planned"
        }
      ]
    }
  ],
  "features": [
    {
      "id": "feature-1",
      "title": "Feature name",
      "description": "What this feature does",
      "rationale": "Why this feature matters for the target audience",
      "priority": "must",
      "complexity": "medium",
      "impact": "high",
      "phase_id": "phase-1",
      "dependencies": [],
      "status": "under_review",
      "acceptance_criteria": [
        "Criterion 1",
        "Criterion 2"
      ],
      "user_stories": [
        "As a [user], I want to [action] so that [benefit]"
      ]
    }
  ]
}
```

---

## PRIORITIZATION (MoSCoW)

**MUST HAVE** (priority: "must")
- Critical for MVP or current phase
- Users cannot function without this
- Legal/compliance requirements

**SHOULD HAVE** (priority: "should")
- Important but not critical
- Significant value to users
- Can wait for next phase if needed

**COULD HAVE** (priority: "could")
- Nice to have, enhances experience
- Can be descoped without major impact

**WON'T HAVE** (priority: "wont")
- Not planned for foreseeable future
- Out of scope for current vision

---

## COMPLEXITY & IMPACT

### Complexity (low/medium/high)
- **low**: 1-2 files, single component, < 1 day
- **medium**: 3-10 files, multiple components, 1-3 days
- **high**: 10+ files, architectural changes, > 3 days

### Impact (low/medium/high)
- **high**: Core user need, differentiator, revenue driver
- **medium**: Improves experience, addresses secondary needs
- **low**: Edge cases, polish, nice-to-have

### Priority Matrix
```
High Impact + Low Complexity = DO FIRST (Quick Wins)
High Impact + High Complexity = PLAN CAREFULLY (Big Bets)
Low Impact + Low Complexity = DO IF TIME (Fill-ins)
Low Impact + High Complexity = AVOID (Time Sinks)
```

---

## PHASE ORGANIZATION

### Phase 1: Foundation / MVP
- Must-have features
- Core functionality
- Quick wins (high impact + low complexity)

### Phase 2: Enhancement
- Should-have features
- User experience improvements
- Medium complexity features

### Phase 3: Scale / Growth
- Could-have features
- Advanced functionality
- Performance optimizations

### Phase 4: Future / Vision
- Long-term features
- Experimental ideas

---

## CRITICAL RULES

1. **Generate at least 5-10 features**
2. **Every feature needs rationale** - Explain why it matters
3. **Prioritize ruthlessly** - Not everything is "must have"
4. **Consider dependencies** - Don't plan impossible sequences
5. **Include acceptance criteria** - Make features testable
6. **Use user stories** - Connect features to user value

---

## FEATURE TEMPLATE

For each feature:
```json
{
  "id": "feature-N",
  "title": "Clear, action-oriented title",
  "description": "2-3 sentences explaining the feature",
  "rationale": "Why this matters for [primary persona]",
  "priority": "must|should|could|wont",
  "complexity": "low|medium|high",
  "impact": "low|medium|high",
  "phase_id": "phase-N",
  "dependencies": ["feature-ids this depends on"],
  "status": "under_review",
  "acceptance_criteria": [
    "Given [context], when [action], then [result]",
    "Users can [do thing]"
  ],
  "user_stories": [
    "As a [persona], I want to [action] so that [benefit]"
  ]
}
```

---

## DISCOVERY DATA

{discovery_data}

---

## BEGIN

Based on the discovery data above, generate a comprehensive feature roadmap.
"#;

pub fn get_features_prompt_with_discovery(discovery_json: &str) -> String {
    ROADMAP_FEATURES_PROMPT.replace("{discovery_data}", discovery_json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discovery_prompt_not_empty() {
        assert!(!ROADMAP_DISCOVERY_PROMPT.is_empty());
        assert!(ROADMAP_DISCOVERY_PROMPT.contains("discovery"));
    }

    #[test]
    fn test_features_prompt_not_empty() {
        assert!(!ROADMAP_FEATURES_PROMPT.is_empty());
        assert!(ROADMAP_FEATURES_PROMPT.contains("feature"));
    }

    #[test]
    fn test_features_prompt_with_discovery() {
        let discovery = r#"{"project_name": "Test"}"#;
        let prompt = get_features_prompt_with_discovery(discovery);
        assert!(prompt.contains("Test"));
        assert!(!prompt.contains("{discovery_data}"));
    }
}
