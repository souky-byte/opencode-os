use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use events::{Event, EventEnvelope};
use opencode_client::apis::configuration::Configuration;
use opencode_core::{
    GenerateRoadmapRequest, Roadmap, RoadmapFeature, RoadmapGenerationStatus, RoadmapStats,
    UpdateFeatureRequest,
};
use orchestrator::services::RoadmapService;
use serde::{Deserialize, Serialize};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tracing::{error, info};
use utoipa::ToSchema;

use crate::config::{ProjectConfig, RoadmapConfig};
use crate::error::AppError;
use crate::state::AppState;

#[derive(Debug, Serialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct RoadmapResponse {
    pub roadmap: Option<Roadmap>,
    pub exists: bool,
    pub stats: Option<RoadmapStats>,
}

#[utoipa::path(
    get,
    path = "/api/roadmap",
    responses(
        (status = 200, description = "Current roadmap", body = RoadmapResponse)
    ),
    tag = "roadmap"
)]
pub async fn get_roadmap(State(state): State<AppState>) -> Result<Json<RoadmapResponse>, AppError> {
    let project = state.project().await?;
    let project_config = ProjectConfig::read(&project.path).await;
    let service = create_roadmap_service(&state, &project.path, &project_config);

    match service.load().await {
        Ok(Some(roadmap)) => {
            let stats = roadmap.stats();
            Ok(Json(RoadmapResponse {
                roadmap: Some(roadmap),
                exists: true,
                stats: Some(stats),
            }))
        }
        Ok(None) => Ok(Json(RoadmapResponse {
            roadmap: None,
            exists: false,
            stats: None,
        })),
        Err(e) => {
            error!(error = %e, "Failed to load roadmap");
            Err(AppError::Internal(e.to_string()))
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/roadmap/generate",
    request_body = GenerateRoadmapRequest,
    responses(
        (status = 202, description = "Generation started"),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Generation failed")
    ),
    tag = "roadmap"
)]
pub async fn generate_roadmap(
    State(state): State<AppState>,
    Json(payload): Json<GenerateRoadmapRequest>,
) -> Result<StatusCode, AppError> {
    let project = state.project().await?;
    let project_config = ProjectConfig::read(&project.path).await;
    let service = create_roadmap_service(&state, &project.path, &project_config);

    // Increment generation ID to cancel any previous generation
    let generation_id = state
        .roadmap_generation_id
        .fetch_add(1, Ordering::SeqCst)
        + 1;

    info!(
        project_path = %project.path.display(),
        force = payload.force,
        generation_id = generation_id,
        "Starting roadmap generation"
    );

    // Reset status to idle first to ensure clean state
    {
        let mut status = state.roadmap_status.write().await;
        info!(
            old_phase = ?status.phase,
            old_progress = status.progress,
            "Resetting roadmap status to idle"
        );
        *status = RoadmapGenerationStatus::idle();
    }

    // Verify reset worked
    {
        let status = state.roadmap_status.read().await;
        info!(
            new_phase = ?status.phase,
            new_progress = status.progress,
            "Status after reset"
        );
    }

    // Publish progress event with reset status so frontend updates immediately
    state.event_bus.publish(EventEnvelope::new(
        Event::RoadmapGenerationProgress {
            phase: "idle".to_string(),
            progress: 0,
            message: "Starting...".to_string(),
        },
    ));

    state
        .event_bus
        .publish(EventEnvelope::new(Event::RoadmapGenerationStarted));

    // Configure service with generation ID for cancellation support
    let service = service.with_generation_id(generation_id, state.roadmap_generation_id.clone());

    let event_bus = state.event_bus.clone();
    let roadmap_status = state.roadmap_status.clone();

    tokio::spawn(async move {
        match service.generate(payload.force).await {
            Ok(roadmap) => {
                info!(
                    features = roadmap.features.len(),
                    phases = roadmap.phases.len(),
                    "Roadmap generation completed"
                );

                event_bus.publish(EventEnvelope::new(Event::RoadmapGenerationCompleted {
                    feature_count: roadmap.features.len(),
                    phase_count: roadmap.phases.len(),
                }));
            }
            Err(e) => {
                error!(error = %e, "Roadmap generation failed");

                // Ensure status is set to error state
                *roadmap_status.write().await = RoadmapGenerationStatus::error(e.to_string());

                event_bus.publish(EventEnvelope::new(Event::RoadmapGenerationFailed {
                    error: e.to_string(),
                }));
            }
        }
    });

    Ok(StatusCode::ACCEPTED)
}

#[utoipa::path(
    get,
    path = "/api/roadmap/status",
    responses(
        (status = 200, description = "Generation status", body = RoadmapGenerationStatus)
    ),
    tag = "roadmap"
)]
pub async fn get_generation_status(
    State(state): State<AppState>,
) -> Result<Json<RoadmapGenerationStatus>, AppError> {
    let status = state.roadmap_status.read().await.clone();
    tracing::debug!(
        phase = ?status.phase,
        progress = status.progress,
        "Returning generation status"
    );
    Ok(Json(status))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct FeaturePathParams {
    pub feature_id: String,
}

#[utoipa::path(
    get,
    path = "/api/roadmap/features/{feature_id}",
    params(
        ("feature_id" = String, Path, description = "Feature ID")
    ),
    responses(
        (status = 200, description = "Feature details", body = RoadmapFeature),
        (status = 404, description = "Feature not found")
    ),
    tag = "roadmap"
)]
pub async fn get_feature(
    State(state): State<AppState>,
    axum::extract::Path(feature_id): axum::extract::Path<String>,
) -> Result<Json<RoadmapFeature>, AppError> {
    let project = state.project().await?;
    let project_config = ProjectConfig::read(&project.path).await;
    let service = create_roadmap_service(&state, &project.path, &project_config);

    let roadmap = service
        .load()
        .await?
        .ok_or_else(|| AppError::NotFound("Roadmap not found".to_string()))?;

    roadmap
        .feature_by_id(&feature_id)
        .cloned()
        .map(Json)
        .ok_or_else(|| AppError::NotFound(format!("Feature {} not found", feature_id)))
}

#[utoipa::path(
    patch,
    path = "/api/roadmap/features/{feature_id}",
    params(
        ("feature_id" = String, Path, description = "Feature ID")
    ),
    request_body = UpdateFeatureRequest,
    responses(
        (status = 200, description = "Feature updated", body = RoadmapFeature),
        (status = 404, description = "Feature not found")
    ),
    tag = "roadmap"
)]
pub async fn update_feature(
    State(state): State<AppState>,
    axum::extract::Path(feature_id): axum::extract::Path<String>,
    Json(payload): Json<UpdateFeatureRequest>,
) -> Result<Json<RoadmapFeature>, AppError> {
    let project = state.project().await?;
    let project_config = ProjectConfig::read(&project.path).await;
    let service = create_roadmap_service(&state, &project.path, &project_config);

    let updated = service
        .store()
        .update_feature(&feature_id, &payload)
        .await
        .map_err(|e| match e {
            orchestrator::OrchestratorError::NotFound(msg) => AppError::NotFound(msg),
            _ => AppError::Internal(e.to_string()),
        })?;

    info!(feature_id = %feature_id, "Feature updated");

    state
        .event_bus
        .publish(EventEnvelope::new(Event::RoadmapFeatureUpdated {
            feature_id: feature_id.clone(),
            status: payload.status.map(|s| s.as_str().to_string()),
        }));

    Ok(Json(updated))
}

#[derive(Debug, Serialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct ConvertToTaskResponse {
    pub task_id: uuid::Uuid,
    pub feature_id: String,
}

#[utoipa::path(
    post,
    path = "/api/roadmap/features/{feature_id}/convert-to-task",
    params(
        ("feature_id" = String, Path, description = "Feature ID")
    ),
    responses(
        (status = 201, description = "Task created from feature", body = ConvertToTaskResponse),
        (status = 404, description = "Feature not found"),
        (status = 400, description = "Feature already linked to a task")
    ),
    tag = "roadmap"
)]
pub async fn convert_feature_to_task(
    State(state): State<AppState>,
    axum::extract::Path(feature_id): axum::extract::Path<String>,
) -> Result<(StatusCode, Json<ConvertToTaskResponse>), AppError> {
    let project = state.project().await?;
    let project_config = ProjectConfig::read(&project.path).await;
    let service = create_roadmap_service(&state, &project.path, &project_config);

    let roadmap = service
        .load()
        .await?
        .ok_or_else(|| AppError::NotFound("Roadmap not found".to_string()))?;

    let feature = roadmap
        .feature_by_id(&feature_id)
        .ok_or_else(|| AppError::NotFound(format!("Feature {} not found", feature_id)))?;

    if feature.linked_task_id.is_some() {
        return Err(AppError::BadRequest(format!(
            "Feature {} is already linked to task {}",
            feature_id,
            feature.linked_task_id.as_ref().unwrap()
        )));
    }

    let description = format!(
        "{}\n\n## Rationale\n{}\n\n## Acceptance Criteria\n{}\n\n## User Stories\n{}",
        feature.description,
        feature.rationale,
        feature
            .acceptance_criteria
            .iter()
            .map(|c| format!("- {}", c))
            .collect::<Vec<_>>()
            .join("\n"),
        feature
            .user_stories
            .iter()
            .map(|s| format!("- {}", s))
            .collect::<Vec<_>>()
            .join("\n")
    );

    let task = opencode_core::Task::new(feature.title.clone(), description);
    let created = project.task_repository.create(&task).await?;

    service
        .store()
        .link_feature_to_task(&feature_id, &created.id.to_string())
        .await?;

    info!(
        feature_id = %feature_id,
        task_id = %created.id,
        "Feature converted to task"
    );

    state
        .event_bus
        .publish(EventEnvelope::new(Event::TaskCreated {
            task_id: created.id,
            title: feature.title.clone(),
        }));

    state
        .event_bus
        .publish(EventEnvelope::new(Event::RoadmapFeatureConverted {
            feature_id: feature_id.clone(),
            task_id: created.id,
        }));

    Ok((
        StatusCode::CREATED,
        Json(ConvertToTaskResponse {
            task_id: created.id,
            feature_id,
        }),
    ))
}

#[utoipa::path(
    delete,
    path = "/api/roadmap",
    responses(
        (status = 204, description = "Roadmap deleted"),
        (status = 404, description = "Roadmap not found")
    ),
    tag = "roadmap"
)]
pub async fn delete_roadmap(State(state): State<AppState>) -> Result<StatusCode, AppError> {
    let project = state.project().await?;
    let project_config = ProjectConfig::read(&project.path).await;
    let service = create_roadmap_service(&state, &project.path, &project_config);

    if !service.exists().await {
        return Err(AppError::NotFound("Roadmap not found".to_string()));
    }

    service.delete().await?;
    *state.roadmap_status.write().await = RoadmapGenerationStatus::idle();

    info!("Roadmap deleted");

    Ok(StatusCode::NO_CONTENT)
}

fn create_roadmap_service(
    state: &AppState,
    project_path: &std::path::Path,
    project_config: &ProjectConfig,
) -> RoadmapService {
    let config = Arc::new(Configuration {
        base_path: state.opencode_url.clone(),
        ..Default::default()
    });

    let mut service = RoadmapService::new(config, project_path, state.roadmap_status.clone())
        .with_event_bus(state.event_bus.clone());

    if let Some(model) = &project_config.roadmap.model {
        service = service.with_model(&model.provider_id, &model.model_id);
    }

    service
}

#[derive(Debug, Serialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct RoadmapSettingsResponse {
    pub config: RoadmapConfig,
}

#[utoipa::path(
    get,
    path = "/api/settings/roadmap",
    responses(
        (status = 200, description = "Roadmap settings", body = RoadmapSettingsResponse)
    ),
    tag = "settings"
)]
pub async fn get_roadmap_settings(
    State(state): State<AppState>,
) -> Result<Json<RoadmapSettingsResponse>, AppError> {
    let project = state.project().await?;
    let config = ProjectConfig::read(&project.path).await;

    Ok(Json(RoadmapSettingsResponse {
        config: config.roadmap,
    }))
}

#[derive(Debug, Deserialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct UpdateRoadmapSettingsRequest {
    pub config: RoadmapConfig,
}

#[utoipa::path(
    put,
    path = "/api/settings/roadmap",
    request_body = UpdateRoadmapSettingsRequest,
    responses(
        (status = 200, description = "Roadmap settings updated", body = RoadmapSettingsResponse)
    ),
    tag = "settings"
)]
pub async fn update_roadmap_settings(
    State(state): State<AppState>,
    Json(payload): Json<UpdateRoadmapSettingsRequest>,
) -> Result<Json<RoadmapSettingsResponse>, AppError> {
    let project = state.project().await?;
    let mut config = ProjectConfig::read(&project.path).await;

    config.roadmap = payload.config;

    config
        .write(&project.path)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to save config: {}", e)))?;

    info!("Roadmap settings updated");

    Ok(Json(RoadmapSettingsResponse {
        config: config.roadmap,
    }))
}
