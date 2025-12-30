# Backend Implementation Plan

> **Dokument:** Implementační plán pro OpenCode Studio backend
> **Verze:** 1.0
> **Datum:** 2024-12-30
> **Status:** Draft

---

## Executive Summary

Tento dokument definuje fázový plán implementace Rust backendu pro OpenCode Studio. Architektura je navržena s důrazem na:

- **Škálovatelnost**: Horizontální škálování, paralelní tasky
- **Modularita**: Oddělené crates pro snadnou údržbu a testování
- **Clean code**: Domain-driven design, trait-based abstractions

---

## Technologický Stack

| Vrstva | Technologie | Verze |
|--------|-------------|-------|
| Runtime | Rust + Tokio | 1.75+ |
| HTTP Server | Axum | 0.8 |
| Database | SQLite + sqlx | 0.8 |
| Serialization | serde + serde_json | 1.0 |
| OpenCode SDK | Generované z OpenAPI | - |
| Type Generation | ts-rs | latest |
| VCS | Jujutsu (jj) | latest |

---

## Crates Architecture

```
crates/
├── core/           # Domain models, traits, events (NO I/O)
├── db/             # SQLite persistence (sqlx)
├── opencode/       # OpenCode HTTP client (generated + wrapper)
├── vcs/            # Version control abstraction (jj, git)
├── orchestrator/   # Task lifecycle, scheduling
├── api/            # Axum HTTP server + WebSocket
└── cli/            # CLI binary (optional, Phase 4)
```

### Dependency Graph

```
                    ┌─────────┐
                    │   cli   │
                    └────┬────┘
                         │
                    ┌────▼────┐
                    │   api   │
                    └────┬────┘
                         │
              ┌──────────┼──────────┐
              │          │          │
         ┌────▼────┐ ┌───▼───┐ ┌────▼────┐
         │orchestr.│ │  db   │ │opencode │
         └────┬────┘ └───┬───┘ └────┬────┘
              │          │          │
              └──────────┼──────────┘
                         │
                    ┌────▼────┐
                    │   vcs   │
                    └────┬────┘
                         │
                    ┌────▼────┐
                    │  core   │
                    └─────────┘
```

---

## Phase 1: Foundation (2-3 týdny)

### Cíl
Základní infrastruktura - projektem strukturovaný workspace, databáze, základní API.

### 1.1 Workspace Setup

**Soubory k vytvoření:**

```
Cargo.toml                          # Workspace root
crates/
├── core/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── domain/
│       │   ├── mod.rs
│       │   ├── task.rs             # Task entity
│       │   ├── session.rs          # Session entity
│       │   └── workspace.rs        # Workspace entity
│       ├── events/
│       │   ├── mod.rs
│       │   └── bus.rs              # Event bus trait + impl
│       └── error.rs                # Unified error types
│
├── db/
│   ├── Cargo.toml
│   ├── migrations/
│   │   └── 001_initial.sql
│   └── src/
│       ├── lib.rs
│       ├── pool.rs                 # Connection pool setup
│       ├── models/
│       │   ├── mod.rs
│       │   └── task.rs             # DB models
│       └── repositories/
│           ├── mod.rs
│           └── task_repo.rs        # TaskRepository impl
│
└── api/
    ├── Cargo.toml
    └── src/
        ├── main.rs
        ├── routes/
        │   ├── mod.rs
        │   ├── health.rs
        │   └── tasks.rs
        ├── state.rs                # AppState
        └── error.rs                # HTTP error handling
```

**Workspace Cargo.toml:**

```toml
[workspace]
resolver = "2"
members = [
    "crates/core",
    "crates/db",
    "crates/api",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
rust-version = "1.75"

[workspace.dependencies]
# Async runtime
tokio = { version = "1.0", features = ["full"] }

# Web framework
axum = { version = "0.8", features = ["macros", "ws"] }
tower-http = { version = "0.5", features = ["cors", "trace"] }

# Database
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Error handling
anyhow = "1.0"
thiserror = "2.0"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Utils
uuid = { version = "1.0", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
async-trait = "0.1"

# Type generation
ts-rs = { version = "10", features = ["uuid-impl", "chrono-impl"] }
```

### 1.2 Core Domain Models

**crates/core/src/domain/task.rs:**

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
pub enum TaskStatus {
    Todo,
    Planning,
    PlanningReview,
    InProgress,
    AiReview,
    Review,
    Done,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
pub struct Task {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub status: TaskStatus,
    pub roadmap_item_id: Option<Uuid>,
    pub workspace_path: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Task {
    pub fn new(title: String, description: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            title,
            description,
            status: TaskStatus::Todo,
            roadmap_item_id: None,
            workspace_path: None,
            created_at: now,
            updated_at: now,
        }
    }
}
```

### 1.3 Database Schema

**crates/db/migrations/001_initial.sql:**

```sql
-- Tasks table
CREATE TABLE IF NOT EXISTS tasks (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'Todo',
    roadmap_item_id TEXT,
    workspace_path TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX idx_tasks_status ON tasks(status);
CREATE INDEX idx_tasks_created_at ON tasks(created_at);

-- Sessions table (OpenCode sessions)
CREATE TABLE IF NOT EXISTS sessions (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL REFERENCES tasks(id),
    opencode_session_id TEXT,
    phase TEXT NOT NULL,  -- 'planning', 'implementation', 'review'
    status TEXT NOT NULL DEFAULT 'pending',
    started_at INTEGER,
    completed_at INTEGER,
    created_at INTEGER NOT NULL
);

CREATE INDEX idx_sessions_task_id ON sessions(task_id);

-- Events log
CREATE TABLE IF NOT EXISTS events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_type TEXT NOT NULL,
    payload TEXT NOT NULL,  -- JSON
    created_at INTEGER NOT NULL
);

CREATE INDEX idx_events_type ON events(event_type);
CREATE INDEX idx_events_created_at ON events(created_at);
```

### 1.4 Basic API Endpoints

**Phase 1 endpoints:**

| Method | Path | Description |
|--------|------|-------------|
| GET | `/health` | Health check |
| GET | `/api/tasks` | List all tasks |
| POST | `/api/tasks` | Create task |
| GET | `/api/tasks/:id` | Get task detail |
| PATCH | `/api/tasks/:id` | Update task |
| DELETE | `/api/tasks/:id` | Delete task |

### 1.5 Deliverables

- [ ] Workspace setup s 3 crates (core, db, api)
- [ ] Domain models (Task, Session, TaskStatus)
- [ ] SQLite database s migrací
- [ ] Basic CRUD API pro tasks
- [ ] Health endpoint
- [ ] Tracing/logging setup
- [ ] `cargo test` passing

### 1.6 Acceptance Criteria

```bash
# Server starts
cargo run --package api

# Health check works
curl http://localhost:3001/health
# => {"status": "ok", "version": "0.1.0"}

# CRUD works
curl -X POST http://localhost:3001/api/tasks \
  -H "Content-Type: application/json" \
  -d '{"title": "Test", "description": "Test task"}'
# => {"id": "uuid...", "status": "Todo", ...}
```

---

## Phase 2: OpenCode Integration (2-3 týdny)

### Cíl
Integrace s OpenCode HTTP Server API, SDK generování, session management.

### 2.1 Nové Crates

```
crates/
├── opencode/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── client.rs           # HTTP client wrapper
│       ├── types.rs            # Generated types (nebo z OpenAPI)
│       ├── session.rs          # Session management
│       └── events.rs           # SSE event handling
│
└── orchestrator/
    ├── Cargo.toml
    └── src/
        ├── lib.rs
        ├── executor.rs         # Task execution logic
        ├── scheduler.rs        # Parallel task scheduling
        └── state_machine.rs    # Task status transitions
```

### 2.2 OpenCode SDK Generation

**Postup:**

```bash
# 1. Spustit OpenCode server
opencode serve --port 4096

# 2. Stáhnout OpenAPI spec
curl http://localhost:4096/doc -o opencode-api.json

# 3. Generovat typy (možnosti):
# A) openapi-generator
openapi-generator generate -i opencode-api.json -g rust -o crates/opencode/src/generated

# B) progenitor (compile-time)
# V Cargo.toml: progenitor = "0.8"
```

**crates/opencode/src/client.rs (wrapper):**

```rust
use reqwest::Client;
use crate::types::*;

pub struct OpenCodeClient {
    base_url: String,
    client: Client,
}

impl OpenCodeClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            client: Client::new(),
        }
    }

    pub async fn create_session(&self, title: Option<String>) -> Result<Session, Error> {
        let resp = self.client
            .post(format!("{}/session", self.base_url))
            .json(&CreateSessionRequest { title, parent_id: None })
            .send()
            .await?;
        
        resp.json().await.map_err(Into::into)
    }

    pub async fn send_prompt(
        &self,
        session_id: &str,
        prompt: &str,
        model: Option<&str>,
    ) -> Result<MessageResponse, Error> {
        let resp = self.client
            .post(format!("{}/session/{}/message", self.base_url, session_id))
            .json(&SendMessageRequest {
                parts: vec![Part::Text { text: prompt.into() }],
                model: model.map(Into::into),
                ..Default::default()
            })
            .send()
            .await?;
        
        resp.json().await.map_err(Into::into)
    }

    pub async fn send_prompt_async(&self, session_id: &str, prompt: &str) -> Result<(), Error> {
        self.client
            .post(format!("{}/session/{}/prompt_async", self.base_url, session_id))
            .json(&SendMessageRequest {
                parts: vec![Part::Text { text: prompt.into() }],
                ..Default::default()
            })
            .send()
            .await?;
        
        Ok(())
    }

    pub async fn abort_session(&self, session_id: &str) -> Result<(), Error> {
        self.client
            .post(format!("{}/session/{}/abort", self.base_url, session_id))
            .send()
            .await?;
        
        Ok(())
    }

    pub fn subscribe_events(&self) -> Result<EventStream, Error> {
        // SSE connection to /event
        todo!()
    }
}
```

### 2.3 SSE Event Handling

```rust
// crates/opencode/src/events.rs
use eventsource_client as sse;
use futures::StreamExt;

pub struct EventStream {
    stream: sse::Client,
}

impl EventStream {
    pub async fn connect(base_url: &str) -> Result<Self, Error> {
        let client = sse::ClientBuilder::for_url(&format!("{}/event", base_url))?
            .build();
        Ok(Self { stream: client })
    }

    pub async fn next_event(&mut self) -> Option<OpenCodeEvent> {
        self.stream.next().await.and_then(|result| {
            result.ok().and_then(|event| {
                serde_json::from_str(&event.data).ok()
            })
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum OpenCodeEvent {
    #[serde(rename = "session.message")]
    SessionMessage { session_id: String, content: String },
    
    #[serde(rename = "session.completed")]
    SessionCompleted { session_id: String },
    
    #[serde(rename = "task.status_changed")]
    TaskStatusChanged { task_id: String, status: String },
    
    // ... další eventy
}
```

### 2.4 Task Orchestrator

**crates/orchestrator/src/executor.rs:**

```rust
use crate::state_machine::TaskStateMachine;
use core::domain::{Task, TaskStatus};
use opencode::OpenCodeClient;

pub struct TaskExecutor {
    opencode: OpenCodeClient,
    state_machine: TaskStateMachine,
}

impl TaskExecutor {
    pub async fn execute_phase(&self, task: &mut Task) -> Result<(), Error> {
        match task.status {
            TaskStatus::Todo => {
                self.transition_to_planning(task).await?;
            }
            TaskStatus::Planning => {
                self.run_planning_session(task).await?;
            }
            TaskStatus::InProgress => {
                self.run_implementation_session(task).await?;
            }
            TaskStatus::AiReview => {
                self.run_review_session(task).await?;
            }
            _ => {}
        }
        Ok(())
    }

    async fn run_planning_session(&self, task: &mut Task) -> Result<(), Error> {
        // 1. Create OpenCode session
        let session = self.opencode.create_session(Some(task.title.clone())).await?;
        
        // 2. Send planning prompt
        let prompt = format!(
            "Analyze this task and create a technical implementation plan:\n\n\
             Title: {}\n\
             Description: {}\n\n\
             Output the plan to: .opencode-studio/kanban/plans/{}.md",
            task.title, task.description, task.id
        );
        
        self.opencode.send_prompt(&session.id, &prompt, None).await?;
        
        // 3. Transition to next state
        self.state_machine.transition(task, TaskStatus::PlanningReview)?;
        
        Ok(())
    }
}
```

### 2.5 API Extensions

**Nové endpoints:**

| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/tasks/:id/transition` | Změna stavu tasku |
| POST | `/api/tasks/:id/execute` | Spustit aktuální fázi |
| GET | `/api/sessions` | List OpenCode sessions |
| GET | `/api/sessions/:id` | Session detail |
| GET | `/api/sessions/:id/messages` | Session messages |

### 2.6 Deliverables

- [ ] OpenCode SDK (generované typy nebo ruční wrapper)
- [ ] SSE event stream handling
- [ ] Task executor s phase logic
- [ ] State machine pro task transitions
- [ ] Session tracking v DB
- [ ] API endpoints pro sessions

### 2.7 Acceptance Criteria

```bash
# Task transition works
curl -X POST http://localhost:3001/api/tasks/{id}/transition \
  -H "Content-Type: application/json" \
  -d '{"status": "Planning"}'

# Execute phase triggers OpenCode
curl -X POST http://localhost:3001/api/tasks/{id}/execute
# => Creates OpenCode session, sends prompt
```

---

## Phase 3: VCS & Workspace Management (2 týdny)

### Cíl
Integrace s Jujutsu/Git, workspace lifecycle, init/cleanup scripty.

### 3.1 VCS Crate

```
crates/
└── vcs/
    ├── Cargo.toml
    └── src/
        ├── lib.rs
        ├── traits.rs           # VersionControl trait
        ├── jj.rs               # Jujutsu implementation
        ├── git.rs              # Git fallback
        └── workspace.rs        # Workspace management
```

**crates/vcs/src/traits.rs:**

```rust
use async_trait::async_trait;
use std::path::PathBuf;

#[async_trait]
pub trait VersionControl: Send + Sync {
    /// Create isolated workspace for a task
    async fn create_workspace(&self, task_id: &str) -> Result<Workspace, Error>;
    
    /// Get diff of changes in workspace
    async fn get_diff(&self, workspace: &Workspace) -> Result<String, Error>;
    
    /// Merge workspace changes back to main
    async fn merge_workspace(&self, workspace: &Workspace) -> Result<MergeResult, Error>;
    
    /// Clean up workspace
    async fn cleanup_workspace(&self, workspace: &Workspace) -> Result<(), Error>;
    
    /// List all active workspaces
    async fn list_workspaces(&self) -> Result<Vec<Workspace>, Error>;
    
    /// Get conflicts in workspace
    async fn get_conflicts(&self, workspace: &Workspace) -> Result<Vec<ConflictFile>, Error>;
}

pub struct Workspace {
    pub task_id: String,
    pub path: PathBuf,
    pub branch_name: String,
    pub created_at: DateTime<Utc>,
}

pub enum MergeResult {
    Success,
    Conflicts(Vec<ConflictFile>),
}
```

**crates/vcs/src/jj.rs:**

```rust
use tokio::process::Command;

pub struct JujutsuVcs {
    repo_path: PathBuf,
    workspace_base: PathBuf,
}

#[async_trait]
impl VersionControl for JujutsuVcs {
    async fn create_workspace(&self, task_id: &str) -> Result<Workspace, Error> {
        let workspace_path = self.workspace_base.join(format!("task-{}", task_id));
        
        // jj new main -m "task-{id}: Start implementation"
        Command::new("jj")
            .args(["new", "main", "-m", &format!("task-{}: Start", task_id)])
            .current_dir(&self.repo_path)
            .output()
            .await?;
        
        // jj workspace add <path> --revision @
        Command::new("jj")
            .args([
                "workspace", "add",
                workspace_path.to_str().unwrap(),
                "--revision", "@"
            ])
            .current_dir(&self.repo_path)
            .output()
            .await?;
        
        Ok(Workspace {
            task_id: task_id.into(),
            path: workspace_path,
            branch_name: format!("task-{}", task_id),
            created_at: Utc::now(),
        })
    }

    async fn get_diff(&self, workspace: &Workspace) -> Result<String, Error> {
        let output = Command::new("jj")
            .args(["diff"])
            .current_dir(&workspace.path)
            .output()
            .await?;
        
        Ok(String::from_utf8_lossy(&output.stdout).into())
    }

    async fn cleanup_workspace(&self, workspace: &Workspace) -> Result<(), Error> {
        // jj workspace forget <name>
        Command::new("jj")
            .args(["workspace", "forget", &workspace.task_id])
            .current_dir(&self.repo_path)
            .output()
            .await?;
        
        // Remove directory
        tokio::fs::remove_dir_all(&workspace.path).await?;
        
        Ok(())
    }
}
```

### 3.2 Workspace Lifecycle

```rust
// crates/orchestrator/src/workspace_manager.rs

pub struct WorkspaceManager {
    vcs: Arc<dyn VersionControl>,
    config: WorkspaceConfig,
}

impl WorkspaceManager {
    pub async fn setup_workspace(&self, task: &Task) -> Result<Workspace, Error> {
        // 1. Create VCS workspace
        let workspace = self.vcs.create_workspace(&task.id.to_string()).await?;
        
        // 2. Run init scripts
        self.run_init_scripts(&workspace).await?;
        
        // 3. Copy/symlink files
        self.setup_files(&workspace).await?;
        
        Ok(workspace)
    }

    async fn run_init_scripts(&self, workspace: &Workspace) -> Result<(), Error> {
        for script in &self.config.init_scripts {
            Command::new("bash")
                .args([script, workspace.path.to_str().unwrap()])
                .output()
                .await?;
        }
        Ok(())
    }

    pub async fn cleanup_workspace(&self, workspace: &Workspace) -> Result<(), Error> {
        // 1. Run cleanup scripts
        for script in &self.config.cleanup_scripts {
            Command::new("bash")
                .args([script, workspace.path.to_str().unwrap()])
                .output()
                .await?;
        }
        
        // 2. Cleanup VCS
        self.vcs.cleanup_workspace(workspace).await?;
        
        Ok(())
    }
}
```

### 3.3 Deliverables

- [ ] VCS trait + Jujutsu implementation
- [ ] Git fallback implementation
- [ ] Workspace manager
- [ ] Init/cleanup script runner
- [ ] API endpoints pro workspaces

### 3.4 Acceptance Criteria

```bash
# Create workspace for task
curl -X POST http://localhost:3001/api/tasks/{id}/workspace

# List workspaces
curl http://localhost:3001/api/workspaces

# Get diff
curl http://localhost:3001/api/workspaces/{id}/diff
```

---

## Phase 4: WebSocket & Real-time (1-2 týdny)

### Cíl
Real-time updates pro frontend, WebSocket integration.

### 4.1 WebSocket Handler

```rust
// crates/api/src/websocket/mod.rs

use axum::extract::ws::{WebSocket, WebSocketUpgrade};
use tokio::sync::broadcast;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/ws", get(websocket_handler))
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    let mut rx = state.event_bus.subscribe();
    
    loop {
        tokio::select! {
            // Broadcast events to client
            Ok(event) = rx.recv() => {
                let json = serde_json::to_string(&event).unwrap();
                if socket.send(Message::Text(json)).await.is_err() {
                    break;
                }
            }
            
            // Handle client messages
            Some(Ok(msg)) = socket.recv() => {
                // Handle ping/pong, commands
            }
        }
    }
}
```

### 4.2 Event Types

```rust
// crates/core/src/events/mod.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    // Task events
    #[serde(rename = "task.created")]
    TaskCreated { task: Task },
    
    #[serde(rename = "task.status_changed")]
    TaskStatusChanged { task_id: Uuid, from: TaskStatus, to: TaskStatus },
    
    #[serde(rename = "task.completed")]
    TaskCompleted { task_id: Uuid },
    
    // Session events
    #[serde(rename = "session.started")]
    SessionStarted { session_id: String, task_id: Uuid },
    
    #[serde(rename = "session.message")]
    SessionMessage { session_id: String, content: String },
    
    #[serde(rename = "session.completed")]
    SessionCompleted { session_id: String },
    
    // Workspace events
    #[serde(rename = "workspace.created")]
    WorkspaceCreated { task_id: Uuid, path: String },
    
    #[serde(rename = "workspace.deleted")]
    WorkspaceDeleted { task_id: Uuid },
}
```

### 4.3 Deliverables

- [ ] WebSocket handler v Axum
- [ ] Event bus (tokio::sync::broadcast)
- [ ] Event types definované
- [ ] Bridge mezi OpenCode SSE a naším WS

---

## Phase 5: Full Kanban Flow (2-3 týdny)

### Cíl
Kompletní TODO → DONE flow s AI planning, implementation, review.

### 5.1 Phase Prompts

```rust
// crates/orchestrator/src/prompts.rs

pub struct PhasePrompts;

impl PhasePrompts {
    pub fn planning(task: &Task) -> String {
        format!(r#"
You are analyzing a development task. Create a detailed implementation plan.

## Task
**Title:** {title}
**Description:** {description}

## Required Output
Save your analysis to: `.opencode-studio/kanban/plans/{id}.md`

The plan should include:
1. Technical analysis
2. Files to modify/create
3. Step-by-step implementation steps
4. Potential risks
5. Estimated complexity (S/M/L/XL)

Do NOT implement anything yet. Only create the plan.
"#, title = task.title, description = task.description, id = task.id)
    }

    pub fn implementation(task: &Task, plan_path: &str) -> String {
        format!(r#"
Implement the following task according to the plan.

## Task
**Title:** {title}
**Plan:** Read from `{plan_path}`

## Instructions
1. Read the plan carefully
2. Implement each step
3. Write tests if applicable
4. Commit your changes

Start implementation now.
"#, title = task.title, plan_path = plan_path)
    }

    pub fn review(task: &Task, diff: &str) -> String {
        format!(r#"
Review the following code changes for task: {title}

## Diff
```
{diff}
```

## Review Criteria
1. Code quality and style
2. Correctness - does it solve the task?
3. Tests - are they adequate?
4. Security concerns
5. Breaking changes

## Output
Save your review to: `.opencode-studio/kanban/reviews/{id}.md`

If approved, respond with: APPROVED
If changes needed, respond with: CHANGES_REQUESTED and explain what needs fixing.
"#, title = task.title, diff = diff, id = task.id)
    }
}
```

### 5.2 Full Flow Implementation

```rust
// crates/orchestrator/src/executor.rs

impl TaskExecutor {
    pub async fn run_full_cycle(&self, task: &mut Task) -> Result<(), Error> {
        // TODO -> PLANNING
        self.transition(task, TaskStatus::Planning).await?;
        self.run_planning_session(task).await?;
        
        // PLANNING -> PLANNING_REVIEW (optional approval)
        if self.config.require_plan_approval {
            self.transition(task, TaskStatus::PlanningReview).await?;
            // Wait for human approval...
            return Ok(());
        }
        
        // -> IN_PROGRESS
        self.transition(task, TaskStatus::InProgress).await?;
        self.setup_workspace(task).await?;
        self.run_implementation_session(task).await?;
        
        // -> AI_REVIEW
        self.transition(task, TaskStatus::AiReview).await?;
        let review_result = self.run_ai_review(task).await?;
        
        match review_result {
            ReviewResult::Approved => {
                self.transition(task, TaskStatus::Review).await?;
            }
            ReviewResult::ChangesRequested(feedback) => {
                // Loop back to implementation
                self.transition(task, TaskStatus::InProgress).await?;
                self.run_implementation_with_feedback(task, &feedback).await?;
            }
        }
        
        Ok(())
    }
}
```

### 5.3 Deliverables

- [ ] Planning phase implementation
- [ ] Implementation phase
- [ ] AI Review phase
- [ ] Human Review support
- [ ] Retry logic pro failed reviews
- [ ] Plan/Review file management

---

## Phase 6: GitHub Integration (2 týdny)

### Cíl
PR creation, CI status, issue sync.

### 6.1 GitHub Crate

```
crates/
└── github/
    ├── Cargo.toml
    └── src/
        ├── lib.rs
        ├── client.rs           # octocrab wrapper
        ├── pr.rs               # PR operations
        └── issues.rs           # Issue sync
```

### 6.2 Deliverables

- [ ] GitHub client (octocrab)
- [ ] Auto PR creation
- [ ] CI status polling
- [ ] Issue import

---

## Phase 7: Frontend Integration (1-2 týdny)

### Cíl
TypeScript types generování, frontend připojení na Rust API.

### 7.1 Type Generation

```rust
// crates/api/src/bin/generate_types.rs

use ts_rs::TS;

fn main() {
    // Generate TypeScript types
    Task::export_all().unwrap();
    TaskStatus::export_all().unwrap();
    Session::export_all().unwrap();
    Event::export_all().unwrap();
    // ...
}
```

```bash
# Generate types
cargo run --bin generate-types

# Output: frontend/src/types/generated.ts
```

### 7.2 Deliverables

- [ ] ts-rs setup pro všechny typy
- [ ] Generated types v frontend/
- [ ] React Query hooks pro API calls
- [ ] WebSocket hook pro real-time

---

## Timeline Summary

| Phase | Trvání | Závislosti |
|-------|--------|------------|
| **Phase 1: Foundation** | 2-3 týdny | - |
| **Phase 2: OpenCode** | 2-3 týdny | Phase 1 |
| **Phase 3: VCS** | 2 týdny | Phase 1 |
| **Phase 4: WebSocket** | 1-2 týdny | Phase 1, 2 |
| **Phase 5: Full Kanban** | 2-3 týdny | Phase 2, 3, 4 |
| **Phase 6: GitHub** | 2 týdny | Phase 5 |
| **Phase 7: Frontend** | 1-2 týdny | Phase 1-5 |

**Celkem: 12-17 týdnů** (s paralelizací některých fází možné zkrátit na 10-12)

---

## Risk Mitigation

| Risk | Pravděpodobnost | Dopad | Mitigace |
|------|-----------------|-------|----------|
| OpenCode API změny | Střední | Vysoký | Abstrakce, fallback na ACP |
| Jujutsu learning curve | Nízká | Střední | Git fallback připraven |
| SQLite scaling limits | Nízká | Střední | Architektura ready pro PostgreSQL |
| Frontend/Backend type drift | Střední | Střední | ts-rs automatické generování |

---

## Next Steps

1. **Založit crates strukturu** - Cargo workspace setup
2. **Core domain models** - Task, Session entities
3. **Basic API** - CRUD endpoints
4. **OpenCode SDK** - Stáhnout spec, generovat typy

---

*Dokument bude aktualizován po každé dokončené fázi.*
