"use client"

export type TaskStatus = "TODO" | "PLANNING" | "IN_PROGRESS" | "AI_REVIEW" | "REVIEW" | "DONE"

export interface Task {
  id: string
  title: string
  description: string
  status: TaskStatus
  priority: "low" | "medium" | "high"
  assignee: "ai" | "human"
  createdAt: string
  updatedAt: string
  roadmapItemId?: string
}

export interface RoadmapItem {
  id: string
  title: string
  status: "planned" | "in_development" | "completed"
  quarter: string
  jtbd: string
  acceptanceCriteria: string[]
  linkedTaskIds: string[]
}

export interface SessionMessage {
  id: string
  role: "agent" | "system"
  content: string
  timestamp: string
}

export interface Session {
  id: string
  taskId: string
  phase: "planning" | "implementation" | "review"
  status: "running" | "completed" | "failed"
  messages: SessionMessage[]
  startedAt: string
}

export interface Project {
  id: string
  name: string
  slug: string
  description: string
  color: string
  vcsBackend: "jj" | "git"
  defaultBranch: string
  taskCount: number
  activeSessionCount: number
  lastActivity: string
}

export const mockTasks: Task[] = [
  {
    id: "task-001",
    title: "Implement Dark Mode",
    description: "Add dark mode toggle with system preference detection and persistent storage",
    status: "REVIEW",
    priority: "high",
    assignee: "ai",
    createdAt: "2025-01-15T10:00:00Z",
    updatedAt: "2025-01-15T14:30:00Z",
    roadmapItemId: "roadmap-001",
  },
  {
    id: "task-002",
    title: "Add User Authentication",
    description: "Implement OAuth2 login with GitHub and Google providers",
    status: "IN_PROGRESS",
    priority: "high",
    assignee: "ai",
    createdAt: "2025-01-14T09:00:00Z",
    updatedAt: "2025-01-15T12:00:00Z",
  },
  {
    id: "task-003",
    title: "Create API Rate Limiting",
    description: "Add rate limiting middleware with configurable thresholds",
    status: "AI_REVIEW",
    priority: "medium",
    assignee: "ai",
    createdAt: "2025-01-13T11:00:00Z",
    updatedAt: "2025-01-15T11:00:00Z",
  },
  {
    id: "task-004",
    title: "Database Migration Script",
    description: "Create migration for new user preferences table",
    status: "PLANNING",
    priority: "medium",
    assignee: "ai",
    createdAt: "2025-01-15T08:00:00Z",
    updatedAt: "2025-01-15T08:30:00Z",
  },
  {
    id: "task-005",
    title: "Fix Navigation Bug",
    description: "Mobile navigation menu not closing after link click",
    status: "TODO",
    priority: "low",
    assignee: "human",
    createdAt: "2025-01-15T07:00:00Z",
    updatedAt: "2025-01-15T07:00:00Z",
  },
  {
    id: "task-006",
    title: "Add Error Boundary",
    description: "Implement React error boundary with fallback UI",
    status: "TODO",
    priority: "medium",
    assignee: "ai",
    createdAt: "2025-01-14T16:00:00Z",
    updatedAt: "2025-01-14T16:00:00Z",
  },
  {
    id: "task-007",
    title: "Optimize Bundle Size",
    description: "Analyze and reduce JavaScript bundle size by lazy loading",
    status: "DONE",
    priority: "medium",
    assignee: "ai",
    createdAt: "2025-01-12T10:00:00Z",
    updatedAt: "2025-01-13T15:00:00Z",
  },
]

export const mockRoadmapItems: RoadmapItem[] = [
  {
    id: "roadmap-001",
    title: "Dark Mode Support",
    status: "in_development",
    quarter: "Q1 2025",
    jtbd: "When working at night or in dark environments, users want a dark interface to reduce eye strain",
    acceptanceCriteria: [
      "Toggle in header",
      "Persist preference",
      "Respect prefers-color-scheme",
      "All components have dark variants",
    ],
    linkedTaskIds: ["task-001"],
  },
  {
    id: "roadmap-002",
    title: "API v2 with GraphQL",
    status: "planned",
    quarter: "Q1 2025",
    jtbd: "Developers need more flexible data fetching to build efficient client applications",
    acceptanceCriteria: ["GraphQL endpoint", "Schema documentation", "Subscription support", "Backward compatibility"],
    linkedTaskIds: [],
  },
  {
    id: "roadmap-003",
    title: "Mobile Application",
    status: "planned",
    quarter: "Q1 2025",
    jtbd: "Users want to access the platform on mobile devices for on-the-go task management",
    acceptanceCriteria: ["iOS app", "Android app", "Push notifications", "Offline support"],
    linkedTaskIds: [],
  },
  {
    id: "roadmap-004",
    title: "Multi-tenant Support",
    status: "planned",
    quarter: "Q2 2025",
    jtbd: "Enterprise customers need isolated workspaces for different teams or projects",
    acceptanceCriteria: ["Tenant isolation", "Custom domains", "SSO integration", "Usage analytics per tenant"],
    linkedTaskIds: [],
  },
  {
    id: "roadmap-005",
    title: "Advanced Analytics",
    status: "planned",
    quarter: "Q2 2025",
    jtbd: "Product managers need insights into AI agent performance and task completion patterns",
    acceptanceCriteria: ["Dashboard with metrics", "Export reports", "Custom date ranges", "Team comparisons"],
    linkedTaskIds: [],
  },
]

export const mockSessions: Session[] = [
  {
    id: "session-001",
    taskId: "task-002",
    phase: "implementation",
    status: "running",
    startedAt: "2025-01-15T12:00:00Z",
    messages: [
      {
        id: "msg-001",
        role: "system",
        content: "Starting implementation session for task-002",
        timestamp: "2025-01-15T12:00:00Z",
      },
      {
        id: "msg-002",
        role: "agent",
        content: "Reading plan from plans/task-002.md...",
        timestamp: "2025-01-15T12:00:05Z",
      },
      {
        id: "msg-003",
        role: "agent",
        content: "Creating auth provider component in src/providers/auth.tsx",
        timestamp: "2025-01-15T12:01:00Z",
      },
      {
        id: "msg-004",
        role: "agent",
        content: "Implementing OAuth callback handler for GitHub...",
        timestamp: "2025-01-15T12:02:30Z",
      },
    ],
  },
  {
    id: "session-002",
    taskId: "task-003",
    phase: "review",
    status: "running",
    startedAt: "2025-01-15T11:00:00Z",
    messages: [
      {
        id: "msg-005",
        role: "system",
        content: "Starting AI review session for task-003",
        timestamp: "2025-01-15T11:00:00Z",
      },
      {
        id: "msg-006",
        role: "agent",
        content: "Analyzing diff for rate limiting implementation...",
        timestamp: "2025-01-15T11:00:10Z",
      },
      {
        id: "msg-007",
        role: "agent",
        content: "Checking code quality and test coverage...",
        timestamp: "2025-01-15T11:01:00Z",
      },
    ],
  },
  {
    id: "session-003",
    taskId: "task-004",
    phase: "planning",
    status: "running",
    startedAt: "2025-01-15T08:30:00Z",
    messages: [
      {
        id: "msg-008",
        role: "system",
        content: "Starting planning session for task-004",
        timestamp: "2025-01-15T08:30:00Z",
      },
      {
        id: "msg-009",
        role: "agent",
        content: "Analyzing task requirements for database migration...",
        timestamp: "2025-01-15T08:30:10Z",
      },
      {
        id: "msg-010",
        role: "agent",
        content: "Generating technical plan with migration steps...",
        timestamp: "2025-01-15T08:31:00Z",
      },
    ],
  },
]

export const mockPlan = `# Implementation Plan: Dark Mode

## Analysis
Task requires implementing dark mode with:
- User toggle
- System preference detection
- Persistent storage

## Technical Steps

1. **Create ThemeContext** (src/context/ThemeContext.tsx)
   - useReducer for state management
   - localStorage for persistence
   - matchMedia for system detection

2. **Update Tailwind Config** (tailwind.config.js)
   - Enable darkMode: 'class'
   - Add dark color variants

3. **Add Toggle Component** (src/components/ThemeToggle.tsx)
   - Sun/Moon icons
   - Dropdown for auto/light/dark

4. **Update Components**
   - Header.tsx - Add toggle
   - All components - Add dark: variants

## Files to Modify
- src/context/ThemeContext.tsx (new)
- tailwind.config.js
- src/components/Header.tsx
- src/components/ThemeToggle.tsx (new)

## Risks
- Flash of unstyled content on page load
- Third-party components may not support dark mode

## Complexity
Medium - 2-3 hours estimated`

export const mockDiff = `diff --git a/src/context/ThemeContext.tsx b/src/context/ThemeContext.tsx
new file mode 100644
--- /dev/null
+++ b/src/context/ThemeContext.tsx
@@ -0,0 +1,45 @@
+import { createContext, useContext, useEffect, useState } from 'react'
+
+type Theme = 'light' | 'dark' | 'system'
+
+interface ThemeContextType {
+  theme: Theme
+  setTheme: (theme: Theme) => void
+}
+
+const ThemeContext = createContext<ThemeContextType | undefined>(undefined)
+
+export function ThemeProvider({ children }: { children: React.ReactNode }) {
+  const [theme, setTheme] = useState<Theme>('system')
+
+  useEffect(() => {
+    const stored = localStorage.getItem('theme') as Theme
+    if (stored) setTheme(stored)
+  }, [])
+
+  useEffect(() => {
+    const root = window.document.documentElement
+    root.classList.remove('light', 'dark')
+
+    if (theme === 'system') {
+      const systemTheme = window.matchMedia('(prefers-color-scheme: dark)').matches
+        ? 'dark'
+        : 'light'
+      root.classList.add(systemTheme)
+    } else {
+      root.classList.add(theme)
+    }
+
+    localStorage.setItem('theme', theme)
+  }, [theme])
+
+  return (
+    <ThemeContext.Provider value={{ theme, setTheme }}>
+      {children}
+    </ThemeContext.Provider>
+  )
+}

diff --git a/src/components/Header.tsx b/src/components/Header.tsx
--- a/src/components/Header.tsx
+++ b/src/components/Header.tsx
@@ -1,5 +1,6 @@
 import { Logo } from './Logo'
 import { Navigation } from './Navigation'
+import { ThemeToggle } from './ThemeToggle'
 
 export function Header() {
   return (
@@ -7,6 +8,7 @@ export function Header() {
       <div className="flex items-center gap-4">
         <Logo />
         <Navigation />
+        <ThemeToggle />
       </div>
     </header>
   )`

export const mockReview = `# AI Review: Dark Mode Implementation

## Summary
Implementation is **APPROVED** with minor suggestions.

## Code Quality ✅
- Clean separation of concerns
- Proper TypeScript types
- Good use of React Context

## Requirements ✅
- [x] Toggle in header
- [x] Persist preference
- [x] Respect prefers-color-scheme
- [x] All components have dark variants

## Tests ⚠️
- Unit tests for ThemeContext: Missing
- Recommend adding tests for:
  - Initial load with stored preference
  - System preference detection
  - Toggle functionality

## Security ✅
- No security concerns
- localStorage usage is appropriate

## Suggestions
1. Add aria-label to toggle button
2. Consider adding transition on theme change
3. Add unit tests before merge

## Decision
**APPROVE** - Ready for human review`

export const mockRoadmapDetails: Record<
  string,
  { description: string; userStories: string[]; technicalNotes: string }
> = {
  "roadmap-001": {
    description: `# Dark Mode Support

## Overview
Implementing a comprehensive dark mode system that provides a comfortable viewing experience in low-light environments while maintaining brand consistency and accessibility.

## Business Value
- **User Retention**: 82% of developers prefer dark mode for coding
- **Accessibility**: Reduces eye strain for extended usage sessions
- **Modern UX**: Expected feature for developer-focused tools

## Target Users
- Developers working late hours
- Users with light sensitivity
- Power users who prefer dark interfaces`,
    userStories: [
      "As a developer, I want to toggle dark mode so I can work comfortably at night",
      "As a user, I want my preference saved so I don't have to set it every time",
      "As a user, I want the app to follow my system settings by default",
    ],
    technicalNotes: `## Technical Approach

### Implementation Strategy
1. **CSS Variables** - Define color tokens that switch based on theme class
2. **React Context** - Manage theme state globally with \`ThemeProvider\`
3. **Local Storage** - Persist user preference across sessions
4. **Media Query** - Detect \`prefers-color-scheme\` for system default

### Components to Update
- \`Header.tsx\` - Add theme toggle button
- \`ThemeProvider.tsx\` - New context provider (create)
- \`globals.css\` - Add dark mode CSS variables
- All UI components - Ensure proper dark variants

### Risks & Mitigations
| Risk | Mitigation |
|------|------------|
| Flash of unstyled content | Use blocking script in \`<head>\` |
| Third-party components | Wrap with theme-aware containers |`,
  },
  "roadmap-002": {
    description: `# API v2 with GraphQL

## Overview
Introducing GraphQL as our primary API interface to provide more flexible and efficient data fetching capabilities for client applications.

## Business Value
- **Developer Experience**: Single endpoint, self-documenting schema
- **Performance**: Clients request only needed data
- **Flexibility**: Supports complex queries without multiple roundtrips`,
    userStories: [
      "As a frontend developer, I want to fetch exactly the data I need in one request",
      "As an API consumer, I want real-time updates via subscriptions",
      "As a developer, I want auto-generated TypeScript types from the schema",
    ],
    technicalNotes: `## Technical Approach

### Stack
- **Server**: Apollo Server v4
- **Client**: Apollo Client with React hooks
- **Codegen**: GraphQL Code Generator for types

### Endpoints
- \`/graphql\` - Main query/mutation endpoint
- \`/graphql/ws\` - WebSocket for subscriptions

### Migration Path
1. Deploy GraphQL alongside REST
2. Migrate internal apps first
3. Deprecate REST endpoints after 6 months`,
  },
  "roadmap-003": {
    description: `# Mobile Application

## Overview
Native mobile applications for iOS and Android to enable task management and AI agent monitoring on the go.

## Business Value
- **Accessibility**: Manage projects from anywhere
- **Notifications**: Instant alerts for AI completions and reviews
- **Market Expansion**: Mobile-first users`,
    userStories: [
      "As a user, I want to review AI-generated code on my phone",
      "As a project manager, I want push notifications when tasks need approval",
      "As a user, I want to access my projects offline",
    ],
    technicalNotes: `## Technical Approach

### Framework
**React Native** with Expo for cross-platform development

### Key Features
- Push notifications via Firebase
- Offline-first with local SQLite
- Biometric authentication
- Code syntax highlighting (read-only)

### API Integration
- GraphQL subscriptions for real-time updates
- Background sync for offline changes`,
  },
  "roadmap-004": {
    description: `# Multi-tenant Support

## Overview
Enable enterprise customers to have isolated workspaces for different teams, projects, or organizations with centralized administration.

## Business Value
- **Enterprise Sales**: Critical requirement for large organizations
- **Security**: Data isolation between tenants
- **Scalability**: Support for organizational hierarchies`,
    userStories: [
      "As an admin, I want to create separate workspaces for different teams",
      "As an enterprise user, I want SSO login with my company credentials",
      "As a billing admin, I want to see usage analytics per tenant",
    ],
    technicalNotes: `## Technical Approach

### Architecture
- **Database**: Schema per tenant with shared connection pool
- **Auth**: SAML 2.0 / OIDC integration for SSO
- **Routing**: Subdomain-based tenant identification

### Security
- Row-level security policies
- Encrypted tenant data at rest
- Audit logging per tenant`,
  },
  "roadmap-005": {
    description: `# Advanced Analytics

## Overview
Comprehensive analytics dashboard providing insights into AI agent performance, task completion patterns, and team productivity metrics.

## Business Value
- **Optimization**: Identify bottlenecks in AI workflows
- **ROI Tracking**: Measure time saved by AI automation
- **Planning**: Data-driven sprint planning`,
    userStories: [
      "As a tech lead, I want to see which task types AI handles best",
      "As a manager, I want weekly reports on team productivity",
      "As an admin, I want to compare AI vs human task completion times",
    ],
    technicalNotes: `## Technical Approach

### Metrics to Track
- Task completion time by phase
- AI approval/rejection rates
- Session duration and token usage
- Error rates by task type

### Storage
- Time-series database (InfluxDB) for metrics
- Aggregation pipelines for reporting
- Data retention: 90 days raw, 2 years aggregated

### Visualization
- Recharts for interactive graphs
- Export to CSV/PDF
- Scheduled email reports`,
  },
}

export const mockProjects: Project[] = [
  {
    id: "proj-001",
    name: "OpenCode Studio",
    slug: "opencode-studio",
    description: "AI-powered autonomous development platform",
    color: "#6366f1",
    vcsBackend: "jj",
    defaultBranch: "main",
    taskCount: 7,
    activeSessionCount: 2,
    lastActivity: "2025-01-15T14:30:00Z",
  },
  {
    id: "proj-002",
    name: "E-commerce API",
    slug: "ecommerce-api",
    description: "Backend services for online store",
    color: "#10b981",
    vcsBackend: "git",
    defaultBranch: "develop",
    taskCount: 12,
    activeSessionCount: 1,
    lastActivity: "2025-01-15T12:00:00Z",
  },
  {
    id: "proj-003",
    name: "Mobile App",
    slug: "mobile-app",
    description: "React Native cross-platform application",
    color: "#f59e0b",
    vcsBackend: "git",
    defaultBranch: "main",
    taskCount: 5,
    activeSessionCount: 0,
    lastActivity: "2025-01-14T18:00:00Z",
  },
  {
    id: "proj-004",
    name: "Data Pipeline",
    slug: "data-pipeline",
    description: "ETL jobs and analytics processing",
    color: "#ec4899",
    vcsBackend: "jj",
    defaultBranch: "trunk",
    taskCount: 3,
    activeSessionCount: 0,
    lastActivity: "2025-01-13T09:00:00Z",
  },
]
