//! File management for plans, reviews, and findings
//!
//! Handles reading/writing of plan and review markdown files and
//! structured findings JSON in the `.opencode-studio/kanban/` directory structure.

use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::fs;
use tracing::{debug, info};
use uuid::Uuid;

use crate::error::{OrchestratorError, Result};

/// Base directory for OpenCode Studio files
const STUDIO_DIR: &str = ".opencode-studio";
/// Directory for kanban-related files
const KANBAN_DIR: &str = "kanban";
/// Directory for plan files
const PLANS_DIR: &str = "plans";
/// Directory for review files
const REVIEWS_DIR: &str = "reviews";
/// Directory for findings files
const FINDINGS_DIR: &str = "findings";
/// Directory for phase summaries
const PHASES_DIR: &str = "phases";

// ============================================================================
// Review Findings Types
// ============================================================================

/// Severity level of a finding
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum FindingSeverity {
    Error,
    Warning,
    Info,
}

impl FindingSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            FindingSeverity::Error => "error",
            FindingSeverity::Warning => "warning",
            FindingSeverity::Info => "info",
        }
    }
}

/// Status of a finding
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum FindingStatus {
    Pending,
    Fixed,
    Skipped,
}

/// A single review finding
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ReviewFinding {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_start: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_end: Option<i32>,
    pub title: String,
    pub description: String,
    pub severity: FindingSeverity,
    pub status: FindingStatus,
}

/// Collection of findings from an AI review
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct ReviewFindings {
    pub task_id: Uuid,
    pub session_id: Uuid,
    pub approved: bool,
    pub created_at: DateTime<Utc>,
    pub summary: String,
    pub findings: Vec<ReviewFinding>,
}

impl ReviewFindings {
    /// Create a new approved review with no findings
    pub fn approved(task_id: Uuid, session_id: Uuid, summary: String) -> Self {
        Self {
            task_id,
            session_id,
            approved: true,
            created_at: Utc::now(),
            summary,
            findings: Vec::new(),
        }
    }

    /// Create a new review with findings
    pub fn with_findings(
        task_id: Uuid,
        session_id: Uuid,
        summary: String,
        findings: Vec<ReviewFinding>,
    ) -> Self {
        Self {
            task_id,
            session_id,
            approved: findings.is_empty(),
            created_at: Utc::now(),
            summary,
            findings,
        }
    }

    /// Count pending findings
    pub fn pending_count(&self) -> usize {
        self.findings
            .iter()
            .filter(|f| f.status == FindingStatus::Pending)
            .count()
    }
}

// ============================================================================
// Multi-Phase Implementation Types
// ============================================================================

/// A parsed plan with detected phases
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct ParsedPlan {
    /// Content before the first phase (intro, overview, etc.)
    pub preamble: String,
    /// Individual implementation phases
    pub phases: Vec<PlanPhase>,
}

impl ParsedPlan {
    /// Check if this is a single-phase (legacy) plan
    pub fn is_single_phase(&self) -> bool {
        self.phases.len() <= 1
    }

    /// Get total number of phases
    pub fn total_phases(&self) -> u32 {
        self.phases.len() as u32
    }
}

/// A single phase within a plan
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct PlanPhase {
    /// Phase number (1-indexed)
    pub number: u32,
    /// Phase title (e.g., "Setup database models")
    pub title: String,
    /// Full content of this phase
    pub content: String,
}

/// Context passed between implementation phases
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct PhaseContext {
    /// Current phase number being executed (1-indexed)
    pub phase_number: u32,
    /// Total number of phases
    pub total_phases: u32,
    /// Summary from the immediately previous phase
    pub previous_summary: Option<PhaseSummary>,
    /// All completed phase summaries
    pub completed_phases: Vec<PhaseSummary>,
}

impl PhaseContext {
    /// Create initial context for starting phased implementation
    pub fn new(total_phases: u32) -> Self {
        Self {
            phase_number: 1,
            total_phases,
            previous_summary: None,
            completed_phases: Vec::new(),
        }
    }

    /// Check if all phases are complete
    pub fn is_complete(&self) -> bool {
        self.phase_number > self.total_phases
    }

    /// Advance to the next phase after completing current one
    pub fn advance(&mut self, summary: PhaseSummary) {
        self.completed_phases.push(summary.clone());
        self.previous_summary = Some(summary);
        self.phase_number += 1;
    }
}

/// Summary of a completed implementation phase
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PhaseSummary {
    /// Phase number that was completed
    pub phase_number: u32,
    /// Phase title
    pub title: String,
    /// Summary of what was done
    pub summary: String,
    /// List of files that were changed
    pub files_changed: Vec<String>,
    /// Notes for the next phase (important context)
    pub notes: Option<String>,
    /// When the phase was completed
    pub completed_at: DateTime<Utc>,
}

impl PhaseSummary {
    /// Create a new phase summary
    pub fn new(
        phase_number: u32,
        title: impl Into<String>,
        summary: impl Into<String>,
        files_changed: Vec<String>,
        notes: Option<String>,
    ) -> Self {
        Self {
            phase_number,
            title: title.into(),
            summary: summary.into(),
            files_changed,
            notes,
            completed_at: Utc::now(),
        }
    }
}

/// Manages plan and review files for tasks
#[derive(Debug, Clone)]
pub struct FileManager {
    /// Base path of the repository
    base_path: PathBuf,
}

impl FileManager {
    /// Create a new FileManager with the given repository base path
    pub fn new(base_path: impl Into<PathBuf>) -> Self {
        Self {
            base_path: base_path.into(),
        }
    }

    /// Get the path to the plans directory
    pub fn plans_dir(&self) -> PathBuf {
        self.base_path
            .join(STUDIO_DIR)
            .join(KANBAN_DIR)
            .join(PLANS_DIR)
    }

    /// Get the path to the reviews directory
    pub fn reviews_dir(&self) -> PathBuf {
        self.base_path
            .join(STUDIO_DIR)
            .join(KANBAN_DIR)
            .join(REVIEWS_DIR)
    }

    /// Get the path to the findings directory
    pub fn findings_dir(&self) -> PathBuf {
        self.base_path
            .join(STUDIO_DIR)
            .join(KANBAN_DIR)
            .join(FINDINGS_DIR)
    }

    /// Get the path to a plan file for a task
    pub fn plan_path(&self, task_id: Uuid) -> PathBuf {
        self.plans_dir().join(format!("{}.md", task_id))
    }

    /// Get the path to a review file for a task
    pub fn review_path(&self, task_id: Uuid) -> PathBuf {
        self.reviews_dir().join(format!("{}.md", task_id))
    }

    /// Get the path to a findings file for a task
    pub fn findings_path(&self, task_id: Uuid) -> PathBuf {
        self.findings_dir().join(format!("{}.json", task_id))
    }

    /// Ensure all required directories exist
    pub async fn ensure_directories(&self) -> Result<()> {
        let plans_dir = self.plans_dir();
        let reviews_dir = self.reviews_dir();
        let findings_dir = self.findings_dir();

        debug!(
            "Ensuring directories exist: {:?}, {:?}, {:?}",
            plans_dir, reviews_dir, findings_dir
        );

        fs::create_dir_all(&plans_dir).await.map_err(|e| {
            OrchestratorError::ExecutionFailed(format!(
                "Failed to create plans directory {:?}: {}",
                plans_dir, e
            ))
        })?;

        fs::create_dir_all(&reviews_dir).await.map_err(|e| {
            OrchestratorError::ExecutionFailed(format!(
                "Failed to create reviews directory {:?}: {}",
                reviews_dir, e
            ))
        })?;

        fs::create_dir_all(&findings_dir).await.map_err(|e| {
            OrchestratorError::ExecutionFailed(format!(
                "Failed to create findings directory {:?}: {}",
                findings_dir, e
            ))
        })?;

        Ok(())
    }

    /// Write a plan file for a task (atomic write via temp file + rename)
    pub async fn write_plan(&self, task_id: Uuid, content: &str) -> Result<PathBuf> {
        self.ensure_directories().await?;
        let path = self.plan_path(task_id);
        let temp_path = self.plans_dir().join(format!(".{}.tmp", task_id));

        info!("Writing plan to {:?}", path);
        
        fs::write(&temp_path, content).await.map_err(|e| {
            OrchestratorError::ExecutionFailed(format!(
                "Failed to write temp plan file {:?}: {}",
                temp_path, e
            ))
        })?;

        fs::rename(&temp_path, &path).await.map_err(|e| {
            OrchestratorError::ExecutionFailed(format!(
                "Failed to rename plan file {:?} -> {:?}: {}",
                temp_path, path, e
            ))
        })?;

        Ok(path)
    }

    /// Read a plan file for a task
    pub async fn read_plan(&self, task_id: Uuid) -> Result<String> {
        let path = self.plan_path(task_id);

        debug!("Reading plan from {:?}", path);
        fs::read_to_string(&path).await.map_err(|e| {
            OrchestratorError::ExecutionFailed(format!(
                "Failed to read plan file {:?}: {}",
                path, e
            ))
        })
    }

    /// Check if a plan exists for a task
    pub async fn plan_exists(&self, task_id: Uuid) -> bool {
        fs::try_exists(self.plan_path(task_id)).await.unwrap_or(false)
    }

    /// Write a review file for a task (atomic write via temp file + rename)
    pub async fn write_review(&self, task_id: Uuid, content: &str) -> Result<PathBuf> {
        self.ensure_directories().await?;
        let path = self.review_path(task_id);
        let temp_path = self.reviews_dir().join(format!(".{}.tmp", task_id));

        info!("Writing review to {:?}", path);

        fs::write(&temp_path, content).await.map_err(|e| {
            OrchestratorError::ExecutionFailed(format!(
                "Failed to write temp review file {:?}: {}",
                temp_path, e
            ))
        })?;

        fs::rename(&temp_path, &path).await.map_err(|e| {
            OrchestratorError::ExecutionFailed(format!(
                "Failed to rename review file {:?} -> {:?}: {}",
                temp_path, path, e
            ))
        })?;

        Ok(path)
    }

    /// Read a review file for a task
    pub async fn read_review(&self, task_id: Uuid) -> Result<String> {
        let path = self.review_path(task_id);

        debug!("Reading review from {:?}", path);
        fs::read_to_string(&path).await.map_err(|e| {
            OrchestratorError::ExecutionFailed(format!(
                "Failed to read review file {:?}: {}",
                path, e
            ))
        })
    }

    /// Check if a review exists for a task
    pub async fn review_exists(&self, task_id: Uuid) -> bool {
        fs::try_exists(self.review_path(task_id)).await.unwrap_or(false)
    }

    /// Delete a plan file for a task
    pub async fn delete_plan(&self, task_id: Uuid) -> Result<()> {
        let path = self.plan_path(task_id);
        if fs::try_exists(&path).await.unwrap_or(false) {
            fs::remove_file(&path).await.map_err(|e| {
                OrchestratorError::ExecutionFailed(format!(
                    "Failed to delete plan file {:?}: {}",
                    path, e
                ))
            })?;
        }
        Ok(())
    }

    /// Delete a review file for a task
    pub async fn delete_review(&self, task_id: Uuid) -> Result<()> {
        let path = self.review_path(task_id);
        if fs::try_exists(&path).await.unwrap_or(false) {
            fs::remove_file(&path).await.map_err(|e| {
                OrchestratorError::ExecutionFailed(format!(
                    "Failed to delete review file {:?}: {}",
                    path, e
                ))
            })?;
        }
        Ok(())
    }

    // ========================================================================
    // Findings Methods
    // ========================================================================

    /// Write findings to a JSON file for a task (atomic write)
    pub async fn write_findings(&self, task_id: Uuid, findings: &ReviewFindings) -> Result<PathBuf> {
        self.ensure_directories().await?;
        let path = self.findings_path(task_id);
        let temp_path = self.findings_dir().join(format!(".{}.tmp", task_id));

        info!("Writing findings to {:?}", path);

        let json = serde_json::to_string_pretty(findings).map_err(|e| {
            OrchestratorError::ExecutionFailed(format!("Failed to serialize findings: {}", e))
        })?;

        fs::write(&temp_path, &json).await.map_err(|e| {
            OrchestratorError::ExecutionFailed(format!(
                "Failed to write temp findings file {:?}: {}",
                temp_path, e
            ))
        })?;

        fs::rename(&temp_path, &path).await.map_err(|e| {
            OrchestratorError::ExecutionFailed(format!(
                "Failed to rename findings file {:?} -> {:?}: {}",
                temp_path, path, e
            ))
        })?;

        Ok(path)
    }

    /// Read findings from a JSON file for a task
    pub async fn read_findings(&self, task_id: Uuid) -> Result<Option<ReviewFindings>> {
        let path = self.findings_path(task_id);

        if !fs::try_exists(&path).await.unwrap_or(false) {
            return Ok(None);
        }

        debug!("Reading findings from {:?}", path);
        let content = fs::read_to_string(&path).await.map_err(|e| {
            OrchestratorError::ExecutionFailed(format!(
                "Failed to read findings file {:?}: {}",
                path, e
            ))
        })?;

        let findings: ReviewFindings = serde_json::from_str(&content).map_err(|e| {
            OrchestratorError::ExecutionFailed(format!(
                "Failed to parse findings file {:?}: {}",
                path, e
            ))
        })?;

        Ok(Some(findings))
    }

    /// Check if findings exist for a task
    pub async fn findings_exists(&self, task_id: Uuid) -> bool {
        fs::try_exists(self.findings_path(task_id))
            .await
            .unwrap_or(false)
    }

    /// Delete findings file for a task
    pub async fn delete_findings(&self, task_id: Uuid) -> Result<()> {
        let path = self.findings_path(task_id);
        if fs::try_exists(&path).await.unwrap_or(false) {
            fs::remove_file(&path).await.map_err(|e| {
                OrchestratorError::ExecutionFailed(format!(
                    "Failed to delete findings file {:?}: {}",
                    path, e
                ))
            })?;
        }
        Ok(())
    }

    /// Update status of specific findings in the file
    pub async fn update_findings_status(
        &self,
        task_id: Uuid,
        finding_ids: &[String],
        status: FindingStatus,
    ) -> Result<()> {
        let mut findings = self
            .read_findings(task_id)
            .await?
            .ok_or_else(|| OrchestratorError::ExecutionFailed("Findings file not found".into()))?;

        for finding in &mut findings.findings {
            if finding_ids.contains(&finding.id) {
                finding.status = status;
            }
        }

        self.write_findings(task_id, &findings).await?;
        Ok(())
    }

    /// Mark all pending findings as skipped
    pub async fn skip_all_findings(&self, task_id: Uuid) -> Result<()> {
        let mut findings = self
            .read_findings(task_id)
            .await?
            .ok_or_else(|| OrchestratorError::ExecutionFailed("Findings file not found".into()))?;

        for finding in &mut findings.findings {
            if finding.status == FindingStatus::Pending {
                finding.status = FindingStatus::Skipped;
            }
        }

        self.write_findings(task_id, &findings).await?;
        Ok(())
    }

    /// Get the relative path for a plan (used in prompts)
    pub fn plan_relative_path(&self, task_id: Uuid) -> String {
        format!(
            "{}/{}/{}/{}.md",
            STUDIO_DIR, KANBAN_DIR, PLANS_DIR, task_id
        )
    }

    /// Get the relative path for a review (used in prompts)
    pub fn review_relative_path(&self, task_id: Uuid) -> String {
        format!(
            "{}/{}/{}/{}.md",
            STUDIO_DIR, KANBAN_DIR, REVIEWS_DIR, task_id
        )
    }

    // ========================================================================
    // Phase Methods (Multi-Phase Implementation)
    // ========================================================================

    /// Get the path to the phases directory for a task
    pub fn phases_dir(&self, task_id: Uuid) -> PathBuf {
        self.base_path
            .join(STUDIO_DIR)
            .join(KANBAN_DIR)
            .join(PHASES_DIR)
            .join(task_id.to_string())
    }

    /// Get the path to the phase context file
    pub fn phase_context_path(&self, task_id: Uuid) -> PathBuf {
        self.phases_dir(task_id).join("context.json")
    }

    /// Get the path to a phase summary file
    pub fn phase_summary_path(&self, task_id: Uuid, phase_number: u32) -> PathBuf {
        self.phases_dir(task_id)
            .join(format!("phase-{}-summary.json", phase_number))
    }

    /// Ensure the phases directory exists for a task
    pub async fn ensure_phases_dir(&self, task_id: Uuid) -> Result<()> {
        let dir = self.phases_dir(task_id);
        fs::create_dir_all(&dir).await.map_err(|e| {
            OrchestratorError::ExecutionFailed(format!(
                "Failed to create phases directory {:?}: {}",
                dir, e
            ))
        })?;
        Ok(())
    }

    /// Write phase context to file (atomic write)
    pub async fn write_phase_context(&self, task_id: Uuid, context: &PhaseContext) -> Result<PathBuf> {
        self.ensure_phases_dir(task_id).await?;
        let path = self.phase_context_path(task_id);
        let temp_path = self.phases_dir(task_id).join(".context.tmp");

        info!(
            task_id = %task_id,
            phase = context.phase_number,
            total = context.total_phases,
            "Writing phase context"
        );

        let json = serde_json::to_string_pretty(context).map_err(|e| {
            OrchestratorError::ExecutionFailed(format!("Failed to serialize phase context: {}", e))
        })?;

        fs::write(&temp_path, &json).await.map_err(|e| {
            OrchestratorError::ExecutionFailed(format!(
                "Failed to write temp phase context {:?}: {}",
                temp_path, e
            ))
        })?;

        fs::rename(&temp_path, &path).await.map_err(|e| {
            OrchestratorError::ExecutionFailed(format!(
                "Failed to rename phase context {:?} -> {:?}: {}",
                temp_path, path, e
            ))
        })?;

        Ok(path)
    }

    /// Read phase context from file
    pub async fn read_phase_context(&self, task_id: Uuid) -> Result<Option<PhaseContext>> {
        let path = self.phase_context_path(task_id);

        if !fs::try_exists(&path).await.unwrap_or(false) {
            return Ok(None);
        }

        debug!("Reading phase context from {:?}", path);
        let content = fs::read_to_string(&path).await.map_err(|e| {
            OrchestratorError::ExecutionFailed(format!(
                "Failed to read phase context {:?}: {}",
                path, e
            ))
        })?;

        let context: PhaseContext = serde_json::from_str(&content).map_err(|e| {
            OrchestratorError::ExecutionFailed(format!(
                "Failed to parse phase context {:?}: {}",
                path, e
            ))
        })?;

        Ok(Some(context))
    }

    /// Write a phase summary to file (atomic write)
    pub async fn write_phase_summary(&self, task_id: Uuid, summary: &PhaseSummary) -> Result<PathBuf> {
        self.ensure_phases_dir(task_id).await?;
        let path = self.phase_summary_path(task_id, summary.phase_number);
        let temp_path = self
            .phases_dir(task_id)
            .join(format!(".phase-{}.tmp", summary.phase_number));

        info!(
            task_id = %task_id,
            phase = summary.phase_number,
            title = %summary.title,
            files_changed = summary.files_changed.len(),
            "Writing phase summary"
        );

        let json = serde_json::to_string_pretty(summary).map_err(|e| {
            OrchestratorError::ExecutionFailed(format!("Failed to serialize phase summary: {}", e))
        })?;

        fs::write(&temp_path, &json).await.map_err(|e| {
            OrchestratorError::ExecutionFailed(format!(
                "Failed to write temp phase summary {:?}: {}",
                temp_path, e
            ))
        })?;

        fs::rename(&temp_path, &path).await.map_err(|e| {
            OrchestratorError::ExecutionFailed(format!(
                "Failed to rename phase summary {:?} -> {:?}: {}",
                temp_path, path, e
            ))
        })?;

        Ok(path)
    }

    /// Read all phase summaries for a task (in order)
    pub async fn read_phase_summaries(&self, task_id: Uuid) -> Result<Vec<PhaseSummary>> {
        let dir = self.phases_dir(task_id);

        if !fs::try_exists(&dir).await.unwrap_or(false) {
            return Ok(Vec::new());
        }

        let mut summaries = Vec::new();
        let mut entries = fs::read_dir(&dir).await.map_err(|e| {
            OrchestratorError::ExecutionFailed(format!(
                "Failed to read phases directory {:?}: {}",
                dir, e
            ))
        })?;

        while let Some(entry) = entries.next_entry().await.map_err(|e| {
            OrchestratorError::ExecutionFailed(format!("Failed to read directory entry: {}", e))
        })? {
            let path = entry.path();
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            // Match phase-N-summary.json pattern
            if file_name.starts_with("phase-") && file_name.ends_with("-summary.json") {
                let content = fs::read_to_string(&path).await.map_err(|e| {
                    OrchestratorError::ExecutionFailed(format!(
                        "Failed to read phase summary {:?}: {}",
                        path, e
                    ))
                })?;

                let summary: PhaseSummary = serde_json::from_str(&content).map_err(|e| {
                    OrchestratorError::ExecutionFailed(format!(
                        "Failed to parse phase summary {:?}: {}",
                        path, e
                    ))
                })?;

                summaries.push(summary);
            }
        }

        // Sort by phase number
        summaries.sort_by_key(|s| s.phase_number);
        Ok(summaries)
    }

    /// Mark a phase as complete in the plan file by adding a checkmark
    pub async fn mark_phase_complete_in_plan(&self, task_id: Uuid, phase_number: u32) -> Result<()> {
        let plan = self.read_plan(task_id).await?;

        // Pattern to match phase headers like "### Phase 1: Title" or "## Phase 1 - Title"
        let patterns = [
            format!(r"(###?\s*Phase\s*{}\s*[:\-])", phase_number),
            format!(r"(###?\s*Fáze\s*{}\s*[:\-])", phase_number),
            format!(r"(###?\s*Step\s*{}\s*[:\-])", phase_number),
            format!(r"(###?\s*Krok\s*{}\s*[:\-])", phase_number),
        ];

        let mut updated_plan = plan.clone();
        let mut marked = false;

        for pattern in &patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                if re.is_match(&updated_plan) {
                    // Add checkmark after the header if not already present
                    updated_plan = re
                        .replace(&updated_plan, |caps: &regex::Captures| {
                            let header = &caps[1];
                            if !updated_plan.contains(&format!("{} ✓", header)) {
                                format!("{} ✓", header)
                            } else {
                                header.to_string()
                            }
                        })
                        .to_string();
                    marked = true;
                    break;
                }
            }
        }

        if marked {
            self.write_plan(task_id, &updated_plan).await?;
            info!(task_id = %task_id, phase = phase_number, "Marked phase complete in plan");
        } else {
            debug!(
                task_id = %task_id,
                phase = phase_number,
                "No matching phase header found in plan to mark complete"
            );
        }

        Ok(())
    }

    /// Delete all phase data for a task
    pub async fn delete_phases(&self, task_id: Uuid) -> Result<()> {
        let dir = self.phases_dir(task_id);
        if fs::try_exists(&dir).await.unwrap_or(false) {
            fs::remove_dir_all(&dir).await.map_err(|e| {
                OrchestratorError::ExecutionFailed(format!(
                    "Failed to delete phases directory {:?}: {}",
                    dir, e
                ))
            })?;
        }
        Ok(())
    }

    /// Check if phase context exists for a task
    pub async fn phase_context_exists(&self, task_id: Uuid) -> bool {
        fs::try_exists(self.phase_context_path(task_id))
            .await
            .unwrap_or(false)
    }
}

impl Default for FileManager {
    fn default() -> Self {
        Self::new(".")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn setup_test_file_manager() -> (FileManager, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let fm = FileManager::new(temp_dir.path());
        (fm, temp_dir)
    }

    #[tokio::test]
    async fn test_ensure_directories() {
        let (fm, _temp_dir) = setup_test_file_manager().await;

        fm.ensure_directories().await.unwrap();

        assert!(fm.plans_dir().exists());
        assert!(fm.reviews_dir().exists());
    }

    #[tokio::test]
    async fn test_write_and_read_plan() {
        let (fm, _temp_dir) = setup_test_file_manager().await;
        let task_id = Uuid::new_v4();
        let content = "# Plan\n\nThis is a test plan.";

        let path = fm.write_plan(task_id, content).await.unwrap();
        assert!(path.exists());

        let read_content = fm.read_plan(task_id).await.unwrap();
        assert_eq!(read_content, content);
    }

    #[tokio::test]
    async fn test_write_and_read_review() {
        let (fm, _temp_dir) = setup_test_file_manager().await;
        let task_id = Uuid::new_v4();
        let content = "# Review\n\nAPPROVED\n\nGreat work!";

        let path = fm.write_review(task_id, content).await.unwrap();
        assert!(path.exists());

        let read_content = fm.read_review(task_id).await.unwrap();
        assert_eq!(read_content, content);
    }

    #[tokio::test]
    async fn test_plan_exists() {
        let (fm, _temp_dir) = setup_test_file_manager().await;
        let task_id = Uuid::new_v4();

        assert!(!fm.plan_exists(task_id).await);

        fm.write_plan(task_id, "test").await.unwrap();

        assert!(fm.plan_exists(task_id).await);
    }

    #[tokio::test]
    async fn test_delete_plan() {
        let (fm, _temp_dir) = setup_test_file_manager().await;
        let task_id = Uuid::new_v4();

        fm.write_plan(task_id, "test").await.unwrap();
        assert!(fm.plan_exists(task_id).await);

        fm.delete_plan(task_id).await.unwrap();
        assert!(!fm.plan_exists(task_id).await);
    }

    #[tokio::test]
    async fn test_relative_paths() {
        let fm = FileManager::new("/repo");
        let task_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

        assert_eq!(
            fm.plan_relative_path(task_id),
            ".opencode-studio/kanban/plans/550e8400-e29b-41d4-a716-446655440000.md"
        );
        assert_eq!(
            fm.review_relative_path(task_id),
            ".opencode-studio/kanban/reviews/550e8400-e29b-41d4-a716-446655440000.md"
        );
    }
}
