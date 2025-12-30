use serde::{Deserialize, Serialize};
use uuid::Uuid;

use events::EventEnvelope;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    Subscribe { filter: Option<SubscriptionFilter> },
    Unsubscribe,
    Ping,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    Event { envelope: EventEnvelope },
    Subscribed { filter: Option<SubscriptionFilter> },
    Unsubscribed,
    Pong,
    Error { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionFilter {
    pub task_ids: Option<Vec<Uuid>>,
}

impl SubscriptionFilter {
    pub fn for_task(task_id: Uuid) -> Self {
        Self {
            task_ids: Some(vec![task_id]),
        }
    }

    pub fn for_tasks(task_ids: Vec<Uuid>) -> Self {
        Self {
            task_ids: Some(task_ids),
        }
    }

    pub fn matches(&self, envelope: &EventEnvelope) -> bool {
        match &self.task_ids {
            Some(ids) => {
                if let Some(event_task_id) = envelope.event.task_id() {
                    ids.contains(&event_task_id)
                } else {
                    true
                }
            }
            None => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use events::Event;

    #[test]
    fn test_client_message_serialize() {
        let msg = ClientMessage::Subscribe { filter: None };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("subscribe"));
    }

    #[test]
    fn test_client_message_deserialize() {
        let json = r#"{"type":"ping"}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();
        assert!(matches!(msg, ClientMessage::Ping));
    }

    #[test]
    fn test_server_message_serialize() {
        let msg = ServerMessage::Pong;
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("pong"));
    }

    #[test]
    fn test_subscription_filter_matches_all() {
        let filter = SubscriptionFilter { task_ids: None };
        let task_id = Uuid::new_v4();
        let envelope = EventEnvelope::new(Event::TaskCreated {
            task_id,
            title: "Test".to_string(),
        });

        assert!(filter.matches(&envelope));
    }

    #[test]
    fn test_subscription_filter_matches_specific_task() {
        let task_id = Uuid::new_v4();
        let other_task_id = Uuid::new_v4();

        let filter = SubscriptionFilter::for_task(task_id);

        let matching_envelope = EventEnvelope::new(Event::TaskCreated {
            task_id,
            title: "Test".to_string(),
        });

        let non_matching_envelope = EventEnvelope::new(Event::TaskCreated {
            task_id: other_task_id,
            title: "Other".to_string(),
        });

        assert!(filter.matches(&matching_envelope));
        assert!(!filter.matches(&non_matching_envelope));
    }

    #[test]
    fn test_subscription_filter_allows_events_without_task_id() {
        let filter = SubscriptionFilter::for_task(Uuid::new_v4());
        let envelope = EventEnvelope::new(Event::Error {
            message: "test".to_string(),
            context: None,
        });

        assert!(filter.matches(&envelope));
    }
}
