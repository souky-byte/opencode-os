pub mod config;
pub mod error;
pub mod opencode_manager;
pub mod project_manager;
pub mod routes;
pub mod state;

use axum::routing::{get, post};
use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};
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
        routes::project::get_project_info,
        routes::projects::open_project,
        routes::projects::init_project,
        routes::projects::get_current_project,
        routes::projects::get_recent_projects,
        routes::projects::remove_recent_project,
        routes::projects::clear_recent_projects,
        routes::projects::validate_project_path,
        routes::list_tasks,
        routes::create_task,
        routes::get_task,
        routes::update_task,
        routes::delete_task,
        routes::transition_task,
        routes::execute_task,
        routes::get_task_plan,
        routes::get_task_findings,
        routes::fix_findings,
        routes::skip_findings,
        routes::get_task_phases,
        routes::list_sessions,
        routes::get_session,
        routes::list_sessions_for_task,
        routes::delete_session,

        routes::sse::events_stream,
        routes::sse::session_activity_stream,
        routes::list_workspaces,
        routes::create_workspace_for_task,
        routes::get_workspace_status,
        routes::get_workspace_diff,
        routes::merge_workspace,
        routes::delete_workspace,
        routes::get_viewed_files,
        routes::set_file_viewed,
        routes::list_comments,
        routes::create_comment,
        routes::delete_comment,
        routes::send_comments_to_fix,
        routes::filesystem::browse_directory,
        routes::opencode::get_providers,
        routes::settings::get_phase_models,
        routes::settings::update_phase_models,
        routes::settings::get_github_settings,
        routes::settings::update_github_settings,
        routes::settings::delete_github_token,
        routes::complete::get_complete_preview,
        routes::complete::complete_task,
        routes::complete::get_user_mode,
        routes::complete::update_user_mode,
        routes::pull_requests::list_pull_requests,
        routes::pull_requests::get_pull_request,
        routes::pull_requests::get_pull_request_diff,
        routes::pull_requests::get_pull_request_files,
        routes::pull_requests::get_pull_request_comments,
        routes::pull_requests::create_review_comment,
        routes::pull_requests::reply_to_comment,
        routes::pull_requests::get_pull_request_reviews,
        routes::pull_requests::fix_from_pr_comments,
    ),
    components(schemas(
        routes::HealthResponse,
        routes::projects::ProjectInfo,
        routes::projects::OpenProjectRequest,
        routes::projects::OpenProjectResponse,
        routes::projects::InitProjectRequest,
        routes::projects::InitProjectResponse,
        routes::projects::CurrentProjectResponse,
        routes::projects::ProjectErrorResponse,
        routes::projects::RecentProject,
        routes::projects::RecentProjectsResponse,
        routes::projects::ValidatePathRequest,
        routes::projects::ValidatePathResponse,
        routes::projects::RemoveRecentRequest,
        routes::projects::RemoveRecentResponse,
        routes::projects::ClearRecentResponse,
        routes::TransitionRequest,
        routes::TransitionResponse,
        routes::ExecuteResponse,
        routes::PlanResponse,
        routes::FindingsResponse,
        routes::FixFindingsRequest,
        routes::PhasesResponse,
        routes::PhaseInfo,
        routes::PhaseStatus,
        routes::WorkspaceResponse,
        routes::WorkspaceStatusResponse,
        routes::DiffResponse,
        routes::MergeRequest,
        routes::MergeResponse,
        routes::ViewedFilesResponse,
        routes::SetViewedRequest,
        routes::ReviewCommentResponse,
        routes::CommentsListResponse,
        routes::CreateCommentRequest,
        routes::SendToFixRequest,
        routes::SendToFixResponse,
        routes::filesystem::BrowseQuery,
        routes::filesystem::BrowseResponse,
        routes::filesystem::DirectoryEntry,
        routes::opencode::OpenCodeModel,
        routes::opencode::OpenCodeProvider,
        routes::opencode::ProvidersResponse,
        routes::settings::PhaseModelsResponse,
        routes::settings::UpdatePhaseModelsRequest,
        routes::settings::GitHubSettingsResponse,
        routes::settings::UpdateGitHubTokenRequest,
        config::ModelSelection,
        config::PhaseModels,
        config::ProjectConfig,
        config::UserMode,
        routes::complete::CompletePreviewResponse,
        routes::complete::CompleteAction,
        routes::complete::PrOptions,
        routes::complete::MergeOptions,
        routes::complete::CompleteTaskRequest,
        routes::complete::CompleteTaskResponse,
        routes::complete::PrInfo,
        routes::complete::MergeResultInfo,
        routes::complete::UserModeResponse,
        routes::complete::UpdateUserModeRequest,
        routes::pull_requests::PullRequestsListResponse,
        routes::pull_requests::PrDiffResponse,
        routes::pull_requests::PrCommentsResponse,
        routes::pull_requests::PrReviewsResponse,
        routes::pull_requests::CreatePrCommentRequest,
        routes::pull_requests::ReplyToCommentRequest,
        routes::pull_requests::FixFromCommentsRequest,
        routes::pull_requests::FixFromCommentsResponse,
        vcs::DiffSummary,
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
        (name = "project", description = "Legacy project info endpoints"),
        (name = "projects", description = "Project management endpoints"),
        (name = "tasks", description = "Task management endpoints"),
        (name = "sessions", description = "Session management endpoints"),
        (name = "events", description = "Real-time event streaming (SSE)"),
        (name = "workspaces", description = "Workspace management endpoints"),
        (name = "comments", description = "Review comments endpoints"),
        (name = "filesystem", description = "Filesystem browsing endpoints"),
        (name = "opencode", description = "OpenCode integration endpoints"),
        (name = "settings", description = "Project settings endpoints"),
        (name = "complete", description = "Task completion and Git workflow endpoints"),
        (name = "pull-requests", description = "GitHub Pull Request management endpoints"),
    )
)]
pub struct ApiDoc;

pub fn create_router(state: AppState) -> Router {
    let app_dir = state.app_dir.clone();

    let api_router = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api/openapi.json", ApiDoc::openapi()))
        .route("/health", get(routes::health_check))
        .route("/api/project", get(routes::project::get_project_info))
        .route("/api/projects/open", post(routes::projects::open_project))
        .route("/api/projects/init", post(routes::projects::init_project))
        .route(
            "/api/projects/current",
            get(routes::projects::get_current_project),
        )
        .route(
            "/api/projects/recent",
            get(routes::projects::get_recent_projects),
        )
        .route(
            "/api/projects/recent/remove",
            post(routes::projects::remove_recent_project),
        )
        .route(
            "/api/projects/recent/clear",
            post(routes::projects::clear_recent_projects),
        )
        .route(
            "/api/projects/validate",
            post(routes::projects::validate_project_path),
        )
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
        .route("/api/tasks/{id}/plan", get(routes::get_task_plan))
        .route("/api/tasks/{id}/findings", get(routes::get_task_findings))
        .route("/api/tasks/{id}/findings/fix", post(routes::fix_findings))
        .route("/api/tasks/{id}/findings/skip", post(routes::skip_findings))
        .route("/api/tasks/{id}/phases", get(routes::get_task_phases))
        .route(
            "/api/tasks/{id}/diff/viewed",
            get(routes::get_viewed_files).post(routes::set_file_viewed),
        )
        .route(
            "/api/tasks/{id}/comments",
            get(routes::list_comments).post(routes::create_comment),
        )
        .route(
            "/api/tasks/{id}/comments/{comment_id}",
            axum::routing::delete(routes::delete_comment),
        )
        .route(
            "/api/tasks/{id}/comments/send-to-fix",
            post(routes::send_comments_to_fix),
        )
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
        .route(
            "/api/sessions/{id}/activity",
            get(routes::sse::session_activity_stream),
        )
        .route("/api/events", get(routes::sse::events_stream))
        .route("/api/workspaces", get(routes::list_workspaces))
        .route(
            "/api/workspaces/{id}",
            get(routes::get_workspace_status).delete(routes::delete_workspace),
        )
        .route("/api/workspaces/{id}/diff", get(routes::get_workspace_diff))
        .route("/api/workspaces/{id}/merge", post(routes::merge_workspace))
        .route(
            "/api/filesystem/browse",
            get(routes::filesystem::browse_directory),
        )
        .route(
            "/api/opencode/providers",
            get(routes::opencode::get_providers),
        )
        .route(
            "/api/settings/models",
            get(routes::settings::get_phase_models).put(routes::settings::update_phase_models),
        )
        .route(
            "/api/settings/user-mode",
            get(routes::complete::get_user_mode).put(routes::complete::update_user_mode),
        )
        .route(
            "/api/settings/github",
            get(routes::settings::get_github_settings)
                .put(routes::settings::update_github_settings)
                .delete(routes::settings::delete_github_token),
        )
        .route(
            "/api/tasks/{id}/complete/preview",
            get(routes::complete::get_complete_preview),
        )
        .route(
            "/api/tasks/{id}/complete",
            post(routes::complete::complete_task),
        )
        // Pull Requests routes
        .route(
            "/api/pull-requests",
            get(routes::pull_requests::list_pull_requests),
        )
        .route(
            "/api/pull-requests/{number}",
            get(routes::pull_requests::get_pull_request),
        )
        .route(
            "/api/pull-requests/{number}/diff",
            get(routes::pull_requests::get_pull_request_diff),
        )
        .route(
            "/api/pull-requests/{number}/files",
            get(routes::pull_requests::get_pull_request_files),
        )
        .route(
            "/api/pull-requests/{number}/comments",
            get(routes::pull_requests::get_pull_request_comments)
                .post(routes::pull_requests::create_review_comment),
        )
        .route(
            "/api/pull-requests/{number}/comments/{comment_id}/reply",
            post(routes::pull_requests::reply_to_comment),
        )
        .route(
            "/api/pull-requests/{number}/reviews",
            get(routes::pull_requests::get_pull_request_reviews),
        )
        .route(
            "/api/pull-requests/{number}/fix",
            post(routes::pull_requests::fix_from_pr_comments),
        )
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state);

    if let Some(app_dir) = app_dir {
        let index_file = app_dir.join("index.html");
        let serve_dir = ServeDir::new(&app_dir).not_found_service(ServeFile::new(&index_file));
        api_router.fallback_service(serve_dir)
    } else {
        api_router
    }
}
