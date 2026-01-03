use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use utoipa::ToSchema;

use crate::error::AppError;
use crate::project_manager::{detect_vcs, ProjectErrorCode, ProjectInfo as ManagerProjectInfo};
use crate::state::AppState;

#[derive(Debug, Serialize, ToSchema)]
pub struct ProjectInfo {
    pub name: String,
    pub path: String,
    pub vcs: String,
    pub tasks_count: i64,
    pub initialized: bool,
}

impl From<ManagerProjectInfo> for ProjectInfo {
    fn from(info: ManagerProjectInfo) -> Self {
        Self {
            name: info.name,
            path: info.path,
            vcs: info.vcs,
            tasks_count: info.tasks_count,
            initialized: info.initialized,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct OpenProjectRequest {
    pub path: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct OpenProjectResponse {
    pub success: bool,
    pub project: Option<ProjectInfo>,
    pub was_initialized: bool,
    pub error: Option<ProjectErrorResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ProjectErrorResponse {
    pub code: String,
    pub message: String,
}

#[utoipa::path(
    post,
    path = "/api/projects/open",
    request_body = OpenProjectRequest,
    responses(
        (status = 200, description = "Project opened successfully", body = OpenProjectResponse),
        (status = 400, description = "Failed to open project", body = OpenProjectResponse)
    ),
    tag = "projects"
)]
pub async fn open_project(
    State(state): State<AppState>,
    Json(payload): Json<OpenProjectRequest>,
) -> Result<Json<OpenProjectResponse>, AppError> {
    let path = PathBuf::from(&payload.path);

    match state.project_manager.open(&path).await {
        Ok(result) => {
            state.global_config.add_recent(&path)?;
            state.global_config.set_last(&path)?;

            Ok(Json(OpenProjectResponse {
                success: true,
                project: Some(result.project.into()),
                was_initialized: result.was_initialized,
                error: None,
            }))
        }
        Err(e) => {
            let code = ProjectErrorCode::from(&e);
            Ok(Json(OpenProjectResponse {
                success: false,
                project: None,
                was_initialized: false,
                error: Some(ProjectErrorResponse {
                    code: format!("{:?}", code),
                    message: e.to_string(),
                }),
            }))
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct InitProjectRequest {
    pub path: String,
    #[serde(default)]
    pub force: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct InitProjectResponse {
    pub success: bool,
    pub project: Option<ProjectInfo>,
    pub already_initialized: bool,
    pub error: Option<ProjectErrorResponse>,
}

#[utoipa::path(
    post,
    path = "/api/projects/init",
    request_body = InitProjectRequest,
    responses(
        (status = 200, description = "Project initialized", body = InitProjectResponse),
        (status = 400, description = "Failed to initialize project", body = InitProjectResponse)
    ),
    tag = "projects"
)]
pub async fn init_project(
    State(state): State<AppState>,
    Json(payload): Json<InitProjectRequest>,
) -> Result<Json<InitProjectResponse>, AppError> {
    let path = PathBuf::from(&payload.path);

    match state.project_manager.init(&path, payload.force).await {
        Ok(result) => Ok(Json(InitProjectResponse {
            success: true,
            project: Some(result.project.into()),
            already_initialized: result.already_initialized,
            error: None,
        })),
        Err(e) => {
            let code = ProjectErrorCode::from(&e);
            Ok(Json(InitProjectResponse {
                success: false,
                project: None,
                already_initialized: false,
                error: Some(ProjectErrorResponse {
                    code: format!("{:?}", code),
                    message: e.to_string(),
                }),
            }))
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CurrentProjectResponse {
    pub project: Option<ProjectInfo>,
}

#[utoipa::path(
    get,
    path = "/api/projects/current",
    responses(
        (status = 200, description = "Current project info", body = CurrentProjectResponse)
    ),
    tag = "projects"
)]
pub async fn get_current_project(State(state): State<AppState>) -> Json<CurrentProjectResponse> {
    let project = match state.project().await {
        Ok(ctx) => {
            let info = ctx.info().await;
            Some(info.into())
        }
        Err(_) => None,
    };

    Json(CurrentProjectResponse { project })
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RecentProject {
    pub path: String,
    pub name: String,
    pub vcs: String,
    pub exists: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RecentProjectsResponse {
    pub projects: Vec<RecentProject>,
}

#[utoipa::path(
    get,
    path = "/api/projects/recent",
    responses(
        (status = 200, description = "List of recent projects", body = RecentProjectsResponse)
    ),
    tag = "projects"
)]
pub async fn get_recent_projects(State(state): State<AppState>) -> Json<RecentProjectsResponse> {
    let recent_paths = state.global_config.get_recent();

    // Only return projects that still exist on disk
    let projects: Vec<RecentProject> = recent_paths
        .into_iter()
        .filter(|path| path.exists() && path.is_dir())
        .map(|path| {
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();
            let vcs = detect_vcs(&path).to_string();

            RecentProject {
                path: path.display().to_string(),
                name,
                vcs,
                exists: true,
            }
        })
        .collect();

    Json(RecentProjectsResponse { projects })
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RemoveRecentRequest {
    pub path: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RemoveRecentResponse {
    pub success: bool,
}

#[utoipa::path(
    post,
    path = "/api/projects/recent/remove",
    request_body = RemoveRecentRequest,
    responses(
        (status = 200, description = "Project removed from recent list", body = RemoveRecentResponse)
    ),
    tag = "projects"
)]
pub async fn remove_recent_project(
    State(state): State<AppState>,
    Json(payload): Json<RemoveRecentRequest>,
) -> Result<Json<RemoveRecentResponse>, AppError> {
    state
        .global_config
        .remove_recent(&PathBuf::from(&payload.path))?;
    Ok(Json(RemoveRecentResponse { success: true }))
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ClearRecentResponse {
    pub success: bool,
}

#[utoipa::path(
    post,
    path = "/api/projects/recent/clear",
    responses(
        (status = 200, description = "Recent projects list cleared", body = ClearRecentResponse)
    ),
    tag = "projects"
)]
pub async fn clear_recent_projects(
    State(state): State<AppState>,
) -> Result<Json<ClearRecentResponse>, AppError> {
    state.global_config.clear_recent()?;
    Ok(Json(ClearRecentResponse { success: true }))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ValidatePathRequest {
    pub path: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ValidatePathResponse {
    pub valid: bool,
    pub exists: bool,
    pub is_vcs_repo: bool,
    pub vcs: Option<String>,
    pub name: Option<String>,
    pub error: Option<String>,
}

#[utoipa::path(
    post,
    path = "/api/projects/validate",
    request_body = ValidatePathRequest,
    responses(
        (status = 200, description = "Path validation result", body = ValidatePathResponse)
    ),
    tag = "projects"
)]
pub async fn validate_project_path(
    Json(payload): Json<ValidatePathRequest>,
) -> Json<ValidatePathResponse> {
    let path = PathBuf::from(&payload.path);

    if !path.exists() {
        return Json(ValidatePathResponse {
            valid: false,
            exists: false,
            is_vcs_repo: false,
            vcs: None,
            name: None,
            error: Some("Path does not exist".to_string()),
        });
    }

    if !path.is_dir() {
        return Json(ValidatePathResponse {
            valid: false,
            exists: true,
            is_vcs_repo: false,
            vcs: None,
            name: None,
            error: Some("Path is not a directory".to_string()),
        });
    }

    let vcs = detect_vcs(&path);
    let is_vcs_repo = vcs != "none";

    let name = path.file_name().and_then(|n| n.to_str()).map(String::from);

    Json(ValidatePathResponse {
        valid: is_vcs_repo,
        exists: true,
        is_vcs_repo,
        vcs: if is_vcs_repo {
            Some(vcs.to_string())
        } else {
            None
        },
        name,
        error: if !is_vcs_repo {
            Some("Not a git or jujutsu repository".to_string())
        } else {
            None
        },
    })
}
