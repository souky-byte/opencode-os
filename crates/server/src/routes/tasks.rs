use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use events::{Event, EventEnvelope};
use opencode_core::{CreateTaskRequest, Task, TaskStatus, UpdateTaskRequest};
use orchestrator::PhaseResult;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::error::AppError;
use crate::state::AppState;

#[utoipa::path(
    get,
    path = "/api/tasks",
    responses(
        (status = 200, description = "List of all tasks", body = Vec<Task>)
    ),
    tag = "tasks"
)]
pub async fn list_tasks(State(state): State<AppState>) -> Result<Json<Vec<Task>>, AppError> {
    let tasks = state.task_repository.find_all().await?;
    Ok(Json(tasks))
}

#[utoipa::path(
    post,
    path = "/api/tasks",
    request_body = CreateTaskRequest,
    responses(
        (status = 201, description = "Task created", body = Task),
        (status = 400, description = "Invalid request")
    ),
    tag = "tasks"
)]
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

#[utoipa::path(
    get,
    path = "/api/tasks/{id}",
    params(
        ("id" = Uuid, Path, description = "Task ID")
    ),
    responses(
        (status = 200, description = "Task found", body = Task),
        (status = 404, description = "Task not found")
    ),
    tag = "tasks"
)]
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

#[utoipa::path(
    patch,
    path = "/api/tasks/{id}",
    params(
        ("id" = Uuid, Path, description = "Task ID")
    ),
    request_body = UpdateTaskRequest,
    responses(
        (status = 200, description = "Task updated", body = Task),
        (status = 404, description = "Task not found")
    ),
    tag = "tasks"
)]
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

#[utoipa::path(
    delete,
    path = "/api/tasks/{id}",
    params(
        ("id" = Uuid, Path, description = "Task ID")
    ),
    responses(
        (status = 204, description = "Task deleted"),
        (status = 404, description = "Task not found")
    ),
    tag = "tasks"
)]
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

#[derive(Debug, Deserialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct TransitionRequest {
    pub status: TaskStatus,
}

#[derive(Debug, Serialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct TransitionResponse {
    pub task: Task,
    pub previous_status: TaskStatus,
}

#[utoipa::path(
    post,
    path = "/api/tasks/{id}/transition",
    params(
        ("id" = Uuid, Path, description = "Task ID")
    ),
    request_body = TransitionRequest,
    responses(
        (status = 200, description = "Task transitioned", body = TransitionResponse),
        (status = 400, description = "Invalid transition"),
        (status = 404, description = "Task not found")
    ),
    tag = "tasks"
)]
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

#[derive(Debug, Serialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct ExecuteResponse {
    pub task: Task,
    pub result: PhaseResultDto,
}

#[derive(Debug, Serialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PhaseResultDto {
    SessionCreated { session_id: String },
    PlanCreated { session_id: String, plan_path: String },
    AwaitingApproval { phase: String },
    ReviewPassed { session_id: String },
    ReviewFailed { session_id: String, feedback: String, iteration: u32 },
    MaxIterationsExceeded { iterations: u32 },
    Completed,
}

impl From<PhaseResult> for PhaseResultDto {
    fn from(result: PhaseResult) -> Self {
        match result {
            PhaseResult::SessionCreated { session_id } => Self::SessionCreated { session_id },
            PhaseResult::PlanCreated { session_id, plan_path } => {
                Self::PlanCreated { session_id, plan_path }
            }
            PhaseResult::AwaitingApproval { phase } => {
                Self::AwaitingApproval { phase: phase.as_str().to_string() }
            }
            PhaseResult::ReviewPassed { session_id } => Self::ReviewPassed { session_id },
            PhaseResult::ReviewFailed { session_id, feedback, iteration } => {
                Self::ReviewFailed { session_id, feedback, iteration }
            }
            PhaseResult::MaxIterationsExceeded { iterations } => {
                Self::MaxIterationsExceeded { iterations }
            }
            PhaseResult::Completed => Self::Completed,
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/tasks/{id}/execute",
    params(
        ("id" = Uuid, Path, description = "Task ID")
    ),
    responses(
        (status = 200, description = "Phase executed", body = ExecuteResponse),
        (status = 404, description = "Task not found"),
        (status = 500, description = "Execution failed")
    ),
    tag = "tasks"
)]
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
