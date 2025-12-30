use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use server::{create_router, state::AppState};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

const STUDIO_DIR: &str = ".opencode-studio";
const CONFIG_FILE: &str = "config.toml";
const DEFAULT_DB_NAME: &str = "studio.db";
const DEFAULT_PORT: u16 = 3001;
const DEFAULT_FRONTEND_PORT: u16 = 3000;

#[derive(Parser)]
#[command(name = "opencode-studio")]
#[command(about = "AI-powered development platform", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(short, long, default_value_t = DEFAULT_PORT)]
    port: u16,

    #[arg(long, default_value = "http://localhost:4096")]
    opencode_url: String,
}

#[derive(Subcommand)]
enum Commands {
    Init,
    Serve {
        #[arg(short, long, default_value_t = DEFAULT_PORT)]
        port: u16,

        #[arg(long, default_value = "http://localhost:4096")]
        opencode_url: String,

        #[arg(long)]
        no_browser: bool,
    },
    Status,
}

#[derive(Debug, Serialize, Deserialize)]
struct StudioConfig {
    project: ProjectConfig,
    server: ServerConfig,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProjectConfig {
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ServerConfig {
    port: u16,
    opencode_url: String,
    frontend_url: String,
}

impl Default for StudioConfig {
    fn default() -> Self {
        Self {
            project: ProjectConfig {
                name: "my-project".to_string(),
            },
            server: ServerConfig {
                port: DEFAULT_PORT,
                opencode_url: "http://localhost:4096".to_string(),
                frontend_url: format!("http://localhost:{}", DEFAULT_FRONTEND_PORT),
            },
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Init) => init_project().await,
        Some(Commands::Serve {
            port,
            opencode_url,
            no_browser,
        }) => serve(port, &opencode_url, !no_browser).await,
        Some(Commands::Status) => status().await,
        None => serve(cli.port, &cli.opencode_url, true).await,
    }
}

async fn init_project() -> Result<()> {
    let cwd = std::env::current_dir()?;
    let studio_dir = cwd.join(STUDIO_DIR);

    if studio_dir.exists() {
        println!("Project already initialized at {}", studio_dir.display());
        return Ok(());
    }

    println!("Initializing OpenCode Studio in {}", cwd.display());

    tokio::fs::create_dir_all(&studio_dir).await?;
    tokio::fs::create_dir_all(studio_dir.join("kanban/plans")).await?;
    tokio::fs::create_dir_all(studio_dir.join("kanban/reviews")).await?;

    let project_name = cwd
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("my-project")
        .to_string();

    let config = StudioConfig {
        project: ProjectConfig {
            name: project_name.clone(),
        },
        ..Default::default()
    };

    let config_path = studio_dir.join(CONFIG_FILE);
    let config_content = toml::to_string_pretty(&config)?;
    tokio::fs::write(&config_path, config_content).await?;

    let db_path = studio_dir.join(DEFAULT_DB_NAME);
    let database_url = format!("sqlite:{}", db_path.display());
    let pool = db::create_pool(&database_url).await?;
    db::run_migrations(&pool).await?;

    println!();
    println!("Initialized OpenCode Studio for '{}'", project_name);
    println!();
    println!("Created:");
    println!("  {}/ ", STUDIO_DIR);
    println!("  ├── {} ", CONFIG_FILE);
    println!("  ├── {} ", DEFAULT_DB_NAME);
    println!("  └── kanban/");
    println!("      ├── plans/");
    println!("      └── reviews/");
    println!();
    println!("Next steps:");
    println!("  1. Run 'opencode-studio' to start the server");
    println!("  2. Open http://localhost:{} in your browser", DEFAULT_FRONTEND_PORT);

    Ok(())
}

async fn serve(port: u16, opencode_url: &str, open_browser: bool) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let studio_dir = cwd.join(STUDIO_DIR);

    let (config, db_path) = if studio_dir.exists() {
        let config_path = studio_dir.join(CONFIG_FILE);
        let config: StudioConfig = if config_path.exists() {
            let content = tokio::fs::read_to_string(&config_path).await?;
            toml::from_str(&content)?
        } else {
            StudioConfig::default()
        };
        let db_path = studio_dir.join(DEFAULT_DB_NAME);
        (config, db_path)
    } else {
        println!("No .opencode-studio directory found.");
        println!("Run 'opencode-studio init' first, or using default configuration.");
        println!();
        (StudioConfig::default(), cwd.join(DEFAULT_DB_NAME))
    };

    init_tracing();

    let database_url = format!("sqlite:{}", db_path.display());
    tracing::info!("Database: {}", db_path.display());
    tracing::info!("OpenCode URL: {}", opencode_url);

    let pool = db::create_pool(&database_url)
        .await
        .context("Failed to create database pool")?;
    db::run_migrations(&pool).await?;

    let state = AppState::new(pool, opencode_url);
    let app = create_router(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;

    let project_name = &config.project.name;
    println!();
    println!("OpenCode Studio - {}", project_name);
    println!("════════════════════════════════════════");
    println!();
    println!("  API Server:  http://localhost:{}", port);
    println!("  Swagger UI:  http://localhost:{}/swagger-ui", port);
    println!("  Frontend:    {}", config.server.frontend_url);
    println!();
    println!("Press Ctrl+C to stop");
    println!();

    if open_browser {
        let frontend_url = config.server.frontend_url.clone();
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            if let Err(e) = open::that(&frontend_url) {
                tracing::warn!("Failed to open browser: {}", e);
            }
        });
    }

    axum::serve(listener, app).await?;

    Ok(())
}

async fn status() -> Result<()> {
    let cwd = std::env::current_dir()?;
    let studio_dir = cwd.join(STUDIO_DIR);

    if !studio_dir.exists() {
        println!("Not an OpenCode Studio project.");
        println!("Run 'opencode-studio init' to initialize.");
        return Ok(());
    }

    let config_path = studio_dir.join(CONFIG_FILE);
    let config: StudioConfig = if config_path.exists() {
        let content = tokio::fs::read_to_string(&config_path).await?;
        toml::from_str(&content)?
    } else {
        StudioConfig::default()
    };

    let db_path = studio_dir.join(DEFAULT_DB_NAME);

    if !db_path.exists() {
        println!("Project: {} (database not initialized)", config.project.name);
        return Ok(());
    }

    let database_url = format!("sqlite:{}", db_path.display());
    let pool = db::create_pool(&database_url).await?;

    let task_repo = db::TaskRepository::new(pool);
    let tasks = task_repo.find_all().await?;

    println!();
    println!("Project: {}", config.project.name);
    println!("Path:    {}", cwd.display());
    println!();

    if tasks.is_empty() {
        println!("No tasks yet.");
    } else {
        println!("Tasks ({}):", tasks.len());
        for task in &tasks {
            let status_str = serde_json::to_string(&task.status)
                .unwrap_or_default()
                .trim_matches('"')
                .to_string();
            let status_icon = match status_str.as_str() {
                "todo" => "○",
                "planning" | "planning_review" => "◐",
                "in_progress" | "ai_review" => "◑",
                "review" => "◕",
                "done" => "●",
                _ => "?",
            };
            println!("  {} [{}] {}", status_icon, status_str, task.title);
        }
    }

    println!();

    Ok(())
}

fn init_tracing() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "opencode_studio=info,server=info,tower_http=info".into()),
        )
        .init();
}
