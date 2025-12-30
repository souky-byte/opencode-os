use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

#[derive(Debug)]
#[allow(dead_code)]
pub enum AppError {
    NotFound(String),
    BadRequest(String),
    Conflict(String),
    Internal(String),
    Database(db::DbError),
    Vcs(vcs::VcsError),
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_type, message) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, "not_found", msg),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "bad_request", msg),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, "conflict", msg),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error", msg),
            AppError::Database(err) => {
                tracing::error!("Database error: {:?}", err);
                match err {
                    db::DbError::TaskNotFound(id) => (
                        StatusCode::NOT_FOUND,
                        "not_found",
                        format!("Task not found: {}", id),
                    ),
                    db::DbError::SessionNotFound(id) => (
                        StatusCode::NOT_FOUND,
                        "not_found",
                        format!("Session not found: {}", id),
                    ),
                    _ => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "database_error",
                        "Database error occurred".to_string(),
                    ),
                }
            }
            AppError::Vcs(err) => {
                tracing::error!("VCS error: {:?}", err);
                match err {
                    vcs::VcsError::WorkspaceNotFound(id) => (
                        StatusCode::NOT_FOUND,
                        "not_found",
                        format!("Workspace not found: {}", id),
                    ),
                    vcs::VcsError::WorkspaceAlreadyExists(id) => (
                        StatusCode::CONFLICT,
                        "conflict",
                        format!("Workspace already exists: {}", id),
                    ),
                    vcs::VcsError::MergeConflict(msg) => {
                        (StatusCode::CONFLICT, "merge_conflict", msg)
                    }
                    _ => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "vcs_error",
                        err.to_string(),
                    ),
                }
            }
        };

        let body = Json(ErrorResponse {
            error: error_type.to_string(),
            message,
        });

        (status, body).into_response()
    }
}

impl From<db::DbError> for AppError {
    fn from(err: db::DbError) -> Self {
        AppError::Database(err)
    }
}

impl From<vcs::VcsError> for AppError {
    fn from(err: vcs::VcsError) -> Self {
        AppError::Vcs(err)
    }
}
