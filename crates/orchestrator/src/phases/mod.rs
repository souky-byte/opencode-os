//! Phase implementations for the orchestrator.
//!
//! This module provides concrete implementations of the [`Phase`] trait
//! for each execution phase:
//!
//! - [`PlanningPhase`] - Generates implementation plan
//! - [`ImplementationPhase`] - Converts plan to code (supports multi-phase)
//! - [`ReviewPhase`] - AI-driven code review with findings
//! - [`FixPhase`] - Addresses review findings

mod fix;
mod implementation;
mod planning;
mod review;

pub use fix::{FixMode, FixPhase};
pub use implementation::{AtomicPhaseContext, ImplementationPhase, PhaseContextState};
pub use planning::PlanningPhase;
pub use review::ReviewPhase;
