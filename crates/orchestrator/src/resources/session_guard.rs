//! RAII guard for session lifecycle management.
//!
//! This module provides automatic session failure handling when
//! the guard goes out of scope without being marked as completed.

use tracing::{debug, warn};
use uuid::Uuid;

use events::{Event, EventBus, EventEnvelope};

/// RAII guard for session lifecycle.
///
/// When this guard is dropped without being marked as completed,
/// it automatically emits a session failed event. This ensures
/// proper error handling even in panic scenarios.
///
/// # Example
///
/// ```ignore
/// let mut guard = SessionGuard::new(session_id, task_id, event_bus);
/// // ... execute session ...
/// guard.mark_completed(); // Must call this on success
/// // If not called, Drop will emit a failure event
/// ```
pub struct SessionGuard {
    session_id: Uuid,
    task_id: Uuid,
    event_bus: Option<EventBus>,
    completed: bool,
}

impl SessionGuard {
    /// Create a new session guard.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The local session ID
    /// * `task_id` - The task ID this session belongs to
    /// * `event_bus` - Optional event bus for emitting failure events
    pub fn new(session_id: Uuid, task_id: Uuid, event_bus: Option<EventBus>) -> Self {
        debug!(
            session_id = %session_id,
            task_id = %task_id,
            "Session guard created"
        );

        Self {
            session_id,
            task_id,
            event_bus,
            completed: false,
        }
    }

    /// Mark the session as completed successfully.
    ///
    /// This prevents the guard from emitting a failure event on drop.
    pub fn mark_completed(&mut self) {
        debug!(
            session_id = %self.session_id,
            "Session marked as completed"
        );
        self.completed = true;
    }

    /// Mark the session as failed with an error.
    ///
    /// This immediately emits a failure event and marks the guard
    /// as completed (to prevent double emission on drop).
    pub fn mark_failed(&mut self, error: &str) {
        debug!(
            session_id = %self.session_id,
            error = %error,
            "Session marked as failed"
        );

        self.emit_failure(Some(error.to_string()));
        self.completed = true;
    }

    /// Check if the session has been completed.
    pub fn is_completed(&self) -> bool {
        self.completed
    }

    /// Get the session ID.
    pub fn session_id(&self) -> Uuid {
        self.session_id
    }

    /// Get the task ID.
    pub fn task_id(&self) -> Uuid {
        self.task_id
    }

    /// Emit a session failure event.
    fn emit_failure(&self, _error: Option<String>) {
        if let Some(ref bus) = self.event_bus {
            let event = Event::SessionEnded {
                session_id: self.session_id,
                task_id: self.task_id,
                success: false,
            };
            bus.publish(EventEnvelope::new(event));
        }
    }
}

impl Drop for SessionGuard {
    fn drop(&mut self) {
        if !self.completed {
            warn!(
                session_id = %self.session_id,
                task_id = %self.task_id,
                "Session guard dropped without completion - emitting failure"
            );

            self.emit_failure(Some("Session terminated unexpectedly".to_string()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guard_initial_state() {
        let guard = SessionGuard::new(Uuid::new_v4(), Uuid::new_v4(), None);

        assert!(!guard.is_completed());
    }

    #[test]
    fn test_guard_mark_completed() {
        let mut guard = SessionGuard::new(Uuid::new_v4(), Uuid::new_v4(), None);

        guard.mark_completed();
        assert!(guard.is_completed());
    }

    #[test]
    fn test_guard_mark_failed() {
        let mut guard = SessionGuard::new(Uuid::new_v4(), Uuid::new_v4(), None);

        guard.mark_failed("test error");
        assert!(guard.is_completed());
    }

    #[test]
    fn test_guard_ids() {
        let session_id = Uuid::new_v4();
        let task_id = Uuid::new_v4();
        let guard = SessionGuard::new(session_id, task_id, None);

        assert_eq!(guard.session_id(), session_id);
        assert_eq!(guard.task_id(), task_id);
    }
}
