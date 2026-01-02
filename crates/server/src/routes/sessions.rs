use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use opencode_core::Session;
use uuid::Uuid;

use crate::error::AppError;
use crate::state::AppState;

#[utoipa::path(
    get,
    path = "/api/sessions",
    responses(
        (status = 200, description = "List of all sessions", body = Vec<Session>)
    ),
    tag = "sessions"
)]
pub async fn list_sessions(State(state): State<AppState>) -> Result<Json<Vec<Session>>, AppError> {
    let project = state.project().await?;
    let sessions = project.session_repository.find_all().await?;
    Ok(Json(sessions))
}

#[utoipa::path(
    get,
    path = "/api/sessions/{id}",
    params(
        ("id" = Uuid, Path, description = "Session ID")
    ),
    responses(
        (status = 200, description = "Session found", body = Session),
        (status = 404, description = "Session not found")
    ),
    tag = "sessions"
)]
pub async fn get_session(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Session>, AppError> {
    let project = state.project().await?;
    let session = project.session_repository.find_by_id(id).await?;

    match session {
        Some(s) => Ok(Json(s)),
        None => Err(AppError::NotFound(format!("Session not found: {}", id))),
    }
}

#[utoipa::path(
    get,
    path = "/api/tasks/{task_id}/sessions",
    params(
        ("task_id" = Uuid, Path, description = "Task ID")
    ),
    responses(
        (status = 200, description = "Sessions for task", body = Vec<Session>)
    ),
    tag = "sessions"
)]
pub async fn list_sessions_for_task(
    State(state): State<AppState>,
    Path(task_id): Path<Uuid>,
) -> Result<Json<Vec<Session>>, AppError> {
    let project = state.project().await?;
    let sessions = project.session_repository.find_by_task_id(task_id).await?;
    Ok(Json(sessions))
}

#[utoipa::path(
    delete,
    path = "/api/sessions/{id}",
    params(
        ("id" = Uuid, Path, description = "Session ID")
    ),
    responses(
        (status = 204, description = "Session deleted"),
        (status = 404, description = "Session not found")
    ),
    tag = "sessions"
)]
pub async fn delete_session(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let project = state.project().await?;
    let deleted = project.session_repository.delete(id).await?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound(format!("Session not found: {}", id)))
    }
}


