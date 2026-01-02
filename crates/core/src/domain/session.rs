use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default, ToSchema, Hash)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[serde(rename_all = "snake_case")]
pub enum SessionPhase {
    #[default]
    Planning,
    Implementation,
    Review,
    /// Fix phase - used to fix issues found during review
    Fix,
}

impl SessionPhase {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planning => "planning",
            Self::Implementation => "implementation",
            Self::Review => "review",
            Self::Fix => "fix",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "planning" => Some(Self::Planning),
            "implementation" => Some(Self::Implementation),
            "review" => Some(Self::Review),
            "fix" => Some(Self::Fix),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    #[default]
    Pending,
    Running,
    Completed,
    Failed,
    Aborted,
}

impl SessionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Aborted => "aborted",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(Self::Pending),
            "running" => Some(Self::Running),
            "completed" => Some(Self::Completed),
            "failed" => Some(Self::Failed),
            "aborted" => Some(Self::Aborted),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct Session {
    pub id: Uuid,
    pub task_id: Uuid,
    pub opencode_session_id: Option<String>,
    pub phase: SessionPhase,
    pub status: SessionStatus,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl Session {
    pub fn new(task_id: Uuid, phase: SessionPhase) -> Self {
        Self {
            id: Uuid::new_v4(),
            task_id,
            opencode_session_id: None,
            phase,
            status: SessionStatus::default(),
            started_at: None,
            completed_at: None,
            created_at: Utc::now(),
        }
    }

    pub fn start(&mut self, opencode_session_id: String) {
        self.opencode_session_id = Some(opencode_session_id);
        self.status = SessionStatus::Running;
        self.started_at = Some(Utc::now());
    }

    pub fn complete(&mut self) {
        self.status = SessionStatus::Completed;
        self.completed_at = Some(Utc::now());
    }

    pub fn fail(&mut self) {
        self.status = SessionStatus::Failed;
        self.completed_at = Some(Utc::now());
    }

    pub fn abort(&mut self) {
        self.status = SessionStatus::Aborted;
        self.completed_at = Some(Utc::now());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let task_id = Uuid::new_v4();
        let session = Session::new(task_id, SessionPhase::Planning);

        assert_eq!(session.task_id, task_id);
        assert_eq!(session.phase, SessionPhase::Planning);
        assert_eq!(session.status, SessionStatus::Pending);
        assert!(session.opencode_session_id.is_none());
    }

    #[test]
    fn test_session_lifecycle() {
        let task_id = Uuid::new_v4();
        let mut session = Session::new(task_id, SessionPhase::Implementation);

        session.start("opencode-123".to_string());
        assert_eq!(session.status, SessionStatus::Running);
        assert_eq!(
            session.opencode_session_id,
            Some("opencode-123".to_string())
        );
        assert!(session.started_at.is_some());

        session.complete();
        assert_eq!(session.status, SessionStatus::Completed);
        assert!(session.completed_at.is_some());
    }

    #[test]
    fn test_session_phase_serialization() {
        assert_eq!(SessionPhase::Planning.as_str(), "planning");
        assert_eq!(SessionPhase::Implementation.as_str(), "implementation");
        assert_eq!(SessionPhase::parse("review"), Some(SessionPhase::Review));
    }
}
