use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use events::{Event, EventEnvelope};
use opencode_core::{CreateTaskRequest, Task, TaskStatus, UpdateTaskRequest};
use orchestrator::ReviewFinding;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, instrument, warn};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::error::AppError;
use crate::state::AppState;
use orchestrator::{parse_plan_phases, PhaseContext, PhaseSummary};

#[utoipa::path(
    get,
    path = "/api/tasks",
    responses(
        (status = 200, description = "List of all tasks", body = Vec<Task>)
    ),
    tag = "tasks"
)]
pub async fn list_tasks(State(state): State<AppState>) -> Result<Json<Vec<Task>>, AppError> {
    let project = state.project().await?;
    let tasks = project.task_repository.find_all().await?;
    Ok(Json(tasks))
}

#[utoipa::path(
    post,
    path = "/api/tasks",
    request_body = CreateTaskRequest,
    responses(
        (status = 201, description = "Task created", body = Task),
        (status = 400, description = "Invalid request")
    ),
    tag = "tasks"
)]
pub async fn create_task(
    State(state): State<AppState>,
    Json(payload): Json<CreateTaskRequest>,
) -> Result<(StatusCode, Json<Task>), AppError> {
    info!(
        title = %payload.title,
        has_description = !payload.description.is_empty(),
        "API: Creating new task"
    );

    if payload.title.trim().is_empty() {
        warn!("API: Task creation rejected - empty title");
        return Err(AppError::BadRequest("Title cannot be empty".to_string()));
    }

    let project = state.project().await?;
    let task = Task::new(payload.title.clone(), payload.description);
    let created = project.task_repository.create(&task).await?;

    info!(
        task_id = %created.id,
        title = %created.title,
        "API: Task created successfully"
    );

    state
        .event_bus
        .publish(EventEnvelope::new(Event::TaskCreated {
            task_id: created.id,
            title: payload.title,
        }));

    Ok((StatusCode::CREATED, Json(created)))
}

#[utoipa::path(
    get,
    path = "/api/tasks/{id}",
    params(
        ("id" = Uuid, Path, description = "Task ID")
    ),
    responses(
        (status = 200, description = "Task found", body = Task),
        (status = 404, description = "Task not found")
    ),
    tag = "tasks"
)]
pub async fn get_task(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Task>, AppError> {
    let project = state.project().await?;
    let task = project.task_repository.find_by_id(id).await?;

    match task {
        Some(t) => Ok(Json(t)),
        None => Err(AppError::NotFound(format!("Task not found: {}", id))),
    }
}

#[utoipa::path(
    patch,
    path = "/api/tasks/{id}",
    params(
        ("id" = Uuid, Path, description = "Task ID")
    ),
    request_body = UpdateTaskRequest,
    responses(
        (status = 200, description = "Task updated", body = Task),
        (status = 404, description = "Task not found")
    ),
    tag = "tasks"
)]
pub async fn update_task(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateTaskRequest>,
) -> Result<Json<Task>, AppError> {
    let project = state.project().await?;
    let updated = project.task_repository.update(id, &payload).await?;

    match updated {
        Some(t) => Ok(Json(t)),
        None => Err(AppError::NotFound(format!("Task not found: {}", id))),
    }
}

#[utoipa::path(
    delete,
    path = "/api/tasks/{id}",
    params(
        ("id" = Uuid, Path, description = "Task ID")
    ),
    responses(
        (status = 204, description = "Task deleted"),
        (status = 404, description = "Task not found")
    ),
    tag = "tasks"
)]
pub async fn delete_task(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let project = state.project().await?;
    let deleted = project.task_repository.delete(id).await?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound(format!("Task not found: {}", id)))
    }
}

#[derive(Debug, Deserialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct TransitionRequest {
    pub status: TaskStatus,
}

#[derive(Debug, Serialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct TransitionResponse {
    pub task: Task,
    pub previous_status: TaskStatus,
}

#[utoipa::path(
    post,
    path = "/api/tasks/{id}/transition",
    params(
        ("id" = Uuid, Path, description = "Task ID")
    ),
    request_body = TransitionRequest,
    responses(
        (status = 200, description = "Task transitioned", body = TransitionResponse),
        (status = 400, description = "Invalid transition"),
        (status = 404, description = "Task not found")
    ),
    tag = "tasks"
)]
#[instrument(skip(state), fields(task_id = %id))]
pub async fn transition_task(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<TransitionRequest>,
) -> Result<Json<TransitionResponse>, AppError> {
    info!(
        task_id = %id,
        target_status = %payload.status.as_str(),
        "API: Task transition requested"
    );

    let project = state.project().await?;
    let task = project.task_repository.find_by_id(id).await?;
    let Some(mut task) = task else {
        warn!(task_id = %id, "API: Task not found for transition");
        return Err(AppError::NotFound(format!("Task not found: {}", id)));
    };

    let previous_status = task.status;
    debug!(
        current_status = %previous_status.as_str(),
        target_status = %payload.status.as_str(),
        "Attempting state transition"
    );

    project
        .task_executor
        .transition(&mut task, payload.status)
        .map_err(|e| {
            warn!(
                task_id = %id,
                from = %previous_status.as_str(),
                to = %payload.status.as_str(),
                error = %e,
                "API: Task transition failed"
            );
            AppError::BadRequest(e.to_string())
        })?;

    let update = UpdateTaskRequest {
        status: Some(task.status),
        ..Default::default()
    };
    project.task_repository.update(id, &update).await?;

    info!(
        task_id = %id,
        from = %previous_status.as_str(),
        to = %task.status.as_str(),
        "API: Task transition completed"
    );

    // Note: TaskStatusChanged event is already emitted by task_executor.transition()

    Ok(Json(TransitionResponse {
        task,
        previous_status,
    }))
}

#[derive(Debug, Serialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct ExecuteResponse {
    pub task: Task,
    pub session_id: String,
    pub opencode_session_id: String,
    pub phase: String,
}

#[utoipa::path(
    post,
    path = "/api/tasks/{id}/execute",
    params(
        ("id" = Uuid, Path, description = "Task ID")
    ),
    responses(
        (status = 202, description = "Execution started", body = ExecuteResponse),
        (status = 404, description = "Task not found"),
        (status = 500, description = "Execution failed to start")
    ),
    tag = "tasks"
)]
#[instrument(skip(state), fields(task_id = %id))]
pub async fn execute_task(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<ExecuteResponse>), AppError> {
    info!(task_id = %id, "API: Task execution requested");

    let project = state.project().await?;
    let task = project.task_repository.find_by_id(id).await?;
    let Some(mut task) = task else {
        warn!(task_id = %id, "API: Task not found for execution");
        return Err(AppError::NotFound(format!("Task not found: {}", id)));
    };

    info!(
        task_id = %id,
        task_title = %task.title,
        current_status = %task.status.as_str(),
        "API: Starting task phase execution"
    );

    let started = project
        .task_executor
        .start_phase_async(&mut task)
        .await
        .map_err(|e| {
            error!(
                task_id = %id,
                error = %e,
                "API: Task execution failed to start"
            );
            AppError::Internal(e.to_string())
        })?;

    let update = UpdateTaskRequest {
        status: Some(task.status),
        ..Default::default()
    };
    project.task_repository.update(id, &update).await?;

    info!(
        task_id = %id,
        session_id = %started.session_id,
        opencode_session_id = %started.opencode_session_id,
        phase = %started.phase.as_str(),
        "API: Execution started"
    );

    Ok((
        StatusCode::ACCEPTED,
        Json(ExecuteResponse {
            task,
            session_id: started.session_id.to_string(),
            opencode_session_id: started.opencode_session_id,
            phase: started.phase.as_str().to_string(),
        }),
    ))
}

#[derive(Debug, Serialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct PlanResponse {
    pub content: String,
    pub exists: bool,
}

#[utoipa::path(
    get,
    path = "/api/tasks/{id}/plan",
    params(
        ("id" = Uuid, Path, description = "Task ID")
    ),
    responses(
        (status = 200, description = "Plan content", body = PlanResponse),
        (status = 404, description = "Task not found")
    ),
    tag = "tasks"
)]
pub async fn get_task_plan(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<PlanResponse>, AppError> {
    let project = state.project().await?;

    // Verify task exists
    let task = project.task_repository.find_by_id(id).await?;
    if task.is_none() {
        return Err(AppError::NotFound(format!("Task not found: {}", id)));
    }

    let file_manager = project.task_executor.file_manager();
    if file_manager.plan_exists(id).await {
        match file_manager.read_plan(id).await {
            Ok(content) => Ok(Json(PlanResponse {
                content,
                exists: true,
            })),
            Err(e) => {
                error!(task_id = %id, error = %e, "Failed to read plan file");
                Ok(Json(PlanResponse {
                    content: String::new(),
                    exists: false,
                }))
            }
        }
    } else {
        Ok(Json(PlanResponse {
            content: String::new(),
            exists: false,
        }))
    }
}

// ============================================================================
// Findings API
// ============================================================================

#[derive(Debug, Serialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct FindingsResponse {
    pub findings: Vec<ReviewFinding>,
    pub summary: String,
    pub approved: bool,
    pub exists: bool,
}

#[utoipa::path(
    get,
    path = "/api/tasks/{id}/findings",
    params(
        ("id" = Uuid, Path, description = "Task ID")
    ),
    responses(
        (status = 200, description = "Task findings", body = FindingsResponse),
        (status = 404, description = "Task not found")
    ),
    tag = "tasks"
)]
pub async fn get_task_findings(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<FindingsResponse>, AppError> {
    let project = state.project().await?;

    // Verify task exists
    let task = project.task_repository.find_by_id(id).await?;
    if task.is_none() {
        return Err(AppError::NotFound(format!("Task not found: {}", id)));
    }

    let file_manager = project.task_executor.file_manager();
    match file_manager.read_findings(id).await {
        Ok(Some(findings)) => Ok(Json(FindingsResponse {
            findings: findings.findings,
            summary: findings.summary,
            approved: findings.approved,
            exists: true,
        })),
        Ok(None) => Ok(Json(FindingsResponse {
            findings: vec![],
            summary: String::new(),
            approved: false,
            exists: false,
        })),
        Err(e) => {
            error!(task_id = %id, error = %e, "Failed to read findings file");
            Ok(Json(FindingsResponse {
                findings: vec![],
                summary: String::new(),
                approved: false,
                exists: false,
            }))
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct FixFindingsRequest {
    /// IDs of findings to fix, or empty to fix all
    pub finding_ids: Option<Vec<String>>,
    /// If true, fix all findings regardless of finding_ids
    pub fix_all: Option<bool>,
}

#[utoipa::path(
    post,
    path = "/api/tasks/{id}/findings/fix",
    params(
        ("id" = Uuid, Path, description = "Task ID")
    ),
    request_body = FixFindingsRequest,
    responses(
        (status = 202, description = "Fix started", body = ExecuteResponse),
        (status = 404, description = "Task not found"),
        (status = 400, description = "Invalid request")
    ),
    tag = "tasks"
)]
#[instrument(skip(state), fields(task_id = %id))]
pub async fn fix_findings(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<FixFindingsRequest>,
) -> Result<(StatusCode, Json<ExecuteResponse>), AppError> {
    info!(task_id = %id, "API: Fix findings requested");

    let project = state.project().await?;
    let task = project.task_repository.find_by_id(id).await?;
    let Some(mut task) = task else {
        return Err(AppError::NotFound(format!("Task not found: {}", id)));
    };

    // Verify task is in ai_review state
    if task.status != TaskStatus::AiReview {
        return Err(AppError::BadRequest(format!(
            "Task must be in ai_review state to fix findings. Current: {}",
            task.status.as_str()
        )));
    }

    // Read current findings
    let file_manager = project.task_executor.file_manager();
    let findings_data = file_manager.read_findings(id).await.map_err(|e| {
        error!(task_id = %id, error = %e, "Failed to read findings");
        AppError::Internal(e.to_string())
    })?;

    let Some(findings_data) = findings_data else {
        return Err(AppError::NotFound("No findings found for this task".to_string()));
    };

    // Determine which findings to fix
    let findings_to_fix: Vec<&ReviewFinding> = if payload.fix_all.unwrap_or(false) {
        findings_data
            .findings
            .iter()
            .filter(|f| f.status == orchestrator::FindingStatus::Pending)
            .collect()
    } else if let Some(ref ids) = payload.finding_ids {
        findings_data
            .findings
            .iter()
            .filter(|f| ids.contains(&f.id) && f.status == orchestrator::FindingStatus::Pending)
            .collect()
    } else {
        return Err(AppError::BadRequest(
            "Either finding_ids or fix_all must be provided".to_string(),
        ));
    };

    if findings_to_fix.is_empty() {
        return Err(AppError::BadRequest("No pending findings to fix".to_string()));
    }

    info!(
        task_id = %id,
        finding_count = findings_to_fix.len(),
        "API: Fixing selected findings"
    );

    // Transition task to Fix state
    project
        .task_executor
        .transition(&mut task, TaskStatus::Fix)
        .map_err(|e| {
            error!(task_id = %id, error = %e, "Failed to transition to fix state");
            AppError::BadRequest(e.to_string())
        })?;

    // Start fix execution (this will run fix phase with MCP)
    let started = project
        .task_executor
        .start_phase_async(&mut task)
        .await
        .map_err(|e| {
            error!(
                task_id = %id,
                error = %e,
                "API: Fix execution failed to start"
            );
            AppError::Internal(e.to_string())
        })?;

    let update = UpdateTaskRequest {
        status: Some(task.status),
        ..Default::default()
    };
    project.task_repository.update(id, &update).await?;

    info!(
        task_id = %id,
        session_id = %started.session_id,
        "API: Fix execution started"
    );

    Ok((
        StatusCode::ACCEPTED,
        Json(ExecuteResponse {
            task,
            session_id: started.session_id.to_string(),
            opencode_session_id: started.opencode_session_id,
            phase: started.phase.as_str().to_string(),
        }),
    ))
}

#[utoipa::path(
    post,
    path = "/api/tasks/{id}/findings/skip",
    params(
        ("id" = Uuid, Path, description = "Task ID")
    ),
    responses(
        (status = 200, description = "Findings skipped, task moved to review", body = Task),
        (status = 404, description = "Task not found"),
        (status = 400, description = "Invalid state")
    ),
    tag = "tasks"
)]
#[instrument(skip(state), fields(task_id = %id))]
pub async fn skip_findings(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Task>, AppError> {
    info!(task_id = %id, "API: Skip findings requested");

    let project = state.project().await?;
    let task = project.task_repository.find_by_id(id).await?;
    let Some(mut task) = task else {
        return Err(AppError::NotFound(format!("Task not found: {}", id)));
    };

    // Verify task is in ai_review state
    if task.status != TaskStatus::AiReview {
        return Err(AppError::BadRequest(format!(
            "Task must be in ai_review state to skip findings. Current: {}",
            task.status.as_str()
        )));
    }

    // Mark all pending findings as skipped
    let file_manager = project.task_executor.file_manager();
    if let Err(e) = file_manager.skip_all_findings(id).await {
        warn!(task_id = %id, error = %e, "Failed to update findings status (continuing anyway)");
    }

    // Transition to review state
    project
        .task_executor
        .transition(&mut task, TaskStatus::Review)
        .map_err(|e| {
            error!(task_id = %id, error = %e, "Failed to transition to review");
            AppError::Internal(e.to_string())
        })?;

    let update = UpdateTaskRequest {
        status: Some(task.status),
        ..Default::default()
    };
    project.task_repository.update(id, &update).await?;

    info!(task_id = %id, "API: Findings skipped, task moved to review");

    Ok(Json(task))
}

// ============================================================================
// Phases API
// ============================================================================

/// Phase status for display
#[derive(Debug, Clone, Serialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[serde(rename_all = "snake_case")]
pub enum PhaseStatus {
    Pending,
    Running,
    Completed,
}

/// Information about a single implementation phase
#[derive(Debug, Clone, Serialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct PhaseInfo {
    /// Phase number (1-indexed)
    pub number: u32,
    /// Phase title
    pub title: String,
    /// Phase content from the plan
    pub content: String,
    /// Current status of this phase
    pub status: PhaseStatus,
    /// Associated session ID (if started)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// Summary of completed phase
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<PhaseSummary>,
}

/// Response for phases endpoint
#[derive(Debug, Serialize, ToSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct PhasesResponse {
    /// Whether this task has multiple phases
    pub is_multi_phase: bool,
    /// Total number of phases
    pub total_phases: u32,
    /// Current phase being executed (1-indexed), None if not started or completed
    pub current_phase: Option<u32>,
    /// List of all phases with their status
    pub phases: Vec<PhaseInfo>,
}

#[utoipa::path(
    get,
    path = "/api/tasks/{id}/phases",
    params(
        ("id" = Uuid, Path, description = "Task ID")
    ),
    responses(
        (status = 200, description = "Phases information", body = PhasesResponse),
        (status = 404, description = "Task not found")
    ),
    tag = "tasks"
)]
pub async fn get_task_phases(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<PhasesResponse>, AppError> {
    let project = state.project().await?;

    // Verify task exists
    let task = project.task_repository.find_by_id(id).await?;
    if task.is_none() {
        return Err(AppError::NotFound(format!("Task not found: {}", id)));
    }

    let file_manager = project.task_executor.file_manager();

    // Check if plan exists
    if !file_manager.plan_exists(id).await {
        return Ok(Json(PhasesResponse {
            is_multi_phase: false,
            total_phases: 0,
            current_phase: None,
            phases: vec![],
        }));
    }

    // Read and parse the plan
    let plan_content = match file_manager.read_plan(id).await {
        Ok(content) => content,
        Err(e) => {
            warn!(task_id = %id, error = %e, "Failed to read plan");
            return Ok(Json(PhasesResponse {
                is_multi_phase: false,
                total_phases: 0,
                current_phase: None,
                phases: vec![],
            }));
        }
    };

    let parsed_plan = parse_plan_phases(&plan_content);

    // Read phase context if it exists
    let phase_context: Option<PhaseContext> = file_manager.read_phase_context(id).await.ok().flatten();

    // Get sessions for this task to determine which phases have sessions
    let sessions = project.session_repository.find_by_task_id(id).await.unwrap_or_default();

    // Build session lookup by phase number
    let session_by_phase: std::collections::HashMap<u32, &opencode_core::Session> = sessions
        .iter()
        .filter_map(|s| s.implementation_phase_number.map(|n| (n, s)))
        .collect();

    // Determine current phase
    let current_phase = if let Some(ref ctx) = phase_context {
        if ctx.is_complete() {
            None
        } else {
            Some(ctx.phase_number)
        }
    } else if !parsed_plan.is_single_phase() {
        // Multi-phase plan but no context yet - not started
        None
    } else {
        // Single-phase plan - check if there's a running session
        let running_session = sessions.iter().find(|s| s.status == opencode_core::SessionStatus::Running);
        if running_session.is_some() {
            Some(1)
        } else {
            None
        }
    };

    // Build phase info list
    let phases: Vec<PhaseInfo> = parsed_plan
        .phases
        .iter()
        .map(|phase| {
            let session = session_by_phase.get(&phase.number);

            // Determine phase status
            let status = if let Some(ref ctx) = phase_context {
                if phase.number < ctx.phase_number {
                    PhaseStatus::Completed
                } else if phase.number == ctx.phase_number {
                    // Check if there's a running session
                    if session.map(|s| s.status == opencode_core::SessionStatus::Running).unwrap_or(false) {
                        PhaseStatus::Running
                    } else {
                        PhaseStatus::Pending
                    }
                } else {
                    PhaseStatus::Pending
                }
            } else {
                // No phase context - check session status
                match session {
                    Some(s) if s.status == opencode_core::SessionStatus::Running => PhaseStatus::Running,
                    Some(s) if s.status == opencode_core::SessionStatus::Completed => PhaseStatus::Completed,
                    _ => PhaseStatus::Pending,
                }
            };

            // Get summary for completed phases
            let summary = phase_context
                .as_ref()
                .and_then(|ctx| {
                    ctx.completed_phases
                        .iter()
                        .find(|s| s.phase_number == phase.number)
                        .cloned()
                });

            PhaseInfo {
                number: phase.number,
                title: phase.title.clone(),
                content: phase.content.clone(),
                status,
                session_id: session.map(|s| s.id.to_string()),
                summary,
            }
        })
        .collect();

    Ok(Json(PhasesResponse {
        is_multi_phase: !parsed_plan.is_single_phase(),
        total_phases: parsed_plan.total_phases(),
        current_phase,
        phases,
    }))
}
