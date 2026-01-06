use axum::extract::State;
use axum::http::StatusCode;
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

    config.write(&project.project_path).await.map_err(|e| {
        error!(error = %e, "Failed to save config");
        AppError::Internal(format!("Failed to save settings: {}", e))
    })?;

    debug!("Phase model settings saved successfully");

    Ok(Json(PhaseModelsResponse {
        phase_models: config.phase_models,
    }))
}

// GitHub Token Settings

#[derive(Debug, Serialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct GitHubSettingsResponse {
    /// Whether a GitHub token is configured (true if token exists)
    pub has_token: bool,
    /// Masked token for display (e.g., "ghp_****abcd")
    pub masked_token: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct UpdateGitHubTokenRequest {
    /// The GitHub personal access token. Set to null or empty string to remove.
    pub token: Option<String>,
}

/// Mask a token for display, showing only prefix and last 4 chars
fn mask_token(token: &str) -> String {
    if token.len() <= 8 {
        return "****".to_string();
    }

    // Check for known prefixes
    let prefix_end =
        if token.starts_with("ghp_") || token.starts_with("gho_") || token.starts_with("ghu_") {
            4
        } else if token.starts_with("github_pat_") {
            11
        } else {
            0
        };

    let suffix_start = token.len().saturating_sub(4);

    if prefix_end > 0 {
        format!("{}****{}", &token[..prefix_end], &token[suffix_start..])
    } else {
        format!("****{}", &token[suffix_start..])
    }
}

#[utoipa::path(
    get,
    path = "/api/settings/github",
    responses(
        (status = 200, description = "GitHub settings", body = GitHubSettingsResponse),
    ),
    tag = "settings"
)]
pub async fn get_github_settings(State(state): State<AppState>) -> Json<GitHubSettingsResponse> {
    debug!("Reading GitHub settings");

    let token = state.global_config.get_github_token();

    Json(GitHubSettingsResponse {
        has_token: token.is_some(),
        masked_token: token.as_ref().map(|t| mask_token(t)),
    })
}

#[utoipa::path(
    put,
    path = "/api/settings/github",
    request_body = UpdateGitHubTokenRequest,
    responses(
        (status = 200, description = "GitHub token updated", body = GitHubSettingsResponse),
        (status = 500, description = "Failed to save settings")
    ),
    tag = "settings"
)]
pub async fn update_github_settings(
    State(state): State<AppState>,
    Json(payload): Json<UpdateGitHubTokenRequest>,
) -> Result<Json<GitHubSettingsResponse>, AppError> {
    info!("Updating GitHub token");

    // Normalize empty string to None
    let token = payload.token.filter(|t| !t.trim().is_empty());

    state
        .global_config
        .set_github_token(token.clone())
        .map_err(|e| {
            error!(error = %e, "Failed to save GitHub token");
            AppError::Internal(format!("Failed to save GitHub settings: {}", e))
        })?;

    debug!("GitHub token saved successfully");

    Ok(Json(GitHubSettingsResponse {
        has_token: token.is_some(),
        masked_token: token.as_ref().map(|t| mask_token(t)),
    }))
}

#[utoipa::path(
    delete,
    path = "/api/settings/github",
    responses(
        (status = 204, description = "GitHub token removed"),
        (status = 500, description = "Failed to remove token")
    ),
    tag = "settings"
)]
pub async fn delete_github_token(State(state): State<AppState>) -> Result<StatusCode, AppError> {
    info!("Removing GitHub token");

    state.global_config.set_github_token(None).map_err(|e| {
        error!(error = %e, "Failed to remove GitHub token");
        AppError::Internal(format!("Failed to remove GitHub token: {}", e))
    })?;

    debug!("GitHub token removed successfully");

    Ok(StatusCode::NO_CONTENT)
}
