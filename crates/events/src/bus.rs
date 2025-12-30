//! Event bus implementation using tokio broadcast channels

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::broadcast;

use crate::types::EventEnvelope;

/// Capacity for the broadcast channel
const DEFAULT_CAPACITY: usize = 1000;

/// Event bus for publishing and subscribing to events
#[derive(Clone)]
pub struct EventBus {
    sender: broadcast::Sender<EventEnvelope>,
    /// Number of events published (for monitoring)
    event_count: Arc<AtomicUsize>,
}

impl EventBus {
    /// Create a new event bus with default capacity
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_CAPACITY)
    }

    /// Create a new event bus with specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self {
            sender,
            event_count: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Publish an event to all subscribers
    ///
    /// Returns the number of subscribers that received the event.
    /// If there are no subscribers, returns 0 (the event is dropped).
    pub fn publish(&self, envelope: EventEnvelope) -> usize {
        self.event_count.fetch_add(1, Ordering::Relaxed);
        self.sender.send(envelope).unwrap_or(0)
    }

    /// Subscribe to events
    ///
    /// Returns a receiver that will receive all published events.
    /// Note: Events published before subscribing will not be received.
    pub fn subscribe(&self) -> broadcast::Receiver<EventEnvelope> {
        self.sender.subscribe()
    }

    /// Get the number of current subscribers
    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }

    /// Get the total number of events published
    pub fn event_count(&self) -> usize {
        self.event_count.load(Ordering::Relaxed)
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for EventBus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventBus")
            .field("subscriber_count", &self.subscriber_count())
            .field("event_count", &self.event_count())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Event;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_publish_subscribe() {
        let bus = EventBus::new();
        let mut rx = bus.subscribe();

        let event = Event::TaskCreated {
            task_id: Uuid::new_v4(),
            title: "Test".to_string(),
        };
        let envelope = EventEnvelope::new(event);

        let sent = bus.publish(envelope.clone());
        assert_eq!(sent, 1);

        let received = rx.recv().await.unwrap();
        assert_eq!(received.id, envelope.id);
    }

    #[tokio::test]
    async fn test_multiple_subscribers() {
        let bus = EventBus::new();
        let mut rx1 = bus.subscribe();
        let mut rx2 = bus.subscribe();

        let event = Event::TaskCreated {
            task_id: Uuid::new_v4(),
            title: "Test".to_string(),
        };
        let envelope = EventEnvelope::new(event);
        let envelope_id = envelope.id;

        let sent = bus.publish(envelope);
        assert_eq!(sent, 2);

        let received1 = rx1.recv().await.unwrap();
        let received2 = rx2.recv().await.unwrap();

        assert_eq!(received1.id, envelope_id);
        assert_eq!(received2.id, envelope_id);
    }

    #[tokio::test]
    async fn test_no_subscribers() {
        let bus = EventBus::new();

        let event = Event::TaskCreated {
            task_id: Uuid::new_v4(),
            title: "Test".to_string(),
        };
        let envelope = EventEnvelope::new(event);

        // No subscribers, event is dropped
        let sent = bus.publish(envelope);
        assert_eq!(sent, 0);
    }

    #[tokio::test]
    async fn test_subscriber_count() {
        let bus = EventBus::new();
        assert_eq!(bus.subscriber_count(), 0);

        let _rx1 = bus.subscribe();
        assert_eq!(bus.subscriber_count(), 1);

        let _rx2 = bus.subscribe();
        assert_eq!(bus.subscriber_count(), 2);

        drop(_rx1);
        // Note: broadcast channel may not immediately reflect dropped receivers
    }

    #[tokio::test]
    async fn test_event_count() {
        let bus = EventBus::new();
        assert_eq!(bus.event_count(), 0);

        let event = Event::Error {
            message: "test".to_string(),
            context: None,
        };
        bus.publish(EventEnvelope::new(event.clone()));
        assert_eq!(bus.event_count(), 1);

        bus.publish(EventEnvelope::new(event));
        assert_eq!(bus.event_count(), 2);
    }

    #[test]
    fn test_clone() {
        let bus1 = EventBus::new();
        let bus2 = bus1.clone();

        let _rx = bus2.subscribe();
        assert_eq!(bus1.subscriber_count(), 1);
        assert_eq!(bus2.subscriber_count(), 1);
    }
}
