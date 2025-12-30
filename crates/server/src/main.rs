mod error;
mod routes;
mod state;

use axum::routing::{get, post};
use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "server=debug,tower_http=debug".into()),
        )
        .init();

    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:./studio.db".to_string());
    let opencode_url =
        std::env::var("OPENCODE_URL").unwrap_or_else(|_| "http://localhost:4096".to_string());

    tracing::info!("Connecting to database: {}", database_url);
    tracing::info!("OpenCode server URL: {}", opencode_url);

    let pool = db::create_pool(&database_url).await?;
    db::run_migrations(&pool).await?;

    tracing::info!("Database migrations completed");

    let state = AppState::new(pool, &opencode_url);

    let app = Router::new()
        .route("/health", get(routes::health_check))
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
        .with_state(state);

    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3001);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    tracing::info!("Server listening on http://0.0.0.0:{}", port);

    axum::serve(listener, app).await?;

    Ok(())
}
