# Product Requirements Document: OpenCode Studio

## 1. Executive Summary

### 1.1 Produkt
**OpenCode Studio** je autonomní AI-powered development platform, která orchestruje OpenCode sessions pro automatizovaný vývoj software. Člověk definuje *co* se má udělat (produktová vrstva), AI plánuje a implementuje *jak* (technická vrstva).

### 1.2 Vize
Transformovat vývoj software z manuálního procesu na orchestraci AI agentů, kde člověk funguje jako product owner a reviewer, nikoliv jako implementátor.

### 1.3 Klíčové principy
- **Autonomie**: Minimální lidská intervence během vývoje
- **Transparentnost**: Veškerá komunikace přes soubory (plány, reviews, roadmapa)
- **Modularita**: Plugovatelné moduly pro různé AI-powered funkce
- **Škálovatelnost**: Připraveno na paralelní běh více agentů

---

## 2. Problem Statement

### 2.1 Současný stav
Vývojáři tráví většinu času implementací, nikoliv plánováním a architekturou. AI coding agenti (Claude Code, OpenCode, Codex) existují, ale:

- Vyžadují konstantní dohled a interakci
- Nemají strukturovaný workflow (planning → implementation → review)
- Chybí orchestrace více paralelních tasků
- Není produktová vrstva (JTBD, user stories) nad technickými tasky

### 2.2 Cílový stav
Člověk vytvoří produktový požadavek → AI naplánuje → AI implementuje → AI review → člověk otestuje a schválí.

---

## 3. User Personas

### 3.1 Primary: Solo Developer / Tech Lead
- Má produktové vize ale limitovaný čas na implementaci
- Chce delegovat rutinní vývoj na AI
- Potřebuje zachovat kontrolu nad kvalitou

### 3.2 Secondary: Small Team Lead
- Řídí malý tým a chce zvýšit output
- Používá AI agenty jako "virtuální vývojáře"
- Potřebuje orchestrovat více paralelních tasků

---

## 4. Core Architecture

### 4.1 Dvouvrstvý model

```
┌─────────────────────────────────────────────────────────────────┐
│                    ROADMAP (produktová vrstva)                   │
│                                                                  │
│  "Co a proč"                                                    │
│  - JTBD (Jobs to be Done)                                       │
│  - User stories                                                 │
│  - Business value                                               │
│  - Success metrics                                              │
│  - Acceptance criteria                                          │
│                                                                  │
└─────────────────────────────────┬───────────────────────────────┘
                                  │
                                  │  [Přesunout do vývoje]
                                  ▼
┌─────────────────────────────────────────────────────────────────┐
│                    KANBAN (implementační vrstva)                 │
│                                                                  │
│  "Jak"                                                          │
│  - Technický plán (PLANNING fáze)                               │
│  - Implementace (IN_PROGRESS)                                   │
│  - Code review (AI_REVIEW, REVIEW)                              │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 4.2 Task Lifecycle

```
┌─────────┐     ┌──────────┐     ┌─────────────┐     ┌───────────┐     ┌──────────┐     ┌──────────┐
│  TODO   │ ──▶ │ PLANNING │ ──▶ │ IN_PROGRESS │ ──▶ │ AI_REVIEW │ ──▶ │  REVIEW  │ ──▶ │   DONE   │
│         │     │          │     │             │     │           │     │          │     │          │
│ člověk  │     │ AUTOMAT  │     │   AUTOMAT   │     │  AUTOMAT  │     │  člověk  │     │  člověk  │
│ vytvoří │     │ AI plán  │     │   OpenCode  │     │  AI check │     │  testuje │     │  mergne  │
└─────────┘     └──────────┘     └─────────────┘     └───────────┘     └──────────┘     └──────────┘
                     │                  ▲                  │                  │
                     ▼                  │                  ▼                  │
               ┌──────────┐             │            ┌───────────┐            │
               │ APPROVE  │             │            │ AI REJECT │            │
               │  PLAN?   │             │            │  automat  │            │
               └──────────┘             │            │  feedback │            │
                 │      │               │            └───────────┘            │
            auto │      │ člověk       │                  │                  │
                 │      │ schválí      └──────────────────┘                  │
                 ▼      ▼                                                    │
            IN_PROGRESS                                                      │
                                                                             ▼
                                             ┌───────────────────────────────┐
                                             │        HUMAN REJECT           │
                                             │  zpět do IN_PROGRESS          │
                                             │  s feedbackem                 │
                                             └───────────────────────────────┘
```

### 4.3 Session Strategy

Každá fáze má vlastní OpenCode session, komunikace přes soubory:

| Fáze | Session | Input (soubor) | Output (soubor) |
|------|---------|----------------|-----------------|
| PLANNING | A | task description | `plans/{id}.md` |
| IN_PROGRESS | B | plán | kód ve workspace |
| AI_REVIEW | C | diff, task description | `reviews/{id}.md` |
| IN_PROGRESS (retry) | D | review feedback | opravený kód |

**Výhody separátních sessions:**
- Čistý kontext pro každou fázi
- Různé modely pro různé fáze (levnější pro planning)
- Nezávislé na session stavu
- Recovery: plán/review je v souboru

---

## 5. Features

### 5.1 Core Module: Kanban

#### 5.1.1 Task Management
- Vytvoření tasku s popisem
- Automatický přechod mezi stavy
- Vazba na Roadmap item (volitelná)
- Historie všech sessions a jejich výstupů

#### 5.1.2 Automatizované fáze

**PLANNING (automat)**
```markdown
Input: Task description
Output: plans/{task_id}.md

Obsah plánu:
- Analýza požadavku
- Technické kroky implementace
- Soubory k změně/vytvoření
- Potenciální rizika
- Odhad komplexity
```

**IN_PROGRESS (automat)**
```markdown
Input: plans/{task_id}.md
Output: Změny v kódu (workspace)

OpenCode session implementuje dle plánu.
```

**AI_REVIEW (automat)**
```markdown
Input: Git diff, task description, plán
Output: reviews/{task_id}.md

Kontroluje:
- Code quality
- Splnění požadavků
- Testy
- Security
- Breaking changes
```

**REVIEW (člověk)**
- Testování ve workspace
- Dev server automaticky spuštěn
- Schválení nebo reject s feedbackem

#### 5.1.3 Konfigurace
```toml
[kanban]
require_plan_approval = false  # true = člověk schvaluje plán
auto_start_dev_server = true
parallel_tasks_limit = 5

[kanban.models]
planning = "claude-sonnet-4-20250514"
implementation = "claude-sonnet-4-20250514"
review = "claude-sonnet-4-20250514"
```

### 5.2 Core Module: Roadmap

#### 5.2.1 Produktová specifikace
Roadmap items jsou produktové dokumenty, nikoliv technické:

```markdown
# Dark Mode pro Dashboard

## Status
🟡 Planned | Q1 2025

## Jobs to be Done
Když pracuji večer nebo v tmavém prostředí,
chci mít tmavé rozhraní,
abych neměl únavu očí a mohl pracovat déle.

## User Stories
- Jako uživatel chci přepnout mezi light/dark mode
- Jako uživatel chci, aby si aplikace pamatovala preferenci
- Jako uživatel chci respektování systémového nastavení

## Business Value
- Konkurenční parita
- Snížení churn rate u power userů

## Success Metrics
- 30% uživatelů aktivuje dark mode do 30 dnů

## Acceptance Criteria
- [ ] Toggle v headeru
- [ ] Persistence preference
- [ ] Respektuje prefers-color-scheme
- [ ] Všechny komponenty mají dark variantu

## Target Users
- Power users
- Vývojáři

## Open Questions
- Má být "auto" třetí možnost?
```

#### 5.2.2 Flow: Roadmap → Kanban

Při přesunu do vývoje:
1. Vytvoří se Kanban task s referencí na roadmap item
2. Task description obsahuje odkaz na produktovou spec
3. PLANNING fáze čte produktovou spec a vytváří technický plán

#### 5.2.3 AI generování roadmapy
```markdown
Trigger: Manuální nebo po analýze projektu
Input:
  - README / vision dokument
  - GitHub issues
  - Dokončené tasky

Output: roadmap/roadmap.md + roadmap/items/*.md
```

### 5.3 Additional Modules

#### 5.3.1 Changelog Generator
```markdown
Trigger: Po DONE tasku nebo před release
Input: Git commits, dokončené tasky
Output: changelog/CHANGELOG.md
```

#### 5.3.2 Documentation Generator
```markdown
Trigger: Manuální nebo po změně API
Input: Zdrojový kód, existující docs
Output: docs/*.md

Varianty:
- Architecture overview
- API dokumentace
- Setup guide
```

#### 5.3.3 Code Insights
```markdown
Trigger: Scheduled nebo manuální
Input: Codebase
Output: insights/*.md

Varianty:
- Tech debt analýza
- Security audit
- Performance bottlenecks
- Test coverage mezery
```

#### 5.3.4 PR Description Generator
```markdown
Trigger: Task jde do REVIEW
Input: Diff, task description, plán
Output: pr-descriptions/{task_id}.md
```

#### 5.3.5 Meeting Notes → Tasks
```markdown
Trigger: Upload meeting notes
Input: Poznámky ze schůzky
Output: Nové tasky v kanbanu
```

---

## 6. Version Control: Jujutsu (jj)

### 6.1 Proč Jujutsu místo Git Worktrees

| Aspekt | Git Worktrees | Jujutsu Workspaces |
|--------|---------------|-------------------|
| Auto-commit | ❌ Musíš `git add` | ✅ Automatické snapshotování |
| Konflikty | ❌ Blokují práci | ✅ First-class, můžeš pokračovat |
| Rebase | ❌ Může selhat | ✅ Vždy uspěje |
| Undo | ❌ Reflog, složité | ✅ `jj undo` |
| AI-friendly | ❌ Agent musí znát git add | ✅ Změny se zachytí automaticky |

### 6.2 Workspace Lifecycle

```bash
# 1. Task jde do IN_PROGRESS
jj new main -m "task-123: Implement dark mode"

# 2. Vytvoř workspace
jj workspace add ../workspaces/task-123 --revision @

# 3. OpenCode pracuje (změny se automaticky zachytí)

# 4. Pro PR
jj bookmark create task-123 -r @
jj git push --bookmark task-123

# 5. Cleanup
jj workspace forget task-123
```

### 6.3 Konfigurace

```toml
[vcs]
backend = "jj"  # "jj" | "git"

[jj]
colocated = true                      # Zachová .git pro kompatibilitu
workspace_base_path = "../.workspaces"
auto_forget_workspace = true
cleanup_delay_hours = 24

[jj.parallel]
max_workspaces = 10
```

### 6.4 Abstrakce pro Git fallback

```rust
#[async_trait]
pub trait VersionControl: Send + Sync {
    async fn create_workspace(&self, task: &Task) -> Result<Workspace>;
    async fn get_diff(&self, workspace: &Workspace) -> Result<String>;
    async fn merge_workspace(&self, workspace: &Workspace) -> Result<MergeResult>;
    async fn cleanup_workspace(&self, workspace: &Workspace) -> Result<()>;
    async fn list_workspaces(&self) -> Result<Vec<Workspace>>;
    async fn get_conflicts(&self, workspace: &Workspace) -> Result<Vec<ConflictFile>>;
}
```

---

## 7. Workspace Configuration

### 7.1 Init Scripts

```bash
#!/bin/bash
# .opencode-studio/scripts/workspace-init.sh

WORKSPACE_PATH=$1
TASK_ID=$2
MAIN_REPO=$3

cd "$WORKSPACE_PATH"

# Symlink node_modules
if [ -d "$MAIN_REPO/node_modules" ]; then
    ln -sf "$MAIN_REPO/node_modules" ./node_modules
fi

# Copy a customize .env
if [ -f "$MAIN_REPO/.env" ]; then
    cp "$MAIN_REPO/.env" ./.env
    echo "TASK_ID=$TASK_ID" >> ./.env
    echo "DATABASE_NAME=myapp_test_$TASK_ID" >> ./.env
fi

# Setup test database
createdb "myapp_test_$TASK_ID" 2>/dev/null || true

# Run migrations
if [ -f "prisma/schema.prisma" ]; then
    npx prisma migrate deploy
fi
```

### 7.2 Cleanup Scripts

```bash
#!/bin/bash
# .opencode-studio/scripts/workspace-cleanup.sh

WORKSPACE_PATH=$1
TASK_ID=$2

# Stop dev servers
pkill -f "node.*$WORKSPACE_PATH" || true

# Drop test database
dropdb "myapp_test_$TASK_ID" 2>/dev/null || true

# Clear Redis namespace
redis-cli KEYS "task:$TASK_ID:*" | xargs -r redis-cli DEL || true
```

### 7.3 Kompletní konfigurace

```toml
[worktree]
base_path = "../.workspaces"
auto_cleanup = true
cleanup_delay_hours = 24
max_parallel_workspaces = 5

[worktree.init]
scripts = [".opencode-studio/scripts/workspace-init.sh"]
copy_files = [".env", ".env.local"]
symlink_dirs = ["node_modules", ".pnpm-store", "target", ".venv"]

[worktree.init.env]
DATABASE_URL = "postgresql://localhost/myapp_test_{task_id}"
PORT = "auto"

[worktree.cleanup]
scripts = [".opencode-studio/scripts/workspace-cleanup.sh"]
actions = ["stop_dev_server", "drop_test_database"]
```

---

## 8. GitHub Integration

### 8.1 Issues Sync
```toml
[github.issues]
sync_enabled = true
sync_labels = ["feature", "enhancement"]
auto_import = false  # Manuální import do roadmapy
```

### 8.2 Pull Requests
```toml
[github.pull_requests]
auto_create = true
draft = true
template = ".github/pull_request_template.md"
title_pattern = "[{task_id}] {task_title}"

[github.pull_requests.reviewers]
auto_assign = true
team = "developers"
```

### 8.3 CI Integration
```toml
[github.actions]
wait_for_ci = true
ci_timeout_minutes = 30
required_checks = ["test", "lint", "build"]
```

---

## 9. Technical Architecture

### 9.1 System Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              FRONTEND (Web UI)                               │
│                           React + TypeScript + Vite                          │
│                                                                              │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐          │
│  │  Kanban  │ │ Roadmap  │ │   Docs   │ │ Insights │ │ Settings │          │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘ └──────────┘          │
│                                                                              │
│                         WebSocket (real-time updates)                        │
└─────────────────────────────────────┬───────────────────────────────────────┘
                                      │
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                              API GATEWAY (Rust/Axum)                         │
│                                                                              │
│  REST API + WebSocket + Authentication (JWT, GitHub OAuth)                  │
└─────────────────────────────────────┬───────────────────────────────────────┘
                                      │
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                              CORE ENGINE (Rust)                              │
│                                                                              │
│  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐  ┌──────────────┐ │
│  │ Module System │  │   Event Bus   │  │ State Machine │  │   Scheduler  │ │
│  └───────────────┘  └───────────────┘  └───────────────┘  └──────────────┘ │
│                                                                              │
│  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐  ┌──────────────┐ │
│  │  VCS Manager  │  │OpenCode Client│  │ GitHub Client │  │Script Runner │ │
│  │  (jj / git)   │  │               │  │               │  │              │ │
│  └───────────────┘  └───────────────┘  └───────────────┘  └──────────────┘ │
└─────────────────────────────────────┬───────────────────────────────────────┘
                                      │
                    ┌─────────────────┼─────────────────┐
                    ▼                 ▼                 ▼
┌─────────────────────┐ ┌─────────────────────┐ ┌─────────────────────┐
│      SQLite DB      │ │    File System      │ │   OpenCode Server   │
│                     │ │                     │ │                     │
│  - Tasks metadata   │ │  - Plans (.md)      │ │  - Sessions         │
│  - Sessions         │ │  - Reviews (.md)    │ │  - Events (SSE)     │
│  - Events log       │ │  - Roadmap (.md)    │ │                     │
│                     │ │  - Workspaces       │ │                     │
└─────────────────────┘ └─────────────────────┘ └─────────────────────┘
```

### 9.2 Project Structure

```
opencode-studio/
├── Cargo.toml
├── crates/
│   ├── core/                    # Core engine, traits, events
│   │   └── src/
│   │       ├── config/
│   │       ├── events/
│   │       ├── state/
│   │       └── traits/
│   │
│   ├── modules/                 # Pluggable modules
│   │   └── src/
│   │       ├── kanban/
│   │       ├── roadmap/
│   │       ├── docs/
│   │       └── insights/
│   │
│   ├── vcs/                     # Version control abstraction
│   │   └── src/
│   │       ├── jj.rs
│   │       ├── git.rs
│   │       └── traits.rs
│   │
│   ├── opencode/                # OpenCode client
│   │   └── src/
│   │       ├── client.rs
│   │       ├── session.rs
│   │       └── events.rs
│   │
│   ├── github/                  # GitHub integration
│   ├── scripts/                 # Script runner
│   ├── db/                      # Database layer
│   ├── api/                     # HTTP API (Axum)
│   └── cli/                     # CLI binary
│
├── frontend/                    # Web UI (React)
│   ├── src/
│   │   ├── components/
│   │   ├── pages/
│   │   ├── hooks/
│   │   └── stores/
│   └── vite.config.ts
│
└── docs/
```

### 9.3 Data Storage (Hybrid)

**SQLite pro:**
- Task metadata (status, timestamps, relations)
- Session tracking
- Event log
- Rychlé queries

**Soubory pro:**
- Plány (`.md`) – čitelné, verzovatelné, OpenCode je může číst
- Reviews (`.md`) – historie rozhodnutí
- Roadmap items (`.md`) – produktová dokumentace
- Konfigurace (`.toml`)

### 9.4 Directory Structure

```
.opencode-studio/
├── config.toml
├── studio.db
│
├── kanban/
│   ├── tasks/
│   │   └── {id}.md
│   ├── plans/
│   │   └── {id}.md
│   └── reviews/
│       └── {id}.md
│
├── roadmap/
│   ├── roadmap.md
│   └── items/
│       └── {id}.md
│
├── changelog/
│   └── CHANGELOG.md
│
├── docs/
│   └── *.md
│
├── insights/
│   └── *.md
│
├── scripts/
│   ├── workspace-init.sh
│   └── workspace-cleanup.sh
│
└── sessions/
    └── {module}_{timestamp}.log
```

### 9.5 OpenCode Integration Strategy

> **Rozhodnutí (2024-12-30):** Po analýze škálovatelnosti a porovnání s vibe-kanban implementací volíme **HTTP Server API** přístup místo ACP (Agent Client Protocol).

#### 9.5.1 Přístupy k integraci

| Přístup | Popis | Výhody | Nevýhody |
|---------|-------|--------|----------|
| **ACP (subprocess)** | `npx opencode-ai acp` | Přímá kontrola, offline | Každý task = nový Node.js proces (~100MB RAM) |
| **HTTP Server API** ✅ | `opencode serve` + REST/SSE | Stateless, škálovatelné, SDK z OpenAPI | Vyžaduje běžící server |

#### 9.5.2 Proč HTTP Server API

1. **Horizontální škálování**: REST je stateless, jeden OpenCode server zvládne více sessions
2. **Resource efficiency**: Jeden server proces vs N Node.js procesů pro N tasků
3. **SDK generování**: OpenCode poskytuje OpenAPI 3.1 spec na `/doc` endpoint
4. **Distributed deployment**: OpenCode server může běžet na remote machine
5. **Paralelní tasky**: PRD specifikuje `parallel_tasks_limit = 5` - HTTP to zvládne efektivněji

#### 9.5.3 OpenCode Server API

OpenCode server (`opencode serve --port 4096`) poskytuje:

```
# Sessions
POST   /session                    # Vytvořit session
GET    /session/:id                # Detail session
POST   /session/:id/message        # Poslat zprávu (sync)
POST   /session/:id/prompt_async   # Poslat zprávu (async)
POST   /session/:id/abort          # Přerušit session
GET    /session/:id/diff           # Získat diff změn

# Real-time
GET    /event                      # SSE stream všech eventů
GET    /global/event               # Globální eventy

# Files & VCS
GET    /file?path=<path>           # List souborů
GET    /vcs                        # VCS info
```

#### 9.5.4 Rust SDK generování

```bash
# OpenCode poskytuje OpenAPI 3.1 spec
curl http://localhost:4096/doc > opencode-api.json

# Generování Rust klienta
cargo install openapi-generator-cli
openapi-generator generate -i opencode-api.json -g rust -o crates/opencode-sdk
```

Alternativně použít `progenitor` crate pro compile-time generování.

#### 9.5.5 Architektura integrace

```
┌─────────────────────────────────────────────────────────────────┐
│                     OpenCode Studio Backend                      │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                 crates/opencode/                          │   │
│  │                                                           │   │
│  │  ┌─────────────────┐    ┌─────────────────┐              │   │
│  │  │  OpenCodeClient │    │  SessionManager │              │   │
│  │  │  (generated SDK)│    │  (state tracking)│              │   │
│  │  └────────┬────────┘    └────────┬────────┘              │   │
│  │           │                      │                        │   │
│  │           ▼                      ▼                        │   │
│  │  ┌─────────────────────────────────────────┐             │   │
│  │  │            EventStream (SSE)             │             │   │
│  │  │  - session.message                       │             │   │
│  │  │  - task.status_changed                   │             │   │
│  │  │  - workspace.created                     │             │   │
│  │  └─────────────────────────────────────────┘             │   │
│  └──────────────────────────────────────────────────────────┘   │
│                              │                                   │
│                              ▼                                   │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                   OpenCode Server                         │   │
│  │                   (standalone process)                    │   │
│  │                   opencode serve --port 4096              │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

#### 9.5.6 Fallback strategie

Pro případ, kdy HTTP server není dostupný, zachováváme možnost ACP fallbacku:

```rust
#[async_trait]
pub trait AgentExecutor: Send + Sync {
    async fn create_session(&self, config: SessionConfig) -> Result<Session>;
    async fn send_prompt(&self, session_id: &str, prompt: &str) -> Result<()>;
    async fn subscribe_events(&self) -> Result<EventStream>;
    async fn abort_session(&self, session_id: &str) -> Result<()>;
}

// Implementace
pub struct HttpAgentExecutor { /* ... */ }  // Primární
pub struct AcpAgentExecutor { /* ... */ }   // Fallback (future)
```

---

## 10. Module System

### 10.1 Module Trait

```rust
#[async_trait]
pub trait AIModule: Send + Sync {
    fn id(&self) -> &'static str;
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn output_paths(&self) -> Vec<PathPattern>;
    
    async fn execute(&self, ctx: ModuleContext) -> Result<ModuleOutput>;
    async fn can_execute(&self, ctx: &ModuleContext) -> Result<bool>;
    async fn cleanup(&self, ctx: &ModuleContext) -> Result<()>;
}

pub struct ModuleContext {
    pub project: Project,
    pub config: ModuleConfig,
    pub opencode: OpenCodeClient,
    pub event_bus: EventBus,
    pub input: ModuleInput,
}

pub struct ModuleInput {
    pub trigger: Trigger,
    pub user_input: Option<String>,
    pub context_files: Vec<PathBuf>,
    pub parameters: HashMap<String, Value>,
}

pub enum Trigger {
    Manual,
    Scheduled(Schedule),
    Event(EventType),
    Hook(HookType),
}
```

### 10.2 Event System

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    // Task events
    TaskCreated { task_id: Uuid, title: String },
    TaskStatusChanged { task_id: Uuid, from: TaskStatus, to: TaskStatus },
    TaskCompleted { task_id: Uuid },
    
    // Session events
    SessionStarted { session_id: String, task_id: Uuid },
    SessionMessage { session_id: String, content: String },
    SessionCompleted { session_id: String },
    
    // VCS events
    WorkspaceCreated { task_id: Uuid, path: PathBuf },
    WorkspaceDeleted { task_id: Uuid },
    
    // GitHub events
    PullRequestCreated { task_id: Uuid, pr_number: u64 },
    CIStatusChanged { task_id: Uuid, status: CIStatus },
    
    // Module events
    ModuleStarted { module_id: String },
    ModuleCompleted { module_id: String, output: ModuleOutput },
}
```

### 10.3 State Machine

```rust
impl TaskStateMachine {
    pub fn transitions() -> HashMap<TaskStatus, Vec<TaskStatus>> {
        hashmap! {
            Todo => vec![Planning],
            Planning => vec![PlanningReview, InProgress, Todo],
            PlanningReview => vec![InProgress, Planning, Todo],
            InProgress => vec![AiReview, Planning, Todo],
            AiReview => vec![Review, InProgress],
            Review => vec![Done, InProgress],
            Done => vec![],
        }
    }
    
    pub async fn transition(
        task: &mut Task,
        to: TaskStatus,
        ctx: &TransitionContext,
    ) -> Result<()> {
        // Validate transition
        // Execute pre-hooks
        // Update state
        // Execute post-hooks (start sessions, create workspaces, etc.)
        // Emit event
    }
}
```

---

## 11. API Design

### 11.1 REST Endpoints

```
# Tasks
GET    /api/tasks
POST   /api/tasks
GET    /api/tasks/{id}
PATCH  /api/tasks/{id}
DELETE /api/tasks/{id}
POST   /api/tasks/{id}/transition    # Změna stavu

# Roadmap
GET    /api/roadmap
GET    /api/roadmap/items
POST   /api/roadmap/items
GET    /api/roadmap/items/{id}
PATCH  /api/roadmap/items/{id}
POST   /api/roadmap/items/{id}/to-kanban  # Přesun do kanbanu

# Sessions
GET    /api/sessions
GET    /api/sessions/{id}
GET    /api/sessions/{id}/messages

# Workspaces
GET    /api/workspaces
POST   /api/workspaces/{id}/dev-server/start
POST   /api/workspaces/{id}/dev-server/stop

# Config
GET    /api/config
PATCH  /api/config

# Modules
GET    /api/modules
POST   /api/modules/{id}/execute
```

### 11.2 WebSocket Events

```typescript
// Client subscribes to events
ws.onmessage = (event) => {
    const data = JSON.parse(event.data);
    
    switch (data.type) {
        case 'task.status_changed':
            // Update kanban board
            break;
        case 'session.message':
            // Show AI output in real-time
            break;
        case 'workspace.created':
            // Enable workspace actions
            break;
    }
};
```

---

## 12. User Interface

### 12.1 Main Layout

```
┌─────────────────────────────────────────────────────────────────┐
│  OpenCode Studio                            [Project ▼] [⚙️]    │
├─────────┬───────────────────────────────────────────────────────┤
│         │                                                       │
│ Kanban  │   [Aktuální view]                                    │
│ Roadmap │                                                       │
│ Docs    │                                                       │
│ Insights│                                                       │
│ Settings│                                                       │
│         │                                                       │
└─────────┴───────────────────────────────────────────────────────┘
```

### 12.2 Kanban View

```
┌─────────────────────────────────────────────────────────────────┐
│  Kanban                                      [+ New Task]       │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  TODO        PLANNING      IN_PROGRESS    AI_REVIEW    REVIEW   │
│  ┌───────┐   ┌───────┐    ┌───────┐      ┌───────┐   ┌───────┐ │
│  │Task A │   │Task B │    │Task C │      │Task D │   │Task E │ │
│  │       │   │ 🤖    │    │ 🤖    │      │ 🤖    │   │ 👤    │ │
│  └───────┘   └───────┘    └───────┘      └───────┘   └───────┘ │
│  ┌───────┐                                                      │
│  │Task F │                                                      │
│  └───────┘                                                      │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘

🤖 = Automatická fáze (AI pracuje)
👤 = Čeká na člověka
```

### 12.3 Task Detail (v REVIEW)

```
┌─────────────────────────────────────────────────────────────────┐
│  Task: Dark Mode Implementation                    [Approve ✓]  │
│  Status: REVIEW                                   [Reject ✗]   │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Tabs: [Plan] [Diff] [AI Review] [Dev Server] [Terminal]       │
│  ─────────────────────────────────────────────────────────────  │
│                                                                 │
│  ## Diff                                                        │
│  + src/context/ThemeContext.tsx (new)                          │
│  ~ src/components/Header.tsx (+15, -3)                         │
│  ~ tailwind.config.js (+8, -0)                                 │
│                                                                 │
│  [View Full Diff]  [Open in VS Code]                           │
│                                                                 │
│  ─────────────────────────────────────────────────────────────  │
│                                                                 │
│  ## Dev Server                                                  │
│  Status: Running on http://localhost:3042                       │
│  [Open in Browser]  [View Logs]  [Restart]                     │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 12.4 Roadmap View

```
┌─────────────────────────────────────────────────────────────────┐
│  Roadmap                                    [+ New Item] [AI ↻] │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Q1 2025                                                        │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │ 🟢 Dark Mode    │  │ 🟡 API v2       │  │ 🔵 Mobile App   │ │
│  │ In Development  │  │ Planned         │  │ Planned         │ │
│  │ [View] [Kanban] │  │ [View] [→Dev]   │  │ [View] [→Dev]   │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
│                                                                 │
│  Q2 2025                                                        │
│  ┌─────────────────┐  ┌─────────────────┐                      │
│  │ 🔵 Multi-tenant │  │ 🔵 Analytics    │                      │
│  │ Planned         │  │ Planned         │                      │
│  │ [View] [→Dev]   │  │ [View] [→Dev]   │                      │
│  └─────────────────┘  └─────────────────┘                      │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## 13. Configuration Reference

### 13.1 Kompletní config.toml

```toml
[project]
name = "my-app"
repository = "git@github.com:user/my-app.git"
default_branch = "main"

# ─────────────────────────────────────────────────────────────────
# VERSION CONTROL
# ─────────────────────────────────────────────────────────────────

[vcs]
backend = "jj"  # "jj" | "git"

[jj]
colocated = true
workspace_base_path = "../.workspaces"
auto_forget_workspace = true
cleanup_delay_hours = 24

[jj.parallel]
max_workspaces = 10

# ─────────────────────────────────────────────────────────────────
# WORKSPACE
# ─────────────────────────────────────────────────────────────────

[workspace]
base_path = "../.workspaces"
auto_cleanup = true
cleanup_delay_hours = 24
max_parallel = 5

[workspace.init]
scripts = [".opencode-studio/scripts/workspace-init.sh"]
copy_files = [".env", ".env.local"]
symlink_dirs = ["node_modules", ".pnpm-store", "target", ".venv"]

[workspace.init.env]
DATABASE_URL = "postgresql://localhost/myapp_test_{task_id}"
PORT = "auto"

[workspace.cleanup]
scripts = [".opencode-studio/scripts/workspace-cleanup.sh"]

# ─────────────────────────────────────────────────────────────────
# KANBAN
# ─────────────────────────────────────────────────────────────────

[kanban]
require_plan_approval = false
auto_start_dev_server = true
parallel_tasks_limit = 5

[kanban.models]
planning = "claude-sonnet-4-20250514"
implementation = "claude-sonnet-4-20250514"
review = "claude-sonnet-4-20250514"

# ─────────────────────────────────────────────────────────────────
# OPENCODE
# ─────────────────────────────────────────────────────────────────

[opencode]
host = "127.0.0.1"
port = 4096
auto_start = true

[opencode.context]
always_include = ["README.md", "ARCHITECTURE.md"]
ignore_patterns = ["node_modules/**", "*.lock", ".git/**"]
max_context_files = 50

# ─────────────────────────────────────────────────────────────────
# GITHUB
# ─────────────────────────────────────────────────────────────────

[github]
enabled = true
auth_method = "token"

[github.issues]
sync_enabled = true
sync_labels = ["feature", "enhancement"]
auto_import = false

[github.pull_requests]
auto_create = true
draft = true
title_pattern = "[{task_id}] {task_title}"

[github.pull_requests.reviewers]
auto_assign = true
team = "developers"

[github.actions]
wait_for_ci = true
required_checks = ["test", "lint", "build"]

# ─────────────────────────────────────────────────────────────────
# DEV SERVER
# ─────────────────────────────────────────────────────────────────

[dev_server]
auto_start_on_review = true
port_range = [3001, 3100]
health_check_path = "/health"

[dev_server.commands]
node = "npm run dev"
rust = "cargo run"
python = "python -m uvicorn main:app --reload"

# ─────────────────────────────────────────────────────────────────
# NOTIFICATIONS
# ─────────────────────────────────────────────────────────────────

[notifications]
on_review_ready = true
on_ai_review_failed = true
on_task_done = true

[notifications.channels]
desktop = true
slack_webhook = ""
```

---

## 14. Success Metrics

### 14.1 Efficiency Metrics
- **Time to first commit**: Čas od vytvoření tasku do prvního kódu
- **Automation rate**: % tasků dokončených bez human intervention v IN_PROGRESS
- **AI review accuracy**: % AI review rozhodnutí shodných s human review

### 14.2 Quality Metrics
- **First-pass success rate**: % tasků které projdou AI review napoprvé
- **Human rejection rate**: % tasků vrácených člověkem v REVIEW
- **Post-merge issues**: Počet bugů nalezených po merge

### 14.3 Adoption Metrics
- **Tasks per day**: Průměrný počet dokončených tasků
- **Parallel tasks**: Průměrný počet současně běžících tasků
- **User engagement**: Čas strávený v REVIEW vs celkový čas tasku

---

## 15. Roadmap

### Phase 1: MVP (4-6 týdnů)
- [x] Core architecture design
- [ ] Kanban module (TODO → DONE flow)
- [ ] OpenCode integration (sessions, SSE)
- [ ] Jujutsu VCS integration
- [ ] Basic web UI
- [ ] SQLite storage

### Phase 2: Automation (4-6 týdnů)
- [ ] PLANNING fáze s AI
- [ ] AI_REVIEW fáze
- [ ] Workspace init/cleanup scripts
- [ ] Dev server management
- [ ] GitHub PR integration

### Phase 3: Produktová vrstva (4-6 týdnů)
- [ ] Roadmap module
- [ ] Roadmap → Kanban flow
- [ ] AI roadmap generation
- [ ] Changelog generator

### Phase 4: Polish & Scale (4-6 týdnů)
- [ ] Documentation generator
- [ ] Code insights module
- [ ] Notifications
- [ ] Multi-project support
- [ ] Performance optimizations

---

## 16. Open Questions

1. **Session recovery**: Jak řešit situaci kdy OpenCode session spadne uprostřed implementace?

2. **Konfliktní merge**: Jak automaticky řešit konflikty při merge do main? Nechat na AI nebo eskalovat člověku?

3. **Cost management**: Jak trackovat a limitovat náklady na AI API calls?

4. **Multi-agent**: Mělo by být možné mít různé agenty pro různé typy tasků? (např. specialized frontend agent)

5. **Rollback**: Jak řešit situaci kdy se merged task ukáže jako problematický?

---

## 17. Appendix

### A. Glossary
- **Task**: Jednotka práce v kanbanu
- **Roadmap Item**: Produktová specifikace feature
- **Workspace**: Izolované vývojové prostředí (jj workspace / git worktree)
- **Session**: OpenCode session pro AI interakci
- **Change ID**: Jujutsu identifikátor změny (stabilní napříč rebases)

### B. Related Projects
- [Vibe Kanban](https://github.com/BloopAI/vibe-kanban) - Inspirace pro orchestraci AI agentů
- [OpenCode](https://github.com/sst/opencode) - AI coding agent
- [Jujutsu](https://github.com/jj-vcs/jj) - Next-gen VCS
