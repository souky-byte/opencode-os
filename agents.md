# OpenCode Studio - Agent Context

> Tento dokument slouzi jako kontext pro AI agenty pracujici na projektu.
> Posledni aktualizace: 2025-12-30

---

## 1. Co je OpenCode Studio

**OpenCode Studio** je autonomni AI-powered development platform, ktera orchestruje OpenCode sessions pro automatizovany vyvoj software.

### Klicove principy
- **Autonomie**: Minimalni lidska intervence behem vyvoje
- **Transparentnost**: Komunikace pres soubory (plany, reviews, roadmapa)
- **Modularita**: Plugovatelne moduly pro ruzne AI-powered funkce
- **Skalovatnost**: Paralelni beh vice agentu

### Dvouvrstva architektura
```
ROADMAP (produktova vrstva) - "Co a proc"
    │
    │  [Presunout do vyvoje]
    ▼
KANBAN (implementacni vrstva) - "Jak"
```

---

## 2. Task Lifecycle (State Machine)

```
TODO → PLANNING → PLANNING_REVIEW → IN_PROGRESS → AI_REVIEW → REVIEW → DONE
       (AI plan)   (optional)        (OpenCode)    (AI check)  (human)
```

### Prechody stavu
| From | Allowed To |
|------|------------|
| Todo | Planning |
| Planning | PlanningReview, Todo |
| PlanningReview | InProgress, Planning |
| InProgress | AiReview, PlanningReview |
| AiReview | Review, InProgress |
| Review | Done, InProgress |
| Done | (terminal) |

### Session Strategy
Kazda faze = vlastni OpenCode session, komunikace pres soubory:

| Faze | Input | Output |
|------|-------|--------|
| PLANNING | task description | `plans/{id}.md` |
| IN_PROGRESS | plan | kod ve workspace |
| AI_REVIEW | diff, task | `reviews/{id}.md` |

---

## 3. Crates Architecture

```
crates/
├── core/           # Domain models, traits (NO I/O)
│   └── domain/     # Task, Session, TaskStatus
├── db/             # SQLite persistence (sqlx)
│   ├── models/     # DB models
│   └── repositories/  # TaskRepository, SessionRepository
├── opencode/       # OpenCode HTTP client
│   ├── client.rs   # OpenCodeClient (create_session, send_message, etc.)
│   ├── types.rs    # Session, Message, SendMessageRequest
│   └── events.rs   # SSE EventStream, OpenCodeEvent
├── orchestrator/   # Task lifecycle, scheduling
│   ├── executor.rs     # TaskExecutor (execute_phase, run_planning_session, etc.)
│   ├── state_machine.rs # TaskStateMachine (validate_transition)
│   ├── prompts.rs      # PhasePrompts (planning, implementation, review)
│   └── files.rs        # FileManager (plans/reviews in .opencode-studio/kanban/)
├── events/         # Event system
│   ├── types.rs    # Event, EventEnvelope, AgentMessageData, ToolExecutionData
│   └── bus.rs      # EventBus (tokio::sync::broadcast)
├── websocket/      # WebSocket real-time updates
│   ├── handler.rs  # ws_handler, WsState
│   └── messages.rs # ClientMessage, ServerMessage, SubscriptionFilter
├── vcs/            # Version control (jj, git)
│   ├── traits.rs   # VersionControl trait, Workspace, MergeResult
│   ├── jj.rs       # Jujutsu implementation
│   ├── git.rs      # Git fallback
│   └── workspace.rs # WorkspaceManager, WorkspaceConfig
├── server/         # Axum HTTP server
│   └── routes/     # health, tasks, sessions, workspaces, ws
└── github/         # GitHub integration (octocrab)
    ├── client.rs   # GitHubClient (create_pull_request, merge, get_ci_status, import_issue)
    ├── types.rs    # PullRequest, Issue, CiStatus, CiState, CreatePrRequest, RepoConfig
    └── error.rs    # GitHubError
```

### Dependency Graph
```
         server
            │
  ┌─────────┼─────────┐
  │         │         │
orchestrator db    opencode
  │         │         │
  └─────────┼─────────┘
            │
          core
```

---

## 4. Current API Endpoints

### Tasks
| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/tasks` | List all tasks |
| POST | `/api/tasks` | Create task |
| GET | `/api/tasks/{id}` | Get task detail |
| PATCH | `/api/tasks/{id}` | Update task |
| DELETE | `/api/tasks/{id}` | Delete task |
| POST | `/api/tasks/{id}/transition` | Change task status |
| POST | `/api/tasks/{id}/execute` | Execute current phase |
| GET | `/api/tasks/{id}/sessions` | List sessions for task |

### Sessions
| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/sessions` | List all sessions |
| GET | `/api/sessions/{id}` | Get session detail |
| DELETE | `/api/sessions/{id}` | Delete session |

### Workspaces
| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/tasks/{id}/workspace` | Create workspace for task |
| GET | `/api/workspaces` | List all workspaces |
| GET | `/api/workspaces/{id}` | Get workspace status |
| GET | `/api/workspaces/{id}/diff` | Get workspace diff |
| POST | `/api/workspaces/{id}/merge` | Merge workspace |
| DELETE | `/api/workspaces/{id}` | Delete/cleanup workspace |

### WebSocket
| Method | Path | Description |
|--------|------|-------------|
| GET | `/ws` | WebSocket connection for real-time events |

### Health
| Method | Path | Description |
|--------|------|-------------|
| GET | `/health` | Health check |

---

## 5. OpenCode Integration

### OpenCodeClient methods
```rust
create_session(title: Option<String>) -> Session
get_session(session_id: &str) -> Session
list_sessions() -> Vec<Session>
send_message(session_id, prompt, model) -> MessageResponse
send_message_async(session_id, prompt) -> ()
abort_session(session_id) -> ()
get_messages(session_id) -> Vec<Message>
```

### SSE Events
```rust
enum OpenCodeEvent {
    SessionMessage { session_id, content }
    SessionCompleted { session_id }
    SessionError { session_id, error }
    TaskStatusChanged { task_id, status }
}
```

### EventStream usage
```rust
let stream = EventStream::new("http://localhost:4096");
let mut receiver = stream.connect().await?;
while let Some(event) = receiver.next_event().await {
    // handle event
}
```

---

## 6. Key Domain Types

### Task (core)
```rust
struct Task {
    id: Uuid,
    title: String,
    description: String,
    status: TaskStatus,
    roadmap_item_id: Option<Uuid>,
    workspace_path: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}
```

### TaskStatus (core)
```rust
enum TaskStatus {
    Todo,
    Planning,
    PlanningReview,
    InProgress,
    AiReview,
    Review,
    Done,
}
```

### Session (core)
```rust
struct Session {
    id: Uuid,
    task_id: Uuid,
    opencode_session_id: Option<String>,
    phase: SessionPhase,  // Planning, Implementation, Review
    status: SessionStatus,  // Pending, Running, Completed, Failed
    started_at: Option<DateTime<Utc>>,
    completed_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}
```

---

## 7. Implementation Phases

### Phase 1: Foundation ✅ DONE
- [x] Workspace setup (core, db, server crates)
- [x] Domain models (Task, Session, TaskStatus)
- [x] SQLite database s migraci
- [x] Basic CRUD API pro tasks
- [x] Health endpoint
- [x] Tracing/logging setup

### Phase 2: OpenCode Integration ✅ DONE
- [x] OpenCode SDK (HTTP client wrapper)
- [x] SSE event stream handling
- [x] Task executor s phase logic
- [x] State machine pro task transitions
- [x] Session tracking v DB
- [x] API endpoints pro sessions

### Phase 3: VCS & Workspace Management ✅ DONE
- [x] VCS trait + Jujutsu implementation
- [x] Git fallback implementation
- [x] Workspace manager
- [x] Init/cleanup script runner
- [x] API endpoints pro workspaces

### Phase 4: WebSocket & Real-time ✅ DONE
- [x] WebSocket handler v Axum
- [x] Event bus (tokio::sync::broadcast)
- [x] Event types (Task, Session, Workspace, Error events)
- [x] WebSocket route at /ws
- [x] Event emission from task routes

### Phase 5: Full Kanban Flow ✅ DONE
- [x] Planning phase implementation (run_planning_session with file output)
- [x] Implementation phase (run_implementation_session with workspace integration)
- [x] AI Review phase (run_ai_review with diff and review file output)
- [x] Human Review support (approve_plan, reject_plan, approve_review, reject_review)
- [x] Retry logic pro failed reviews (max_review_iterations enforcement)
- [x] File management module (plans/reviews in .opencode-studio/kanban/)
- [x] VCS WorkspaceManager integration for real diffs
- [x] EventBus integration for event emission
- [x] Session persistence to DB
- [x] run_full_cycle() for complete TODO→DONE automation

### Phase 6: GitHub Integration ✅ DONE
- [x] GitHub client (octocrab wrapper)
- [x] Auto PR creation (create_pull_request, merge_pull_request, close_pull_request)
- [x] CI status polling (get_ci_status, wait_for_ci, get_pr_ci_status)
- [x] Issue import (get_issue, list_issues, import_issue)

### Phase 7: Frontend Integration ✅ DONE
- [x] ts-rs setup pro vsechny typy
- [x] Generated types v frontend/src/types/generated/
- [x] OpenAPI spec (utoipa) + Swagger UI
- [x] React Query hooks (Orval generated)
- [x] WebSocket hook (`useStudioWebSocket`)

#### OpenAPI & SDK Generation
```bash
# Start server (required for OpenAPI spec)
DATABASE_URL=sqlite:./studio.db cargo run --package server --bin server

# Generate React Query SDK from OpenAPI
cd frontend && pnpm generate:api
```

**Available endpoints:**
- `GET /api/openapi.json` - OpenAPI 3.1 spec
- `GET /swagger-ui` - Swagger UI documentation

**Generated SDK location:** `frontend/src/api/generated/`
- 50 TypeScript files (hooks + models)
- React Query hooks: `useListTasks`, `useCreateTask`, `useGetTask`, `useUpdateTask`, `useDeleteTask`, `useTransitionTask`, `useExecuteTask`
- Sessions: `useListSessions`, `useGetSession`, `useDeleteSession`, `useListSessionsForTask`
- Workspaces: `useListWorkspaces`, `useGetWorkspaceStatus`, `useGetWorkspaceDiff`, `useMergeWorkspace`, `useDeleteWorkspace`, `useCreateWorkspaceForTask`
- Health: `useHealthCheck`

**Usage example:**
```tsx
import { useListTasks, useCreateTask } from '~/api/generated/tasks/tasks';
import { Task } from '~/api/generated/model';

function TaskList() {
  const { data: tasks, isLoading } = useListTasks();
  const createTask = useCreateTask();
  
  const handleCreate = () => {
    createTask.mutate({ data: { title: 'New Task', description: 'Description' } });
  };
  
  return <div>{tasks?.map(t => <div key={t.id}>{t.title}</div>)}</div>;
}
```

#### WebSocket Hook

**Location:** `frontend/src/hooks/useStudioWebSocket.ts`

Real-time event streaming from the backend. Auto-connects, handles reconnection with exponential backoff, and provides typed events.

**Features:**
- Auto-connect on mount
- Reconnection with exponential backoff (max 5 attempts)
- Ping/pong keep-alive (30s interval)
- Task-specific subscription filtering
- Fully typed events from `~/types/generated`

**Usage example:**
```tsx
import { useStudioWebSocket } from '~/hooks/useStudioWebSocket';
import type { Event, EventEnvelope } from '~/types/generated';

function TaskMonitor({ taskId }: { taskId: string }) {
  const { connectionState, isSubscribed } = useStudioWebSocket({
    taskIds: [taskId],
    onEvent: (event: Event, envelope: EventEnvelope) => {
      switch (event.type) {
        case 'task.status_changed':
          console.log(`Task ${event.task_id}: ${event.from_status} → ${event.to_status}`);
          break;
        case 'agent.message':
          console.log(`Agent: ${event.message.content}`);
          break;
        case 'session.ended':
          console.log(`Session ended: ${event.success ? 'success' : 'failed'}`);
          break;
      }
    },
    onConnectionChange: (state) => console.log('WS:', state),
  });

  return <div>Connection: {connectionState}, Subscribed: {isSubscribed}</div>;
}
```

**Environment variable:** `NEXT_PUBLIC_WS_URL` (default: `ws://localhost:3001/ws`)

### Phase 8: CLI Tool ✅ DONE
- [x] `opencode-studio` CLI binary
- [x] `init` command - initialize project
- [x] `serve` command - start server + open browser
- [x] `status` command - show project status
- [x] `/api/project` endpoint for project info

#### CLI Installation
```bash
cargo install --path crates/cli
```

#### CLI Usage
```bash
# Initialize a new project
cd /path/to/my-project
opencode-studio init

# Start the server (opens browser automatically)
opencode-studio

# Start server on custom port
opencode-studio --port 4000

# Start server without opening browser
opencode-studio serve --no-browser

# Check project status
opencode-studio status
```

#### Project Structure After Init
```
my-project/
├── .opencode-studio/
│   ├── config.toml       # Project configuration
│   ├── studio.db         # SQLite database
│   └── kanban/
│       ├── plans/        # Generated plans
│       └── reviews/      # AI reviews
├── .git/ or .jj/         # VCS (auto-detected)
└── ... your project files
```

#### config.toml Format
```toml
[project]
name = "my-project"

[server]
port = 3001
opencode_url = "http://localhost:4096"
frontend_url = "http://localhost:3000"
```

---

## 8. Tech Stack

| Layer | Technology |
|-------|------------|
| Runtime | Rust + Tokio |
| HTTP Server | Axum 0.8 |
| Database | SQLite + sqlx 0.8 |
| Serialization | serde + serde_json |
| Error handling | anyhow + thiserror |
| Logging | tracing |
| OpenCode | HTTP client (reqwest) + SSE (eventsource-stream) |
| GitHub | octocrab 0.41 |

---

## 9. File Structure

```
.opencode-studio/
├── config.toml
├── studio.db
├── kanban/
│   ├── tasks/{id}.md
│   ├── plans/{id}.md
│   └── reviews/{id}.md
├── roadmap/
│   ├── roadmap.md
│   └── items/{id}.md
├── scripts/
│   ├── workspace-init.sh
│   └── workspace-cleanup.sh
└── sessions/
    └── {module}_{timestamp}.log
```

---

## 10. Running the Project

```bash
# Run all tests
cargo test --workspace

# Run server
DATABASE_URL=sqlite:./studio.db cargo run --package server

# Server runs on http://localhost:3001
```

### Environment Variables
| Variable | Default | Description |
|----------|---------|-------------|
| DATABASE_URL | sqlite:./studio.db | SQLite connection |
| OPENCODE_URL | http://localhost:4096 | OpenCode server URL |
| PORT | 3001 | Server port |

---

## 11. Coding Conventions

1. **Crate naming**: Avoid reserved names (`core` → `opencode_core`)
2. **Error handling**: Use `thiserror` for custom errors, `anyhow` for application errors
3. **Parsing**: Use `str.parse()` instead of `FromStr::from_str()`
4. **Type safety**: Never use `as any`, `@ts-ignore`, `@ts-expect-error`
5. **Tests**: Each module should have unit tests

---

## 12. Test Coverage

| Crate | Tests | Status |
|-------|-------|--------|
| db | 10 | ✅ |
| events | 12 | ✅ |
| github | 11 | ✅ |
| opencode | 2 | ✅ |
| opencode_core | 8 | ✅ |
| orchestrator | 24 | ✅ |
| vcs | 12 | ✅ |
| websocket | 9 | ✅ |
| server (integration) | 31 | ✅ |
| **Total** | **119** | ✅ All passing |

### Server Integration Tests Breakdown
- **L1 API Tests (14)**: CRUD, transitions, health, sessions, validation
- **L2 Flow Tests (4)**: Kanban flow, planning phase, session DB, auto-transitions
- **L3 Workspace Tests (6)**: Create, list, status, diff, delete, not-found (git worktree)
- **L3 E2E jj Tests (5)**: Create, multiple, cleanup, diff, status (real jj workspaces)
- **Additional (2)**: Health, workspaces empty list
