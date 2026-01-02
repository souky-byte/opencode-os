use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use db::ReviewCommentRepository;
use opencode_core::{TaskStatus, UpdateTaskRequest};
use orchestrator::UserReviewComment;
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::error::AppError;
use crate::state::AppState;

// ============================================================================
// Types
// ============================================================================

#[derive(Debug, Serialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct ReviewCommentResponse {
    pub id: String,
    pub task_id: String,
    pub file_path: String,
    pub line_start: i64,
    pub line_end: i64,
    pub side: String,
    pub content: String,
    pub status: String,
    pub created_at: i64,
}

impl From<db::ReviewComment> for ReviewCommentResponse {
    fn from(c: db::ReviewComment) -> Self {
        Self {
            id: c.id,
            task_id: c.task_id,
            file_path: c.file_path,
            line_start: c.line_start,
            line_end: c.line_end,
            side: c.side,
            content: c.content,
            status: c.status,
            created_at: c.created_at,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct CommentsListResponse {
    pub comments: Vec<ReviewCommentResponse>,
}

#[derive(Debug, Deserialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct CreateCommentRequest {
    pub file_path: String,
    pub line_start: i64,
    pub line_end: i64,
    #[serde(default = "default_side")]
    pub side: String,
    pub content: String,
}

fn default_side() -> String {
    "new".to_string()
}

#[derive(Debug, Deserialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct SendToFixRequest {
    pub comment_ids: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct SendToFixResponse {
    pub session_id: String,
    pub comments_count: usize,
}

// ============================================================================
// Handlers
// ============================================================================

#[utoipa::path(
    get,
    path = "/api/tasks/{task_id}/comments",
    params(
        ("task_id" = String, Path, description = "Task ID")
    ),
    responses(
        (status = 200, description = "List of comments", body = CommentsListResponse)
    ),
    tag = "comments"
)]
pub async fn list_comments(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<CommentsListResponse>, AppError> {
    let project = state.project().await?;
    let repo = ReviewCommentRepository::new(project.pool.clone());

    let comments = repo.find_by_task_id(&task_id).await?;

    Ok(Json(CommentsListResponse {
        comments: comments.into_iter().map(Into::into).collect(),
    }))
}

#[utoipa::path(
    post,
    path = "/api/tasks/{task_id}/comments",
    params(
        ("task_id" = String, Path, description = "Task ID")
    ),
    request_body = CreateCommentRequest,
    responses(
        (status = 201, description = "Comment created", body = ReviewCommentResponse)
    ),
    tag = "comments"
)]
pub async fn create_comment(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
    Json(payload): Json<CreateCommentRequest>,
) -> Result<(StatusCode, Json<ReviewCommentResponse>), AppError> {
    let project = state.project().await?;
    let repo = ReviewCommentRepository::new(project.pool.clone());

    let id = Uuid::new_v4().to_string();
    let comment = repo
        .create(
            &id,
            &task_id,
            &payload.file_path,
            payload.line_start,
            payload.line_end,
            &payload.side,
            &payload.content,
        )
        .await?;

    Ok((StatusCode::CREATED, Json(comment.into())))
}

#[utoipa::path(
    delete,
    path = "/api/tasks/{task_id}/comments/{comment_id}",
    params(
        ("task_id" = String, Path, description = "Task ID"),
        ("comment_id" = String, Path, description = "Comment ID")
    ),
    responses(
        (status = 204, description = "Comment deleted"),
        (status = 404, description = "Comment not found")
    ),
    tag = "comments"
)]
pub async fn delete_comment(
    State(state): State<AppState>,
    Path((task_id, comment_id)): Path<(String, String)>,
) -> Result<StatusCode, AppError> {
    let project = state.project().await?;
    let repo = ReviewCommentRepository::new(project.pool.clone());

    // Verify comment exists and belongs to this task
    let comment = repo.find_by_id(&comment_id).await?;
    match comment {
        Some(c) if c.task_id == task_id => {
            repo.delete(&comment_id).await?;
            Ok(StatusCode::NO_CONTENT)
        }
        Some(_) => Err(AppError::NotFound(format!(
            "Comment {} does not belong to task {}",
            comment_id, task_id
        ))),
        None => Err(AppError::NotFound(format!(
            "Comment not found: {}",
            comment_id
        ))),
    }
}

#[utoipa::path(
    post,
    path = "/api/tasks/{task_id}/comments/send-to-fix",
    params(
        ("task_id" = String, Path, description = "Task ID")
    ),
    request_body = SendToFixRequest,
    responses(
        (status = 202, description = "Fix session started", body = SendToFixResponse),
        (status = 400, description = "No comments selected")
    ),
    tag = "comments"
)]
pub async fn send_comments_to_fix(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
    Json(payload): Json<SendToFixRequest>,
) -> Result<(StatusCode, Json<SendToFixResponse>), AppError> {
    if payload.comment_ids.is_empty() {
        return Err(AppError::BadRequest(
            "No comments selected for fix".to_string(),
        ));
    }

    let project = state.project().await?;
    let repo = ReviewCommentRepository::new(project.pool.clone());

    // Get the selected comments
    let db_comments = repo.find_by_ids(&payload.comment_ids).await?;

    if db_comments.is_empty() {
        return Err(AppError::NotFound("No valid comments found".to_string()));
    }

    let task_uuid = uuid::Uuid::parse_str(&task_id)
        .map_err(|_| AppError::BadRequest(format!("Invalid task ID: {}", task_id)))?;

    // Get the task
    let mut task = project
        .task_repository
        .find_by_id(task_uuid)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Task not found: {}", task_id)))?;

    info!(
        task_id = %task_id,
        comment_count = db_comments.len(),
        "Starting fix session with user comments"
    );

    // Convert db comments to UserReviewComment for the prompt
    let user_comments: Vec<UserReviewComment> = db_comments
        .iter()
        .map(|c| UserReviewComment {
            file_path: c.file_path.clone(),
            line_start: c.line_start,
            line_end: c.line_end,
            side: c.side.clone(),
            content: c.content.clone(),
        })
        .collect();

    // Transition task to Fix state
    project
        .task_executor
        .transition(&mut task, TaskStatus::Fix)
        .map_err(|e| {
            error!(task_id = %task_id, error = %e, "Failed to transition to fix state");
            AppError::BadRequest(e.to_string())
        })?;

    // Start fix execution with user comments
    let started = project
        .task_executor
        .start_fix_with_comments(&task, &user_comments)
        .await
        .map_err(|e| {
            error!(task_id = %task_id, error = %e, "Failed to start fix session");
            AppError::Internal(e.to_string())
        })?;

    // Update task status in database
    let update = UpdateTaskRequest {
        status: Some(task.status),
        ..Default::default()
    };
    project.task_repository.update(task_uuid, &update).await?;

    // Mark comments as sent
    repo.update_status_bulk(&payload.comment_ids, "sent")
        .await?;

    info!(
        task_id = %task_id,
        session_id = %started.session_id,
        "Fix session started with {} user comments",
        db_comments.len()
    );

    Ok((
        StatusCode::ACCEPTED,
        Json(SendToFixResponse {
            session_id: started.session_id.to_string(),
            comments_count: db_comments.len(),
        }),
    ))
}
