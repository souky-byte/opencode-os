use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;
use console::Term;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use server::{create_router, state::AppState};
use std::io::Write;
use std::path::PathBuf;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

const STUDIO_DIR: &str = ".opencode-studio";
const CONFIG_FILE: &str = "config.toml";
const GLOBAL_CONFIG_FILE: &str = "global.toml";
const DEFAULT_DB_NAME: &str = "studio.db";
const DEFAULT_PORT: u16 = 3001;
const MAX_RECENT_PROJECTS: usize = 10;

const APP_DIR: &str = "app";
const APP_VERSION_FILE: &str = "version.txt";
const FRONTEND_RELEASE_URL: &str =
    "https://github.com/souky-byte/opencode-studio/releases/latest/download/frontend.zip";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

const LOGO: &str = r#"
   ____                   ______          __   
  / __ \____  ___  ____  / ____/___  ____/ /__ 
 / / / / __ \/ _ \/ __ \/ /   / __ \/ __  / _ \
/ /_/ / /_/ /  __/ / / / /___/ /_/ / /_/ /  __/
\____/ .___/\___/_/ /_/\____/\____/\__,_/\___/ 
    /_/                                   Studio
"#;

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
    /// Initialize OpenCode Studio in a project directory
    Init {
        /// Path to the project directory (defaults to current directory)
        #[arg(value_name = "PATH")]
        path: Option<PathBuf>,
    },
    /// Start the OpenCode Studio server
    Serve {
        /// Path to the project directory (defaults to current directory)
        #[arg(value_name = "PATH")]
        path: Option<PathBuf>,

        #[arg(short, long, default_value_t = DEFAULT_PORT)]
        port: u16,

        #[arg(long, default_value = "http://localhost:4096")]
        opencode_url: String,

        #[arg(long)]
        no_browser: bool,
    },
    /// Show project status
    Status {
        /// Path to the project directory (defaults to current directory)
        #[arg(value_name = "PATH")]
        path: Option<PathBuf>,
    },
    /// Update the frontend app to the latest version
    Update,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct StudioConfig {
    project: ProjectConfig,
    server: ServerConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ProjectConfig {
    name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ServerConfig {
    port: u16,
    opencode_url: String,
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
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct GlobalConfig {
    #[serde(default)]
    recent_projects: Vec<String>,
    last_project: Option<String>,
}

fn get_global_config_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not determine home directory")?;
    Ok(home.join(STUDIO_DIR))
}

fn get_app_dir() -> Result<PathBuf> {
    Ok(get_global_config_dir()?.join(APP_DIR))
}

async fn load_global_config() -> GlobalConfig {
    let Ok(config_dir) = get_global_config_dir() else {
        return GlobalConfig::default();
    };
    let config_path = config_dir.join(GLOBAL_CONFIG_FILE);

    if !config_path.exists() {
        return GlobalConfig::default();
    }

    match tokio::fs::read_to_string(&config_path).await {
        Ok(content) => toml::from_str(&content).unwrap_or_default(),
        Err(_) => GlobalConfig::default(),
    }
}

async fn save_global_config(config: &GlobalConfig) -> Result<()> {
    let config_dir = get_global_config_dir()?;
    tokio::fs::create_dir_all(&config_dir).await?;

    let config_path = config_dir.join(GLOBAL_CONFIG_FILE);
    let content = toml::to_string_pretty(config)?;
    tokio::fs::write(&config_path, content).await?;
    Ok(())
}

async fn add_recent_project(path: &std::path::Path) {
    let path_str = path.display().to_string();
    let mut config = load_global_config().await;

    config.recent_projects.retain(|p| p != &path_str);
    config.recent_projects.insert(0, path_str.clone());
    config.recent_projects.truncate(MAX_RECENT_PROJECTS);
    config.last_project = Some(path_str);

    let _ = save_global_config(&config).await;
}

async fn check_app_version(app_dir: &std::path::Path) -> Option<String> {
    let version_file = app_dir.join(APP_VERSION_FILE);
    tokio::fs::read_to_string(version_file).await.ok()
}

async fn download_frontend(app_dir: &PathBuf, show_progress: bool) -> Result<()> {
    let temp_zip = app_dir.parent().unwrap().join("frontend.zip");

    tokio::fs::create_dir_all(app_dir).await?;

    let client = reqwest::Client::new();
    let response = client
        .get(FRONTEND_RELEASE_URL)
        .send()
        .await
        .context("Failed to download frontend")?;

    if !response.status().is_success() {
        anyhow::bail!(
            "Failed to download frontend: HTTP {}",
            response.status()
        );
    }

    let total_size = response.content_length().unwrap_or(0);

    let pb = if show_progress && total_size > 0 {
        let pb = ProgressBar::new(total_size);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("     {spinner:.cyan} [{bar:30.cyan/dim}] {bytes}/{total_bytes}")
                .unwrap()
                .progress_chars("█▓░"),
        );
        Some(pb)
    } else {
        None
    };

    let mut file = std::fs::File::create(&temp_zip)?;
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.context("Error downloading chunk")?;
        file.write_all(&chunk)?;
        if let Some(ref pb) = pb {
            pb.inc(chunk.len() as u64);
        }
    }

    if let Some(pb) = pb {
        pb.finish_and_clear();
    }

    let zip_file = std::fs::File::open(&temp_zip)?;
    let mut archive = zip::ZipArchive::new(zip_file)?;

    if app_dir.exists() {
        tokio::fs::remove_dir_all(app_dir).await?;
    }
    tokio::fs::create_dir_all(app_dir).await?;

    archive.extract(app_dir)?;

    tokio::fs::remove_file(&temp_zip).await?;

    let version_file = app_dir.join(APP_VERSION_FILE);
    tokio::fs::write(&version_file, CURRENT_VERSION).await?;

    Ok(())
}

async fn ensure_frontend_app() -> Result<PathBuf> {
    let app_dir = get_app_dir()?;

    if !app_dir.exists() || !app_dir.join("index.html").exists() {
        println!();
        println!(
            "  {} {}",
            "📦".yellow(),
            "Downloading OpenCode Studio app...".white()
        );

        match download_frontend(&app_dir, true).await {
            Ok(()) => {
                println!(
                    "  {} {}",
                    "✓".green().bold(),
                    "App downloaded successfully".green()
                );
            }
            Err(e) => {
                println!(
                    "  {} {}",
                    "✗".red(),
                    format!("Failed to download app: {}", e).red()
                );
                println!();
                println!(
                    "     {}",
                    "The app will still work, but you'll need the frontend running separately."
                        .dimmed()
                );
                println!(
                    "     {}",
                    "Or run 'opencode-studio update' to retry the download.".dimmed()
                );
                return Err(e);
            }
        }
    }

    Ok(app_dir)
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Init { path }) => init_project(path).await,
        Some(Commands::Serve {
            path,
            port,
            opencode_url,
            no_browser,
        }) => serve(path, port, &opencode_url, !no_browser).await,
        Some(Commands::Status { path }) => status(path).await,
        Some(Commands::Update) => update_frontend().await,
        None => serve(None, cli.port, &cli.opencode_url, true).await,
    }
}

async fn update_frontend() -> Result<()> {
    let app_dir = get_app_dir()?;

    println!();
    println!(
        "  {} {}",
        "🔄".cyan(),
        "Updating OpenCode Studio app...".white()
    );

    if let Some(current) = check_app_version(&app_dir).await {
        println!("     {} {}", "Current version:".dimmed(), current.trim());
    }

    download_frontend(&app_dir, true).await?;

    println!(
        "  {} {} {}",
        "✓".green().bold(),
        "Updated to version".green(),
        CURRENT_VERSION.cyan()
    );
    println!();

    Ok(())
}

async fn resolve_project_path(path: Option<PathBuf>) -> Result<PathBuf> {
    let project_path = match path {
        Some(p) => {
            if p.is_absolute() {
                p
            } else {
                std::env::current_dir()?.join(p)
            }
        }
        None => {
            let cwd = std::env::current_dir()?;
            if cwd.join(STUDIO_DIR).exists()
                || cwd.join(".git").exists()
                || cwd.join(".jj").exists()
            {
                cwd
            } else {
                let global_config = load_global_config().await;
                if let Some(last) = global_config.last_project {
                    let last_path = PathBuf::from(&last);
                    if last_path.exists() {
                        println!("{} Using last project: {}", "→".cyan(), last.dimmed());
                        last_path
                    } else {
                        cwd
                    }
                } else {
                    cwd
                }
            }
        }
    };

    let canonical = project_path
        .canonicalize()
        .with_context(|| format!("Path does not exist: {}", project_path.display()))?;

    Ok(canonical)
}

fn validate_vcs_project(path: &std::path::Path) -> Result<()> {
    let has_git = path.join(".git").exists();
    let has_jj = path.join(".jj").exists();

    if !has_git && !has_jj {
        anyhow::bail!(
            "Not a git/jj repository: {}\nInitialize with 'git init' or 'jj init' first.",
            path.display()
        );
    }

    Ok(())
}

async fn init_project_internal(cwd: &std::path::Path, silent: bool) -> Result<StudioConfig> {
    let studio_dir = cwd.join(STUDIO_DIR);

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

    if !silent {
        print_init_success(&project_name);
    }

    add_recent_project(cwd).await;

    Ok(config)
}

fn print_init_success(project_name: &str) {
    println!();
    println!(
        "  {} Initialized OpenCode Studio for '{}'",
        "✓".green().bold(),
        project_name.cyan()
    );
    println!();
    println!("  {}", "Created:".dimmed());
    println!("    {}/ ", STUDIO_DIR.yellow());
    println!("    ├── {} ", CONFIG_FILE.dimmed());
    println!("    ├── {} ", DEFAULT_DB_NAME.dimmed());
    println!("    └── {}/", "kanban".dimmed());
    println!("        ├── {}/", "plans".dimmed());
    println!("        └── {}/", "reviews".dimmed());
    println!();
}

async fn init_project(path: Option<PathBuf>) -> Result<()> {
    let cwd = resolve_project_path(path).await?;
    validate_vcs_project(&cwd)?;
    let studio_dir = cwd.join(STUDIO_DIR);

    if studio_dir.exists() {
        println!(
            "  {} Project already initialized at {}",
            "ℹ".blue(),
            studio_dir.display().to_string().dimmed()
        );
        return Ok(());
    }

    println!(
        "  {} Initializing in {}",
        "→".cyan(),
        cwd.display().to_string().dimmed()
    );

    init_project_internal(&cwd, false).await?;

    println!("  {}", "Next steps:".bold());
    println!(
        "    {} Run {} to start",
        "1.".dimmed(),
        "opencode-studio".cyan()
    );
    println!();

    Ok(())
}

async fn load_studio_config(studio_dir: &std::path::Path) -> Result<StudioConfig> {
    let config_path = studio_dir.join(CONFIG_FILE);
    if config_path.exists() {
        let content = tokio::fs::read_to_string(&config_path).await?;
        Ok(toml::from_str(&content)?)
    } else {
        Ok(StudioConfig::default())
    }
}

fn print_banner(project_name: &str, port: u16) {
    let term = Term::stdout();
    let _ = term.clear_screen();

    for (i, line) in LOGO.lines().enumerate() {
        let colored_line = match i % 3 {
            0 => line.cyan(),
            1 => line.blue(),
            _ => line.magenta(),
        };
        println!("{}", colored_line);
    }

    println!();

    let box_width = 50;
    let separator = "─".repeat(box_width);

    println!("  {}", format!("┌{}┐", separator).dimmed());
    println!(
        "  {}  {} {}{}",
        "│".dimmed(),
        "Project:".dimmed(),
        project_name.white().bold(),
        " ".repeat(box_width.saturating_sub(11 + project_name.len())) + &"│".dimmed().to_string()
    );
    println!("  {}", format!("├{}┤", separator).dimmed());

    let app_url = format!("http://localhost:{}", port);
    let api_url = format!("http://localhost:{}/api", port);
    let docs_url = format!("http://localhost:{}/swagger-ui", port);

    print_url_line("App", &app_url, box_width);
    print_url_line("API", &api_url, box_width);
    print_url_line("Docs", &docs_url, box_width);

    println!("  {}", format!("└{}┘", separator).dimmed());
    println!();

    println!(
        "  {} {} {}",
        "●".green(),
        "Server running".green(),
        "Press Ctrl+C to stop".dimmed()
    );
    println!();
}

fn print_url_line(label: &str, url: &str, box_width: usize) {
    let content = format!("  {}:  {}", label.dimmed(), url.cyan());
    let visible_len = label.len() + url.len() + 4;
    let padding = box_width.saturating_sub(visible_len);
    println!(
        "  {} {}{}{}",
        "│".dimmed(),
        content,
        " ".repeat(padding),
        "│".dimmed()
    );
}

fn print_auto_init_banner() {
    println!();
    println!(
        "  {} {}",
        "⚡".yellow(),
        "First time setup detected".white().bold()
    );
    println!("     {}", "Initializing OpenCode Studio...".dimmed());
    println!();
}

async fn serve(
    path: Option<PathBuf>,
    port: u16,
    opencode_url: &str,
    open_browser: bool,
) -> Result<()> {
    let cwd = resolve_project_path(path).await?;
    validate_vcs_project(&cwd)?;
    add_recent_project(&cwd).await;
    let studio_dir = cwd.join(STUDIO_DIR);

    let config = if studio_dir.exists() {
        load_studio_config(&studio_dir).await?
    } else {
        print_auto_init_banner();
        init_project_internal(&cwd, true).await?
    };

    let app_dir = ensure_frontend_app().await.ok();

    init_tracing();

    tracing::debug!("Project: {}", cwd.display());
    tracing::debug!("OpenCode URL: {}", opencode_url);

    let state = AppState::new(opencode_url);
    let state = if let Some(ref app_dir) = app_dir {
        state.with_app_dir(app_dir.clone())
    } else {
        state
    };

    state
        .open_project(&cwd)
        .await
        .context("Failed to open project")?;

    let app = create_router(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;

    print_banner(&config.project.name, port);

    if open_browser {
        let url = format!("http://localhost:{}", port);
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            if let Err(e) = open::that(&url) {
                tracing::warn!("Failed to open browser: {}", e);
            }
        });
    }

    axum::serve(listener, app).await?;

    Ok(())
}

async fn status(path: Option<PathBuf>) -> Result<()> {
    let cwd = resolve_project_path(path).await?;
    let studio_dir = cwd.join(STUDIO_DIR);

    if !studio_dir.exists() {
        println!();
        println!("  {} Not an OpenCode Studio project.", "✗".red());
        println!(
            "     Run {} to initialize.",
            "opencode-studio init".cyan()
        );
        println!();
        return Ok(());
    }

    let config = load_studio_config(&studio_dir).await?;
    let db_path = studio_dir.join(DEFAULT_DB_NAME);

    if !db_path.exists() {
        println!();
        println!(
            "  {} Project: {} {}",
            "ℹ".blue(),
            config.project.name.cyan(),
            "(database not initialized)".dimmed()
        );
        println!();
        return Ok(());
    }

    let database_url = format!("sqlite:{}", db_path.display());
    let pool = db::create_pool(&database_url).await?;

    let task_repo = db::TaskRepository::new(pool);
    let tasks = task_repo.find_all().await?;

    println!();
    println!(
        "  {} {}",
        "◆".magenta(),
        config.project.name.white().bold()
    );
    println!("    {}", cwd.display().to_string().dimmed());
    println!();

    if tasks.is_empty() {
        println!("  {} No tasks yet.", "○".dimmed());
    } else {
        println!("  {} ({}):", "Tasks".bold(), tasks.len());
        println!();

        for task in &tasks {
            let status_str = serde_json::to_string(&task.status)
                .unwrap_or_default()
                .trim_matches('"')
                .to_string();

            let (icon, color) = match status_str.as_str() {
                "todo" => ("○", "white"),
                "planning" => ("◐", "yellow"),
                "planning_review" => ("◑", "yellow"),
                "in_progress" => ("◑", "blue"),
                "ai_review" => ("◕", "cyan"),
                "review" => ("◕", "magenta"),
                "done" => ("●", "green"),
                _ => ("?", "white"),
            };

            let colored_icon = match color {
                "yellow" => icon.yellow(),
                "blue" => icon.blue(),
                "cyan" => icon.cyan(),
                "magenta" => icon.magenta(),
                "green" => icon.green(),
                _ => icon.white(),
            };

            println!(
                "    {} {} {}",
                colored_icon,
                task.title.white(),
                format!("[{}]", status_str).dimmed()
            );
        }
    }

    println!();

    Ok(())
}

fn init_tracing() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "warn".into()),
        )
        .init();
}
