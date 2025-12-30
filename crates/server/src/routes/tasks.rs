use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use events::{Event, EventEnvelope};
use opencode_core::{CreateTaskRequest, Task, TaskStatus, UpdateTaskRequest};
use orchestrator::PhaseResult;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;
use crate::state::AppState;

pub async fn list_tasks(State(state): State<AppState>) -> Result<Json<Vec<Task>>, AppError> {
    let tasks = state.task_repository.find_all().await?;
    Ok(Json(tasks))
}

pub async fn create_task(
    State(state): State<AppState>,
    Json(payload): Json<CreateTaskRequest>,
) -> Result<(StatusCode, Json<Task>), AppError> {
    if payload.title.trim().is_empty() {
        return Err(AppError::BadRequest("Title cannot be empty".to_string()));
    }

    let task = Task::new(payload.title.clone(), payload.description);
    let created = state.task_repository.create(&task).await?;

    state
        .event_bus
        .publish(EventEnvelope::new(Event::TaskCreated {
            task_id: created.id,
            title: payload.title,
        }));

    Ok((StatusCode::CREATED, Json(created)))
}

pub async fn get_task(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Task>, AppError> {
    let task = state.task_repository.find_by_id(id).await?;

    match task {
        Some(t) => Ok(Json(t)),
        None => Err(AppError::NotFound(format!("Task not found: {}", id))),
    }
}

pub async fn update_task(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateTaskRequest>,
) -> Result<Json<Task>, AppError> {
    let updated = state.task_repository.update(id, &payload).await?;

    match updated {
        Some(t) => Ok(Json(t)),
        None => Err(AppError::NotFound(format!("Task not found: {}", id))),
    }
}

pub async fn delete_task(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let deleted = state.task_repository.delete(id).await?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound(format!("Task not found: {}", id)))
    }
}

#[derive(Debug, Deserialize)]
pub struct TransitionRequest {
    pub status: TaskStatus,
}

#[derive(Debug, Serialize)]
pub struct TransitionResponse {
    pub task: Task,
    pub previous_status: TaskStatus,
}

pub async fn transition_task(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<TransitionRequest>,
) -> Result<Json<TransitionResponse>, AppError> {
    let task = state.task_repository.find_by_id(id).await?;
    let Some(mut task) = task else {
        return Err(AppError::NotFound(format!("Task not found: {}", id)));
    };

    let previous_status = task.status;

    state
        .task_executor
        .transition(&mut task, payload.status)
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    let update = UpdateTaskRequest {
        status: Some(task.status),
        ..Default::default()
    };
    state.task_repository.update(id, &update).await?;

    state
        .event_bus
        .publish(EventEnvelope::new(Event::TaskStatusChanged {
            task_id: id,
            from_status: format!("{:?}", previous_status),
            to_status: format!("{:?}", task.status),
        }));

    Ok(Json(TransitionResponse {
        task,
        previous_status,
    }))
}

#[derive(Debug, Serialize)]
pub struct ExecuteResponse {
    pub task: Task,
    pub result: PhaseResultDto,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PhaseResultDto {
    SessionCreated { session_id: String },
    AwaitingApproval,
    ReviewPassed,
    ReviewFailed { feedback: String },
    Completed,
}

impl From<PhaseResult> for PhaseResultDto {
    fn from(result: PhaseResult) -> Self {
        match result {
            PhaseResult::SessionCreated { session_id } => Self::SessionCreated { session_id },
            PhaseResult::AwaitingApproval => Self::AwaitingApproval,
            PhaseResult::ReviewPassed => Self::ReviewPassed,
            PhaseResult::ReviewFailed { feedback } => Self::ReviewFailed { feedback },
            PhaseResult::Completed => Self::Completed,
        }
    }
}

pub async fn execute_task(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ExecuteResponse>, AppError> {
    let task = state.task_repository.find_by_id(id).await?;
    let Some(mut task) = task else {
        return Err(AppError::NotFound(format!("Task not found: {}", id)));
    };

    let result = state
        .task_executor
        .execute_phase(&mut task)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let update = UpdateTaskRequest {
        status: Some(task.status),
        ..Default::default()
    };
    state.task_repository.update(id, &update).await?;

    Ok(Json(ExecuteResponse {
        task,
        result: result.into(),
    }))
}
