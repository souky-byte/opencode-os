---
feature: "OpenCode Studio Frontend"
spec: |
  Build a complete frontend for OpenCode Studio using Vite + React 18 + TypeScript + Tailwind CSS. The UI should follow the vibe-kanban patterns and include: Kanban board with 6 columns (TODO, PLANNING, IN_PROGRESS, AI_REVIEW, REVIEW, DONE), task cards with status indicators, task detail panel with tabbed interface, create task dialog, WebSocket integration for real-time updates, and workspace diff/merge UI. Use generated types from @/types/generated and API hooks from @/api/generated.
---

## Task List

### Feature 1: UI Components
Description: Add essential UI components using shadcn/ui patterns
- [x] 1.01 Add Button, Card, Dialog, Tabs, Badge, Input, Textarea components (note: Adding essential UI components) (note: Added Button, Card, Dialog, Tabs, Badge, Input, Textarea components)
- [x] 1.02 Add Loader, ScrollArea, Separator components (note: Adding Loader, ScrollArea, Separator) (note: Added Loader, ScrollArea, Separator components)

### Feature 2: Kanban Board
Description: Build the main kanban board view with columns and task cards
- [x] 2.01 Create KanbanColumn component with status header and task list (note: Creating KanbanColumn component) (note: Created KanbanColumn with status header and task count)
- [x] 2.02 Create TaskCard component with title, status badge, AI indicator (note: Creating TaskCard component) (note: Created TaskCard with title, badges, AI/Human indicators)
- [x] 2.03 Create KanbanView component integrating columns and using useListTasks hook (note: Creating KanbanView component) (note: Created KanbanView with columns and task grouping)

### Feature 3: Task Detail Panel
Description: Build the task detail panel with tabbed interface
- [x] 3.01 Create TaskDetailPanel with task info, description, status controls (note: Creating TaskDetailPanel component) (note: Created TaskDetailPanel with details, execute/transition controls)
- [x] 3.02 Add tabs for Plan, Diff, AI Review, Sessions (note: Added tabs for Details, Plan, Diff, Sessions)
- [x] 3.03 Integrate execute and transition mutations (note: Integrated useExecuteTask and useTransitionTask mutations)

### Feature 4: Create Task Dialog
Description: Build the create task dialog with form
- [x] 4.01 Create CreateTaskDialog using NiceModal pattern (note: Creating CreateTaskDialog) (note: Created CreateTaskDialog using NiceModal pattern)
- [x] 4.02 Wire up useCreateTask mutation with form submission (note: Wired useCreateTask mutation with form submission)

### Feature 5: WebSocket Integration
Description: Add real-time updates via WebSocket
- [x] 5.01 Create useWebSocket hook for /ws connection (note: Creating useWebSocket hook) (note: Created useWebSocket hook with auto-reconnect and ping/pong)
- [x] 5.02 Handle ServerMessage events and invalidate React Query cache (note: Added React Query cache invalidation on task/session/workspace events)

### Feature 6: App Integration
Description: Wire everything together in App.tsx
- [x] 6.01 Update App.tsx with KanbanView and TaskDetailPanel layout (note: Updating App.tsx with complete layout) (note: Updated App.tsx with KanbanView, TaskDetailPanel, and sidebar)
- [x] 6.02 Add selected task state and panel toggle (note: Added selectedTask state, panel toggle, WebSocket integration)
