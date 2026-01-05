use server::{create_router, opencode_manager::OpenCodeManager, state::AppState};
use std::path::PathBuf;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "server=debug,wiki=info,tower_http=debug".into()),
        )
        .init();

    let opencode_url =
        std::env::var("OPENCODE_URL").unwrap_or_else(|_| "http://localhost:4096".to_string());

    tracing::info!("OpenCode server URL: {}", opencode_url);

    // Ensure OpenCode server is running
    let mut _opencode_manager = OpenCodeManager::new(&opencode_url);
    _opencode_manager.ensure_running().await?;

    let state = AppState::new(&opencode_url);

    if let Some(project_path) = std::env::var("PROJECT_PATH").ok().map(PathBuf::from) {
        tracing::info!("Opening project from PROJECT_PATH: {:?}", project_path);
        state.open_project(&project_path).await?;
    } else if let Ok(database_url) = std::env::var("DATABASE_URL") {
        if database_url.starts_with("sqlite:") {
            let db_path = database_url
                .strip_prefix("sqlite:")
                .unwrap_or(&database_url);
            let db_path = PathBuf::from(db_path);

            if let Some(studio_dir) = db_path.parent() {
                if let Some(project_path) = studio_dir.parent() {
                    if project_path.join(".git").exists() || project_path.join(".jj").exists() {
                        tracing::info!("Opening project from DATABASE_URL: {:?}", project_path);
                        state.open_project(project_path).await?;
                    }
                }
            }
        }
    } else {
        match state.auto_open_last_project().await {
            Ok(true) => tracing::info!("Auto-opened last project"),
            Ok(false) => tracing::info!("No project to auto-open"),
            Err(e) => tracing::warn!("Failed to auto-open last project: {}", e),
        }
    }

    let app = create_router(state);

    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3001);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    tracing::info!("Server listening on http://0.0.0.0:{}", port);

    axum::serve(listener, app).await?;

    Ok(())
}
