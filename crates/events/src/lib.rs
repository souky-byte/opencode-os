//! Event system for OpenCode Studio
//!
//! This crate provides the event bus and event types for real-time
//! communication between components.

mod bus;
mod types;

pub use bus::EventBus;
pub use types::*;
