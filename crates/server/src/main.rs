use server::{create_router, opencode_manager::OpenCodeManager, state::AppState};
use std::path::PathBuf;
use tokio::signal;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Find the frontend assets directory
fn find_app_dir() -> Option<PathBuf> {
    let exe_path = std::env::current_exe().ok()?;
    let exe_dir = exe_path.parent()?;

    // Check locations in order of preference:
    // 1. APP_DIR environment variable
    if let Ok(app_dir) = std::env::var("APP_DIR") {
        let path = PathBuf::from(app_dir);
        if path.join("index.html").exists() {
            return Some(path);
        }
    }

    // 2. dist/frontend (npm package structure - relative to binary)
    let npm_path = exe_dir.join("../dist/frontend");
    if npm_path.join("index.html").exists() {
        return Some(npm_path.canonicalize().ok()?);
    }

    // 3. frontend/dist (development structure - relative to cwd)
    let dev_path = PathBuf::from("frontend/dist");
    if dev_path.join("index.html").exists() {
        return Some(dev_path.canonicalize().ok()?);
    }

    // 4. Relative to binary (for release builds)
    let release_path = exe_dir.join("../../frontend/dist");
    if release_path.join("index.html").exists() {
        return Some(release_path.canonicalize().ok()?);
    }

    None
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "server=debug,orchestrator=debug,wiki=info,tower_http=debug".into()),
        )
        .init();

    let opencode_url =
        std::env::var("OPENCODE_URL").unwrap_or_else(|_| "http://localhost:4096".to_string());

    tracing::info!("OpenCode server URL: {}", opencode_url);

    // Ensure OpenCode server is running
    let mut _opencode_manager = OpenCodeManager::new(&opencode_url);
    _opencode_manager.ensure_running().await?;

    // Find and configure frontend assets
    let app_dir = find_app_dir();
    if let Some(ref dir) = app_dir {
        tracing::info!("Serving frontend from: {:?}", dir);
    } else {
        tracing::warn!("Frontend assets not found - API-only mode");
    }

    let state = if let Some(dir) = app_dir {
        AppState::new(&opencode_url).with_app_dir(dir)
    } else {
        AppState::new(&opencode_url)
    };

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

    // Run server with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    // Explicitly shutdown OpenCode when server stops
    tracing::info!("Shutting down OpenCode server...");
    _opencode_manager.shutdown();

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received Ctrl+C, shutting down...");
        }
        _ = terminate => {
            tracing::info!("Received terminate signal, shutting down...");
        }
    }
}
