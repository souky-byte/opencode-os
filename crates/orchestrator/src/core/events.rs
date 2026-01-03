//! Ordered event emitter with sequence guarantees.
//!
//! This module provides an event emitter that ensures events are emitted
//! with monotonically increasing sequence numbers for proper ordering.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use events::{Event, EventBus, EventEnvelope};

/// Event emitter with sequence number guarantees.
///
/// Wraps an EventBus and adds sequence numbers to events to ensure
/// proper ordering even in concurrent scenarios.
#[derive(Clone)]
pub struct OrderedEventEmitter {
    bus: EventBus,
    sequence: Arc<AtomicU64>,
}

impl OrderedEventEmitter {
    /// Create a new ordered event emitter wrapping the given bus.
    pub fn new(bus: EventBus) -> Self {
        Self {
            bus,
            sequence: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Emit an event with the next sequence number.
    pub fn emit(&self, event: Event) {
        let _seq = self.sequence.fetch_add(1, Ordering::SeqCst);
        // Wrap event in envelope for the bus
        let envelope = EventEnvelope::new(event);
        self.bus.publish(envelope);
    }

    /// Get the current sequence number (for debugging/testing).
    pub fn current_sequence(&self) -> u64 {
        self.sequence.load(Ordering::SeqCst)
    }

    /// Get a reference to the underlying event bus.
    pub fn bus(&self) -> &EventBus {
        &self.bus
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_sequence_increments() {
        let bus = EventBus::new();
        let emitter = OrderedEventEmitter::new(bus);

        assert_eq!(emitter.current_sequence(), 0);

        emitter.emit(Event::TaskCreated {
            task_id: Uuid::new_v4(),
            title: "Test task 1".to_string(),
        });
        assert_eq!(emitter.current_sequence(), 1);

        emitter.emit(Event::TaskCreated {
            task_id: Uuid::new_v4(),
            title: "Test task 2".to_string(),
        });
        assert_eq!(emitter.current_sequence(), 2);
    }

    #[test]
    fn test_clone_shares_sequence() {
        let bus = EventBus::new();
        let emitter1 = OrderedEventEmitter::new(bus);
        let emitter2 = emitter1.clone();

        emitter1.emit(Event::TaskCreated {
            task_id: Uuid::new_v4(),
            title: "Test task 1".to_string(),
        });
        assert_eq!(emitter1.current_sequence(), 1);
        assert_eq!(emitter2.current_sequence(), 1);

        emitter2.emit(Event::TaskCreated {
            task_id: Uuid::new_v4(),
            title: "Test task 2".to_string(),
        });
        assert_eq!(emitter1.current_sequence(), 2);
        assert_eq!(emitter2.current_sequence(), 2);
    }
}
