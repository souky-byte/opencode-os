use chrono::{DateTime, TimeZone, Utc};
use opencode_core::{Session, SessionPhase, SessionStatus};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SessionRow {
    pub id: String,
    pub task_id: String,
    pub opencode_session_id: Option<String>,
    pub phase: String,
    pub status: String,
    pub started_at: Option<i64>,
    pub completed_at: Option<i64>,
    pub created_at: i64,
}

impl SessionRow {
    pub fn into_domain(self) -> Session {
        Session {
            id: Uuid::parse_str(&self.id).unwrap_or_default(),
            task_id: Uuid::parse_str(&self.task_id).unwrap_or_default(),
            opencode_session_id: self.opencode_session_id,
            phase: SessionPhase::parse(&self.phase).unwrap_or_default(),
            status: SessionStatus::parse(&self.status).unwrap_or_default(),
            started_at: self.started_at.map(timestamp_to_datetime),
            completed_at: self.completed_at.map(timestamp_to_datetime),
            created_at: timestamp_to_datetime(self.created_at),
        }
    }
}

impl From<&Session> for SessionRow {
    fn from(session: &Session) -> Self {
        Self {
            id: session.id.to_string(),
            task_id: session.task_id.to_string(),
            opencode_session_id: session.opencode_session_id.clone(),
            phase: session.phase.as_str().to_string(),
            status: session.status.as_str().to_string(),
            started_at: session.started_at.map(datetime_to_timestamp),
            completed_at: session.completed_at.map(datetime_to_timestamp),
            created_at: datetime_to_timestamp(session.created_at),
        }
    }
}

fn timestamp_to_datetime(ts: i64) -> DateTime<Utc> {
    Utc.timestamp_opt(ts, 0).unwrap()
}

fn datetime_to_timestamp(dt: DateTime<Utc>) -> i64 {
    dt.timestamp()
}
