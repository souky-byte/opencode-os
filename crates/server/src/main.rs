use server::{create_router, state::AppState};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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
