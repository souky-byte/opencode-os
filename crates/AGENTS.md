# OpenCode Studio: Crates

## OVERVIEW

9-crate Rust workspace. Core domain logic, persistence, orchestration, and HTTP server.

## CRATE MAP

| Crate | Purpose | Key Exports | Tests |
|:------|:--------|:------------|:------|
| `core` | Domain models (NO I/O) | `Task`, `Session`, `TaskStatus`, `SessionPhase` | 10 |
| `db` | SQLite persistence (sqlx) | `TaskRepository`, `SessionRepository`, `create_pool` | 12 |
| `opencode-client` | OpenAPI-generated SDK | `apis::DefaultApi`, `Configuration` | 0 |
| `orchestrator` | Task lifecycle engine | `TaskExecutor`, `TaskStateMachine`, `FileManager` | 36 |
| `vcs` | VCS abstraction | `VersionControl`, `WorkspaceManager`, `JujutsuVcs`, `GitVcs` | 20 |
| `events` | Event bus | `EventBus`, `TaskEvent`, `SessionEvent` | 8 |
| `github` | GitHub API (octocrab) | `GitHubClient`, `PullRequest`, `Issue` | 11 |
| `server` | Axum HTTP + SSE | `AppState`, `router`, `OpenApi` | 12 |
| `cli` | Binary: `opencode-studio` | Commands: init, serve, status, update | 0 |

## DEPENDENCY GRAPH

```
server (aggregates all)
├── orchestrator → core, opencode-client, db, vcs, events
├── db → core
├── vcs → core
├── github → core
└── cli → db, server (uses path deps - tech debt)

Foundational (no internal deps): core, events, opencode-client
```

## WHERE TO LOOK

| Task | Crate | File |
|:-----|:------|:-----|
| Add domain type | `core` | `src/domain/mod.rs` |
| Add DB table | `db` | `migrations/*.sql` + `src/models/` + `src/repositories/` |
| Add API route | `server` | `src/routes/` + update `src/lib.rs` OpenAPI |
| Task state transitions | `orchestrator` | `src/state_machine.rs` |
| AI prompts | `orchestrator` | `src/prompts.rs` |
| VCS operations | `vcs` | `src/jj.rs` (primary), `src/git.rs` (fallback) |
| Event emission | `events` | `src/types.rs` for new event types |
| GitHub integration | `github` | `src/client.rs` |

## CONVENTIONS

- `core` exports as `opencode_core` (reserved word collision)
- Crate errors: `thiserror` with dedicated `error.rs`
- Workspace deps in root `Cargo.toml`, not per-crate
- All domain mutations through `orchestrator` state machine
- Inline tests: `#[cfg(test)] mod tests { ... }`

## ANTI-PATTERNS

- Direct `Task`/`Session` status mutation (use state machine)
- I/O in `core` crate (pure domain only)
- `unwrap()` in non-test code (use `?` or explicit error)
- `len() > 0` (use `!is_empty()`)

## TEST COMMANDS

```bash
cargo test --workspace                    # All 109 tests
cargo test -p orchestrator               # Single crate
cargo test -p server -- --nocapture      # With output
```
