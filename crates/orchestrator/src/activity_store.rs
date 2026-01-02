use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};

use axum::extract::ws::Message as WsMessage;
use chrono::{DateTime, Utc};
use db::{CreateSessionActivity, SessionActivityRepository};
use futures::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use uuid::Uuid;

const HISTORY_BYTES_LIMIT: usize = 10 * 1024 * 1024;
const CHANNEL_CAPACITY: usize = 1000;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SessionActivityMsg {
    ToolCall {
        id: String,
        tool_name: String,
        #[cfg_attr(feature = "typescript", ts(type = "Record<string, unknown> | null"))]
        args: Option<serde_json::Value>,
        timestamp: DateTime<Utc>,
    },
    ToolResult {
        id: String,
        tool_name: String,
        #[cfg_attr(feature = "typescript", ts(type = "Record<string, unknown> | null"))]
        args: Option<serde_json::Value>,
        result: String,
        success: bool,
        timestamp: DateTime<Utc>,
    },
    AgentMessage {
        id: String,
        content: String,
        is_partial: bool,
        timestamp: DateTime<Utc>,
    },
    Reasoning {
        id: String,
        content: String,
        timestamp: DateTime<Utc>,
    },
    StepStart {
        id: String,
        step_name: Option<String>,
        timestamp: DateTime<Utc>,
    },
    JsonPatch {
        #[cfg_attr(feature = "typescript", ts(type = "unknown[]"))]
        patch: json_patch::Patch,
        timestamp: DateTime<Utc>,
    },
    Finished {
        success: bool,
        error: Option<String>,
        timestamp: DateTime<Utc>,
    },
}

impl SessionActivityMsg {
    pub fn id(&self) -> Option<&str> {
        match self {
            Self::ToolCall { id, .. } => Some(id),
            Self::ToolResult { id, .. } => Some(id),
            Self::AgentMessage { id, .. } => Some(id),
            Self::Reasoning { id, .. } => Some(id),
            Self::StepStart { id, .. } => Some(id),
            Self::JsonPatch { .. } => None,
            Self::Finished { .. } => None,
        }
    }

    pub fn approx_bytes(&self) -> usize {
        const OVERHEAD: usize = 64;
        match self {
            Self::ToolCall { tool_name, args, .. } => {
                OVERHEAD + tool_name.len() + args.as_ref().map(|a| a.to_string().len()).unwrap_or(0)
            }
            Self::ToolResult { tool_name, result, args, .. } => {
                OVERHEAD
                    + tool_name.len()
                    + result.len()
                    + args.as_ref().map(|a| a.to_string().len()).unwrap_or(0)
            }
            Self::AgentMessage { content, .. } => OVERHEAD + content.len(),
            Self::Reasoning { content, .. } => OVERHEAD + content.len(),
            Self::StepStart { step_name, .. } => {
                OVERHEAD + step_name.as_ref().map(|s| s.len()).unwrap_or(0)
            }
            Self::JsonPatch { patch, .. } => {
                OVERHEAD + serde_json::to_string(patch).map(|s| s.len()).unwrap_or(2)
            }
            Self::Finished { error, .. } => OVERHEAD + error.as_ref().map(|e| e.len()).unwrap_or(0),
        }
    }

    pub fn to_ws_message(&self) -> Result<WsMessage, serde_json::Error> {
        let json = serde_json::to_string(self)?;
        Ok(WsMessage::Text(json.into()))
    }

    pub fn to_ws_message_unchecked(&self) -> WsMessage {
        let json = match self {
            Self::Finished { .. } => serde_json::to_string(self)
                .unwrap_or_else(|_| r#"{"type":"finished","success":false}"#.to_string()),
            _ => serde_json::to_string(self)
                .unwrap_or_else(|_| r#"{"type":"error","message":"serialization_failed"}"#.to_string()),
        };
        WsMessage::Text(json.into())
    }

    pub fn tool_call(id: impl Into<String>, tool_name: impl Into<String>, args: Option<serde_json::Value>) -> Self {
        Self::ToolCall {
            id: id.into(),
            tool_name: tool_name.into(),
            args,
            timestamp: Utc::now(),
        }
    }

    pub fn tool_result(
        id: impl Into<String>,
        tool_name: impl Into<String>,
        args: Option<serde_json::Value>,
        result: impl Into<String>,
        success: bool,
    ) -> Self {
        Self::ToolResult {
            id: id.into(),
            tool_name: tool_name.into(),
            args,
            result: result.into(),
            success,
            timestamp: Utc::now(),
        }
    }

    pub fn agent_message(id: impl Into<String>, content: impl Into<String>, is_partial: bool) -> Self {
        Self::AgentMessage {
            id: id.into(),
            content: content.into(),
            is_partial,
            timestamp: Utc::now(),
        }
    }

    pub fn finished(success: bool, error: Option<String>) -> Self {
        Self::Finished {
            success,
            error,
            timestamp: Utc::now(),
        }
    }
}

#[derive(Clone)]
struct StoredMsg {
    msg: SessionActivityMsg,
    bytes: usize,
}

struct StoreInner {
    history: VecDeque<StoredMsg>,
    total_bytes: usize,
}

pub struct SessionActivityStore {
    session_id: Uuid,
    inner: RwLock<StoreInner>,
    sender: broadcast::Sender<SessionActivityMsg>,
    repository: Option<SessionActivityRepository>,
}

impl SessionActivityStore {
    pub fn new(session_id: Uuid) -> Self {
        let (sender, _) = broadcast::channel(CHANNEL_CAPACITY);
        Self {
            session_id,
            inner: RwLock::new(StoreInner {
                history: VecDeque::with_capacity(64),
                total_bytes: 0,
            }),
            sender,
            repository: None,
        }
    }

    pub fn with_repository(mut self, repository: SessionActivityRepository) -> Self {
        self.repository = Some(repository);
        self
    }

    pub fn session_id(&self) -> Uuid {
        self.session_id
    }

    /// Load historical activities from DB into the in-memory store.
    /// Call this after creating the store to populate it with persisted activities.
    pub async fn load_from_db(&self) -> Result<usize, db::DbError> {
        let Some(ref repo) = self.repository else {
            return Ok(0);
        };

        let activities = repo.find_by_session_id(self.session_id).await?;
        let count = activities.len();

        let mut inner = self.inner.write().unwrap();
        for activity in activities {
            // Reconstruct SessionActivityMsg from stored data
            if let Some(msg) = self.activity_to_msg(&activity) {
                let bytes = msg.approx_bytes();

                // Check memory limit
                while inner.total_bytes.saturating_add(bytes) > HISTORY_BYTES_LIMIT {
                    if let Some(front) = inner.history.pop_front() {
                        inner.total_bytes = inner.total_bytes.saturating_sub(front.bytes);
                    } else {
                        break;
                    }
                }

                inner.history.push_back(StoredMsg { msg, bytes });
                inner.total_bytes = inner.total_bytes.saturating_add(bytes);
            }
        }

        Ok(count)
    }

    fn activity_to_msg(&self, activity: &db::models::SessionActivity) -> Option<SessionActivityMsg> {
        // Reconstruct the message from the stored JSON data
        // The data field should contain the full serialized message
        serde_json::from_value(activity.data.clone()).ok()
    }

    pub fn push(&self, msg: SessionActivityMsg) {
        let _ = self.sender.send(msg.clone());

        // Persist to DB asynchronously if repository is available
        if let Some(ref repo) = self.repository {
            let repo = repo.clone();
            let session_id = self.session_id;
            let msg_clone = msg.clone();
            tokio::spawn(async move {
                let activity_type = match &msg_clone {
                    SessionActivityMsg::ToolCall { .. } => "tool_call",
                    SessionActivityMsg::ToolResult { .. } => "tool_result",
                    SessionActivityMsg::AgentMessage { .. } => "agent_message",
                    SessionActivityMsg::Reasoning { .. } => "reasoning",
                    SessionActivityMsg::StepStart { .. } => "step_start",
                    SessionActivityMsg::JsonPatch { .. } => "json_patch",
                    SessionActivityMsg::Finished { .. } => "finished",
                };
                let activity_id = msg_clone.id().map(|s| s.to_string());
                let data = serde_json::to_value(&msg_clone).unwrap_or(serde_json::Value::Null);

                let create = CreateSessionActivity::new(
                    session_id,
                    activity_type,
                    activity_id,
                    data,
                );

                if let Err(e) = repo.create(&create).await {
                    tracing::warn!("Failed to persist activity to DB: {:?}", e);
                }
            });
        }

        let bytes = msg.approx_bytes();
        let mut inner = self.inner.write().unwrap();

        while inner.total_bytes.saturating_add(bytes) > HISTORY_BYTES_LIMIT {
            if let Some(front) = inner.history.pop_front() {
                inner.total_bytes = inner.total_bytes.saturating_sub(front.bytes);
            } else {
                break;
            }
        }

        inner.history.push_back(StoredMsg { msg, bytes });
        inner.total_bytes = inner.total_bytes.saturating_add(bytes);
    }

    pub fn push_tool_call(&self, id: impl Into<String>, tool_name: impl Into<String>, args: Option<serde_json::Value>) {
        self.push(SessionActivityMsg::tool_call(id, tool_name, args));
    }

    pub fn push_tool_result(
        &self,
        id: impl Into<String>,
        tool_name: impl Into<String>,
        args: Option<serde_json::Value>,
        result: impl Into<String>,
        success: bool,
    ) {
        self.push(SessionActivityMsg::tool_result(id, tool_name, args, result, success));
    }

    pub fn push_agent_message(&self, id: impl Into<String>, content: impl Into<String>, is_partial: bool) {
        self.push(SessionActivityMsg::agent_message(id, content, is_partial));
    }

    pub fn push_patch(&self, patch: json_patch::Patch) {
        self.push(SessionActivityMsg::JsonPatch {
            patch,
            timestamp: Utc::now(),
        });
    }

    pub fn push_finished(&self, success: bool, error: Option<String>) {
        self.push(SessionActivityMsg::finished(success, error));
    }

    pub fn subscribe(&self) -> broadcast::Receiver<SessionActivityMsg> {
        self.sender.subscribe()
    }

    pub fn get_history(&self) -> Vec<SessionActivityMsg> {
        self.inner
            .read()
            .unwrap()
            .history
            .iter()
            .map(|s| s.msg.clone())
            .collect()
    }

    pub fn history_plus_stream(
        &self,
    ) -> impl Stream<Item = Result<SessionActivityMsg, std::io::Error>> + Send + 'static {
        let history = self.get_history();
        let rx = self.subscribe();

        let hist_stream = futures::stream::iter(history.into_iter().map(Ok::<_, std::io::Error>));
        let live_stream = BroadcastStream::new(rx).filter_map(
            |res: Result<SessionActivityMsg, _>| async move { res.ok().map(Ok::<_, std::io::Error>) },
        );

        hist_stream.chain(live_stream)
    }

    pub fn history_len(&self) -> usize {
        self.inner.read().unwrap().history.len()
    }

    pub fn history_bytes(&self) -> usize {
        self.inner.read().unwrap().total_bytes
    }

    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

#[derive(Clone, Default)]
pub struct SessionActivityRegistry {
    stores: Arc<RwLock<HashMap<Uuid, Arc<SessionActivityStore>>>>,
    repository: Option<SessionActivityRepository>,
}

impl SessionActivityRegistry {
    pub fn new() -> Self {
        Self {
            stores: Arc::new(RwLock::new(HashMap::new())),
            repository: None,
        }
    }

    pub fn with_repository(mut self, repository: SessionActivityRepository) -> Self {
        self.repository = Some(repository);
        self
    }

    pub fn get_or_create(&self, session_id: Uuid) -> Arc<SessionActivityStore> {
        {
            let stores = self.stores.read().unwrap();
            if let Some(store) = stores.get(&session_id) {
                return Arc::clone(store);
            }
        }

        let mut stores = self.stores.write().unwrap();
        if let Some(store) = stores.get(&session_id) {
            return Arc::clone(store);
        }

        let store = if let Some(ref repo) = self.repository {
            SessionActivityStore::new(session_id).with_repository(repo.clone())
        } else {
            SessionActivityStore::new(session_id)
        };
        let store = Arc::new(store);
        stores.insert(session_id, Arc::clone(&store));
        store
    }

    /// Get or create a store, loading history from DB if available.
    /// This is an async version that ensures historical data is loaded.
    pub async fn get_or_create_with_history(&self, session_id: Uuid) -> Arc<SessionActivityStore> {
        // Check if already exists
        {
            let stores = self.stores.read().unwrap();
            if let Some(store) = stores.get(&session_id) {
                return Arc::clone(store);
            }
        }

        // Create the store outside the lock
        let store = if let Some(ref repo) = self.repository {
            SessionActivityStore::new(session_id).with_repository(repo.clone())
        } else {
            SessionActivityStore::new(session_id)
        };
        let store = Arc::new(store);

        // Load from DB if repository is available (before acquiring write lock)
        if self.repository.is_some() {
            if let Err(e) = store.load_from_db().await {
                tracing::warn!("Failed to load activities from DB for session {}: {:?}", session_id, e);
            }
        }

        // Now acquire write lock and insert (or return existing if race)
        let mut stores = self.stores.write().unwrap();
        if let Some(existing) = stores.get(&session_id) {
            // Another task created the store while we were loading
            return Arc::clone(existing);
        }

        stores.insert(session_id, Arc::clone(&store));
        store
    }

    pub fn get(&self, session_id: &Uuid) -> Option<Arc<SessionActivityStore>> {
        self.stores.read().unwrap().get(session_id).cloned()
    }

    pub fn remove(&self, session_id: &Uuid) -> Option<Arc<SessionActivityStore>> {
        self.stores.write().unwrap().remove(session_id)
    }

    pub fn session_ids(&self) -> Vec<Uuid> {
        self.stores.read().unwrap().keys().copied().collect()
    }

    pub fn len(&self) -> usize {
        self.stores.read().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.stores.read().unwrap().is_empty()
    }

    pub fn repository(&self) -> Option<&SessionActivityRepository> {
        self.repository.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_activity_msg_creation() {
        let tool_call = SessionActivityMsg::tool_call("tc-1", "read_file", None);
        assert!(matches!(tool_call, SessionActivityMsg::ToolCall { .. }));
        assert_eq!(tool_call.id(), Some("tc-1"));

        let tool_result = SessionActivityMsg::tool_result(
            "tc-1",
            "read_file",
            None,
            "file contents",
            true,
        );
        assert!(matches!(tool_result, SessionActivityMsg::ToolResult { .. }));

        let finished = SessionActivityMsg::finished(true, None);
        assert!(matches!(finished, SessionActivityMsg::Finished { .. }));
        assert_eq!(finished.id(), None);
    }

    #[test]
    fn test_activity_msg_serialization() {
        let msg = SessionActivityMsg::tool_call("tc-1", "bash", Some(serde_json::json!({"command": "ls"})));
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("tool_call"));
        assert!(json.contains("bash"));
        assert!(json.contains("tc-1"));
    }

    #[test]
    fn test_store_push_and_history() {
        let store = SessionActivityStore::new(Uuid::new_v4());

        store.push_tool_call("tc-1", "read_file", None);
        store.push_tool_result("tc-1", "read_file", None, "contents", true);
        store.push_agent_message("msg-1", "Hello", false);

        let history = store.get_history();
        assert_eq!(history.len(), 3);
        assert!(matches!(history[0], SessionActivityMsg::ToolCall { .. }));
        assert!(matches!(history[1], SessionActivityMsg::ToolResult { .. }));
        assert!(matches!(history[2], SessionActivityMsg::AgentMessage { .. }));
    }

    #[tokio::test]
    async fn test_store_broadcast() {
        let store = Arc::new(SessionActivityStore::new(Uuid::new_v4()));

        let mut rx = store.subscribe();

        store.push_tool_call("tc-1", "test_tool", None);

        let msg = rx.recv().await.unwrap();
        assert!(matches!(msg, SessionActivityMsg::ToolCall { tool_name, .. } if tool_name == "test_tool"));
    }

    #[test]
    fn test_registry_get_or_create() {
        let registry = SessionActivityRegistry::new();
        let session_id = Uuid::new_v4();

        let store1 = registry.get_or_create(session_id);
        let store2 = registry.get_or_create(session_id);

        assert_eq!(store1.session_id(), store2.session_id());
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_registry_remove() {
        let registry = SessionActivityRegistry::new();
        let session_id = Uuid::new_v4();

        registry.get_or_create(session_id);
        assert_eq!(registry.len(), 1);

        registry.remove(&session_id);
        assert!(registry.is_empty());
    }

    #[test]
    fn test_ws_message_conversion() {
        let msg = SessionActivityMsg::tool_call("tc-1", "bash", None);
        let ws_msg = msg.to_ws_message().unwrap();
        assert!(matches!(ws_msg, WsMessage::Text(_)));
    }

    #[tokio::test]
    async fn test_history_plus_stream_returns_history_first() {
        use futures::StreamExt;

        let store = Arc::new(SessionActivityStore::new(Uuid::new_v4()));

        store.push_tool_call("tc-1", "read_file", None);
        store.push_tool_result("tc-1", "read_file", None, "contents", true);
        store.push_agent_message("msg-1", "Processing...", false);

        let mut stream = std::pin::pin!(store.history_plus_stream());

        let msg1 = stream.next().await.unwrap().unwrap();
        assert!(matches!(msg1, SessionActivityMsg::ToolCall { id, .. } if id == "tc-1"));

        let msg2 = stream.next().await.unwrap().unwrap();
        assert!(matches!(msg2, SessionActivityMsg::ToolResult { id, .. } if id == "tc-1"));

        let msg3 = stream.next().await.unwrap().unwrap();
        assert!(matches!(msg3, SessionActivityMsg::AgentMessage { id, .. } if id == "msg-1"));
    }

    #[tokio::test]
    async fn test_history_plus_stream_then_live() {
        use futures::StreamExt;
        use tokio::time::{timeout, Duration};

        let store = Arc::new(SessionActivityStore::new(Uuid::new_v4()));

        store.push_tool_call("tc-1", "read_file", None);

        let store_clone = Arc::clone(&store);
        let mut stream = std::pin::pin!(store.history_plus_stream());

        let history_msg = stream.next().await.unwrap().unwrap();
        assert!(matches!(history_msg, SessionActivityMsg::ToolCall { id, .. } if id == "tc-1"));

        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(10)).await;
            store_clone.push_tool_call("tc-2", "write_file", None);
        });

        let live_msg = timeout(Duration::from_millis(100), stream.next())
            .await
            .expect("should receive live message")
            .unwrap()
            .unwrap();
        assert!(matches!(live_msg, SessionActivityMsg::ToolCall { id, .. } if id == "tc-2"));
    }

    #[tokio::test]
    async fn test_reconnect_gets_full_history() {
        use futures::StreamExt;
        use tokio::time::{timeout, Duration};

        let store = Arc::new(SessionActivityStore::new(Uuid::new_v4()));

        store.push_tool_call("tc-1", "read_file", None);
        store.push_tool_result("tc-1", "read_file", None, "contents", true);

        {
            let mut stream1 = std::pin::pin!(store.history_plus_stream());
            let _ = stream1.next().await;
            let _ = stream1.next().await;
        }

        store.push_agent_message("msg-1", "Done", false);

        let mut stream2 = std::pin::pin!(store.history_plus_stream());

        let msg1 = timeout(Duration::from_millis(100), stream2.next())
            .await
            .expect("should get history msg 1")
            .unwrap()
            .unwrap();
        let msg2 = timeout(Duration::from_millis(100), stream2.next())
            .await
            .expect("should get history msg 2")
            .unwrap()
            .unwrap();
        let msg3 = timeout(Duration::from_millis(100), stream2.next())
            .await
            .expect("should get history msg 3")
            .unwrap()
            .unwrap();

        assert!(matches!(msg1, SessionActivityMsg::ToolCall { .. }));
        assert!(matches!(msg2, SessionActivityMsg::ToolResult { .. }));
        assert!(matches!(msg3, SessionActivityMsg::AgentMessage { .. }));
    }

    #[test]
    fn test_finished_message() {
        let store = SessionActivityStore::new(Uuid::new_v4());

        store.push_tool_call("tc-1", "bash", None);
        store.push_finished(true, None);

        let history = store.get_history();
        assert_eq!(history.len(), 2);

        match &history[1] {
            SessionActivityMsg::Finished { success, error, .. } => {
                assert!(*success);
                assert!(error.is_none());
            }
            _ => panic!("Expected Finished message"),
        }
    }

    #[test]
    fn test_finished_with_error() {
        let store = SessionActivityStore::new(Uuid::new_v4());

        store.push_finished(false, Some("Task failed".to_string()));

        let history = store.get_history();
        assert_eq!(history.len(), 1);

        match &history[0] {
            SessionActivityMsg::Finished { success, error, .. } => {
                assert!(!*success);
                assert_eq!(error.as_deref(), Some("Task failed"));
            }
            _ => panic!("Expected Finished message"),
        }
    }
}
