//! OpenCode SSE event subscriber for real-time session monitoring.
//!
//! Subscribes to OpenCode's `/event` SSE endpoint to receive real-time updates
//! about session status, message parts, and other events.

use eventsource_stream::{Event as SseEvent, Eventsource};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

/// OpenCode event types we care about
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OpenCodeEvent {
    #[serde(rename = "session.status")]
    SessionStatus { properties: SessionStatusProps },
    #[serde(rename = "session.idle")]
    SessionIdle { properties: SessionIdleProps },
    #[serde(rename = "session.updated")]
    SessionUpdated { properties: SessionUpdatedProps },
    #[serde(rename = "message.part.updated")]
    MessagePartUpdated { properties: MessagePartUpdatedProps },
    #[serde(rename = "message.updated")]
    MessageUpdated { properties: MessageUpdatedProps },
    // Direct activity events (streamed during execution)
    #[serde(rename = "step_start")]
    StepStart {
        id: String,
        step_name: Option<String>,
        timestamp: String,
    },
    #[serde(rename = "reasoning")]
    Reasoning {
        id: String,
        content: String,
        timestamp: String,
    },
    #[serde(rename = "agent_message")]
    AgentMessage {
        id: String,
        content: String,
        is_partial: bool,
        timestamp: String,
    },
    #[serde(rename = "finished")]
    Finished {
        success: bool,
        error: Option<String>,
        timestamp: String,
    },
    // Tool events
    #[serde(rename = "tool_call")]
    ToolCall {
        id: String,
        tool_name: String,
        #[serde(default)]
        args: Option<serde_json::Value>,
        timestamp: String,
    },
    #[serde(rename = "tool_result")]
    ToolResult {
        id: String,
        tool_name: String,
        #[serde(default)]
        args: Option<serde_json::Value>,
        result: String,
        success: bool,
        timestamp: String,
    },
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStatusProps {
    #[serde(rename = "sessionID")]
    pub session_id: String,
    pub status: SessionStatusValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStatusValue {
    #[serde(rename = "type")]
    pub status_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionIdleProps {
    #[serde(rename = "sessionID")]
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionUpdatedProps {
    #[serde(rename = "sessionID")]
    pub session_id: String,
    #[serde(default)]
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessagePartUpdatedProps {
    pub part: serde_json::Value,
    #[serde(default)]
    pub delta: Option<String>,
}

impl MessagePartUpdatedProps {
    /// Extract session_id from the part object (OpenCode puts it inside part.sessionID)
    pub fn session_id(&self) -> Option<String> {
        self.part
            .get("sessionID")
            .and_then(|v| v.as_str())
            .map(String::from)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageUpdatedProps {
    pub message: serde_json::Value,
}

/// Parsed session status from OpenCode
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionStatus {
    Running,
    Idle,
    Completed,
    Error,
    Unknown(String),
}

impl From<&str> for SessionStatus {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "running" | "busy" => SessionStatus::Running,
            "idle" => SessionStatus::Idle,
            "completed" => SessionStatus::Completed,
            "error" => SessionStatus::Error,
            other => SessionStatus::Unknown(other.to_string()),
        }
    }
}

/// Configuration for the OpenCode event subscriber
#[derive(Debug, Clone)]
pub struct OpenCodeEventConfig {
    pub base_url: String,
    pub session_id: String,
    pub directory: String,
}

/// Events emitted by the subscriber for the executor to handle
#[derive(Debug, Clone)]
pub enum ExecutorEvent {
    /// Session became idle (AI finished responding)
    SessionIdle { session_id: String },
    /// Session status changed
    StatusChanged {
        session_id: String,
        status: SessionStatus,
    },
    /// Message part updated (tool call, text, etc.)
    MessagePartUpdated {
        session_id: String,
        part: serde_json::Value,
        delta: Option<String>,
    },
    /// Direct activity event (step_start, reasoning, agent_message, finished)
    DirectActivity {
        activity: crate::activity_store::SessionActivityMsg,
    },
    /// Connection error
    Error { message: String },
    /// Stream ended
    Disconnected,
}

/// OpenCode SSE event subscriber
pub struct OpenCodeEventSubscriber {
    config: OpenCodeEventConfig,
    client: reqwest::Client,
}

impl OpenCodeEventSubscriber {
    pub fn new(
        base_url: impl Into<String>,
        session_id: impl Into<String>,
        directory: impl Into<String>,
    ) -> Self {
        Self {
            config: OpenCodeEventConfig {
                base_url: base_url.into(),
                session_id: session_id.into(),
                directory: directory.into(),
            },
            client: reqwest::Client::new(),
        }
    }

    /// Subscribe to OpenCode events and return a channel receiver for executor events
    pub fn subscribe(self) -> mpsc::Receiver<ExecutorEvent> {
        let (tx, rx) = mpsc::channel(100);
        let config = self.config.clone();
        let client = self.client.clone();

        tokio::spawn(async move {
            if let Err(e) = Self::run_subscription(client, config, tx.clone()).await {
                error!(error = %e, "OpenCode event subscription failed");
                let _ = tx
                    .send(ExecutorEvent::Error {
                        message: e.to_string(),
                    })
                    .await;
            }
            let _ = tx.send(ExecutorEvent::Disconnected).await;
        });

        rx
    }

    async fn run_subscription(
        client: reqwest::Client,
        config: OpenCodeEventConfig,
        tx: mpsc::Sender<ExecutorEvent>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url = format!(
            "{}/event?directory={}",
            config.base_url,
            urlencoding::encode(&config.directory)
        );
        info!(url = %url, session_id = %config.session_id, directory = %config.directory, "Subscribing to OpenCode SSE events");

        let response = client
            .get(&url)
            .header("Accept", "text/event-stream")
            .send()
            .await?;

        if !response.status().is_success() {
            error!(status = %response.status(), "Failed to connect to OpenCode SSE");
            return Err(format!("Failed to connect to SSE: {}", response.status()).into());
        }

        info!(session_id = %config.session_id, "SSE connection established, waiting for events");

        let stream = response.bytes_stream();
        let mut event_stream = stream.eventsource();
        let mut event_count = 0u32;

        while let Some(event_result) = event_stream.next().await {
            match event_result {
                Ok(event) => {
                    event_count += 1;
                    debug!(
                        session_id = %config.session_id,
                        event_type = %event.event,
                        data_len = event.data.len(),
                        event_count = event_count,
                        "Received SSE event"
                    );

                    if let Some(executor_event) =
                        Self::process_sse_event(&event, &config.session_id)
                    {
                        let is_idle = matches!(executor_event, ExecutorEvent::SessionIdle { .. });

                        if is_idle {
                            info!(
                                session_id = %config.session_id,
                                event_count = event_count,
                                "Session became idle, stopping SSE subscription"
                            );
                        }

                        if tx.send(executor_event).await.is_err() {
                            debug!("Receiver dropped, stopping subscription");
                            break;
                        }

                        // Stop on idle - session completed
                        if is_idle {
                            break;
                        }
                    }
                }
                Err(e) => {
                    warn!(error = %e, session_id = %config.session_id, "SSE stream error");
                    // Continue on transient errors
                }
            }
        }

        info!(session_id = %config.session_id, event_count = event_count, "SSE subscription ended");
        Ok(())
    }

    /// Parse RFC3339 timestamp, falling back to current time with a warning if parsing fails
    fn parse_timestamp(timestamp: &str) -> chrono::DateTime<chrono::Utc> {
        chrono::DateTime::parse_from_rfc3339(timestamp)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(|e| {
                warn!(error = %e, timestamp = %timestamp, "Failed to parse timestamp, using current time");
                chrono::Utc::now()
            })
    }

    fn process_sse_event(event: &SseEvent, target_session_id: &str) -> Option<ExecutorEvent> {
        // Skip empty events or comments
        if event.data.is_empty() {
            return None;
        }

        let parsed: OpenCodeEvent = match serde_json::from_str(&event.data) {
            Ok(e) => e,
            Err(e) => {
                debug!(error = %e, data = %event.data, "Failed to parse OpenCode event");
                return None;
            }
        };

        // Log what type was parsed
        debug!(parsed_type = ?std::mem::discriminant(&parsed), "Parsed OpenCode event type");

        match parsed {
            OpenCodeEvent::SessionIdle { properties } => {
                if properties.session_id == target_session_id {
                    Some(ExecutorEvent::SessionIdle {
                        session_id: properties.session_id,
                    })
                } else {
                    None
                }
            }
            OpenCodeEvent::SessionStatus { properties } => {
                if properties.session_id == target_session_id {
                    let status_type = &properties.status.status_type;
                    info!(
                        session_id = %properties.session_id,
                        status_type = %status_type,
                        "Received session.status event"
                    );
                    // Treat "idle" status as SessionIdle event
                    if status_type == "idle" {
                        Some(ExecutorEvent::SessionIdle {
                            session_id: properties.session_id,
                        })
                    } else {
                        Some(ExecutorEvent::StatusChanged {
                            session_id: properties.session_id,
                            status: SessionStatus::from(status_type.as_str()),
                        })
                    }
                } else {
                    None
                }
            }
            OpenCodeEvent::MessagePartUpdated { properties } => {
                // Extract session_id from the part object, or use target session as fallback
                let event_session_id = properties
                    .session_id()
                    .unwrap_or_else(|| target_session_id.to_string());

                debug!(
                    event_session_id = %event_session_id,
                    target_session_id = %target_session_id,
                    part_type = ?properties.part.get("type"),
                    "Processing MessagePartUpdated"
                );

                if event_session_id == target_session_id {
                    info!(
                        session_id = %event_session_id,
                        part_type = ?properties.part.get("type"),
                        "Forwarding MessagePartUpdated to executor"
                    );
                    Some(ExecutorEvent::MessagePartUpdated {
                        session_id: event_session_id,
                        part: properties.part,
                        delta: properties.delta,
                    })
                } else {
                    debug!(
                        event_session_id = %event_session_id,
                        target_session_id = %target_session_id,
                        "Ignoring MessagePartUpdated for different session"
                    );
                    None
                }
            }
            OpenCodeEvent::SessionUpdated { properties } => {
                if properties.session_id == target_session_id {
                    debug!(session_id = %properties.session_id, "Session updated");
                }
                None
            }
            OpenCodeEvent::MessageUpdated { .. } => None,
            // Direct activity events - convert to SessionActivityMsg and forward
            OpenCodeEvent::StepStart {
                id,
                step_name,
                timestamp,
            } => {
                info!(id = %id, "Received step_start event");
                let ts = Self::parse_timestamp(&timestamp);
                Some(ExecutorEvent::DirectActivity {
                    activity: crate::activity_store::SessionActivityMsg::StepStart {
                        id,
                        step_name,
                        timestamp: ts,
                    },
                })
            }
            OpenCodeEvent::Reasoning {
                id,
                content,
                timestamp,
            } => {
                debug!(id = %id, content_len = content.len(), "Received reasoning event");
                let ts = Self::parse_timestamp(&timestamp);
                Some(ExecutorEvent::DirectActivity {
                    activity: crate::activity_store::SessionActivityMsg::Reasoning {
                        id,
                        content,
                        timestamp: ts,
                    },
                })
            }
            OpenCodeEvent::AgentMessage {
                id,
                content,
                is_partial,
                timestamp,
            } => {
                info!(id = %id, is_partial = is_partial, "Received agent_message event");
                let ts = Self::parse_timestamp(&timestamp);
                Some(ExecutorEvent::DirectActivity {
                    activity: crate::activity_store::SessionActivityMsg::AgentMessage {
                        id,
                        content,
                        is_partial,
                        timestamp: ts,
                    },
                })
            }
            OpenCodeEvent::Finished {
                success,
                error,
                timestamp,
            } => {
                info!(success = success, "Received finished event");
                let ts = Self::parse_timestamp(&timestamp);
                Some(ExecutorEvent::DirectActivity {
                    activity: crate::activity_store::SessionActivityMsg::Finished {
                        success,
                        error,
                        timestamp: ts,
                    },
                })
            }
            OpenCodeEvent::ToolCall {
                id,
                tool_name,
                args,
                timestamp,
            } => {
                info!(id = %id, tool_name = %tool_name, "Received tool_call event");
                let ts = Self::parse_timestamp(&timestamp);
                Some(ExecutorEvent::DirectActivity {
                    activity: crate::activity_store::SessionActivityMsg::ToolCall {
                        id,
                        tool_name,
                        args,
                        timestamp: ts,
                    },
                })
            }
            OpenCodeEvent::ToolResult {
                id,
                tool_name,
                args,
                result,
                success,
                timestamp,
            } => {
                info!(id = %id, tool_name = %tool_name, success = success, "Received tool_result event");
                let ts = Self::parse_timestamp(&timestamp);
                Some(ExecutorEvent::DirectActivity {
                    activity: crate::activity_store::SessionActivityMsg::ToolResult {
                        id,
                        tool_name,
                        args,
                        result,
                        success,
                        timestamp: ts,
                    },
                })
            }
            OpenCodeEvent::Unknown => None,
        }
    }
}

/// Helper to wait for session completion with timeout
pub async fn wait_for_session_completion(
    base_url: &str,
    session_id: &str,
    directory: &str,
    timeout: std::time::Duration,
) -> Result<(), String> {
    let subscriber = OpenCodeEventSubscriber::new(base_url, session_id, directory);
    let mut rx = subscriber.subscribe();

    let result = tokio::time::timeout(timeout, async {
        while let Some(event) = rx.recv().await {
            match event {
                ExecutorEvent::SessionIdle { .. } => return Ok(()),
                ExecutorEvent::StatusChanged {
                    status: SessionStatus::Completed,
                    ..
                } => return Ok(()),
                ExecutorEvent::StatusChanged {
                    status: SessionStatus::Error,
                    ..
                } => {
                    return Err("Session ended with error".to_string());
                }
                ExecutorEvent::Error { message } => return Err(message),
                ExecutorEvent::Disconnected => return Err("Connection lost".to_string()),
                _ => continue,
            }
        }
        Err("Event stream ended unexpectedly".to_string())
    })
    .await;

    match result {
        Ok(inner) => inner,
        Err(_) => Err("Timeout waiting for session completion".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_session_idle() {
        let json = r#"{"type":"session.idle","properties":{"sessionID":"ses_123"}}"#;
        let event: OpenCodeEvent = serde_json::from_str(json).unwrap();

        match event {
            OpenCodeEvent::SessionIdle { properties } => {
                assert_eq!(properties.session_id, "ses_123");
            }
            _ => panic!("Expected SessionIdle"),
        }
    }

    #[test]
    fn test_parse_session_status() {
        let json = r#"{"type":"session.status","properties":{"sessionID":"ses_456","status":{"type":"running"}}}"#;
        let event: OpenCodeEvent = serde_json::from_str(json).unwrap();

        match event {
            OpenCodeEvent::SessionStatus { properties } => {
                assert_eq!(properties.session_id, "ses_456");
                assert_eq!(properties.status.status_type, "running");
            }
            _ => panic!("Expected SessionStatus"),
        }
    }

    #[test]
    fn test_parse_unknown_event() {
        let json = r#"{"type":"some.unknown.event","properties":{}}"#;
        let event: OpenCodeEvent = serde_json::from_str(json).unwrap();
        assert!(matches!(event, OpenCodeEvent::Unknown));
    }

    #[test]
    fn test_session_status_from_str() {
        assert_eq!(SessionStatus::from("running"), SessionStatus::Running);
        assert_eq!(SessionStatus::from("idle"), SessionStatus::Idle);
        assert_eq!(SessionStatus::from("IDLE"), SessionStatus::Idle);
        assert!(matches!(
            SessionStatus::from("custom"),
            SessionStatus::Unknown(_)
        ));
    }
}
