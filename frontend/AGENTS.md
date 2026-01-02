# OpenCode Studio: Frontend

## OVERVIEW

React + Vite SPA. React Query for server state, Zustand for client state, Orval-generated API hooks.

## STRUCTURE

```
frontend/src/
├── api/generated/     # Orval-generated React Query hooks (DO NOT EDIT)
├── components/        # Feature-based organization
│   ├── ui/           # Shared primitives (shadcn/ui)
│   ├── kanban/       # KanbanView, KanbanColumn, TaskCard
│   ├── sessions/     # SessionCard, SessionsList
│   ├── task-detail/  # TaskDetailPanel
│   ├── activity/     # ActivityFeed, ActivityItem
│   └── dialogs/      # CreateTaskDialog, ProjectPickerDialog
├── hooks/            # Custom hooks (SSE, real-time)
├── stores/           # Zustand stores (project, sidebar)
├── lib/              # Utilities (api-fetcher, modals)
└── types/generated/  # ts-rs generated types (DO NOT EDIT)
```

## WHERE TO LOOK

| Task | Location |
|:-----|:---------|
| Add UI component | `components/ui/` (shadcn pattern) |
| Add feature component | `components/{feature}/` |
| Add API hook | Regenerate: `pnpm generate:api` |
| Add client state | `stores/use{Name}Store.ts` |
| Add SSE hook | `hooks/useEventStream.ts` pattern |

## PATTERNS

**State Management:**
- Server state → React Query (via Orval hooks)
- Client state → Zustand stores
- SSE events → Custom hooks with auto-reconnect

**Component Pattern:**
```tsx
// Feature component
export function TaskCard({ task }: { task: Task }) {
  const { data, isLoading } = useGetTask(task.id);
  const mutation = useUpdateTask();
  // ...
}
```

**SSE Pattern:**
```tsx
const { isConnected } = useEventStream({
  onEvent: (event) => {
    if (event.type === "TaskStatusChanged") {
      queryClient.invalidateQueries({ queryKey: ["tasks"] });
    }
  }
});
```

## CONVENTIONS

- Biome: Tabs, 100 chars, double quotes, trailing commas
- Named exports only (no default except pages/configs)
- `useImportType` / `useExportType` enforced
- No barrel files (direct imports only)

## ANTI-PATTERNS

- `as any`, `@ts-ignore`, `@ts-expect-error` → NEVER
- `console.log` → use proper logging or remove
- Empty catch blocks → handle errors explicitly
- Floating promises → always await or void
- Editing `api/generated/` or `types/generated/` → regenerate instead

## GENERATED CODE

Two sources of generated code (never edit directly):

| Source | Output | Regenerate |
|:-------|:-------|:-----------|
| OpenAPI spec | `api/generated/` | `pnpm generate:api` |
| Rust types (ts-rs) | `types/generated/` | Backend build |

## COMMANDS

```bash
pnpm dev              # Vite dev server
pnpm build            # Production build
pnpm generate:api     # Regenerate API hooks from OpenAPI
pnpm lint             # Biome check
```

## DEPS

- `@tanstack/react-query` - Server state
- `zustand` - Client state  
- `@nice-modal-react` - Modal management
- `tailwindcss` - Styling
- `shadcn/ui` - Component primitives
