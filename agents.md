# OpenCode Studio

**Generated:** 2025-12-31 | **Commit:** 1f74380 | **Branch:** ui

## OVERVIEW

AI-powered development orchestration platform. Rust backend (Axum + SQLite) + React frontend. Automates task lifecycle TODO→DONE via OpenCode sessions.

## STRUCTURE

```
opencode-os/
├── crates/           # Rust workspace (9 crates) → see crates/AGENTS.md
│   ├── server/       # Axum HTTP + SSE (entry: main.rs)
│   ├── cli/          # opencode-studio CLI binary
│   ├── orchestrator/ # Task lifecycle, state machine, prompts
│   ├── opencode-client/ # OpenAPI-generated OpenCode SDK
│   ├── db/           # SQLite persistence (sqlx)
│   ├── vcs/          # Jujutsu/Git abstraction
│   ├── events/       # Event bus (tokio::broadcast)
│   ├── github/       # GitHub API (octocrab)
│   └── core/         # Domain models (NO I/O) → exports as opencode_core
├── frontend/         # React + Vite → see frontend/AGENTS.md
├── docs/             # Architecture docs
└── .opencode-studio/ # Runtime: config.toml, studio.db, kanban/{plans,reviews}
```

## WHERE TO LOOK

| Task | Location | Notes |
|------|----------|-------|
| Add API endpoint | `crates/server/src/routes/` | Add to lib.rs OpenAPI schema |
| Task state logic | `crates/orchestrator/src/state_machine.rs` | TaskStateMachine |
| OpenCode integration | `crates/orchestrator/src/executor.rs` | Thin orchestrator, delegates to services |
| Phase services | `crates/orchestrator/src/services/` | planning, implementation, review, fix phases |
| VCS operations | `crates/vcs/src/` | jj.rs primary, git.rs fallback |
| Frontend component | `frontend/src/components/` | Feature dirs: kanban/, sessions/, task-detail/ |
| Generated types | `frontend/src/types/generated/` | ts-rs from Rust |
| Generated API hooks | `frontend/src/api/generated/` | Orval from OpenAPI |
| Domain models | `crates/core/src/domain/` | Task, Session, TaskStatus |
| DB migrations | `crates/db/migrations/` | SQLite schema |
| AI prompts | `crates/orchestrator/src/prompts.rs` | Planning/review prompts |

## TASK LIFECYCLE

```
TODO → PLANNING → PLANNING_REVIEW → IN_PROGRESS → AI_REVIEW → REVIEW → DONE
       (AI plan)   (optional)        (OpenCode)    (AI check)  (human)
```

Each phase = separate OpenCode session. Files in `.opencode-studio/kanban/`.

## KEY TYPES

```rust
// crates/core/src/domain/task.rs
struct Task { id, title, description, status: TaskStatus, roadmap_item_id, workspace_path }
enum TaskStatus { Todo, Planning, PlanningReview, InProgress, AiReview, Review, Done }

// crates/core/src/domain/session.rs  
struct Session { id, task_id, opencode_session_id, phase: SessionPhase, status: SessionStatus }
enum SessionPhase { Planning, Implementation, Review }
enum SessionStatus { Pending, Running, Completed, Failed, Aborted }
```

## DEPENDENCY GRAPH

```
cli → db, server
server → core, opencode_client, orchestrator, db, vcs, events, github
orchestrator → core, opencode_client, db, vcs, events
github, vcs, db → core
events, opencode_client, core → (no internal deps)
```

## API SURFACE

25+ REST endpoints across 8 modules:
- `/api/tasks` - Task CRUD + lifecycle transitions
- `/api/sessions` - Session management + activity SSE
- `/api/workspaces` - VCS workspace operations
- `/api/projects` - Project open/init/validate
- `/api/events` - Global SSE event stream
- `/api/openapi.json` - Generated spec
- `/swagger-ui` - Interactive docs

## CONVENTIONS

- **Crate naming**: `core` → `opencode_core` (reserved word)
- **Error handling**: `thiserror` for crate errors, `anyhow` for app errors
- **Parsing**: `str.parse()` not `FromStr::from_str()`
- **Type safety**: NEVER `as any`, `@ts-ignore`, `@ts-expect-error`
- **Biome**: Tabs, 100 char lines, double quotes, trailing commas
- **CI**: `clippy -D warnings`, `cargo fmt --check`

## ANTI-PATTERNS

- Empty catch blocks (`noEmptyBlockStatements: warn`)
- Barrel files (`noBarrelFile` + `noReExportAll` enabled)
- Floating promises (`noFloatingPromises: error`)
- `console.log` in prod (`noConsole: warn`)
- Hardcoded secrets (`noSecrets: error`)
- Path deps instead of workspace deps (known tech debt in cli crate)

## COMMANDS

```bash
# Dev (backend + frontend concurrent)
pnpm dev

# Backend only
DATABASE_URL=sqlite:./studio.db cargo run --package server

# Frontend only
cd frontend && pnpm dev

# Tests
cargo test --workspace
cargo test -p orchestrator  # 55 tests
cargo clippy --workspace --all-features -- -D warnings

# Generate frontend SDK
cd frontend && pnpm generate:api

# CLI
cargo install --path crates/cli
opencode-studio init    # Initialize project
opencode-studio serve   # Start server
```

## ENV VARIABLES

| Variable | Default | Description |
|----------|---------|-------------|
| DATABASE_URL | sqlite:./studio.db | SQLite connection |
| OPENCODE_URL | http://localhost:4096 | OpenCode server |
| PORT | 3001 | Backend port |

## NOTES

- **OpenAPI**: `/api/openapi.json`, Swagger at `/swagger-ui`
- **SSE**: `/api/events` (global), `/api/sessions/{id}/activity` (per-session)
- **VCS**: Jujutsu preferred, Git fallback auto-detected
- **Type gen**: ts-rs (Rust→TS), Orval (OpenAPI→React Query)
- **Test pattern**: Inline `#[cfg(test)]` modules, no separate test dirs
