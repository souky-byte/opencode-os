use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};
use utoipa::ToSchema;

use crate::config::{ModelSelection, PhaseModels, ProjectConfig};
use crate::error::AppError;
use crate::state::AppState;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct PhaseModelsResponse {
    pub phase_models: PhaseModels,
}

#[derive(Debug, Deserialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct UpdatePhaseModelsRequest {
    pub planning: Option<ModelSelection>,
    pub implementation: Option<ModelSelection>,
    pub review: Option<ModelSelection>,
    pub fix: Option<ModelSelection>,
}

#[utoipa::path(
    get,
    path = "/api/settings/models",
    responses(
        (status = 200, description = "Current phase model settings", body = PhaseModelsResponse),
        (status = 500, description = "Failed to read settings")
    ),
    tag = "settings"
)]
pub async fn get_phase_models(
    State(state): State<AppState>,
) -> Result<Json<PhaseModelsResponse>, AppError> {
    debug!("Reading phase model settings");

    let project = state.project().await?;
    let config = ProjectConfig::read(&project.project_path).await;

    Ok(Json(PhaseModelsResponse {
        phase_models: config.phase_models,
    }))
}

#[utoipa::path(
    put,
    path = "/api/settings/models",
    request_body = UpdatePhaseModelsRequest,
    responses(
        (status = 200, description = "Settings updated", body = PhaseModelsResponse),
        (status = 500, description = "Failed to save settings")
    ),
    tag = "settings"
)]
pub async fn update_phase_models(
    State(state): State<AppState>,
    Json(payload): Json<UpdatePhaseModelsRequest>,
) -> Result<Json<PhaseModelsResponse>, AppError> {
    info!("Updating phase model settings");

    let project = state.project().await?;
    let mut config = ProjectConfig::read(&project.project_path).await;

    config.phase_models = PhaseModels {
        planning: payload.planning,
        implementation: payload.implementation,
        review: payload.review,
        fix: payload.fix,
    };

    config
        .write(&project.project_path)
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to save config");
            AppError::Internal(format!("Failed to save settings: {}", e))
        })?;

    debug!("Phase model settings saved successfully");

    Ok(Json(PhaseModelsResponse {
        phase_models: config.phase_models,
    }))
}
