use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use db::DiffViewedRepository;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use vcs::{MergeResult, Workspace};

use crate::error::AppError;
use crate::state::AppState;

#[derive(Debug, Serialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct WorkspaceResponse {
    pub task_id: String,
    pub path: String,
    pub branch_name: String,
    pub status: String,
    pub created_at: String,
}

impl From<Workspace> for WorkspaceResponse {
    fn from(ws: Workspace) -> Self {
        Self {
            task_id: ws.task_id,
            path: ws.path.display().to_string(),
            branch_name: ws.branch_name,
            status: format!("{:?}", ws.status).to_lowercase(),
            created_at: ws.created_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WorkspaceStatusResponse {
    pub task_id: String,
    pub status: String,
}

#[utoipa::path(
    post,
    path = "/api/tasks/{task_id}/workspace",
    params(
        ("task_id" = Uuid, Path, description = "Task ID")
    ),
    responses(
        (status = 201, description = "Workspace created", body = WorkspaceResponse),
        (status = 404, description = "Task not found")
    ),
    tag = "workspaces"
)]
pub async fn create_workspace_for_task(
    State(state): State<AppState>,
    Path(task_id): Path<Uuid>,
) -> Result<(StatusCode, Json<WorkspaceResponse>), AppError> {
    let project = state.project().await?;
    let task = project.task_repository.find_by_id(task_id).await?;

    let Some(_task) = task else {
        return Err(AppError::NotFound(format!("Task not found: {}", task_id)));
    };

    let workspace = project
        .workspace_manager
        .setup_workspace(&task_id.to_string())
        .await?;

    Ok((StatusCode::CREATED, Json(workspace.into())))
}

#[utoipa::path(
    get,
    path = "/api/workspaces",
    responses(
        (status = 200, description = "List of all workspaces", body = Vec<WorkspaceResponse>)
    ),
    tag = "workspaces"
)]
pub async fn list_workspaces(
    State(state): State<AppState>,
) -> Result<Json<Vec<WorkspaceResponse>>, AppError> {
    let project = state.project().await?;
    let workspaces = project.workspace_manager.list_workspaces().await?;
    Ok(Json(workspaces.into_iter().map(Into::into).collect()))
}

#[derive(Debug, Serialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct DiffResponse {
    pub task_id: String,
    pub diff: String,
}

#[utoipa::path(
    get,
    path = "/api/workspaces/{task_id}/diff",
    params(
        ("task_id" = String, Path, description = "Task ID")
    ),
    responses(
        (status = 200, description = "Workspace diff", body = DiffResponse),
        (status = 404, description = "Workspace not found")
    ),
    tag = "workspaces"
)]
pub async fn get_workspace_diff(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<DiffResponse>, AppError> {
    let project = state.project().await?;
    let workspaces = project.workspace_manager.list_workspaces().await?;

    let workspace = workspaces
        .into_iter()
        .find(|ws| ws.task_id == task_id)
        .ok_or_else(|| AppError::NotFound(format!("Workspace not found: {}", task_id)))?;

    let diff = project.workspace_manager.get_diff(&workspace).await?;

    Ok(Json(DiffResponse {
        task_id: workspace.task_id,
        diff,
    }))
}

#[utoipa::path(
    get,
    path = "/api/workspaces/{task_id}",
    params(
        ("task_id" = String, Path, description = "Task ID")
    ),
    responses(
        (status = 200, description = "Workspace status", body = WorkspaceStatusResponse),
        (status = 404, description = "Workspace not found")
    ),
    tag = "workspaces"
)]
pub async fn get_workspace_status(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<WorkspaceStatusResponse>, AppError> {
    let project = state.project().await?;
    let workspaces = project.workspace_manager.list_workspaces().await?;

    let workspace = workspaces
        .into_iter()
        .find(|ws| ws.task_id == task_id)
        .ok_or_else(|| AppError::NotFound(format!("Workspace not found: {}", task_id)))?;

    let status = project.workspace_manager.get_status(&workspace).await?;

    Ok(Json(WorkspaceStatusResponse {
        task_id: workspace.task_id,
        status,
    }))
}

#[derive(Debug, Deserialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct MergeRequest {
    pub message: String,
}

#[derive(Debug, Serialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[serde(tag = "result", rename_all = "snake_case")]
pub enum MergeResponse {
    Success,
    Conflicts { files: Vec<String> },
}

impl From<MergeResult> for MergeResponse {
    fn from(result: MergeResult) -> Self {
        match result {
            MergeResult::Success => MergeResponse::Success,
            MergeResult::Conflicts { files } => MergeResponse::Conflicts {
                files: files
                    .into_iter()
                    .map(|f| f.path.display().to_string())
                    .collect(),
            },
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/workspaces/{task_id}/merge",
    params(
        ("task_id" = String, Path, description = "Task ID")
    ),
    request_body = MergeRequest,
    responses(
        (status = 200, description = "Merge result", body = MergeResponse),
        (status = 404, description = "Workspace not found")
    ),
    tag = "workspaces"
)]
pub async fn merge_workspace(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
    Json(payload): Json<MergeRequest>,
) -> Result<Json<MergeResponse>, AppError> {
    let project = state.project().await?;
    let workspaces = project.workspace_manager.list_workspaces().await?;

    let workspace = workspaces
        .into_iter()
        .find(|ws| ws.task_id == task_id)
        .ok_or_else(|| AppError::NotFound(format!("Workspace not found: {}", task_id)))?;

    let result = project
        .workspace_manager
        .merge_workspace(&workspace, &payload.message)
        .await?;

    Ok(Json(result.into()))
}

#[utoipa::path(
    delete,
    path = "/api/workspaces/{task_id}",
    params(
        ("task_id" = String, Path, description = "Task ID")
    ),
    responses(
        (status = 204, description = "Workspace deleted"),
        (status = 404, description = "Workspace not found")
    ),
    tag = "workspaces"
)]
pub async fn delete_workspace(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<StatusCode, AppError> {
    let project = state.project().await?;
    let workspaces = project.workspace_manager.list_workspaces().await?;

    let workspace = workspaces
        .into_iter()
        .find(|ws| ws.task_id == task_id)
        .ok_or_else(|| AppError::NotFound(format!("Workspace not found: {}", task_id)))?;

    project
        .workspace_manager
        .cleanup_workspace(&workspace)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Diff Viewed Files Endpoints
// ============================================================================

#[derive(Debug, Serialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct ViewedFilesResponse {
    pub viewed_files: Vec<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct SetViewedRequest {
    pub file_path: String,
    pub viewed: bool,
}

#[utoipa::path(
    get,
    path = "/api/tasks/{task_id}/diff/viewed",
    params(
        ("task_id" = String, Path, description = "Task ID")
    ),
    responses(
        (status = 200, description = "List of viewed files", body = ViewedFilesResponse)
    ),
    tag = "workspaces"
)]
pub async fn get_viewed_files(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<ViewedFilesResponse>, AppError> {
    let project = state.project().await?;
    let repo = DiffViewedRepository::new(project.pool.clone());

    let viewed_files = repo.get_viewed_files(&task_id).await?;

    Ok(Json(ViewedFilesResponse { viewed_files }))
}

#[utoipa::path(
    post,
    path = "/api/tasks/{task_id}/diff/viewed",
    params(
        ("task_id" = String, Path, description = "Task ID")
    ),
    request_body = SetViewedRequest,
    responses(
        (status = 204, description = "Viewed status updated")
    ),
    tag = "workspaces"
)]
pub async fn set_file_viewed(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
    Json(payload): Json<SetViewedRequest>,
) -> Result<StatusCode, AppError> {
    let project = state.project().await?;
    let repo = DiffViewedRepository::new(project.pool.clone());

    if payload.viewed {
        repo.mark_viewed(&task_id, &payload.file_path).await?;
    } else {
        repo.unmark_viewed(&task_id, &payload.file_path).await?;
    }

    Ok(StatusCode::NO_CONTENT)
}
