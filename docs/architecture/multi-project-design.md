# Multi-Project Workspace Selection - Backend Design

## Overview

Enable users to select and switch between multiple projects at runtime, with automatic initialization of `.opencode-studio/` structure when opening a new project.

## User Flow

1. User opens opencode-studio
2. Frontend shows project picker (recent projects + folder selector)
3. User selects a folder (must be git/jj repo)
4. Backend validates, auto-inits if needed, switches context
5. User can switch projects at any time without restarting

## Architecture Decision

**Approach: Hot-swap AppState with Arc<RwLock<ProjectContext>>**

The server maintains a single `AppState` but with a swappable `ProjectContext` that holds project-specific resources (DB pool, repositories, executor, workspace manager).

```
AppState (immutable shell)
├── project_context: Arc<RwLock<Option<ProjectContext>>>  // Swappable
├── opencode_url: String                                   // Shared
├── global_config: GlobalConfigManager                     // Shared
└── event_bus: EventBus                                    // Shared (broadcast project changes)

ProjectContext (per-project)
├── path: PathBuf
├── pool: SqlitePool
├── task_repository: TaskRepository
├── session_repository: SessionRepository
├── task_executor: Arc<TaskExecutor>
├── workspace_manager: Arc<WorkspaceManager>
└── vcs: Arc<dyn VersionControl>
```

### Why This Approach?

1. **No server restart** - Seamless project switching
2. **Clean isolation** - Each project has its own DB and resources
3. **Graceful cleanup** - Can close old DB connections before switching
4. **WebSocket continuity** - Existing connections stay alive, receive project change events

## API Design

### New Endpoints

```
POST /api/projects/open
POST /api/projects/init
GET  /api/projects/current
GET  /api/projects/recent     (exists, rename from /api/project/recent)
POST /api/projects/validate   (exists, rename from /api/project/validate)
```

### POST /api/projects/open

Opens a project, auto-initializing if needed.

**Request:**
```json
{
  "path": "/Users/dev/my-project"
}
```

**Response (success):**
```json
{
  "success": true,
  "project": {
    "name": "my-project",
    "path": "/Users/dev/my-project",
    "vcs": "git",
    "tasks_count": 12,
    "initialized": true,
    "was_initialized": false  // true if we just created .opencode-studio/
  }
}
```

**Response (error):**
```json
{
  "success": false,
  "error": {
    "code": "NOT_VCS_REPO",
    "message": "Path is not a git or jujutsu repository"
  }
}
```

**Error codes:**
- `PATH_NOT_FOUND` - Path doesn't exist
- `NOT_DIRECTORY` - Path is not a directory
- `NOT_VCS_REPO` - Not a git/jj repository
- `INIT_FAILED` - Failed to create .opencode-studio structure
- `DB_CONNECT_FAILED` - Failed to connect to project database

### POST /api/projects/init

Explicitly initializes a project without switching to it. Useful for "init in background" scenarios.

**Request:**
```json
{
  "path": "/Users/dev/my-project",
  "force": false  // if true, reinitialize even if exists
}
```

**Response:**
```json
{
  "success": true,
  "already_initialized": false,
  "project": { ... }
}
```

### GET /api/projects/current

Returns currently active project, or null if none.

**Response:**
```json
{
  "project": {
    "name": "my-project",
    "path": "/Users/dev/my-project",
    "vcs": "git",
    "tasks_count": 12,
    "initialized": true
  }
}
```

**Response (no project):**
```json
{
  "project": null
}
```

## Implementation Plan

### 1. New Module: `crates/server/src/project_manager.rs`

```rust
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ProjectManager {
    context: Arc<RwLock<Option<ProjectContext>>>,
    opencode_url: String,
    global_config: GlobalConfigManager,
}

pub struct ProjectContext {
    pub path: PathBuf,
    pub pool: SqlitePool,
    pub task_repository: TaskRepository,
    pub session_repository: SessionRepository,
    pub task_executor: Arc<TaskExecutor>,
    pub workspace_manager: Arc<WorkspaceManager>,
}

impl ProjectManager {
    /// Open a project, initializing if needed
    pub async fn open(&self, path: &Path) -> Result<ProjectInfo, ProjectError>;
    
    /// Initialize project structure without opening
    pub async fn init(&self, path: &Path, force: bool) -> Result<InitResult, ProjectError>;
    
    /// Get current project context
    pub async fn current(&self) -> Option<ProjectContext>;
    
    /// Close current project (cleanup)
    pub async fn close(&self) -> Result<(), ProjectError>;
}

impl ProjectContext {
    /// Create from path (connects to DB, sets up executor)
    pub async fn new(path: PathBuf, opencode_url: &str) -> Result<Self, ProjectError>;
}
```

### 2. Update: `crates/server/src/state.rs`

```rust
#[derive(Clone)]
pub struct AppState {
    pub project_manager: Arc<ProjectManager>,
    pub event_bus: EventBus,
    pub opencode_url: String,
}

impl AppState {
    pub async fn new(opencode_url: &str) -> Self {
        let event_bus = EventBus::new();
        let global_config = GlobalConfigManager::new();
        let project_manager = Arc::new(ProjectManager::new(
            opencode_url.to_string(),
            global_config,
        ));
        
        Self {
            project_manager,
            event_bus,
            opencode_url: opencode_url.to_string(),
        }
    }
    
    /// Convenience: get current project context or error
    pub async fn project(&self) -> Result<ProjectContext, NoProjectError> {
        self.project_manager.current().await.ok_or(NoProjectError)
    }
}
```

### 3. Update Route Handlers

Existing task/session routes need to handle "no project" state:

```rust
pub async fn list_tasks(State(state): State<AppState>) -> Result<Json<...>, AppError> {
    let project = state.project().await?;  // Returns 400 if no project
    let tasks = project.task_repository.find_all().await?;
    Ok(Json(tasks))
}
```

### 4. Global Config Manager

```rust
// ~/.opencode-studio/global.toml
[projects]
recent = [
    "/Users/dev/project-a",
    "/Users/dev/project-b",
]
last = "/Users/dev/project-a"

[preferences]
auto_open_last = true
max_recent = 10
```

```rust
pub struct GlobalConfigManager {
    path: PathBuf,  // ~/.opencode-studio/global.toml
}

impl GlobalConfigManager {
    pub fn add_recent(&self, path: &Path) -> Result<(), ConfigError>;
    pub fn get_recent(&self) -> Vec<PathBuf>;
    pub fn set_last(&self, path: &Path) -> Result<(), ConfigError>;
    pub fn get_last(&self) -> Option<PathBuf>;
}
```

### 5. Project Initialization

When opening a project that lacks `.opencode-studio/`:

```rust
async fn init_project_structure(path: &Path) -> Result<(), InitError> {
    let studio_dir = path.join(".opencode-studio");
    
    // Create directory structure
    fs::create_dir_all(&studio_dir)?;
    fs::create_dir_all(studio_dir.join("kanban/plans"))?;
    fs::create_dir_all(studio_dir.join("kanban/reviews"))?;
    
    // Create config.toml with defaults
    let config = ProjectConfig::default();
    fs::write(
        studio_dir.join("config.toml"),
        toml::to_string_pretty(&config)?
    )?;
    
    // Database will be created by sqlx on first connect
    // (create_if_missing: true)
    
    Ok(())
}
```

### 6. WebSocket Events

Broadcast project changes to connected clients:

```rust
#[derive(Serialize, Clone)]
#[serde(tag = "type")]
pub enum ServerMessage {
    // ... existing variants
    
    #[serde(rename = "project_changed")]
    ProjectChanged {
        project: Option<ProjectInfo>,
    },
    
    #[serde(rename = "project_closed")]
    ProjectClosed,
}
```

## Migration Path

### Phase 1: Add ProjectManager (backward compatible)

1. Create `project_manager.rs` with `ProjectContext`
2. Refactor `AppState` to use `ProjectManager` internally
3. Keep existing `DATABASE_URL` env var working
4. Auto-open project from `DATABASE_URL` path on startup

### Phase 2: Add new endpoints

1. Add `/api/projects/open` endpoint
2. Add `/api/projects/current` endpoint
3. Add `/api/projects/init` endpoint
4. Update existing `/api/project/*` routes to `/api/projects/*`

### Phase 3: Handle no-project state

1. Update all route handlers to require `state.project()`
2. Return proper error when no project is open
3. Frontend handles "select project" flow

### Phase 4: Cleanup

1. Remove hardcoded `DATABASE_URL` dependency
2. Add startup option to auto-open last project
3. Add CLI support for project selection

## Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum ProjectError {
    #[error("Path not found: {0}")]
    PathNotFound(PathBuf),
    
    #[error("Path is not a directory: {0}")]
    NotDirectory(PathBuf),
    
    #[error("Not a git or jujutsu repository: {0}")]
    NotVcsRepo(PathBuf),
    
    #[error("Failed to initialize project: {0}")]
    InitFailed(String),
    
    #[error("Database connection failed: {0}")]
    DbConnectFailed(#[from] sqlx::Error),
    
    #[error("No project is currently open")]
    NoProjectOpen,
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

## Directory Structure

```
~/.opencode-studio/                    # Global config
├── global.toml                        # Recent projects, preferences

/path/to/project/                      # User's project
├── .opencode-studio/                  # Project-specific
│   ├── config.toml                    # Project config
│   ├── studio.db                      # SQLite database
│   ├── studio.db-wal                  # WAL file
│   ├── studio.db-shm                  # Shared memory
│   └── kanban/
│       ├── plans/                     # AI-generated plans
│       └── reviews/                   # AI-generated reviews
├── .git/ or .jj/                      # VCS
└── ...                                # Project files
```

## Frontend Integration

The frontend needs to:

1. **On load**: Call `GET /api/projects/current`
   - If project → show main UI
   - If null → show project picker

2. **Project picker UI**:
   - List recent projects from `GET /api/projects/recent`
   - Folder picker button (uses native file dialog via Tauri or similar)
   - Call `POST /api/projects/open` on selection

3. **Project switch**:
   - Show loading state during switch
   - Invalidate all React Query caches on project change
   - Re-subscribe WebSocket with new context

4. **Handle WebSocket `project_changed` event**:
   - Refresh all data
   - Update UI to reflect new project

## Testing Strategy

1. **Unit tests**: `ProjectManager` with mock filesystem
2. **Integration tests**: Full open/switch/close cycle
3. **Concurrency tests**: Multiple rapid switches
4. **Cleanup tests**: Verify DB connections closed properly

## Open Questions

1. **Multiple simultaneous projects?** - Current design is single-project. Multi-project would need separate task executor instances per project, more complex.

2. **Remote projects?** - Git clone from URL? Future enhancement.

3. **Project templates?** - Pre-configured `.opencode-studio/` structures? Future enhancement.
