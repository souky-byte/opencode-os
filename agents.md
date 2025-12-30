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
│   └── prompts.rs      # PhasePrompts (planning, implementation, review)
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
└── github/         # [Phase 6] GitHub integration
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

### Phase 5: Full Kanban Flow 🔜 NEXT
- [ ] Planning phase implementation
- [ ] Implementation phase
- [ ] AI Review phase
- [ ] Human Review support
- [ ] Retry logic pro failed reviews

### Phase 6: GitHub Integration
- [ ] GitHub client (octocrab)
- [ ] Auto PR creation
- [ ] CI status polling
- [ ] Issue import

### Phase 7: Frontend Integration
- [ ] ts-rs setup pro vsechny typy
- [ ] Generated types v frontend/
- [ ] React Query hooks
- [ ] WebSocket hook

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
| opencode | 2 | ✅ |
| opencode_core | 8 | ✅ |
| orchestrator | 7 | ✅ |
| vcs | 12 | ✅ |
| websocket | 9 | ✅ |
| server | 0 | (no tests yet) |
| **Total** | **60** | ✅ All passing |
