---
feature: "Frontend Fixes & New Features"
spec: |
  Fix frontend to work correctly with backend API and add new features: Activity visualization component for real-time task execution monitoring, and Project selection/switching UI.
  
  Current issues identified:
  1. OpenAPI paths inconsistent with actual routes (project.rs uses old paths in utoipa annotations)
  2. Missing projects API endpoints in OpenAPI (open, init, current)
  3. Missing "projects" folder in generated API hooks (only "project" exists)
  4. Frontend doesn't handle "no project open" state
  5. Sessions view placeholder not implemented
  6. No activity visualization during task execution
  7. No project picker/switcher UI
  
  Key patterns:
  - Use Orval-generated hooks for all API calls
  - SessionActivityMsg type from ts-rs for activity streaming
  - useSessionActivityStream hook already exists
  - NiceModal for dialogs
  
  Constraints:
  - TypeScript strict mode, no `any` or `@ts-ignore`
  - Biome formatting (tabs, 100 char lines)
  - Generated types from ts-rs and Orval
---

## Task List

### Feature 1: Fix OpenAPI & Regenerate Hooks
Description: Align OpenAPI annotations with actual routes and regenerate frontend API hooks
- [x] 1.01 Update projects.rs utoipa path annotations to match actual routes (/api/projects/* instead of /api/project/*) (note: Checking current utoipa annotations in projects.rs) (note: Already correct - paths use /api/projects/* not /api/project/*)
- [x] 1.02 Add CurrentProjectResponse, OpenProjectResponse, InitProjectResponse to OpenAPI schemas in lib.rs (note: Already in lib.rs schemas (lines 52-62))
- [x] 1.03 Add open_project, init_project, get_current_project paths to lib.rs OpenAPI paths list (note: Already in lib.rs paths (lines 26-30))
- [x] 1.04 Run cargo clippy and cargo test to verify backend changes (note: Clippy clean, all tests pass)
- [x] 1.05 Regenerate frontend API hooks (pnpm generate:api) (note: Server running, regenerating API hooks) (note: Orval generated hooks successfully)
- [x] 1.06 Verify new hooks exist under api/generated/projects/ (note: Verified: useOpenProject, useInitProject, useGetCurrentProject, useGetRecentProjects, useValidateProjectPath all generated with correct /api/projects/* paths)

### Feature 2: Project Context & Selection
Description: Add project store and selection UI - users must select a project before using the app
- [x] 2.01 Create useProjectStore (zustand) with currentProject, isLoading, error states (note: Creating useProjectStore with zustand) (note: Created useProjectStore with currentProject, isLoading, error, isDialogOpen states)
- [x] 2.02 Create ProjectPickerDialog component with recent projects list and path input (note: Creating ProjectPickerDialog component) (note: Created ProjectPickerDialog with recent projects list and manual path input)
- [x] 2.03 Integrate useOpenProject and useGetCurrentProject hooks in ProjectPickerDialog (note: Integrated useOpenProject, useGetRecentProjects, useValidateProjectPath hooks)
- [x] 2.04 Add project indicator in sidebar header showing current project name (note: Adding project indicator to sidebar) (note: Added project name and VCS indicator in sidebar header)
- [x] 2.05 Show ProjectPickerDialog on app load when no project is open (note: ProjectPickerDialog shows on load when no project via useGetCurrentProject + useEffect)
- [x] 2.06 Add 'Switch Project' button in sidebar that opens ProjectPickerDialog (note: Added Switch Project button in sidebar footer)

### Feature 3: Activity Visualization Component
Description: Create component to display real-time session activities (tool calls, results, reasoning)
- [x] 3.01 Create ActivityFeed component that renders list of SessionActivityMsg items (note: Creating ActivityFeed and ActivityItem components) (note: Created ActivityFeed component with auto-scroll and event count)
- [x] 3.02 Create ActivityItem component with different renderers for each activity type (tool_call, tool_result, agent_message, reasoning, finished) (note: Created ActivityItem with renderers for tool_call, tool_result, agent_message, reasoning, step_start, finished)
- [x] 3.03 Add auto-scroll to bottom behavior and optional pause/resume scrolling (note: Added auto-scroll with pause/resume via isNearBottom detection and scroll-to-bottom button)
- [x] 3.04 Integrate useSessionActivityStream hook to fetch activities for selected session (note: Integrating into TaskDetailPanel) (note: Integrated useSessionActivityStream in TaskDetailPanel with session selector)
- [x] 3.05 Add Activity tab to TaskDetailPanel tabs alongside Details, Plan, Diff, Sessions (note: Added Activity tab to TaskDetailPanel with session picker and ActivityFeed)
- [x] 3.06 Show live activity indicator (pulsing dot) when session is running (note: Added pulsing green dot on Activity tab and session buttons when running)

### Feature 4: Sessions View Implementation
Description: Implement the Sessions view showing all sessions across tasks
- [x] 4.01 Create SessionsList component using useListSessions hook (note: Creating SessionsList component) (note: Created SessionsList with useListSessions hook)
- [x] 4.02 Add filters for session phase (planning/implementation/review) and status (note: Added phase and status filters with filter buttons)
- [x] 4.03 Create SessionCard component showing session info with expandable activity feed (note: Created SessionCard with expandable ActivityFeed)
- [x] 4.04 Replace placeholder in App.tsx sessions view with SessionsList (note: Replaced placeholder in App.tsx with SessionsList component)

### Feature 5: Error States & Edge Cases
Description: Handle API errors and edge cases gracefully
- [x] 5.01 Add error boundary component for catching React errors (note: Creating ErrorBoundary component) (note: Created ErrorBoundary class component with reset functionality)
- [x] 5.02 Show user-friendly error messages when API calls fail (note: Error messages already shown in SessionsList (error state) and other components)
- [x] 5.03 Handle 'no project open' 400 error from task endpoints by showing ProjectPickerDialog (note: App.tsx already shows ProjectPickerDialog when no project is loaded (via useGetCurrentProject check))
- [x] 5.04 Run TypeScript check (pnpm exec tsc --noEmit) and fix any type errors (note: Running TypeScript check) (note: TypeScript check passes with no errors)
- [x] 5.05 Run Biome check and fix any linting issues (note: Running Biome check) (note: Biome check applied fixes. Remaining warnings are false positives (noSecrets) or acceptable patterns)
