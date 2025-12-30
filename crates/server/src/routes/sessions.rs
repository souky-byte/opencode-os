use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use opencode_core::Session;
use uuid::Uuid;

use crate::error::AppError;
use crate::state::AppState;

pub async fn list_sessions(State(state): State<AppState>) -> Result<Json<Vec<Session>>, AppError> {
    let sessions = state.session_repository.find_all().await?;
    Ok(Json(sessions))
}

pub async fn get_session(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Session>, AppError> {
    let session = state.session_repository.find_by_id(id).await?;

    match session {
        Some(s) => Ok(Json(s)),
        None => Err(AppError::NotFound(format!("Session not found: {}", id))),
    }
}

pub async fn list_sessions_for_task(
    State(state): State<AppState>,
    Path(task_id): Path<Uuid>,
) -> Result<Json<Vec<Session>>, AppError> {
    let sessions = state.session_repository.find_by_task_id(task_id).await?;
    Ok(Json(sessions))
}

pub async fn delete_session(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let deleted = state.session_repository.delete(id).await?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound(format!("Session not found: {}", id)))
    }
}
