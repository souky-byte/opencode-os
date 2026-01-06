use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// ============================================
// Enums
// ============================================

/// Priority level for roadmap features (MoSCoW method)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, Default, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[serde(rename_all = "snake_case")]
pub enum RoadmapPriority {
    /// Critical for MVP - users cannot function without this
    Must,
    /// Important but not critical - significant value
    #[default]
    Should,
    /// Nice to have - enhances experience
    Could,
    /// Not planned for foreseeable future
    Wont,
}

impl RoadmapPriority {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Must => "must",
            Self::Should => "should",
            Self::Could => "could",
            Self::Wont => "wont",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "must" => Some(Self::Must),
            "should" => Some(Self::Should),
            "could" => Some(Self::Could),
            "wont" | "won't" => Some(Self::Wont),
            _ => None,
        }
    }
}

/// Complexity level for roadmap features
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, Default, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[serde(rename_all = "snake_case")]
pub enum RoadmapComplexity {
    /// 1-2 files, single component, < 1 day
    Low,
    /// 3-10 files, multiple components, 1-3 days
    #[default]
    Medium,
    /// 10+ files, architectural changes, > 3 days
    High,
}

impl RoadmapComplexity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "low" => Some(Self::Low),
            "medium" => Some(Self::Medium),
            "high" => Some(Self::High),
            _ => None,
        }
    }
}

/// Impact level for roadmap features
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, Default, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[serde(rename_all = "snake_case")]
pub enum RoadmapImpact {
    /// Edge cases, polish, nice-to-have
    Low,
    /// Improves experience, addresses secondary needs
    #[default]
    Medium,
    /// Core user need, differentiator, revenue driver
    High,
}

impl RoadmapImpact {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "low" => Some(Self::Low),
            "medium" => Some(Self::Medium),
            "high" => Some(Self::High),
            _ => None,
        }
    }
}

/// Status of a roadmap feature
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, Default, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[serde(rename_all = "snake_case")]
pub enum RoadmapFeatureStatus {
    /// Feature is being reviewed/considered
    #[default]
    UnderReview,
    /// Feature is planned for implementation
    Planned,
    /// Feature is currently being implemented (linked to task)
    InProgress,
    /// Feature implementation is complete
    Done,
}

impl RoadmapFeatureStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::UnderReview => "under_review",
            Self::Planned => "planned",
            Self::InProgress => "in_progress",
            Self::Done => "done",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "under_review" | "idea" => Some(Self::UnderReview),
            "planned" => Some(Self::Planned),
            "in_progress" => Some(Self::InProgress),
            "done" => Some(Self::Done),
            _ => None,
        }
    }
}

/// Status of a roadmap phase
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[serde(rename_all = "snake_case")]
pub enum RoadmapPhaseStatus {
    /// Phase is planned but not started
    #[default]
    Planned,
    /// Phase is currently in progress
    InProgress,
    /// Phase is completed
    Completed,
}

impl RoadmapPhaseStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "planned" => Some(Self::Planned),
            "in_progress" => Some(Self::InProgress),
            "completed" => Some(Self::Completed),
            _ => None,
        }
    }
}

/// Overall status of the roadmap
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[serde(rename_all = "snake_case")]
pub enum RoadmapStatus {
    /// Roadmap is in draft/initial state
    #[default]
    Draft,
    /// Roadmap is actively being worked on
    Active,
    /// Roadmap is archived
    Archived,
}

impl RoadmapStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Active => "active",
            Self::Archived => "archived",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "draft" => Some(Self::Draft),
            "active" => Some(Self::Active),
            "archived" => Some(Self::Archived),
            _ => None,
        }
    }
}

/// Status of roadmap generation process
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[serde(rename_all = "snake_case")]
pub enum RoadmapGenerationPhase {
    /// No generation in progress
    #[default]
    Idle,
    /// Analyzing project structure
    Analyzing,
    /// Discovering target audience and product vision
    Discovering,
    /// Generating features and phases
    Generating,
    /// Generation completed successfully
    Complete,
    /// Generation failed with error
    Error,
}

impl RoadmapGenerationPhase {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Analyzing => "analyzing",
            Self::Discovering => "discovering",
            Self::Generating => "generating",
            Self::Complete => "complete",
            Self::Error => "error",
        }
    }

    pub fn is_active(&self) -> bool {
        matches!(self, Self::Analyzing | Self::Discovering | Self::Generating)
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "idle" => Some(Self::Idle),
            "analyzing" => Some(Self::Analyzing),
            "discovering" => Some(Self::Discovering),
            "generating" => Some(Self::Generating),
            "complete" => Some(Self::Complete),
            "error" => Some(Self::Error),
            _ => None,
        }
    }
}

// ============================================
// Structs
// ============================================

/// Target audience for the product
#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct TargetAudience {
    /// Primary user persona
    pub primary: String,
    /// Secondary user personas
    #[serde(default)]
    pub secondary: Vec<String>,
    /// Pain points the target audience faces
    #[serde(default)]
    pub pain_points: Vec<String>,
    /// Goals the target audience wants to achieve
    #[serde(default)]
    pub goals: Vec<String>,
    /// Usage context (when/where/how they use the product)
    #[serde(default)]
    pub usage_context: Option<String>,
}

/// A milestone within a roadmap phase
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct RoadmapMilestone {
    /// Unique identifier
    pub id: String,
    /// Milestone title
    pub title: String,
    /// Milestone description
    pub description: String,
    /// Feature IDs included in this milestone
    #[serde(default)]
    pub features: Vec<String>,
    /// Milestone status
    #[serde(default)]
    pub status: RoadmapPhaseStatus,
}

/// A phase in the roadmap (e.g., Foundation, Enhancement, Scale)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct RoadmapPhase {
    /// Unique identifier
    pub id: String,
    /// Phase name
    pub name: String,
    /// Phase description
    pub description: String,
    /// Order in the roadmap (1-based)
    pub order: u32,
    /// Phase status
    #[serde(default)]
    pub status: RoadmapPhaseStatus,
    /// Feature IDs in this phase
    #[serde(default)]
    pub features: Vec<String>,
    /// Milestones within this phase
    #[serde(default)]
    pub milestones: Vec<RoadmapMilestone>,
}

/// A feature in the roadmap
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct RoadmapFeature {
    /// Unique identifier
    pub id: String,
    /// Feature title
    pub title: String,
    /// Feature description
    pub description: String,
    /// Rationale - why this feature matters
    #[serde(default)]
    pub rationale: String,
    /// Priority (MoSCoW)
    #[serde(default)]
    pub priority: RoadmapPriority,
    /// Complexity estimate
    #[serde(default)]
    pub complexity: RoadmapComplexity,
    /// Impact estimate
    #[serde(default)]
    pub impact: RoadmapImpact,
    /// Phase this feature belongs to
    pub phase_id: String,
    /// Feature IDs this depends on
    #[serde(default)]
    pub dependencies: Vec<String>,
    /// Current status
    #[serde(default)]
    pub status: RoadmapFeatureStatus,
    /// Acceptance criteria
    #[serde(default)]
    pub acceptance_criteria: Vec<String>,
    /// User stories
    #[serde(default)]
    pub user_stories: Vec<String>,
    /// Linked task ID (when converted to task)
    #[serde(default)]
    pub linked_task_id: Option<String>,
}

/// The complete roadmap
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct Roadmap {
    /// Unique identifier
    pub id: String,
    /// Project name
    pub project_name: String,
    /// Roadmap version
    #[serde(default = "default_version")]
    pub version: String,
    /// Product vision statement
    pub vision: String,
    /// Target audience information
    #[serde(default)]
    pub target_audience: TargetAudience,
    /// Roadmap phases
    #[serde(default)]
    pub phases: Vec<RoadmapPhase>,
    /// All features in the roadmap
    #[serde(default)]
    pub features: Vec<RoadmapFeature>,
    /// Overall roadmap status
    #[serde(default)]
    pub status: RoadmapStatus,
    /// Creation timestamp
    #[serde(default = "default_now")]
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    #[serde(default = "default_now")]
    pub updated_at: DateTime<Utc>,
}

fn default_version() -> String {
    "1.0".to_string()
}

fn default_now() -> DateTime<Utc> {
    Utc::now()
}

impl Roadmap {
    /// Create a new empty roadmap
    pub fn new(project_name: impl Into<String>, vision: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: format!("roadmap-{}", now.timestamp_millis()),
            project_name: project_name.into(),
            version: default_version(),
            vision: vision.into(),
            target_audience: TargetAudience::default(),
            phases: Vec::new(),
            features: Vec::new(),
            status: RoadmapStatus::Draft,
            created_at: now,
            updated_at: now,
        }
    }

    /// Get features by status
    pub fn features_by_status(&self, status: RoadmapFeatureStatus) -> Vec<&RoadmapFeature> {
        self.features
            .iter()
            .filter(|f| f.status == status)
            .collect()
    }

    /// Get features by phase
    pub fn features_by_phase(&self, phase_id: &str) -> Vec<&RoadmapFeature> {
        self.features
            .iter()
            .filter(|f| f.phase_id == phase_id)
            .collect()
    }

    /// Get feature by ID
    pub fn feature_by_id(&self, id: &str) -> Option<&RoadmapFeature> {
        self.features.iter().find(|f| f.id == id)
    }

    /// Get mutable feature by ID
    pub fn feature_by_id_mut(&mut self, id: &str) -> Option<&mut RoadmapFeature> {
        self.features.iter_mut().find(|f| f.id == id)
    }

    /// Get statistics about the roadmap
    pub fn stats(&self) -> RoadmapStats {
        let mut by_priority = std::collections::HashMap::new();
        let mut by_status = std::collections::HashMap::new();
        let mut by_complexity = std::collections::HashMap::new();

        for feature in &self.features {
            *by_priority.entry(feature.priority).or_insert(0) += 1;
            *by_status.entry(feature.status).or_insert(0) += 1;
            *by_complexity.entry(feature.complexity).or_insert(0) += 1;
        }

        RoadmapStats {
            total_features: self.features.len(),
            total_phases: self.phases.len(),
            by_priority,
            by_status,
            by_complexity,
        }
    }
}

/// Statistics about a roadmap
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct RoadmapStats {
    pub total_features: usize,
    pub total_phases: usize,
    #[serde(default)]
    pub by_priority: std::collections::HashMap<RoadmapPriority, usize>,
    #[serde(default)]
    pub by_status: std::collections::HashMap<RoadmapFeatureStatus, usize>,
    #[serde(default)]
    pub by_complexity: std::collections::HashMap<RoadmapComplexity, usize>,
}

/// Status of roadmap generation
#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct RoadmapGenerationStatus {
    /// Current phase
    #[serde(default)]
    pub phase: RoadmapGenerationPhase,
    /// Progress percentage (0-100)
    #[serde(default)]
    pub progress: u8,
    /// Current message/status
    #[serde(default)]
    pub message: String,
    /// Error message if phase is Error
    #[serde(default)]
    pub error: Option<String>,
}

impl RoadmapGenerationStatus {
    pub fn idle() -> Self {
        Self {
            phase: RoadmapGenerationPhase::Idle,
            progress: 0,
            message: String::new(),
            error: None,
        }
    }

    pub fn analyzing() -> Self {
        Self {
            phase: RoadmapGenerationPhase::Analyzing,
            progress: 10,
            message: "Analyzing project structure...".to_string(),
            error: None,
        }
    }

    pub fn discovering() -> Self {
        Self {
            phase: RoadmapGenerationPhase::Discovering,
            progress: 40,
            message: "Discovering target audience and product vision...".to_string(),
            error: None,
        }
    }

    pub fn generating() -> Self {
        Self {
            phase: RoadmapGenerationPhase::Generating,
            progress: 70,
            message: "Generating feature roadmap...".to_string(),
            error: None,
        }
    }

    pub fn complete() -> Self {
        Self {
            phase: RoadmapGenerationPhase::Complete,
            progress: 100,
            message: "Roadmap generation complete!".to_string(),
            error: None,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            phase: RoadmapGenerationPhase::Error,
            progress: 0,
            message: "Generation failed".to_string(),
            error: Some(message.into()),
        }
    }
}

// ============================================
// Request/Response types for API
// ============================================

/// Request to generate a roadmap
#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct GenerateRoadmapRequest {
    /// Force regeneration even if roadmap exists
    #[serde(default)]
    pub force: bool,
}

/// Request to update a feature
#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct UpdateFeatureRequest {
    pub status: Option<RoadmapFeatureStatus>,
    pub phase_id: Option<String>,
    pub priority: Option<RoadmapPriority>,
    pub linked_task_id: Option<String>,
}

/// Response for convert feature to task
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct ConvertToTaskResponse {
    pub task_id: String,
    pub feature_id: String,
}

// ============================================
// Tests
// ============================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_parsing() {
        assert_eq!(RoadmapPriority::parse("must"), Some(RoadmapPriority::Must));
        assert_eq!(
            RoadmapPriority::parse("SHOULD"),
            Some(RoadmapPriority::Should)
        );
        assert_eq!(RoadmapPriority::parse("wont"), Some(RoadmapPriority::Wont));
        assert_eq!(RoadmapPriority::parse("won't"), Some(RoadmapPriority::Wont));
        assert_eq!(RoadmapPriority::parse("invalid"), None);
    }

    #[test]
    fn test_feature_status_parsing() {
        assert_eq!(
            RoadmapFeatureStatus::parse("under_review"),
            Some(RoadmapFeatureStatus::UnderReview)
        );
        assert_eq!(
            RoadmapFeatureStatus::parse("idea"),
            Some(RoadmapFeatureStatus::UnderReview)
        );
        assert_eq!(
            RoadmapFeatureStatus::parse("in_progress"),
            Some(RoadmapFeatureStatus::InProgress)
        );
    }

    #[test]
    fn test_roadmap_creation() {
        let roadmap = Roadmap::new("Test Project", "Build something amazing");

        assert_eq!(roadmap.project_name, "Test Project");
        assert_eq!(roadmap.vision, "Build something amazing");
        assert_eq!(roadmap.status, RoadmapStatus::Draft);
        assert!(roadmap.features.is_empty());
        assert!(roadmap.phases.is_empty());
    }

    #[test]
    fn test_roadmap_stats() {
        let mut roadmap = Roadmap::new("Test", "Vision");
        roadmap.features.push(RoadmapFeature {
            id: "f1".to_string(),
            title: "Feature 1".to_string(),
            description: "Desc".to_string(),
            rationale: String::new(),
            priority: RoadmapPriority::Must,
            complexity: RoadmapComplexity::Low,
            impact: RoadmapImpact::High,
            phase_id: "p1".to_string(),
            dependencies: Vec::new(),
            status: RoadmapFeatureStatus::Planned,
            acceptance_criteria: Vec::new(),
            user_stories: Vec::new(),
            linked_task_id: None,
        });
        roadmap.features.push(RoadmapFeature {
            id: "f2".to_string(),
            title: "Feature 2".to_string(),
            description: "Desc".to_string(),
            rationale: String::new(),
            priority: RoadmapPriority::Should,
            complexity: RoadmapComplexity::Medium,
            impact: RoadmapImpact::Medium,
            phase_id: "p1".to_string(),
            dependencies: Vec::new(),
            status: RoadmapFeatureStatus::UnderReview,
            acceptance_criteria: Vec::new(),
            user_stories: Vec::new(),
            linked_task_id: None,
        });

        let stats = roadmap.stats();
        assert_eq!(stats.total_features, 2);
        assert_eq!(stats.by_priority.get(&RoadmapPriority::Must), Some(&1));
        assert_eq!(stats.by_priority.get(&RoadmapPriority::Should), Some(&1));
    }

    #[test]
    fn test_generation_status() {
        let status = RoadmapGenerationStatus::analyzing();
        assert_eq!(status.phase, RoadmapGenerationPhase::Analyzing);
        assert_eq!(status.progress, 10);

        let error = RoadmapGenerationStatus::error("Something went wrong");
        assert_eq!(error.phase, RoadmapGenerationPhase::Error);
        assert_eq!(error.error, Some("Something went wrong".to_string()));
    }
}
