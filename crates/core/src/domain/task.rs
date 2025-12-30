use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    #[default]
    Todo,
    Planning,
    PlanningReview,
    InProgress,
    AiReview,
    Review,
    Done,
}

impl TaskStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Todo => "todo",
            Self::Planning => "planning",
            Self::PlanningReview => "planning_review",
            Self::InProgress => "in_progress",
            Self::AiReview => "ai_review",
            Self::Review => "review",
            Self::Done => "done",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "todo" => Some(Self::Todo),
            "planning" => Some(Self::Planning),
            "planning_review" => Some(Self::PlanningReview),
            "in_progress" => Some(Self::InProgress),
            "ai_review" => Some(Self::AiReview),
            "review" => Some(Self::Review),
            "done" => Some(Self::Done),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub status: TaskStatus,
    pub roadmap_item_id: Option<Uuid>,
    pub workspace_path: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Task {
    pub fn new(title: impl Into<String>, description: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            title: title.into(),
            description: description.into(),
            status: TaskStatus::default(),
            roadmap_item_id: None,
            workspace_path: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_id(mut self, id: Uuid) -> Self {
        self.id = id;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskRequest {
    pub title: String,
    pub description: String,
    pub roadmap_item_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateTaskRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<TaskStatus>,
    pub workspace_path: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_creation() {
        let task = Task::new("Test Task", "Test Description");

        assert_eq!(task.title, "Test Task");
        assert_eq!(task.description, "Test Description");
        assert_eq!(task.status, TaskStatus::Todo);
        assert!(task.roadmap_item_id.is_none());
        assert!(task.workspace_path.is_none());
    }

    #[test]
    fn test_task_status_serialization() {
        assert_eq!(TaskStatus::Todo.as_str(), "todo");
        assert_eq!(TaskStatus::InProgress.as_str(), "in_progress");
        assert_eq!(TaskStatus::AiReview.as_str(), "ai_review");
    }

    #[test]
    fn test_task_status_parsing() {
        assert_eq!(TaskStatus::parse("todo"), Some(TaskStatus::Todo));
        assert_eq!(
            TaskStatus::parse("in_progress"),
            Some(TaskStatus::InProgress)
        );
        assert_eq!(TaskStatus::parse("invalid"), None);
    }

    #[test]
    fn test_task_with_id() {
        let id = Uuid::new_v4();
        let task = Task::new("Test", "Description").with_id(id);

        assert_eq!(task.id, id);
    }
}
