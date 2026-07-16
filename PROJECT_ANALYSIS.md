# FocusOS — Complete Project Analysis

> **Project Name:** FocusOS
> **Version:** 0.1.0
> **Type:** Desktop Activity Tracker (Windows-first)
> **Tech Stack:** Tauri 2 + Rust + React + TypeScript + SQLite
> **Architecture:** Monorepo (npm workspaces + Cargo workspace)

---

## Table of Contents

1. [Project Overview](#1-project-overview)
2. [Root Directory Structure](#2-root-directory-structure)
3. [Workspace Configuration](#3-workspace-configuration)
4. [Rust Crates (`rust/`)](#4-rust-crates)
5. [Desktop App (`apps/desktop/`)](#5-desktop-app)
6. [Shared Packages (`packages/`)](#6-shared-packages)
7. [Documentation (`docs/`)](#7-documentation)
8. [Data Flow Pipeline](#8-data-flow-pipeline)
9. [Database Schema](#9-database-schema)
10. [Key Design Decisions](#10-key-design-decisions)

---

## 1. Project Overview

**FocusOS** is an open-source, privacy-first desktop activity tracker for Windows — the desktop equivalent of Android Digital Wellbeing. It tracks screen time, application usage, idle periods, and system events (lock/unlock, sleep/wake, boot/shutdown). All data stays local on the user's machine in a SQLite database. No cloud sync, no AI features, no gamification.

### MVP Goals
- Detect the active application with high accuracy
- Measure active screen time while excluding idle periods
- Record application sessions with start/end times
- Handle lock, unlock, sleep, wake, shutdown, and reboot
- Display daily and weekly statistics through a responsive dashboard
- Run continuously at negligible CPU/RAM overhead (< 1% CPU, < 40 MB RAM)
- Never send activity data off the local machine

---

## 2. Root Directory Structure

```
D:\pc tracker\
├── apps/                          # Frontend applications (Tauri desktop app)
│   └── desktop/                   # Main desktop application
├── packages/                      # Shared npm packages
│   ├── analytics/                 # Formatting helpers for the frontend
│   ├── shared-types/              # TypeScript types mirrored from Rust
│   └── ui/                        # Shared React UI components
├── rust/                          # Rust backend crates
│   ├── collector/                 # Active window + process polling
│   ├── session/                   # Session manager, app switch detection
│   ├── idle/                      # Idle detection engine
│   └── database/                  # SQLite access, migrations, queries
├── docs/                          # Project documentation
├── scripts/                       # (empty) Build/utility scripts
├── assets/                        # (empty) Static assets
├── .opencode/                     # OpenCode skills and config
│   └── skills/                    # UI/UX Pro Max skill
├── Cargo.toml                     # Rust workspace manifest
├── Cargo.lock                     # Rust dependency lock file
├── package.json                   # npm workspace manifest
├── package-lock.json              # npm dependency lock file
├── start.bat                      # Development startup script
├── FocusOS_Master_Build_Prompt.md # Master build specification
├── readme.md                      # (empty) Project README
└── .gitignore                     # Git ignore rules (node_modules only)
```

---

## 3. Workspace Configuration

### Root `package.json`
- **Name:** `focus-os`
- **Workspaces:** `apps/*`, `packages/*`
- **Scripts:**
  - `dev` — Runs the desktop app in dev mode
  - `build` — Builds the desktop app
  - `tauri` — Tauri CLI wrapper

### Root `Cargo.toml` (Rust Workspace)
- **Members:** `rust/collector`, `rust/session`, `rust/idle`, `rust/database`, `apps/desktop/src-tauri`
- **Resolver:** 2 (Rust 2021 edition)

### `start.bat`
Development startup script that:
1. Adds MSYS2 MinGW64 and Cargo bin to PATH
2. Changes to the desktop app directory
3. Runs `npx tauri dev`

---

## 4. Rust Crates (`rust/`)

The Rust backend is split into four focused crates, each with a single responsibility. This follows the project's architecture: separate concerns so each module can be tested and understood independently.

---

### 4.1 `rust/collector/` — Activity Collector

**Purpose:** Raw Windows API calls to detect the currently active window. Makes no business logic decisions — just samples the foreground window.

**Files:**
| File | Purpose |
|------|---------|
| `Cargo.toml` | Dependencies: `windows` crate for Win32 API, `serde`, `chrono` |
| `src/lib.rs` | Core collection logic — polls `GetForegroundWindow`, `GetWindowText`, `GetWindowThreadProcessId` |

**Key Types:**
- `ActivitySample` — Struct containing `exe_name`, `window_title`, `pid`, `timestamp`

**How it works:**
- Calls Windows APIs every 1 second to get the foreground window
- Extracts the executable name, window title, and process ID
- Returns an `ActivitySample` to the Session Manager
- Does NOT write to the database directly

**Why separate:** Isolates raw Windows API calls from the rest of the codebase. This means other modules never import `windows-rs` directly, making testing and potential porting easier.

---

### 4.2 `rust/session/` — Session Manager

**Purpose:** Tracks the currently active session, detects app switches, and manages session lifecycle (open, merge, close).

**Files:**
| File | Purpose |
|------|---------|
| `Cargo.toml` | Dependencies: `tracing`, `serde`, `chrono`, `collector`, `database` |
| `src/lib.rs` | Session state machine with heartbeat-merge logic, unit tests |

**Key Types:**
- `SessionManager` — Main state machine that tracks the current open session
- `CurrentSession` — In-memory representation of the active session
- `SessionRecord` — Complete session data (app, start/end times, duration, idle seconds, productive seconds)
- `MergeResult` — Enum: `Merged` (heartbeat extended session), `Switched` (app changed, old session closed), `NoSession` (no prior session)

**Key Constants:**
- `HEARTBEAT_TOLERANCE_SECS = 3` — If same app sends heartbeat within 3 seconds, extend the session instead of closing/reopening

**How it works:**
1. Receives `ActivitySample` from collector
2. If same app + same window title → merge (extend current session in memory, no DB write)
3. If app changed → close current session (calculate duration, idle time, productive time), open new session
4. On idle start/end → update current session's `idle_seconds` (idle doesn't end a session, it's tracked within one)
5. On lock/sleep/shutdown → force-close current session immediately

**Unit Tests:**
- `first_sample_opens_session` — First activity sample opens a new session
- `same_app_within_tolerance_merges` — Same app within 3s merges without DB write
- `app_switch_closes_session` — App change closes old session, opens new one
- `idle_time_tracks_within_session` — Idle time accumulates within a session
- `close_current_returns_record` — Force-close returns complete SessionRecord
- `title_change_triggers_switch` — Window title change triggers app switch

**Why separate:** Session logic is decoupled from the polling loop, so app-switch logic can be unit-tested without OS calls.

---

### 4.3 `rust/idle/` — Idle Detection Engine

**Purpose:** Detects user inactivity (mouse/keyboard) using Windows APIs. Uses a 5-minute threshold.

**Files:**
| File | Purpose |
|------|---------|
| `Cargo.toml` | Dependencies: `tracing`, `chrono` |
| `src/lib.rs` | Idle detection using `GetLastInputInfo` Win32 API, state machine |

**Key Types:**
- `IdleDetector` — State machine that tracks active/idle transitions
- `IdleState` — Enum: `Active`, `Idle { since: i64 }`

**Key Constants:**
- `IDLE_THRESHOLD_SECONDS = 300` (5 minutes) — Time without input before marking as idle

**How it works:**
1. Uses Win32 `GetLastInputInfo` to get time of last user input (mouse/keyboard)
2. Compares against `GetTickCount` to calculate idle duration
3. If idle >= 5 minutes → transitions to `Idle { since: timestamp }`
4. If user provides input → transitions back to `Active`
5. Returns the current state each tick for the collector to act on

**Win32 FFI:**
- `GetLastInputInfo` — Gets the last input event timestamp
- `GetTickCount` — Gets system uptime in milliseconds
- Uses raw FFI bindings (the `windows` crate v0.58 doesn't expose these with the project's feature set)

**Why separate:** Idle detection has a distinct input source (`GetLastInputInfo`) and a fixed threshold. Keeping it separate means the polling loop doesn't need to know about idle logic.

---

### 4.4 `rust/database/` — Database Layer

**Purpose:** All SQLite access — migrations, inserts, queries, analytics aggregations. Single point of database interaction.

**Files:**
| File | Purpose |
|------|---------|
| `Cargo.toml` | Dependencies: `sqlx` (SQLite), `tracing`, `serde`, `chrono`, `tokio` |
| `src/lib.rs` | All database operations: pool creation, CRUD, analytics queries |
| `migrations/20240715000001_initial.sql` | Initial schema migration |

**Key Functions:**

| Function | Purpose |
|----------|---------|
| `create_pool(path)` | Creates SQLite connection pool, runs migrations |
| `upsert_app(pool, exe_name)` | Inserts app if not exists, returns app ID |
| `insert_session(pool, ...)` | Inserts a completed session record |
| `insert_system_event(pool, type, time, details)` | Logs system events (BOOT, SHUTDOWN, LOCK, UNLOCK, SLEEP, WAKE) |
| `insert_idle_event(pool, start, end, duration)` | Logs idle periods |
| `get_today_stats(pool, date)` | Today's total productive time + average session length |
| `get_top_apps(pool, date, range)` | Top 20 apps by total usage over N days |
| `get_longest_session(pool, date)` | Longest single session today |
| `get_most_opened_app(pool, date, range)` | App with most sessions opened |
| `get_idle_percentage(pool, date, range)` | Idle time / total time ratio |
| `get_timeline(pool, date)` | Ordered list of today's sessions for timeline view |
| `get_daily_totals(pool, days)` | Daily screen time totals for trend charts |
| `get_app_history(pool, exe_name, days)` | Session history for a specific app |

**Design Notes:**
- Connection pool with `max_connections(1)` (SQLite doesn't benefit from concurrent writes)
- All queries are parameterized (no string-built SQL)
- Analytics computed via SQL aggregation (not loading rows into Rust)
- Migrations run automatically on app startup

**Why separate:** Schema changes don't touch other modules. Queries can be tuned independently. Database code is testable without OS or UI dependencies.

---

## 5. Desktop App (`apps/desktop/`)

The main Tauri 2 application — a desktop shell with a React frontend.

---

### 5.1 Root Files

| File | Purpose |
|------|---------|
| `package.json` | App dependencies and scripts |
| `tsconfig.json` | TypeScript configuration |
| `vite.config.ts` | Vite build configuration |
| `index.html` | HTML entry point (referenced by Vite) |

**Dependencies:**
- **UI:** React 19, Tailwind CSS 4, shadcn/ui, Lucide React icons
- **Charts:** Recharts
- **State:** Zustand
- **Backend:** Tauri API v2
- **Build:** Vite 6, TypeScript 5

**Dev Scripts:**
- `dev` — Starts Vite dev server
- `build` — TypeScript check + Vite build
- `tauri` — Tauri CLI wrapper

**Vite Config:**
- Path alias: `@` → `./src`
- Dev server on port 1420
- HMR on port 1421 (for Tauri remote dev)
- Ignores `src-tauri/` in file watcher

---

### 5.2 `src-tauri/` — Rust Backend

**Files:**
| File | Purpose |
|------|---------|
| `Cargo.toml` | Rust dependencies for the Tauri app |
| `build.rs` | Tauri build script (minimal — just calls `tauri_build::build()`) |
| `src/main.rs` | Entry point — calls `focus_os_lib::run()` |
| `src/lib.rs` | App setup: initializes tracing, database pool, collector service, Tauri commands |
| `src/commands.rs` | 9 Tauri commands exposed to the frontend |
| `src/collector_service.rs` | Background collector service with system event detection |
| `tauri.conf.json` | Tauri configuration (window size, build paths, bundle settings) |
| `capabilities/default.json` | Tauri permissions for the main window |
| `icons/` | App icons in various sizes (PNG, ICO, ICNS) |
| `gen/schemas/` | Auto-generated Tauri schemas |

**`Cargo.toml` Key Dependencies:**
- `tauri` v2 — Desktop shell framework
- `tauri-plugin-shell` — Shell integration
- `windows` v0.58 — Win32 API bindings (with specific features: Foundation, WindowsAndMessaging, Input, ProcessStatus, Threading, RemoteDesktop)
- `sqlx` v0.8 — Async SQLite
- `tokio` — Async runtime
- `chrono` — Date/time handling
- `tracing` / `tracing-subscriber` — Structured logging

**`src/lib.rs` — App Initialization:**
1. Initializes tracing with env-filter (defaults to `focus_os=debug`)
2. Creates SQLite connection pool (`focus_os.db`)
3. Creates `CollectorService` with shared state
4. Registers 9 Tauri commands
5. Starts the collector service on app setup

**`src/commands.rs` — Tauri Commands (9 total):**

| Command | Returns | Purpose |
|---------|---------|---------|
| `get_today_stats` | `(i64, f64)` | Total screen time + avg session length |
| `get_top_apps(days)` | `Vec<(String, i64, i64)>` | Top apps with usage seconds and session count |
| `get_longest_session` | `Option<(String, i64)>` | Longest session today |
| `get_most_opened_app(days)` | `Option<(String, i64)>` | App with most sessions |
| `get_idle_percentage(days)` | `f64` | Idle time ratio |
| `get_timeline` | `Vec<...>` | Today's session timeline |
| `get_current_session` | `CurrentSessionInfo` | Currently active session |
| `get_daily_totals(days)` | `Vec<(String, i64)>` | Daily screen time for charts |
| `get_app_history(exe_name, days)` | `Vec<...>` | Session history for a specific app |

**`src/collector_service.rs` — Background Collector:**
- Runs in a Tokio task, polls every 1 second
- Detects system events: LOCK/UNLOCK (via `OpenInputDesktop`), SLEEP/WAKE (via null foreground window detection × 3 polls)
- On LOCK/SLEEP: force-closes current session, logs system event
- On UNLOCK/WAKE: logs system event, resets idle detector
- On BOOT: logs BOOT event
- On SHUTDOWN: logs SHUTDOWN event
- On normal operation: samples activity, merges heartbeats, manages idle detection
- Updates `SharedCurrentSession` state for the frontend to read

**Tauri Configuration:**
- Window: 1024×768, resizable, not fullscreen
- Build: frontend at `../dist`, dev at `http://localhost:1420`
- Bundle: targets all platforms
- Security: CSP disabled (for development)

---

### 5.3 `src/` — React Frontend

**Files:**
| File | Purpose |
|------|---------|
| `main.tsx` | React entry point — renders App with StrictMode, forces dark mode |
| `App.tsx` | Root component — tab navigation (Dashboard, Timeline, Statistics, Apps) |
| `Dashboard.tsx` | Main dashboard — current session, stats grid, activity level, top apps |
| `Timeline.tsx` | Visual timeline of today's sessions with color-coded bars |
| `Statistics.tsx` | Statistical view with daily trends chart and top apps bar chart |
| `Applications.tsx` | App browser — list view with search, detail view with session history |
| `index.css` | Global styles — Tailwind CSS, shadcn/ui theme, dark mode, CSS variables |
| `lib/utils.ts` | `cn()` utility — Tailwind class merging with clsx |
| `vite-env.d.ts` | Vite environment type declarations |

**`App.tsx` — Tab Navigation:**
- 4 tabs: Dashboard, Timeline, Statistics, Apps
- Uses shadcn/ui Tabs component
- Header shows FocusOS logo + "privacy-first · local-only" tagline

**`Dashboard.tsx` — Main Dashboard:**
- **Current Session Card:** Shows active app name, window title, duration, start time
- **Stats Grid:** Today's screen time, average session, idle percentage, longest session
- **Activity Level:** Progress bar showing active vs idle ratio
- **Top Apps:** Ranked list with usage bars (top 8)
- Polls every 3 seconds for live updates
- Uses `@focus-os/analytics` for formatting

**`Timeline.tsx` — Visual Timeline:**
- Shows all of today's sessions as colored bars on a timeline
- Each app gets a deterministic color based on name hash
- Visual bars show start/end times with relative positioning
- Polls every 5 seconds for live updates

**`Statistics.tsx` — Statistical Analysis:**
- Range selector: 7, 14, or 30 days
- Summary cards: total time, avg session, idle %, most used app
- Daily Screen Time: Area chart (Recharts) showing daily trends
- Top Applications: Horizontal bar chart (Recharts) showing app usage
- Fetches data on range change

**`Applications.tsx` — App Browser:**
- List view: All apps with usage, session count, search functionality
- Detail view: Click an app to see its session history
- Range selector: Today, 7 days, 30 days
- Session history shows window title, timestamps, duration
- Search filtering by app name

**`index.css` — Theme & Styling:**
- Imports: Tailwind CSS, tw-animate-css, shadcn/ui, Geist Variable font
- Light and dark mode CSS variables using oklch color space
- Full shadcn/ui theme with custom properties
- Dark mode is default (forced in `main.tsx`)

**UI Components (shadcn/ui):**
| Component | File |
|-----------|------|
| Button | `components/ui/button.tsx` |
| Card | `components/ui/card.tsx` |
| Progress | `components/ui/progress.tsx` |
| ScrollArea | `components/ui/scroll-area.tsx` |
| Tabs | `components/ui/tabs.tsx` |

---

## 6. Shared Packages (`packages/`)

### 6.1 `packages/analytics/` — Formatting Helpers

**Package:** `@focus-os/analytics`
**Type:** Pure TypeScript, no dependencies

**Exports:**
| Function | Purpose |
|----------|---------|
| `formatDuration(seconds)` | "2h 15m", "45m 30s", "30s" |
| `formatTime(ts)` | Unix timestamp → localized time (HH:MM) |
| `formatDate(ts)` | Unix timestamp → localized date (Mon DD) |
| `formatPercentage(part, total)` | "75%" |
| `cleanExeName(exe)` | "chrome.exe" → "chrome" |

**Design:** Formatting only — no computation, no aggregation. The Rust backend owns all logic; this package just formats for display.

---

### 6.2 `packages/shared-types/` — TypeScript Types

**Package:** `@focus-os/shared-types`
**Type:** Pure TypeScript, no dependencies

**Exports:**
| Interface | Purpose |
|-----------|---------|
| `ActivitySample` | Raw activity sample from collector |
| `Session` | Complete session record |
| `AppInfo` | Application metadata |
| `SystemEvent` | System event (boot, lock, etc.) |
| `IdleEvent` | Idle period record |
| `DailyStats` | Aggregated daily statistics |
| `AppUsage` | Per-app usage summary |

**Design:** Manually kept in sync with Rust structs (codegen is a future roadmap item).

---

### 6.3 `packages/ui/` — Shared React Components

**Package:** `@focus-os/ui`
**Dependencies:** React 19

**Exports:**
| Component | Purpose |
|-----------|---------|
| `Card` | Simple card wrapper with title and border |

**Status:** Minimal — only one component so far. Most UI uses shadcn/ui components directly in the desktop app.

---

## 7. Documentation (`docs/`)

| File | Purpose |
|------|---------|
| `architecture.md` | High-level pipeline diagram and module rationale |
| `database-design.md` | Full schema documentation with column descriptions |
| `event-flow.md` | Worked example of event processing (Chrome → VS Code switch) |
| `coding-standards.md` | Rust, React, and Database coding standards |

**`architecture.md`:**
Documents the pipeline: Windows OS → Collector → Session Manager → Idle Detection → Event Processor → SQLite → Analytics → React Dashboard. Explains why each module is separate.

**`database-design.md`:**
Full schema for 4 tables: `apps`, `sessions`, `system_events`, `idle_events`. Documents indexes and migration strategy.

**`event-flow.md`:**
Step-by-step walkthrough: Chrome becomes active at 10:00, stays for 26 minutes, VS Code switches at 10:26. Shows how sessions are opened/closed and when DB writes happen. Key rule: "only write on state change, never on a fixed timer."

**`coding-standards.md`:**
- Rust: feature-based modules, no global mutable state, Result errors, tracing logging
- React: functional components, hooks only, strict TypeScript, feature-based folder structure
- Database: migrations only, parameterized queries, indexed columns

---

## 8. Data Flow Pipeline

```
┌─────────────────────────────────────────────────────────────────┐
│                        Windows OS                               │
│  (foreground window, input events, power state, lock state)     │
└──────────────────────────┬──────────────────────────────────────┘
                           │ Win32 APIs
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│  rust/collector (ActivitySample)                                │
│  - GetForegroundWindow → exe_name, window_title, pid            │
│  - Polls every 1 second                                         │
│  - No business logic, just raw sampling                         │
└──────────────────────────┬──────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│  rust/session (SessionManager)                                  │
│  - Heartbeat merge: same app within 3s tolerance → extend       │
│  - App switch: close old session, open new one                  │
│  - Idle tracking: accumulate idle_seconds within session        │
│  - Force-close on lock/sleep/shutdown                           │
└──────────────────────────┬──────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│  rust/idle (IdleDetector)                                       │
│  - GetLastInputInfo → idle duration                             │
│  - Threshold: 300 seconds (5 minutes)                           │
│  - State machine: Active ↔ Idle                                 │
└──────────────────────────┬──────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│  rust/database (SQLite)                                         │
│  - Upsert apps, insert sessions, log system/idle events         │
│  - Analytics queries (aggregation via SQL)                      │
│  - Migrations run on startup                                    │
└──────────────────────────┬──────────────────────────────────────┘
                           │ Tauri Commands
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│  React Frontend (apps/desktop/src/)                             │
│  - Dashboard: live stats, current session, top apps             │
│  - Timeline: visual session bars for today                      │
│  - Statistics: daily trends, app rankings (Recharts)            │
│  - Applications: app browser with search and history            │
└─────────────────────────────────────────────────────────────────┘
```

---

## 9. Database Schema

### `apps`
| Column | Type | Notes |
|--------|------|-------|
| id | INTEGER PK AUTOINCREMENT | |
| name | TEXT NOT NULL | Display name |
| exe_name | TEXT NOT NULL UNIQUE | e.g. `chrome.exe` — main lookup key |
| icon_path | TEXT NULL | Cached extracted icon |
| category | TEXT NULL | Optional user- or heuristic-assigned |

### `sessions`
| Column | Type | Notes |
|--------|------|-------|
| id | INTEGER PK AUTOINCREMENT | |
| app_id | INTEGER NOT NULL FK → apps(id) | |
| window_title | TEXT NULL | |
| started_at | INTEGER NOT NULL | Unix timestamp |
| ended_at | INTEGER NULL | Null while session is still open |
| duration_seconds | INTEGER NULL | Computed on close |
| idle_seconds | INTEGER NOT NULL DEFAULT 0 | Idle time during this session |
| productive_seconds | INTEGER NULL | `duration_seconds - idle_seconds` |

### `system_events`
| Column | Type | Notes |
|--------|------|-------|
| id | INTEGER PK AUTOINCREMENT | |
| event_type | TEXT NOT NULL | BOOT, SHUTDOWN, LOCK, UNLOCK, SLEEP, WAKE |
| timestamp | INTEGER NOT NULL | |
| details | TEXT NULL | Free-form JSON if needed |

### `idle_events`
| Column | Type | Notes |
|--------|------|-------|
| id | INTEGER PK AUTOINCREMENT | |
| started_at | INTEGER NOT NULL | |
| ended_at | INTEGER NULL | |
| duration | INTEGER NULL | Computed on close |

### Indexes
- `sessions(started_at)` — Backs timeline and date-range queries
- `sessions(app_id)` — Backs per-app analytics
- `idle_events(started_at)` — Backs idle queries

---

## 10. Key Design Decisions

1. **Write-on-change, not write-on-tick:** The collector only writes to the database when something changes (app switch, idle start/end, system event). Never writes once per second. This keeps the database small and performance high.

2. **Heartbeat merge (inspired by ActivityWatch):** If the same app sends a heartbeat within 3 seconds, extend the current session in memory instead of closing/reopening. This absorbs dropped polls or brief hiccups without fragmenting one real session into multiple DB rows.

3. **Frontend is thin:** The React frontend never computes durations, aggregates, or analytics. It only renders what the Rust backend returns via Tauri commands. All logic lives in Rust.

4. **Privacy by design:** Zero network calls anywhere in the collector, database, or analytics layers. All data stays local in SQLite.

5. **Feature-based module separation:** Each Rust crate has a single responsibility. Collector doesn't know about sessions, sessions don't know about the database, idle detection is independent. This makes testing and maintenance easier.

6. **Performance budget:** < 1% CPU, < 40 MB RAM, < 2 second cold launch. Every design choice is measured against these targets.

---

*Analysis generated from source code inspection. All files read and verified against actual codebase state.*
