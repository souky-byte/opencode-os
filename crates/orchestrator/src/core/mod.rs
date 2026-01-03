//! Core abstractions for the orchestrator execution model.
//!
//! This module provides the foundational types and traits for unified phase execution:
//!
//! - [`Phase`] - Trait that all execution phases implement
//! - [`PhaseConfig`] - Configuration for session execution
//! - [`PhaseOutcome`] - Result of phase processing
//! - [`ExecutionEngine`] - Unified execution engine for all phases
//! - [`OrderedEventEmitter`] - Event emitter with sequence guarantees

mod events;
mod execution;
mod phase;

pub use events::OrderedEventEmitter;
pub use execution::{AcquiredResources, ExecutionEngine};
pub use phase::{
    McpServerSpec, McpServerType, Phase, PhaseConfig, PhaseMetadata, PhaseOutcome,
    ResourceRequirements, SessionOutput,
};
