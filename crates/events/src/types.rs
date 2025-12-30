//! Event types for the OpenCode Studio event system

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Envelope wrapping all events with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    /// Unique event ID
    pub id: Uuid,
    /// When the event occurred
    pub timestamp: DateTime<Utc>,
    /// The actual event
    pub event: Event,
}

impl EventEnvelope {
    /// Create a new event envelope with auto-generated ID and timestamp
    pub fn new(event: Event) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event,
        }
    }
}

/// All possible events in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Event {
    // Task events
    /// A new task was created
    #[serde(rename = "task.created")]
    TaskCreated { task_id: Uuid, title: String },

    /// Task was updated (title, description, etc.)
    #[serde(rename = "task.updated")]
    TaskUpdated { task_id: Uuid },

    /// Task status changed
    #[serde(rename = "task.status_changed")]
    TaskStatusChanged {
        task_id: Uuid,
        from_status: String,
        to_status: String,
    },

    // Session events
    /// OpenCode session started
    #[serde(rename = "session.started")]
    SessionStarted { session_id: Uuid, task_id: Uuid },

    /// OpenCode session ended
    #[serde(rename = "session.ended")]
    SessionEnded {
        session_id: Uuid,
        task_id: Uuid,
        success: bool,
    },

    /// Message from OpenCode agent
    #[serde(rename = "agent.message")]
    AgentMessage {
        session_id: Uuid,
        task_id: Uuid,
        message: AgentMessageData,
    },

    /// Tool execution by agent
    #[serde(rename = "tool.execution")]
    ToolExecution {
        session_id: Uuid,
        task_id: Uuid,
        tool: ToolExecutionData,
    },

    // Workspace events
    /// Workspace created for a task
    #[serde(rename = "workspace.created")]
    WorkspaceCreated { task_id: Uuid, path: String },

    /// Workspace was merged
    #[serde(rename = "workspace.merged")]
    WorkspaceMerged { task_id: Uuid, success: bool },

    /// Workspace was deleted
    #[serde(rename = "workspace.deleted")]
    WorkspaceDeleted { task_id: Uuid },

    // System events
    /// Generic error event
    #[serde(rename = "error")]
    Error {
        message: String,
        context: Option<String>,
    },
}

/// Data for agent message events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessageData {
    /// The message content
    pub content: String,
    /// Message role (assistant, user, system)
    pub role: String,
    /// Whether this is a partial/streaming message
    pub is_partial: bool,
}

/// Data for tool execution events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionData {
    /// Tool name
    pub name: String,
    /// Tool input (JSON string or summary)
    pub input: Option<String>,
    /// Tool output (truncated if large)
    pub output: Option<String>,
    /// Whether the tool succeeded
    pub success: bool,
}

impl Event {
    /// Get the task ID associated with this event, if any
    pub fn task_id(&self) -> Option<Uuid> {
        match self {
            Event::TaskCreated { task_id, .. } => Some(*task_id),
            Event::TaskUpdated { task_id } => Some(*task_id),
            Event::TaskStatusChanged { task_id, .. } => Some(*task_id),
            Event::SessionStarted { task_id, .. } => Some(*task_id),
            Event::SessionEnded { task_id, .. } => Some(*task_id),
            Event::AgentMessage { task_id, .. } => Some(*task_id),
            Event::ToolExecution { task_id, .. } => Some(*task_id),
            Event::WorkspaceCreated { task_id, .. } => Some(*task_id),
            Event::WorkspaceMerged { task_id, .. } => Some(*task_id),
            Event::WorkspaceDeleted { task_id } => Some(*task_id),
            Event::Error { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_envelope_creation() {
        let event = Event::TaskCreated {
            task_id: Uuid::new_v4(),
            title: "Test task".to_string(),
        };
        let envelope = EventEnvelope::new(event);

        assert!(!envelope.id.is_nil());
        assert!(envelope.timestamp <= Utc::now());
    }

    #[test]
    fn test_event_serialization() {
        let event = Event::TaskStatusChanged {
            task_id: Uuid::new_v4(),
            from_status: "Todo".to_string(),
            to_status: "Planning".to_string(),
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("task.status_changed"));
        assert!(json.contains("from_status"));
        assert!(json.contains("to_status"));
    }

    #[test]
    fn test_event_deserialization() {
        let json = r#"{"type":"task.created","task_id":"550e8400-e29b-41d4-a716-446655440000","title":"Test"}"#;
        let event: Event = serde_json::from_str(json).unwrap();

        match event {
            Event::TaskCreated { task_id, title } => {
                assert_eq!(title, "Test");
                assert!(!task_id.is_nil());
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn test_event_task_id() {
        let task_id = Uuid::new_v4();

        let event = Event::TaskCreated {
            task_id,
            title: "Test".to_string(),
        };
        assert_eq!(event.task_id(), Some(task_id));

        let error_event = Event::Error {
            message: "test".to_string(),
            context: None,
        };
        assert_eq!(error_event.task_id(), None);
    }

    #[test]
    fn test_agent_message_data() {
        let data = AgentMessageData {
            content: "Hello".to_string(),
            role: "assistant".to_string(),
            is_partial: false,
        };

        let json = serde_json::to_string(&data).unwrap();
        assert!(json.contains("Hello"));
        assert!(json.contains("assistant"));
    }

    #[test]
    fn test_tool_execution_data() {
        let data = ToolExecutionData {
            name: "read_file".to_string(),
            input: Some("/path/to/file".to_string()),
            output: Some("file contents".to_string()),
            success: true,
        };

        let json = serde_json::to_string(&data).unwrap();
        assert!(json.contains("read_file"));
        assert!(json.contains("success"));
    }
}
