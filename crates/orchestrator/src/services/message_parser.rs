use opencode_client::models::Part;
use tracing::debug;
use uuid::Uuid;

use crate::activity_store::SessionActivityMsg;
use crate::error::{OrchestratorError, Result};
use crate::files::{FindingSeverity, FindingStatus, ReviewFinding, ReviewFindings};

#[derive(Debug, serde::Deserialize)]
pub struct RawReviewResponse {
    pub approved: bool,
    pub summary: String,
    #[serde(default)]
    pub findings: Vec<RawFinding>,
}

#[derive(Debug, serde::Deserialize)]
pub struct RawFinding {
    #[serde(default)]
    pub file_path: Option<String>,
    #[serde(default)]
    pub line_start: Option<i32>,
    #[serde(default)]
    pub line_end: Option<i32>,
    pub title: String,
    pub description: String,
    #[serde(default = "default_severity")]
    pub severity: String,
}

fn default_severity() -> String {
    "warning".to_string()
}

pub struct MessageParser;

impl MessageParser {
    pub fn extract_text_from_parts(parts: &[Part]) -> String {
        parts
            .iter()
            .filter_map(|part| {
                if part.r#type == opencode_client::models::part::Type::Text {
                    part.text.as_deref()
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn parse_message_parts(parts: &[Part]) -> Vec<SessionActivityMsg> {
        use opencode_client::models::part::Type;

        let mut activities = Vec::new();

        for part in parts {
            match part.r#type {
                Type::Text => {
                    let id = format!("text-{}", Uuid::new_v4());
                    let text = part.text.as_deref().unwrap_or("");
                    activities.push(SessionActivityMsg::agent_message(&id, text, false));
                }
                Type::Reasoning => {
                    let id = format!("reasoning-{}", Uuid::new_v4());
                    activities.push(SessionActivityMsg::Reasoning {
                        id,
                        content: part.text.clone().unwrap_or_default(),
                        timestamp: chrono::Utc::now(),
                    });
                }
                Type::Tool => {
                    let call_id = part.call_id.as_deref().unwrap_or("");
                    let tool_name = part.tool.as_deref().unwrap_or("unknown");

                    if let Some(ref state) = part.state {
                        let output = state.output.as_deref().unwrap_or("");
                        let error = state.error.as_deref().unwrap_or("");
                        let is_completed = !output.is_empty() || !error.is_empty();

                        if is_completed {
                            let success = error.is_empty();
                            let result = if success { output } else { error };
                            activities.push(SessionActivityMsg::tool_result(
                                call_id, tool_name, None, result, success,
                            ));
                        } else {
                            activities
                                .push(SessionActivityMsg::tool_call(call_id, tool_name, None));
                        }
                    } else {
                        activities.push(SessionActivityMsg::tool_call(call_id, tool_name, None));
                    }
                }
                Type::StepStart => {
                    let id = format!("step-{}", Uuid::new_v4());
                    activities.push(SessionActivityMsg::StepStart {
                        id,
                        step_name: None,
                        timestamp: chrono::Utc::now(),
                    });
                }
                _ => {
                    debug!("Skipping part type: {:?}", part.r#type);
                }
            }
        }

        activities
    }

    pub fn parse_sse_part(part: &serde_json::Value) -> Option<SessionActivityMsg> {
        let part_type = part.get("type")?.as_str()?;
        let id = part.get("id").and_then(|v| v.as_str()).unwrap_or("unknown");

        match part_type {
            "text" => {
                let text = part.get("text").and_then(|v| v.as_str()).unwrap_or("");
                let is_partial = part.get("time").and_then(|t| t.get("end")).is_none();
                Some(SessionActivityMsg::agent_message(id, text, is_partial))
            }
            "reasoning" => {
                let content = part.get("text").and_then(|v| v.as_str()).unwrap_or("");
                Some(SessionActivityMsg::Reasoning {
                    id: id.to_string(),
                    content: content.to_string(),
                    timestamp: chrono::Utc::now(),
                })
            }
            "tool" => {
                let call_id = part.get("callID").and_then(|v| v.as_str()).unwrap_or(id);
                let tool_name = part
                    .get("tool")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let state = part.get("state");

                let status = state
                    .and_then(|s| s.get("status"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("pending");

                if status == "completed" || status == "error" {
                    let success = status == "completed";
                    let output = state
                        .and_then(|s| s.get("output"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let error = state
                        .and_then(|s| s.get("error"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let result = if success { output } else { error };

                    Some(SessionActivityMsg::tool_result(
                        call_id, tool_name, None, result, success,
                    ))
                } else {
                    Some(SessionActivityMsg::tool_call(call_id, tool_name, None))
                }
            }
            "step-start" => Some(SessionActivityMsg::StepStart {
                id: id.to_string(),
                step_name: None,
                timestamp: chrono::Utc::now(),
            }),
            _ => {
                debug!(part_type = %part_type, "Skipping unknown SSE part type");
                None
            }
        }
    }

    pub fn parse_review_json(
        content: &str,
        task_id: Uuid,
        session_id: Uuid,
    ) -> Result<ReviewFindings> {
        let json_str = Self::extract_json_from_response(content);

        let raw: RawReviewResponse = serde_json::from_str(&json_str).map_err(|e| {
            tracing::warn!(
                error = %e,
                content_preview = %content.chars().take(500).collect::<String>(),
                "Failed to parse review JSON, falling back to text parsing"
            );
            OrchestratorError::ExecutionFailed(format!("Failed to parse review JSON: {}", e))
        })?;

        let findings: Vec<ReviewFinding> = raw
            .findings
            .into_iter()
            .enumerate()
            .map(|(i, f)| ReviewFinding {
                id: format!("finding-{}", i + 1),
                file_path: f.file_path,
                line_start: f.line_start,
                line_end: f.line_end,
                title: f.title,
                description: f.description,
                severity: match f.severity.to_lowercase().as_str() {
                    "error" => FindingSeverity::Error,
                    "info" => FindingSeverity::Info,
                    _ => FindingSeverity::Warning,
                },
                status: FindingStatus::Pending,
            })
            .collect();

        Ok(ReviewFindings::with_findings(
            task_id,
            session_id,
            raw.summary,
            findings,
        ))
    }

    pub fn extract_json_from_response(content: &str) -> String {
        if let Some(start) = content.find("```json") {
            if let Some(end) = content[start..]
                .find("```\n")
                .or(content[start..].rfind("```"))
            {
                let json_start = start + 7;
                let json_content = &content[json_start..start + end];
                return json_content.trim().to_string();
            }
        }

        if let Some(start) = content.find("```\n{") {
            if let Some(end) = content[start + 4..].find("\n```") {
                return content[start + 4..start + 4 + end].trim().to_string();
            }
        }

        if let Some(start) = content.find('{') {
            if let Some(end) = content.rfind('}') {
                return content[start..=end].to_string();
            }
        }

        content.to_string()
    }

    pub fn parse_review_response(content: &str) -> ReviewResult {
        let content_upper = content.to_uppercase();

        if content_upper.contains("APPROVED") && !content_upper.contains("NOT APPROVED") {
            ReviewResult::Approved
        } else if content_upper.contains("CHANGES_REQUESTED")
            || content_upper.contains("CHANGES REQUESTED")
            || content_upper.contains("REJECTED")
        {
            let feedback = content
                .lines()
                .skip_while(|line| {
                    let upper = line.to_uppercase();
                    !upper.contains("CHANGES_REQUESTED")
                        && !upper.contains("CHANGES REQUESTED")
                        && !upper.contains("REJECTED")
                        && !upper.contains("FEEDBACK")
                        && !upper.contains("ISSUES")
                })
                .skip(1)
                .collect::<Vec<_>>()
                .join("\n")
                .trim()
                .to_string();

            if feedback.is_empty() {
                ReviewResult::ChangesRequested(content.to_string())
            } else {
                ReviewResult::ChangesRequested(feedback)
            }
        } else {
            ReviewResult::ChangesRequested(
                "Review response unclear. Manual review required.".to_string(),
            )
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReviewResult {
    Approved,
    ChangesRequested(String),
    FindingsDetected(usize),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_review_approved() {
        let content = "## Review\n\nThe code looks good.\n\nAPPROVED";
        let result = MessageParser::parse_review_response(content);
        assert_eq!(result, ReviewResult::Approved);
    }

    #[test]
    fn test_parse_review_changes_requested() {
        let content = "## Review\n\nCHANGES_REQUESTED\n\n- Fix the error handling\n- Add tests";
        let result = MessageParser::parse_review_response(content);
        match result {
            ReviewResult::ChangesRequested(feedback) => {
                assert!(feedback.contains("Fix the error handling"));
            }
            _ => panic!("Expected ChangesRequested"),
        }
    }

    #[test]
    fn test_extract_json_from_markdown() {
        let content = "Some text\n```json\n{\"approved\": true}\n```\nMore text";
        let json = MessageParser::extract_json_from_response(content);
        assert!(json.contains("approved"));
    }

    #[test]
    fn test_extract_json_raw() {
        let content = "Response: {\"approved\": false, \"summary\": \"test\"}";
        let json = MessageParser::extract_json_from_response(content);
        assert!(json.starts_with('{'));
        assert!(json.ends_with('}'));
    }
}
