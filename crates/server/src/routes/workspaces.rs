use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use vcs::{MergeResult, Workspace};

use crate::error::AppError;
use crate::state::AppState;

#[derive(Debug, Serialize)]
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

pub async fn create_workspace_for_task(
    State(state): State<AppState>,
    Path(task_id): Path<Uuid>,
) -> Result<(StatusCode, Json<WorkspaceResponse>), AppError> {
    let task = state.task_repository.find_by_id(task_id).await?;

    let Some(_task) = task else {
        return Err(AppError::NotFound(format!("Task not found: {}", task_id)));
    };

    let workspace = state
        .workspace_manager
        .setup_workspace(&task_id.to_string())
        .await?;

    Ok((StatusCode::CREATED, Json(workspace.into())))
}

pub async fn list_workspaces(
    State(state): State<AppState>,
) -> Result<Json<Vec<WorkspaceResponse>>, AppError> {
    let workspaces = state.workspace_manager.list_workspaces().await?;
    Ok(Json(workspaces.into_iter().map(Into::into).collect()))
}

#[derive(Debug, Serialize)]
pub struct DiffResponse {
    pub task_id: String,
    pub diff: String,
}

pub async fn get_workspace_diff(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<DiffResponse>, AppError> {
    let workspaces = state.workspace_manager.list_workspaces().await?;

    let workspace = workspaces
        .into_iter()
        .find(|ws| ws.task_id == task_id)
        .ok_or_else(|| AppError::NotFound(format!("Workspace not found: {}", task_id)))?;

    let diff = state.workspace_manager.get_diff(&workspace).await?;

    Ok(Json(DiffResponse {
        task_id: workspace.task_id,
        diff,
    }))
}

pub async fn get_workspace_status(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let workspaces = state.workspace_manager.list_workspaces().await?;

    let workspace = workspaces
        .into_iter()
        .find(|ws| ws.task_id == task_id)
        .ok_or_else(|| AppError::NotFound(format!("Workspace not found: {}", task_id)))?;

    let status = state.workspace_manager.get_status(&workspace).await?;

    Ok(Json(serde_json::json!({
        "task_id": workspace.task_id,
        "status": status
    })))
}

#[derive(Debug, Deserialize)]
pub struct MergeRequest {
    pub message: String,
}

#[derive(Debug, Serialize)]
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

pub async fn merge_workspace(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
    Json(payload): Json<MergeRequest>,
) -> Result<Json<MergeResponse>, AppError> {
    let workspaces = state.workspace_manager.list_workspaces().await?;

    let workspace = workspaces
        .into_iter()
        .find(|ws| ws.task_id == task_id)
        .ok_or_else(|| AppError::NotFound(format!("Workspace not found: {}", task_id)))?;

    let result = state
        .workspace_manager
        .merge_workspace(&workspace, &payload.message)
        .await?;

    Ok(Json(result.into()))
}

pub async fn delete_workspace(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<StatusCode, AppError> {
    let workspaces = state.workspace_manager.list_workspaces().await?;

    let workspace = workspaces
        .into_iter()
        .find(|ws| ws.task_id == task_id)
        .ok_or_else(|| AppError::NotFound(format!("Workspace not found: {}", task_id)))?;

    state
        .workspace_manager
        .cleanup_workspace(&workspace)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
