use opencode_core::TaskStatus;

use crate::error::{OrchestratorError, Result};

pub struct TaskStateMachine;

impl TaskStateMachine {
    pub fn validate_transition(from: &TaskStatus, to: &TaskStatus) -> Result<()> {
        let allowed = Self::allowed_transitions(from);

        if allowed.contains(to) {
            Ok(())
        } else {
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
            TaskStatus::AiReview => vec![TaskStatus::Review, TaskStatus::InProgress],
            TaskStatus::Review => vec![TaskStatus::Done, TaskStatus::InProgress],
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
            TaskStatus::AiReview => Some(TaskStatus::Review),
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
