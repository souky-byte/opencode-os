---
feature: "Multi-Project Workspace Selection"
spec: |
  Enable runtime project switching in OpenCode Studio. Users can select folders (git/jj repos), auto-initialize .opencode-studio/ structure, and switch between projects without server restart. Architecture uses hot-swappable ProjectContext via Arc<RwLock<Option<ProjectContext>>>.
---

## Task List

### Feature 1: ProjectManager Core
Description: Create ProjectManager and ProjectContext structs with core logic for project lifecycle
- [x] 1.01 Create crates/server/src/project_manager.rs with ProjectError enum and basic structs (note: Starting ProjectManager module creation) (note: Created project_manager.rs with ProjectError, ProjectConfig, ProjectInfo, ProjectContext, ProjectManager structs. 6 unit tests passing.)
- [x] 1.02 Implement ProjectContext::new() - creates DB pool, repositories, executor, workspace manager from path (note: Implemented as part of 1.01 - ProjectContext::new() creates DB pool, repositories, executor, workspace manager from path)
- [x] 1.03 Implement init_project_structure() - creates .opencode-studio/ dirs and config.toml (note: Implemented as part of 1.01 - init_project_structure() creates .opencode-studio/ dirs and config.toml)
- [x] 1.04 Implement ProjectManager with open(), init(), current(), close() methods (note: Implemented as part of 1.01 - ProjectManager with open(), init(), current(), close() methods)
- [x] 1.05 Add unit tests for ProjectManager (note: 6 unit tests added for VCS detection, project init, config loading, error codes)

### Feature 2: GlobalConfigManager
Description: Manage ~/.opencode-studio/global.toml for recent projects and preferences
- [x] 2.01 Create GlobalConfig and GlobalConfigManager structs (note: Creating GlobalConfig and GlobalConfigManager structs) (note: GlobalConfig and GlobalConfigManager structs created with recent_projects, last_project, auto_open_last, max_recent fields)
- [x] 2.02 Implement add_recent(), get_recent(), set_last(), get_last() methods (note: Implemented add_recent(), get_recent(), set_last(), get_last() with deduplication and truncation. 4 tests added.)
- [x] 2.03 Add atomic file write with tempfile for safe config updates (note: Atomic file write using .tmp extension + rename pattern for safe config updates)

### Feature 3: Refactor AppState
Description: Update AppState to use ProjectManager while maintaining backward compatibility
- [x] 3.01 Refactor AppState to hold ProjectManager instead of direct repositories (note: Refactoring AppState to use ProjectManager) (note: Refactored AppState to use ProjectManager. Removed direct repositories, now uses project() method to access ProjectContext.)
- [x] 3.02 Add AppState::project() convenience method that returns Result<ProjectContext, NoProjectError> (note: Added AppState::project() returning Result<ProjectContext, ProjectError>, open_project(), auto_open_last_project() methods)
- [x] 3.03 Update main.rs to use new AppState initialization with auto-open from DATABASE_URL (note: Updated main.rs with PROJECT_PATH env, DATABASE_URL fallback, and auto_open_last_project() support)
- [x] 3.04 Ensure all existing tests pass with refactored AppState (note: All 31 integration tests pass, 10 unit tests pass, clippy clean)

### Feature 4: New API Endpoints
Description: Add /api/projects/* endpoints for project management
- [x] 4.01 Create crates/server/src/routes/projects.rs with OpenAPI schemas (note: Creating projects.rs with OpenAPI schemas) (note: Created projects.rs with ProjectInfo, OpenProjectRequest/Response, InitProjectRequest/Response, CurrentProjectResponse, RecentProject, RecentProjectsResponse, ValidatePathRequest/Response schemas)
- [x] 4.02 Implement POST /api/projects/open endpoint (note: POST /api/projects/open implemented with auto-init, global config updates, error handling)
- [x] 4.03 Implement POST /api/projects/init endpoint (note: POST /api/projects/init implemented with force flag support)
- [x] 4.04 Implement GET /api/projects/current endpoint (note: GET /api/projects/current implemented returning current ProjectInfo or null)
- [x] 4.05 Migrate GET /api/project/recent to /api/projects/recent (note: GET /api/projects/recent implemented returning list of RecentProject with exists/vcs info)
- [x] 4.06 Migrate POST /api/project/validate to /api/projects/validate (note: POST /api/projects/validate implemented with path validation, VCS detection, name extraction)
- [x] 4.07 Register new routes in lib.rs and update OpenAPI schema (note: Routes registered in lib.rs, OpenAPI schema updated with all new types and endpoints, tests passing)

### Feature 5: Route Handler Updates
Description: Update existing route handlers to use ProjectContext
- [x] 5.01 Update tasks.rs handlers to use state.project()? (note: Already done in Feature 3 - tasks.rs uses state.project().await?)
- [x] 5.02 Update sessions.rs handlers to use state.project()? (note: Already done in Feature 3 - sessions.rs uses state.project().await?)
- [x] 5.03 Update workspaces.rs handlers to use state.project()? (note: Already done in Feature 3 - workspaces.rs uses state.project().await?)
- [x] 5.04 Add proper error response for NoProjectOpen error (note: Already done in Feature 3 - AppError::Project handles NoProjectOpen with 400 status)

### Feature 6: WebSocket Events
Description: Broadcast project changes to connected clients
- [x] 6.01 Add ProjectChanged and ProjectClosed variants to ServerMessage (note: Added ProjectOpened and ProjectClosed variants to Event enum in events/types.rs with path, name, was_initialized fields)
- [x] 6.02 Emit project_changed event when ProjectManager::open() succeeds (note: ProjectManager::open() emits ProjectOpened event via event_bus.publish() after successful project opening)
- [x] 6.03 Emit project_closed event when ProjectManager::close() called (note: ProjectManager::close() emits ProjectClosed event via event_bus.publish() when closing a project)

### Feature 7: Integration Tests
Description: End-to-end tests for project switching flow
- [x] 7.01 Add integration test for open project flow (new + existing) (note: Added 10 integration tests for project management: open/init/validate/switch/recent endpoints)
- [x] 7.02 Add integration test for project switching (note: test_project_switching verifies switching between two projects updates current project correctly)
- [x] 7.03 Add integration test for no-project error handling (note: test_tasks_require_project verifies 400 error when no project is open)
- [x] 7.04 Run cargo test --workspace and cargo clippy, fix any issues (note: All 131 tests pass across 10 crates, clippy clean with -D warnings)
