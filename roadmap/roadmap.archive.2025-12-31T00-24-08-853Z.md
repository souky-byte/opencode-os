---
feature: "Project Selection"
spec: |
  Add ability to select which git project folder the app works with. Users should be able to pick a folder from their PC when starting the app. Implement CLI argument support and onboarding UI flow.
---

## Task List

### Feature 1: CLI Project Path Argument
Description: Allow specifying project path via CLI: opencode-studio serve /path/to/project
- [x] 1.01 Update CLI serve command to accept optional project path argument (note: Starting CLI serve command update with project path argument) (note: Added optional path argument to Init, Serve, and Status commands)
- [x] 1.02 Pass project path to server initialization instead of using current_dir() (note: Added AppState::with_repo_path() and CLI passes resolved path to it)
- [x] 1.03 Validate path exists and contains .git or .jj directory (note: Added resolve_project_path() and validate_vcs_project() functions)

### Feature 2: Project Config Persistence
Description: Store selected project path in config file for persistence across restarts
- [x] 2.01 Create config module to read/write .opencode-studio/config.toml (note: Checking existing config implementation) (note: Created GlobalConfig with load/save functions for ~/.opencode-studio/global.toml)
- [x] 2.02 Add project_path field to config (note: Added recent_projects and last_project to GlobalConfig)
- [x] 2.03 Load project path from config if not provided via CLI (note: resolve_project_path now loads last_project from global config if no path provided)

### Feature 3: Onboarding UI
Description: Show project selection dialog when no project is configured
- [x] 3.01 Add API endpoint GET /api/project/status to check if project is configured (note: Adding API endpoints for project configuration) (note: Added GET /api/project (with initialized field), GET /api/project/recent, and frontend hooks)
- [x] 3.02 Add API endpoint POST /api/project/select to set project path (note: Added POST /api/project/validate endpoint and useValidateProjectPath hook)
- [x] 3.03 Create ProjectSelector component with folder input (note: Creating ProjectSelector component) (note: Created ProjectSelector component with path validation and recent projects list)
- [x] 3.04 Show onboarding modal when project not configured (note: Implementing onboarding modal in AppShell) (note: Added onboarding Dialog to AppShell, shows when projectInfo.initialized is false. Fixed tsconfig to use explicit includes like vibe-kanban.)
- [x] 3.05 Add recent projects list to quick-select (note: Already implemented in ProjectSelector component - shows recent projects from useGetRecentProjects)
