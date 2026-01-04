# OpenCode Studio: Crates

## OVERVIEW

11-crate Rust workspace. Core domain logic, persistence, orchestration, wiki generation, and HTTP server.

## CRATE MAP

| Crate | Purpose | Key Exports | Tests |
|:------|:--------|:------------|:------|
| `core` | Domain models (NO I/O) | `Task`, `Session`, `TaskStatus`, `SessionPhase` | 10 |
| `db` | SQLite persistence (sqlx) | `TaskRepository`, `SessionRepository`, `create_pool` | 12 |
| `opencode-client` | OpenAPI-generated SDK | `apis::DefaultApi`, `Configuration` | 0 |
| `orchestrator` | Task lifecycle engine | `TaskExecutor`, `TaskStateMachine`, `services::*` | 55 |
| `vcs` | VCS abstraction | `VersionControl`, `WorkspaceManager`, `JujutsuVcs`, `GitVcs` | 20 |
| `events` | Event bus | `EventBus`, `TaskEvent`, `SessionEvent` | 8 |
| `github` | GitHub API (octocrab) | `GitHubClient`, `PullRequest`, `Issue` | 11 |
| `wiki` | AI wiki generation | `WikiEngine`, `WikiSyncService`, `CodeIndexer`, `RagEngine` | 64 |
| `mcp-wiki` | MCP server for wiki | `WikiService`, tools: search_code, ask_codebase | 7 |
| `server` | Axum HTTP + SSE | `AppState`, `router`, `OpenApi` | 20 |
| `cli` | Binary: `opencode-studio` | Commands: init, serve, status, update | 0 |

## DEPENDENCY GRAPH

```
server (aggregates all)
├── orchestrator → core, opencode-client, db, vcs, events, wiki
├── db → core
├── vcs → core
├── github → core
├── wiki → (external: rusqlite, sqlite-vec, reqwest, tiktoken-rs)
├── mcp-wiki → wiki
└── cli → db, server (uses path deps - tech debt)

Foundational (no internal deps): core, events, opencode-client, wiki
```

## WHERE TO LOOK

| Task | Crate | File |
|:-----|:------|:-----|
| Add domain type | `core` | `src/domain/mod.rs` |
| Add DB table | `db` | `migrations/*.sql` + `src/models/` + `src/repositories/` |
| Add API route | `server` | `src/routes/` + update `src/lib.rs` OpenAPI |
| Task state transitions | `orchestrator` | `src/state_machine.rs` |
| AI prompts | `orchestrator` | `src/prompts.rs` |
| Planning phase | `orchestrator` | `src/services/planning_phase.rs` |
| Implementation phase | `orchestrator` | `src/services/implementation_phase.rs` |
| Review phase | `orchestrator` | `src/services/review_phase.rs` |
| Fix phase | `orchestrator` | `src/services/fix_phase.rs` |
| OpenCode API calls | `orchestrator` | `src/services/opencode_client.rs` |
| Message parsing | `orchestrator` | `src/services/message_parser.rs` |
| VCS operations | `vcs` | `src/jj.rs` (primary), `src/git.rs` (fallback) |
| Event emission | `events` | `src/types.rs` for new event types |
| GitHub integration | `github` | `src/client.rs` |
| Wiki indexing | `wiki` | `src/indexer/mod.rs` |
| Wiki generation | `wiki` | `src/generator/mod.rs` |
| Wiki RAG Q&A | `wiki` | `src/rag/mod.rs` |
| Wiki sync | `wiki` | `src/sync.rs` |
| Wiki MCP tools | `mcp-wiki` | `src/lib.rs` |

## ORCHESTRATOR SERVICES

The `orchestrator` crate uses a modular service architecture in `src/services/`:

| Service | Purpose | Lines |
|:--------|:--------|------:|
| `executor_context.rs` | Shared context, config, transitions, persistence | 243 |
| `planning_phase.rs` | Planning phase execution | 136 |
| `implementation_phase.rs` | Implementation + phased execution | 646 |
| `review_phase.rs` | AI review with JSON fallback | 353 |
| `fix_phase.rs` | Fix iteration handling | 269 |
| `opencode_client.rs` | OpenCode session/prompt API | 210 |
| `message_parser.rs` | SSE parsing, ReviewResult extraction | 327 |
| `mcp_manager.rs` | MCP server lifecycle | 108 |

The main `executor.rs` (~530 lines) delegates to these services.

## WIKI CRATE

AI-powered codebase documentation with semantic search. Uses rusqlite + sqlite-vec (NOT sqlx).

### Architecture

```
wiki/src/
├── domain/           # Data models: CodeChunk, WikiPage, IndexStatus
├── openrouter/       # OpenRouter API client with retry logic
├── vector_store/     # SQLite + sqlite-vec for embeddings
├── chunker/          # Token-based text splitting
├── indexer/          # File reading, chunking, embedding creation
├── generator/        # Wiki page generation with Mermaid diagrams
├── rag/              # RAG engine for Q&A
└── sync.rs           # WikiSyncService for auto-sync
```

### Key Types

| Type | Purpose |
|:-----|:--------|
| `WikiConfig` | Configuration: branches, API key, models, db_path |
| `WikiEngine` | Main orchestrator for indexing and search |
| `WikiSyncService` | Sync service with needs_reindex, sync_if_needed |
| `CodeIndexer` | Creates embeddings, stores chunks |
| `WikiGenerator` | Generates pages with AI |
| `RagEngine` | Q&A with conversation history |
| `VectorStore` | rusqlite + sqlite-vec operations |

### MCP Wiki Server

Binary: `opencode-mcp-wiki`. Provides 5 tools:
- `search_code`: Semantic code search
- `get_documentation`: Retrieve wiki pages
- `ask_codebase`: RAG Q&A
- `list_wiki_pages`: Structure navigation
- `get_index_status`: Indexing status

Environment: `OPENROUTER_API_KEY` (required), `OPENCODE_WIKI_*` (optional).

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
cargo test --workspace                    # All tests
cargo test -p orchestrator               # 55 tests
cargo test -p wiki                       # 64 tests
cargo test -p server -- --nocapture      # 20 tests with output
cargo clippy --workspace -- -D warnings  # Lint check
```
