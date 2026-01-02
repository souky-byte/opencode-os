use axum::extract::State;
use axum::Json;
use serde::Serialize;
use utoipa::ToSchema;

use crate::routes::projects::CurrentProjectResponse;
use crate::state::AppState;

#[derive(Debug, Serialize, ToSchema)]
pub struct LegacyProjectInfo {
    pub name: String,
    pub path: String,
    pub vcs: String,
    pub tasks_count: i64,
    pub initialized: bool,
}

#[utoipa::path(
    get,
    path = "/api/project",
    responses(
        (status = 200, description = "Current project info (legacy)", body = CurrentProjectResponse)
    ),
    tag = "project"
)]
pub async fn get_project_info(State(state): State<AppState>) -> Json<CurrentProjectResponse> {
    let project = match state.project().await {
        Ok(ctx) => {
            let info = ctx.info().await;
            Some(crate::routes::projects::ProjectInfo::from(info))
        }
        Err(_) => None,
    };

    Json(CurrentProjectResponse { project })
}
