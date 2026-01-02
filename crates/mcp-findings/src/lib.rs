//! MCP Server for AI Review Findings
//!
//! This crate provides an MCP (Model Context Protocol) server that enables
//! AI models to create structured review findings during code review sessions.
//!
//! The server exposes tools like:
//! - `create_finding` - Create a new code review finding
//! - `list_findings` - List all findings for the current task
//! - `approve_review` - Mark the review as approved (no issues found)
//! - `complete_review` - Complete the review with findings

use orchestrator::{FileManager, FindingSeverity, FindingStatus, ReviewFinding, ReviewFindings};
use rmcp::{
    handler::server::{router::tool::ToolRouter, tool::Parameters},
    model::{ErrorData as McpError, *},
    schemars, tool, tool_handler, tool_router, ServerHandler,
};
use serde::Deserialize;
use std::borrow::Cow;
use std::future::Future;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn};
use uuid::Uuid;

/// Request to create a new finding
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateFindingRequest {
    /// The file path where the issue was found (optional for general findings)
    #[schemars(description = "The file path where the issue was found")]
    pub file_path: Option<String>,

    /// Starting line number (optional)
    #[schemars(description = "Starting line number of the issue")]
    pub line_start: Option<i32>,

    /// Ending line number (optional)
    #[schemars(description = "Ending line number of the issue")]
    pub line_end: Option<i32>,

    /// Short title describing the issue (max 100 chars)
    #[schemars(description = "Short title describing the issue (max 100 chars)")]
    pub title: String,

    /// Detailed description of the issue
    #[schemars(description = "Detailed description of the issue and why it should be fixed")]
    pub description: String,

    /// Severity level: "error", "warning", or "info"
    #[schemars(description = "Severity level: error (must fix), warning (should fix), info (suggestion)")]
    pub severity: String,
}

/// Request to complete the review
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CompleteReviewRequest {
    /// Overall summary of the review
    #[schemars(description = "Overall summary of the code review")]
    pub summary: String,

    /// Whether the code is approved (no blocking issues)
    #[schemars(description = "Whether the code is approved (true if no error-level issues)")]
    pub approved: bool,
}

/// Request to get a specific finding
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetFindingRequest {
    /// The ID of the finding to get
    #[schemars(description = "The ID of the finding (e.g., 'finding-1')")]
    pub finding_id: String,
}

/// Request to mark a finding as fixed
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct MarkFixedRequest {
    /// The ID of the finding to mark as fixed
    #[schemars(description = "The ID of the finding to mark as fixed (e.g., 'finding-1')")]
    pub finding_id: String,
}

/// MCP Findings Service for code review
#[derive(Clone)]
pub struct FindingsService {
    task_id: Uuid,
    session_id: Uuid,
    workspace_path: PathBuf,
    findings: Arc<Mutex<Vec<ReviewFinding>>>,
    summary: Arc<Mutex<Option<String>>>,
    approved: Arc<Mutex<Option<bool>>>,
    file_manager: Arc<FileManager>,
    tool_router: ToolRouter<FindingsService>,
}

impl FindingsService {
    /// Create a new findings service for a specific task and session
    pub fn new(task_id: Uuid, session_id: Uuid, workspace_path: PathBuf) -> Self {
        let file_manager = Arc::new(FileManager::new(workspace_path.clone()));
        Self {
            task_id,
            session_id,
            workspace_path,
            findings: Arc::new(Mutex::new(Vec::new())),
            summary: Arc::new(Mutex::new(None)),
            approved: Arc::new(Mutex::new(None)),
            file_manager,
            tool_router: Self::tool_router(),
        }
    }

    /// Get the collected findings
    pub async fn get_findings(&self) -> ReviewFindings {
        let findings = self.findings.lock().await.clone();
        let summary = self.summary.lock().await.clone().unwrap_or_default();

        ReviewFindings::with_findings(self.task_id, self.session_id, summary, findings)
    }

    /// Check if review is complete
    pub async fn is_complete(&self) -> bool {
        self.approved.lock().await.is_some()
    }

    /// Save findings to file
    pub async fn save_findings(&self) -> anyhow::Result<()> {
        let review_findings = self.get_findings().await;
        self.file_manager
            .write_findings(self.task_id, &review_findings)
            .await?;
        info!(
            task_id = %self.task_id,
            finding_count = review_findings.findings.len(),
            "Findings saved to file"
        );
        Ok(())
    }
}

#[tool_router]
impl FindingsService {
    #[tool(description = "Create a new code review finding. Use this to report issues found during review.")]
    async fn create_finding(
        &self,
        Parameters(request): Parameters<CreateFindingRequest>,
    ) -> Result<CallToolResult, McpError> {
        let mut findings = self.findings.lock().await;
        let finding_id = format!("finding-{}", findings.len() + 1);

        let severity = match request.severity.to_lowercase().as_str() {
            "error" => FindingSeverity::Error,
            "info" => FindingSeverity::Info,
            _ => FindingSeverity::Warning,
        };

        let finding = ReviewFinding {
            id: finding_id.clone(),
            file_path: request.file_path.clone(),
            line_start: request.line_start,
            line_end: request.line_end,
            title: request.title.clone(),
            description: request.description.clone(),
            severity,
            status: FindingStatus::Pending,
        };

        findings.push(finding);

        info!(
            task_id = %self.task_id,
            finding_id = %finding_id,
            title = %request.title,
            severity = %request.severity,
            "Created finding"
        );

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Finding created: {} ({})",
            finding_id, request.title
        ))]))
    }

    #[tool(description = "List all findings for this task. Returns both existing findings from file and any newly created in this session.")]
    async fn list_findings(&self) -> Result<CallToolResult, McpError> {
        // First try to load existing findings from file
        let file_findings = match self.file_manager.read_findings(self.task_id).await {
            Ok(Some(existing)) => existing.findings,
            _ => Vec::new(),
        };

        // Combine with session findings
        let session_findings = self.findings.lock().await;
        let mut all_findings: Vec<_> = file_findings
            .iter()
            .chain(session_findings.iter())
            .collect();

        // Deduplicate by ID
        all_findings.sort_by(|a, b| a.id.cmp(&b.id));
        all_findings.dedup_by(|a, b| a.id == b.id);

        if all_findings.is_empty() {
            return Ok(CallToolResult::success(vec![Content::text(
                "No findings found.",
            )]));
        }

        let list = all_findings
            .iter()
            .map(|f| {
                let location = match (&f.file_path, f.line_start) {
                    (Some(path), Some(line)) => format!(" at {}:{}", path, line),
                    (Some(path), None) => format!(" in {}", path),
                    _ => String::new(),
                };
                let status = match f.status {
                    FindingStatus::Pending => "",
                    FindingStatus::Fixed => " [FIXED]",
                    FindingStatus::Skipped => " [SKIPPED]",
                };
                format!(
                    "- {} [{}]{}{}: {}",
                    f.id,
                    f.severity.as_str(),
                    status,
                    location,
                    f.title
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Findings ({}):\n{}",
            all_findings.len(),
            list
        ))]))
    }

    #[tool(description = "Get detailed information about a specific finding by its ID.")]
    async fn get_finding(
        &self,
        Parameters(request): Parameters<GetFindingRequest>,
    ) -> Result<CallToolResult, McpError> {
        // First check session findings
        let session_findings = self.findings.lock().await;
        if let Some(f) = session_findings.iter().find(|f| f.id == request.finding_id) {
            let location = match (&f.file_path, f.line_start, f.line_end) {
                (Some(path), Some(start), Some(end)) if start != end => {
                    format!("Location: {}:{}-{}", path, start, end)
                }
                (Some(path), Some(line), _) => format!("Location: {}:{}", path, line),
                (Some(path), None, _) => format!("File: {}", path),
                _ => "Location: Not specified".to_string(),
            };
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "Finding: {}\n\nTitle: {}\nSeverity: {}\nStatus: {:?}\n{}\n\nDescription:\n{}",
                f.id, f.title, f.severity.as_str(), f.status, location, f.description
            ))]));
        }
        drop(session_findings);

        // Then check file findings
        if let Ok(Some(existing)) = self.file_manager.read_findings(self.task_id).await {
            if let Some(f) = existing.findings.iter().find(|f| f.id == request.finding_id) {
                let location = match (&f.file_path, f.line_start, f.line_end) {
                    (Some(path), Some(start), Some(end)) if start != end => {
                        format!("Location: {}:{}-{}", path, start, end)
                    }
                    (Some(path), Some(line), _) => format!("Location: {}:{}", path, line),
                    (Some(path), None, _) => format!("File: {}", path),
                    _ => "Location: Not specified".to_string(),
                };
                return Ok(CallToolResult::success(vec![Content::text(format!(
                    "Finding: {}\n\nTitle: {}\nSeverity: {}\nStatus: {:?}\n{}\n\nDescription:\n{}",
                    f.id, f.title, f.severity.as_str(), f.status, location, f.description
                ))]));
            }
        }

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Finding '{}' not found.",
            request.finding_id
        ))]))
    }

    #[tool(description = "Mark a finding as fixed after you've addressed the issue. This updates the findings file.")]
    async fn mark_fixed(
        &self,
        Parameters(request): Parameters<MarkFixedRequest>,
    ) -> Result<CallToolResult, McpError> {
        // Load existing findings from file
        let mut review_findings = match self.file_manager.read_findings(self.task_id).await {
            Ok(Some(existing)) => existing,
            Ok(None) => {
                return Ok(CallToolResult::success(vec![Content::text(
                    "No findings file found. Nothing to mark as fixed.",
                )]));
            }
            Err(e) => {
                return Err(McpError {
                    code: ErrorCode(-32603),
                    message: Cow::from(format!("Failed to read findings: {}", e)),
                    data: None,
                });
            }
        };

        // Find and update the finding
        let mut found = false;
        for finding in &mut review_findings.findings {
            if finding.id == request.finding_id {
                finding.status = FindingStatus::Fixed;
                found = true;
                break;
            }
        }

        if !found {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "Finding '{}' not found.",
                request.finding_id
            ))]));
        }

        // Save updated findings to file
        if let Err(e) = self
            .file_manager
            .write_findings(self.task_id, &review_findings)
            .await
        {
            return Err(McpError {
                code: ErrorCode(-32603),
                message: Cow::from(format!("Failed to save findings: {}", e)),
                data: None,
            });
        }

        info!(
            task_id = %self.task_id,
            finding_id = %request.finding_id,
            "Finding marked as fixed"
        );

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Finding '{}' marked as fixed.",
            request.finding_id
        ))]))
    }

    #[tool(description = "Approve the review. Use this when the code has no issues or only info-level suggestions.")]
    async fn approve_review(
        &self,
        Parameters(request): Parameters<CompleteReviewRequest>,
    ) -> Result<CallToolResult, McpError> {
        *self.summary.lock().await = Some(request.summary.clone());
        *self.approved.lock().await = Some(true);

        // Save findings to file
        if let Err(e) = self.save_findings().await {
            warn!(error = %e, "Failed to save findings");
            return Err(McpError {
                code: ErrorCode(-32603),
                message: Cow::from(format!("Failed to save findings: {}", e)),
                data: None,
            });
        }

        info!(
            task_id = %self.task_id,
            "Review approved"
        );

        Ok(CallToolResult::success(vec![Content::text(
            "Review approved. No blocking issues found.",
        )]))
    }

    #[tool(description = "Complete the review with findings. Use this when there are issues that need to be fixed.")]
    async fn complete_review(
        &self,
        Parameters(request): Parameters<CompleteReviewRequest>,
    ) -> Result<CallToolResult, McpError> {
        let findings = self.findings.lock().await;
        let error_count = findings
            .iter()
            .filter(|f| matches!(f.severity, FindingSeverity::Error))
            .count();
        let warning_count = findings
            .iter()
            .filter(|f| matches!(f.severity, FindingSeverity::Warning))
            .count();
        drop(findings);

        *self.summary.lock().await = Some(request.summary.clone());
        *self.approved.lock().await = Some(request.approved && error_count == 0);

        // Save findings to file
        if let Err(e) = self.save_findings().await {
            warn!(error = %e, "Failed to save findings");
            return Err(McpError {
                code: ErrorCode(-32603),
                message: Cow::from(format!("Failed to save findings: {}", e)),
                data: None,
            });
        }

        info!(
            task_id = %self.task_id,
            error_count = error_count,
            warning_count = warning_count,
            approved = request.approved,
            "Review completed"
        );

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Review completed. {} errors, {} warnings. Approved: {}",
            error_count, warning_count, request.approved
        ))]))
    }
}

#[tool_handler]
impl ServerHandler for FindingsService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "opencode-findings".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
            instructions: Some(
                "Use this server to report code review findings. \
                 Call create_finding for each issue found, then call \
                 approve_review (if no issues) or complete_review (if issues found)."
                    .to_string(),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_finding() {
        let service = FindingsService::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            PathBuf::from("/tmp/test"),
        );

        // Create a finding
        let request = CreateFindingRequest {
            file_path: Some("src/main.rs".to_string()),
            line_start: Some(42),
            line_end: Some(45),
            title: "Missing error handling".to_string(),
            description: "Function should handle errors".to_string(),
            severity: "error".to_string(),
        };

        let result = service
            .create_finding(Parameters(request))
            .await
            .unwrap();

        assert!(matches!(result, CallToolResult { .. }));

        // Check findings
        let findings = service.get_findings().await;
        assert_eq!(findings.findings.len(), 1);
        assert_eq!(findings.findings[0].title, "Missing error handling");
    }
}
