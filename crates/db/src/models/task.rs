use chrono::{DateTime, TimeZone, Utc};
use opencode_core::{Task, TaskStatus};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TaskRow {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: String,
    pub roadmap_item_id: Option<String>,
    pub workspace_path: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl TaskRow {
    pub fn into_domain(self) -> Task {
        Task {
            id: Uuid::parse_str(&self.id).unwrap_or_default(),
            title: self.title,
            description: self.description,
            status: TaskStatus::parse(&self.status).unwrap_or_default(),
            roadmap_item_id: self.roadmap_item_id.and_then(|s| Uuid::parse_str(&s).ok()),
            workspace_path: self.workspace_path,
            created_at: timestamp_to_datetime(self.created_at),
            updated_at: timestamp_to_datetime(self.updated_at),
        }
    }
}

impl From<&Task> for TaskRow {
    fn from(task: &Task) -> Self {
        Self {
            id: task.id.to_string(),
            title: task.title.clone(),
            description: task.description.clone(),
            status: task.status.as_str().to_string(),
            roadmap_item_id: task.roadmap_item_id.map(|id| id.to_string()),
            workspace_path: task.workspace_path.clone(),
            created_at: datetime_to_timestamp(task.created_at),
            updated_at: datetime_to_timestamp(task.updated_at),
        }
    }
}

fn timestamp_to_datetime(ts: i64) -> DateTime<Utc> {
    Utc.timestamp_opt(ts, 0).unwrap()
}

fn datetime_to_timestamp(dt: DateTime<Utc>) -> i64 {
    dt.timestamp()
}
