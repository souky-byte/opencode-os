use opencode_core::TaskStatus;
use tracing::{debug, warn};

use crate::error::{OrchestratorError, Result};

pub struct TaskStateMachine;

impl TaskStateMachine {
    pub fn validate_transition(from: &TaskStatus, to: &TaskStatus) -> Result<()> {
        let allowed = Self::allowed_transitions(from);

        if allowed.contains(to) {
            debug!(
                from = %from.as_str(),
                to = %to.as_str(),
                "State transition validated"
            );
            Ok(())
        } else {
            warn!(
                from = %from.as_str(),
                to = %to.as_str(),
                allowed = ?allowed.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
                "Invalid state transition attempted"
            );
            Err(OrchestratorError::InvalidTransition {
                from: from.as_str().to_string(),
                to: to.as_str().to_string(),
            })
        }
    }

    fn allowed_transitions(from: &TaskStatus) -> Vec<TaskStatus> {
        match from {
            TaskStatus::Todo => vec![TaskStatus::Planning],
            TaskStatus::Planning => vec![TaskStatus::PlanningReview, TaskStatus::Todo],
            TaskStatus::PlanningReview => vec![TaskStatus::InProgress, TaskStatus::Planning],
            TaskStatus::InProgress => vec![TaskStatus::AiReview, TaskStatus::PlanningReview],
            // AiReview can go to: Fix (fix findings), Review (skip/approved), InProgress (back to impl)
            TaskStatus::AiReview => {
                vec![TaskStatus::Fix, TaskStatus::Review, TaskStatus::InProgress]
            }
            // Fix goes back to AiReview for re-review after fixing
            TaskStatus::Fix => vec![TaskStatus::AiReview],
            // Review can go to: Done (approved), InProgress (request changes), Fix (fix remaining findings)
            TaskStatus::Review => vec![TaskStatus::Done, TaskStatus::InProgress, TaskStatus::Fix],
            TaskStatus::Done => vec![],
        }
    }

    pub fn can_transition(from: &TaskStatus, to: &TaskStatus) -> bool {
        Self::validate_transition(from, to).is_ok()
    }

    pub fn next_status(current: &TaskStatus) -> Option<TaskStatus> {
        match current {
            TaskStatus::Todo => Some(TaskStatus::Planning),
            TaskStatus::Planning => Some(TaskStatus::PlanningReview),
            TaskStatus::PlanningReview => Some(TaskStatus::InProgress),
            TaskStatus::InProgress => Some(TaskStatus::AiReview),
            // From AiReview, default next is Review (approved/skip path)
            // Use transition_to_fix() for the fix path
            TaskStatus::AiReview => Some(TaskStatus::Review),
            // Fix goes back to AiReview
            TaskStatus::Fix => Some(TaskStatus::AiReview),
            TaskStatus::Review => Some(TaskStatus::Done),
            TaskStatus::Done => None,
        }
    }

    pub fn previous_status(current: &TaskStatus) -> Option<TaskStatus> {
        match current {
            TaskStatus::Todo => None,
            TaskStatus::Planning => Some(TaskStatus::Todo),
            TaskStatus::PlanningReview => Some(TaskStatus::Planning),
            TaskStatus::InProgress => Some(TaskStatus::PlanningReview),
            TaskStatus::AiReview => Some(TaskStatus::InProgress),
            // Fix comes after AiReview
            TaskStatus::Fix => Some(TaskStatus::AiReview),
            TaskStatus::Review => Some(TaskStatus::AiReview),
            TaskStatus::Done => Some(TaskStatus::Review),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_transitions() {
        assert!(TaskStateMachine::can_transition(
            &TaskStatus::Todo,
            &TaskStatus::Planning
        ));
        assert!(TaskStateMachine::can_transition(
            &TaskStatus::Planning,
            &TaskStatus::PlanningReview
        ));
        assert!(TaskStateMachine::can_transition(
            &TaskStatus::InProgress,
            &TaskStatus::AiReview
        ));
    }

    #[test]
    fn test_invalid_transitions() {
        assert!(!TaskStateMachine::can_transition(
            &TaskStatus::Todo,
            &TaskStatus::Done
        ));
        assert!(!TaskStateMachine::can_transition(
            &TaskStatus::Planning,
            &TaskStatus::Done
        ));
        assert!(!TaskStateMachine::can_transition(
            &TaskStatus::Done,
            &TaskStatus::Todo
        ));
    }

    #[test]
    fn test_backward_transitions() {
        assert!(TaskStateMachine::can_transition(
            &TaskStatus::Planning,
            &TaskStatus::Todo
        ));
        assert!(TaskStateMachine::can_transition(
            &TaskStatus::InProgress,
            &TaskStatus::PlanningReview
        ));
    }

    #[test]
    fn test_next_status() {
        assert_eq!(
            TaskStateMachine::next_status(&TaskStatus::Todo),
            Some(TaskStatus::Planning)
        );
        assert_eq!(TaskStateMachine::next_status(&TaskStatus::Done), None);
    }
}
