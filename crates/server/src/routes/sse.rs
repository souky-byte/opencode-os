use std::collections::VecDeque;
use std::convert::Infallible;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use axum::extract::{Path, Query, State};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::IntoResponse;
use futures::stream::{Stream, StreamExt};
use serde::Deserialize;
use tokio_stream::wrappers::BroadcastStream;
use uuid::Uuid;

use crate::error::AppError;
use crate::state::AppState;

pub const DEFAULT_EVENT_BUFFER_SIZE: usize = 1000;
pub const SSE_KEEP_ALIVE_INTERVAL: Duration = Duration::from_secs(15);

#[derive(Debug, Deserialize)]
pub struct EventsQuery {
    pub task_ids: Option<String>,
}

pub struct EventBuffer {
    events: VecDeque<events::EventEnvelope>,
    max_size: usize,
}

impl EventBuffer {
    pub fn new(max_size: usize) -> Self {
        Self {
            events: VecDeque::with_capacity(max_size),
            max_size,
        }
    }

    pub fn push(&mut self, envelope: events::EventEnvelope) {
        if self.events.len() >= self.max_size {
            self.events.pop_front();
        }
        self.events.push_back(envelope);
    }

    pub fn events_after(&self, event_id: Uuid) -> Vec<events::EventEnvelope> {
        let mut found = false;
        self.events
            .iter()
            .filter_map(|envelope| {
                if found {
                    Some(envelope.clone())
                } else if envelope.id == event_id {
                    found = true;
                    None
                } else {
                    None
                }
            })
            .collect()
    }

    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.events.len()
    }
}

pub type SharedEventBuffer = Arc<RwLock<EventBuffer>>;

fn parse_task_ids(task_ids: Option<&str>) -> Option<Vec<Uuid>> {
    task_ids.map(|s| {
        s.split(',')
            .filter_map(|id| id.trim().parse().ok())
            .collect()
    })
}

fn envelope_to_sse_event(envelope: &events::EventEnvelope) -> Result<Event, Infallible> {
    let event_type = match &envelope.event {
        events::Event::TaskCreated { .. } => "task.created",
        events::Event::TaskUpdated { .. } => "task.updated",
        events::Event::TaskStatusChanged { .. } => "task.status_changed",
        events::Event::SessionStarted { .. } => "session.started",
        events::Event::SessionEnded { .. } => "session.ended",
        events::Event::PhaseCompleted { .. } => "phase.completed",
        events::Event::PhaseContinuing { .. } => "phase.continuing",
        events::Event::AgentMessage { .. } => "agent.message",
        events::Event::ToolExecution { .. } => "tool.execution",
        events::Event::WorkspaceCreated { .. } => "workspace.created",
        events::Event::WorkspaceMerged { .. } => "workspace.merged",
        events::Event::WorkspaceDeleted { .. } => "workspace.deleted",
        events::Event::ProjectOpened { .. } => "project.opened",
        events::Event::ProjectClosed { .. } => "project.closed",
        events::Event::Error { .. } => "error",
    };

    let data = serde_json::to_string(&envelope).unwrap_or_else(|_| "{}".to_string());

    Ok(Event::default()
        .id(envelope.id.to_string())
        .event(event_type)
        .data(data))
}

fn activity_to_sse_event(
    msg: &orchestrator::SessionActivityMsg,
    seq: u64,
) -> Result<Event, Infallible> {
    let event_type = match msg {
        orchestrator::SessionActivityMsg::ToolCall { .. } => "tool_call",
        orchestrator::SessionActivityMsg::ToolResult { .. } => "tool_result",
        orchestrator::SessionActivityMsg::AgentMessage { .. } => "agent_message",
        orchestrator::SessionActivityMsg::Reasoning { .. } => "reasoning",
        orchestrator::SessionActivityMsg::StepStart { .. } => "step_start",
        orchestrator::SessionActivityMsg::JsonPatch { .. } => "json_patch",
        orchestrator::SessionActivityMsg::Finished { .. } => "finished",
    };

    let data = serde_json::to_string(msg).unwrap_or_else(|_| "{}".to_string());

    Ok(Event::default()
        .id(seq.to_string())
        .event(event_type)
        .data(data))
}

#[utoipa::path(
    get,
    path = "/api/events",
    params(
        ("task_ids" = Option<String>, Query, description = "Comma-separated task IDs to filter events"),
    ),
    responses(
        (status = 200, description = "SSE event stream"),
    ),
    tag = "events"
)]
pub async fn events_stream(
    State(state): State<AppState>,
    Query(query): Query<EventsQuery>,
    headers: axum::http::HeaderMap,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let task_ids = parse_task_ids(query.task_ids.as_deref());
    let last_event_id = headers
        .get("Last-Event-ID")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<Uuid>().ok());

    let buffer = Arc::clone(&state.event_buffer);
    let buffer_for_live = Arc::clone(&buffer);

    let rx = state.event_bus.subscribe();

    let missed_events = if let Some(event_id) = last_event_id {
        buffer
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .events_after(event_id)
    } else {
        vec![]
    };

    let missed_stream =
        futures::stream::iter(missed_events.into_iter().map(|e| envelope_to_sse_event(&e)));

    let live_stream = BroadcastStream::new(rx).filter_map(move |result| {
        let task_ids = task_ids.clone();
        let buffer = Arc::clone(&buffer_for_live);

        async move {
            match result {
                Ok(envelope) => {
                    buffer
                        .write()
                        .unwrap_or_else(|poisoned| poisoned.into_inner())
                        .push(envelope.clone());

                    if let Some(ref ids) = task_ids {
                        if let Some(event_task_id) = envelope.event.task_id() {
                            if !ids.contains(&event_task_id) {
                                return None;
                            }
                        }
                    }

                    Some(envelope_to_sse_event(&envelope))
                }
                Err(e) => {
                    tracing::warn!("SSE broadcast error: {:?}", e);
                    None
                }
            }
        }
    });

    let stream = missed_stream.chain(live_stream);

    Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(SSE_KEEP_ALIVE_INTERVAL)
            .text("keep-alive"),
    )
}

#[utoipa::path(
    get,
    path = "/api/sessions/{id}/activity",
    params(
        ("id" = Uuid, Path, description = "Session ID"),
    ),
    responses(
        (status = 200, description = "SSE activity stream"),
        (status = 404, description = "Session not found"),
    ),
    tag = "sessions"
)]
pub async fn session_activity_stream(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    headers: axum::http::HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    let project = state.project().await?;

    // Check if session exists in DB if no in-memory store
    let store = project.activity_registry.get(&id);
    if store.is_none() {
        let session = project.session_repository.find_by_id(id).await?;
        if session.is_none() {
            return Err(AppError::NotFound(format!("Session not found: {}", id)));
        }
    }

    // Use async version that loads historical activities from DB
    let store = project
        .activity_registry
        .get_or_create_with_history(id)
        .await;

    let last_event_id: Option<u64> = headers
        .get("Last-Event-ID")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse().ok());

    let history = store.get_history();
    let rx = store.subscribe();

    let start_seq = last_event_id.map(|id| id + 1).unwrap_or(0);

    let history_to_send: Vec<_> = history
        .into_iter()
        .enumerate()
        .skip(start_seq as usize)
        .collect();

    let next_seq = start_seq + history_to_send.len() as u64;
    let seq_counter = Arc::new(std::sync::atomic::AtomicU64::new(next_seq));

    let history_stream = futures::stream::iter(
        history_to_send
            .into_iter()
            .map(|(idx, msg)| activity_to_sse_event(&msg, idx as u64)),
    );

    let seq_counter_clone = Arc::clone(&seq_counter);
    let live_stream = BroadcastStream::new(rx).filter_map(move |result| {
        let seq_counter = Arc::clone(&seq_counter_clone);
        async move {
            match result {
                Ok(msg) => {
                    let seq = seq_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    Some(activity_to_sse_event(&msg, seq))
                }
                Err(e) => {
                    tracing::warn!("Session activity SSE broadcast error: {:?}", e);
                    None
                }
            }
        }
    });

    let stream = history_stream.chain(live_stream);

    Ok(Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(SSE_KEEP_ALIVE_INTERVAL)
            .text("keep-alive"),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_task_ids_none() {
        assert!(parse_task_ids(None).is_none());
    }

    #[test]
    fn test_parse_task_ids_empty() {
        assert!(parse_task_ids(Some("")).unwrap().is_empty());
    }

    #[test]
    fn test_parse_task_ids_single() {
        let uuid1 = Uuid::new_v4();
        let result = parse_task_ids(Some(&uuid1.to_string())).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], uuid1);
    }

    #[test]
    fn test_parse_task_ids_multiple() {
        let uuid1 = Uuid::new_v4();
        let uuid2 = Uuid::new_v4();
        let uuid3 = Uuid::new_v4();
        let input = format!("{},{},{}", uuid1, uuid2, uuid3);
        let result = parse_task_ids(Some(&input)).unwrap();
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_parse_task_ids_with_spaces() {
        let uuid1 = Uuid::new_v4();
        let uuid2 = Uuid::new_v4();
        let input = format!("{} , {}", uuid1, uuid2);
        let result = parse_task_ids(Some(&input)).unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_parse_task_ids_filters_invalid() {
        let uuid1 = Uuid::new_v4();
        let uuid2 = Uuid::new_v4();
        let input = format!("{},invalid,{}", uuid1, uuid2);
        let result = parse_task_ids(Some(&input)).unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_event_buffer_events_after() {
        let mut buffer = EventBuffer::new(3);

        let e1 = events::EventEnvelope::new(events::Event::TaskCreated {
            task_id: Uuid::new_v4(),
            title: "Task 1".to_string(),
        });
        let e2 = events::EventEnvelope::new(events::Event::TaskCreated {
            task_id: Uuid::new_v4(),
            title: "Task 2".to_string(),
        });
        let e3 = events::EventEnvelope::new(events::Event::TaskCreated {
            task_id: Uuid::new_v4(),
            title: "Task 3".to_string(),
        });

        let id1 = e1.id;
        let id2 = e2.id;

        buffer.push(e1);
        buffer.push(e2);
        buffer.push(e3.clone());

        let after_first = buffer.events_after(id1);
        assert_eq!(after_first.len(), 2);
        assert_eq!(after_first[0].id, id2);

        let after_second = buffer.events_after(id2);
        assert_eq!(after_second.len(), 1);
        assert_eq!(after_second[0].id, e3.id);

        let after_nonexistent = buffer.events_after(Uuid::new_v4());
        assert!(after_nonexistent.is_empty());
    }

    #[test]
    fn test_event_buffer_evicts_oldest() {
        let mut buffer = EventBuffer::new(2);

        let e1 = events::EventEnvelope::new(events::Event::TaskCreated {
            task_id: Uuid::new_v4(),
            title: "Task 1".to_string(),
        });
        let e2 = events::EventEnvelope::new(events::Event::TaskCreated {
            task_id: Uuid::new_v4(),
            title: "Task 2".to_string(),
        });
        let e3 = events::EventEnvelope::new(events::Event::TaskCreated {
            task_id: Uuid::new_v4(),
            title: "Task 3".to_string(),
        });

        let id1 = e1.id;
        let id2 = e2.id;
        let id3 = e3.id;

        buffer.push(e1);
        buffer.push(e2);
        buffer.push(e3);

        assert_eq!(buffer.len(), 2);
        let after_e1 = buffer.events_after(id1);
        assert!(after_e1.is_empty());
        let after_e2 = buffer.events_after(id2);
        assert_eq!(after_e2.len(), 1);
        assert_eq!(after_e2[0].id, id3);
    }

    #[test]
    fn test_envelope_to_sse_event_does_not_panic() {
        let envelope = events::EventEnvelope::new(events::Event::TaskCreated {
            task_id: Uuid::new_v4(),
            title: "Test".to_string(),
        });

        let _event = envelope_to_sse_event(&envelope).unwrap();
    }

    #[test]
    fn test_activity_to_sse_event_does_not_panic() {
        let msg = orchestrator::SessionActivityMsg::tool_call("tc-1", "bash", None);
        let _event = activity_to_sse_event(&msg, 42).unwrap();
    }
}
