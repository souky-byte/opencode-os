use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};
use tracing::{debug, error};
use utoipa::ToSchema;

use crate::error::AppError;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct OpenCodeModel {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct OpenCodeProvider {
    pub id: String,
    pub name: String,
    pub models: Vec<OpenCodeModel>,
}

#[derive(Debug, Serialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct ProvidersResponse {
    pub providers: Vec<OpenCodeProvider>,
}

#[utoipa::path(
    get,
    path = "/api/opencode/providers",
    responses(
        (status = 200, description = "List of connected providers with their models", body = ProvidersResponse),
        (status = 500, description = "Failed to fetch providers from OpenCode")
    ),
    tag = "opencode"
)]
pub async fn get_providers(State(state): State<AppState>) -> Result<Json<ProvidersResponse>, AppError> {
    debug!("Fetching providers from OpenCode");

    let project = state.project().await?;
    let opencode_config = project.task_executor.opencode_config();

    let response = opencode_client::apis::default_api::provider_list(opencode_config, None)
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to fetch providers from OpenCode");
            AppError::Internal(format!("Failed to fetch providers: {}", e))
        })?;

    let connected_set: std::collections::HashSet<&str> =
        response.connected.iter().map(|s| s.as_str()).collect();

    let providers: Vec<OpenCodeProvider> = response
        .all
        .into_iter()
        .filter(|p| connected_set.contains(p.id.as_str()))
        .map(|p| OpenCodeProvider {
            id: p.id.clone(),
            name: p.name.clone(),
            models: p
                .models
                .into_iter()
                .map(|(model_id, model_data)| OpenCodeModel {
                    id: model_id,
                    name: model_data.name,
                })
                .collect(),
        })
        .collect();

    debug!(provider_count = providers.len(), "Providers fetched successfully");

    Ok(Json(ProvidersResponse { providers }))
}
