pub mod error;
pub mod routes;
pub mod state;

use axum::routing::{get, post};
use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use state::AppState;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "OpenCode Studio API",
        version = "0.1.0",
        description = "API for OpenCode Studio - AI-powered development platform"
    ),
    paths(
        routes::health_check,
        routes::get_project_info,
        routes::list_tasks,
        routes::create_task,
        routes::get_task,
        routes::update_task,
        routes::delete_task,
        routes::transition_task,
        routes::execute_task,
        routes::list_sessions,
        routes::get_session,
        routes::list_sessions_for_task,
        routes::delete_session,
        routes::list_workspaces,
        routes::create_workspace_for_task,
        routes::get_workspace_status,
        routes::get_workspace_diff,
        routes::merge_workspace,
        routes::delete_workspace,
    ),
    components(schemas(
        routes::HealthResponse,
        routes::ProjectInfo,
        routes::TransitionRequest,
        routes::TransitionResponse,
        routes::ExecuteResponse,
        routes::PhaseResultDto,
        routes::WorkspaceResponse,
        routes::WorkspaceStatusResponse,
        routes::DiffResponse,
        routes::MergeRequest,
        routes::MergeResponse,
        opencode_core::Task,
        opencode_core::TaskStatus,
        opencode_core::CreateTaskRequest,
        opencode_core::UpdateTaskRequest,
        opencode_core::Session,
        opencode_core::SessionPhase,
        opencode_core::SessionStatus,
    )),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "project", description = "Project info endpoints"),
        (name = "tasks", description = "Task management endpoints"),
        (name = "sessions", description = "Session management endpoints"),
        (name = "workspaces", description = "Workspace management endpoints"),
    )
)]
pub struct ApiDoc;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api/openapi.json", ApiDoc::openapi()))
        .route("/health", get(routes::health_check))
        .route("/api/project", get(routes::get_project_info))
        .route(
            "/api/tasks",
            get(routes::list_tasks).post(routes::create_task),
        )
        .route(
            "/api/tasks/{id}",
            get(routes::get_task)
                .patch(routes::update_task)
                .delete(routes::delete_task),
        )
        .route("/api/tasks/{id}/transition", post(routes::transition_task))
        .route("/api/tasks/{id}/execute", post(routes::execute_task))
        .route(
            "/api/tasks/{id}/sessions",
            get(routes::list_sessions_for_task),
        )
        .route(
            "/api/tasks/{id}/workspace",
            post(routes::create_workspace_for_task),
        )
        .route("/api/sessions", get(routes::list_sessions))
        .route(
            "/api/sessions/{id}",
            get(routes::get_session).delete(routes::delete_session),
        )
        .route("/api/workspaces", get(routes::list_workspaces))
        .route(
            "/api/workspaces/{id}",
            get(routes::get_workspace_status).delete(routes::delete_workspace),
        )
        .route("/api/workspaces/{id}/diff", get(routes::get_workspace_diff))
        .route("/api/workspaces/{id}/merge", post(routes::merge_workspace))
        .route("/ws", get(routes::websocket_handler))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state)
}
