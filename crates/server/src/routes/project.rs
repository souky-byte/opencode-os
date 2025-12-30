use axum::{extract::State, Json};
use serde::Serialize;
use utoipa::ToSchema;

use crate::state::AppState;

#[derive(Debug, Serialize, ToSchema)]
pub struct ProjectInfo {
    pub name: String,
    pub path: String,
    pub vcs: String,
    pub tasks_count: i64,
}

#[utoipa::path(
    get,
    path = "/api/project",
    responses(
        (status = 200, description = "Current project info", body = ProjectInfo)
    ),
    tag = "project"
)]
pub async fn get_project_info(State(state): State<AppState>) -> Json<ProjectInfo> {
    let name = state
        .repo_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let path = state.repo_path.display().to_string();

    let vcs = if state.repo_path.join(".jj").exists() {
        "jujutsu"
    } else if state.repo_path.join(".git").exists() {
        "git"
    } else {
        "none"
    }
    .to_string();

    let tasks_count = state
        .task_repository
        .find_all()
        .await
        .map(|t| t.len() as i64)
        .unwrap_or(0);

    Json(ProjectInfo {
        name,
        path,
        vcs,
        tasks_count,
    })
}
