use chrono::{DateTime, TimeZone, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SessionActivityRow {
    pub id: i64,
    pub session_id: String,
    pub activity_type: String,
    pub activity_id: Option<String>,
    pub data: String,
    pub created_at: i64,
}

/// Domain model for session activity
#[derive(Debug, Clone)]
pub struct SessionActivity {
    pub id: i64,
    pub session_id: Uuid,
    pub activity_type: String,
    pub activity_id: Option<String>,
    pub data: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

impl SessionActivityRow {
    pub fn into_domain(self) -> SessionActivity {
        SessionActivity {
            id: self.id,
            session_id: Uuid::parse_str(&self.session_id).unwrap_or_default(),
            activity_type: self.activity_type,
            activity_id: self.activity_id,
            data: serde_json::from_str(&self.data).unwrap_or(serde_json::Value::Null),
            created_at: timestamp_to_datetime(self.created_at),
        }
    }
}

/// Input for creating a new activity
#[derive(Debug, Clone)]
pub struct CreateSessionActivity {
    pub session_id: Uuid,
    pub activity_type: String,
    pub activity_id: Option<String>,
    pub data: serde_json::Value,
}

impl CreateSessionActivity {
    pub fn new(
        session_id: Uuid,
        activity_type: impl Into<String>,
        activity_id: Option<String>,
        data: serde_json::Value,
    ) -> Self {
        Self {
            session_id,
            activity_type: activity_type.into(),
            activity_id,
            data,
        }
    }
}

fn timestamp_to_datetime(ts: i64) -> DateTime<Utc> {
    Utc.timestamp_opt(ts, 0).unwrap()
}

pub fn datetime_to_timestamp(dt: DateTime<Utc>) -> i64 {
    dt.timestamp()
}
