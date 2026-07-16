# FocusOS — Master Build Prompt (Full Project, Phase 0 → Milestone 10)

> **How to use this file**
> Paste this entire document as the first message of a fresh agent session running on **DeepSeek V4 Flash** (reasoning effort: `high`, `xhigh` for the architecture/schema decisions in Sections 2–5). This model has a 1M-token context window — do not summarize or truncate this spec mid-session; keep the whole document in context for the full build so later milestones stay consistent with earlier decisions.
> This project is governed by two agent skills that must stay active for the entire session:
> - **Ponytail** (`full` mode) — every implementation decision must pass its six-rung decision ladder before new code is written.
> - **UI/UX Pro Max** — must be invoked for every screen/component built in Milestone 8 onward.
> Both are declared as hard constraints in Section 1. Do not relax them without the user's explicit instruction.

---

## 0. Role & Operating Rules for the Agent

You are the lead engineer building **FocusOS**, an open-source, privacy-first desktop activity tracker (Windows-first, Tauri 2 + Rust + React). You work milestone by milestone, in the exact order defined in Section 9. Follow these operating rules for the whole session:

1. **Plan before you code.** For each milestone, restate the goal in one sentence, list the files you will touch/create, then implement.
2. **Ponytail decision ladder — mandatory for every new piece of code:**
   1. Does this need to exist at all? (YAGNI — if the MVP scope in Section 1.2 doesn't require it, skip it.)
   2. Does something in this repo already do it? Reuse it.
   3. Does the Rust/JS standard library cover it? Use it.
   4. Does the OS/platform or browser have a native feature for it? Use it (e.g. native OS notifications, not a custom toast library).
   5. Is a dependency already in `Cargo.toml`/`package.json`? Use it before adding a new crate/package.
   6. Can it be one line / a small function? Prefer that over a new abstraction, class hierarchy, or config system.
   Only after all six checks fail should you write a new module. Never skip this for architectural code (collector, idle detection, session manager) — those need to be *correct*, not clever; "minimal" there means no speculative extensibility, not sloppy error handling.
3. **UI/UX Pro Max — mandatory for all frontend/dashboard work (Milestones 8–9).** Before generating any React screen or component, run the skill's design-system generation for a "privacy-first productivity desktop app" brief (see Section 8.1 for the exact brief to feed it), then implement against the tokens it returns. Do not hand-roll colors/spacing/fonts without going through the skill first.
4. **Never violate the non-goals in Section 1.3.** If a milestone seems to require AI, cloud, blocking, or gamification, stop and flag it instead of building it.
5. **After each milestone**, run the build (`cargo check` / `cargo build`, `npm run build`), confirm it compiles, and report: what was built, what was deliberately skipped per the Ponytail ladder, and what's next.
6. **Privacy is a correctness requirement, not a nice-to-have.** No network calls anywhere in the collector, database, or analytics layers. If you ever import an HTTP client in `rust/`, that's a bug — stop and reconsider.

---

## 1. Project Definition

### 1.1 Vision
Build the most accurate, privacy-first, open-source desktop activity tracker for Windows — the desktop equivalent of Android Digital Wellbeing — starting with screen time and application usage.

### 1.2 MVP Scope (v1.0 — the only thing you're building right now)
The app must answer exactly these questions, and nothing more:
- How long was the PC actively used today?
- Which applications were used?
- How long was spent in each app?
- When was the user idle?
- What does the day's activity timeline look like?

### 1.3 Non-Goals (do not build these, even if it seems easy or "nice to add")
No AI features. No app/website blocking. No Pomodoro timer. No cloud sync. No focus mode. No productivity scoring. No notifications/reminders beyond what's needed for core tracking. No plugin system. No mobile companion. (These live in the Future Roadmap in Section 10 — out of scope for this build.)

### 1.4 Definition of Success for v1.0
Ship when the app can, unassisted:
- Detect the active application with high accuracy.
- Measure active screen time while excluding idle periods.
- Record application sessions with start/end times.
- Correctly handle lock, unlock, sleep, wake, shutdown, and reboot.
- Store all data locally in SQLite with zero cloud dependency.
- Display daily and weekly statistics through a responsive dashboard.
- Run continuously at negligible CPU/RAM overhead (targets in Section 7).
- Never send activity data off the local machine.

---

## 2. Technology Stack (locked — do not substitute without asking)

| Layer | Choice | Why (for your own reasoning, not to relitigate) |
|---|---|---|
| Desktop shell | **Tauri 2** | lightweight, fast startup, small binary, low RAM, native Rust backend |
| Backend | **Rust** | collection, monitoring, OS events, idle detection, DB, background services |
| Frontend | **React + TypeScript** | dashboard, charts, timeline, statistics, settings |
| State management | **Zustand** | minimal boilerplate, fits Ponytail's "smallest thing that works" |
| Database | **SQLite** | fast, offline, zero-config, handles millions of rows |
| ORM/DB access | **Diesel or SQLx** (pick SQLx unless you hit a specific reason to prefer Diesel's compile-time schema checks — state your choice and move on) |
| Charts | **Recharts** |
| Notifications | **Native OS notifications only** (via Tauri's notification plugin) — no custom toast system |

Windows OS surfaces to research/integrate in Rust (Phase 1 research, Milestone 1–2 implementation):
- Active Window APIs (`GetForegroundWindow` + friends via `windows-rs` or `winapi`)
- Process APIs (executable name, PID)
- Idle detection (`GetLastInputInfo`)
- Lock screen / session change events (`WM_WTSSESSION_CHANGE` / `WTSRegisterSessionNotification`)
- Sleep/Wake events (`WM_POWERBROADCAST`)

---

## 3. High-Level Architecture

Implement this exact pipeline. Each stage is a distinct Rust module; keep boundaries clean because Milestones 4–7 build these one at a time.

```
Windows OS
   │
   ▼
Windows API Layer            (rust/collector — raw OS calls, no business logic)
   │
   ▼
Activity Collector Service   (rust/collector — polls active window every 1s)
   │
   ▼
Session Manager              (rust/session — tracks current session, detects app switches)
   │
   ▼
Idle Detection Engine        (rust/idle — mouse/keyboard inactivity, 5-minute threshold)
   │
   ▼
Event Processor              (turns raw events into session/idle records)
   │
   ▼
SQLite Database               (rust/database — insert/update/query)
   │
   ▼
Analytics Service             (rust/database or a thin analytics module — aggregates for the UI)
   │
   ▼
React Dashboard                (apps/desktop frontend — reads via Tauri commands)
```

The frontend **never computes** durations, idle time, or aggregates — it only renders what the Rust backend returns. This mirrors the "controllers stay thin, backend owns logic" principle Vihang uses on his other projects (SCPR); apply the same discipline here.

---

## 4. Project Structure

Scaffold exactly this layout in Milestone 1:

```
focus-os/
├── apps/
│   ├── desktop/            # Tauri app shell + React frontend
│   └── website/            # (stub only — not built in this MVP pass)
├── packages/
│   ├── shared-types/       # TS types mirrored from Rust structs (keep in sync manually for MVP; codegen is a future-roadmap item, not now)
│   ├── ui/                 # shared React components
│   └── analytics/          # frontend-side formatting helpers only (no computation)
├── rust/
│   ├── collector/          # active window + process + idle polling
│   ├── idle/               # idle detection engine
│   ├── database/           # SQLite access, migrations, queries
│   └── session/            # session manager, event processor
├── docs/                   # architecture doc, DB design doc (Milestone 0 deliverables — see Section 6)
├── scripts/
├── assets/
└── README.md
```

---

## 5. Database Design

SQLite, four tables. Use migrations (SQLx `migrate!` or Diesel migrations) from day one — never hand-edit schema.

**`apps`**
| column | type | notes |
|---|---|---|
| id | INTEGER PK AUTOINCREMENT | |
| name | TEXT NOT NULL | display name |
| exe_name | TEXT NOT NULL UNIQUE | e.g. `chrome.exe` — index this, it's the main lookup key |
| icon_path | TEXT NULL | cached extracted icon |
| category | TEXT NULL | optional, user- or heuristic-assigned |

**`sessions`**
| column | type | notes |
|---|---|---|
| id | INTEGER PK AUTOINCREMENT | |
| app_id | INTEGER NOT NULL REFERENCES apps(id) | index this |
| window_title | TEXT NULL | |
| started_at | INTEGER NOT NULL | unix timestamp |
| ended_at | INTEGER NULL | null while session is still open |
| duration_seconds | INTEGER NULL | computed on close, not on read |
| idle_seconds | INTEGER NOT NULL DEFAULT 0 | idle time that occurred during this session |
| productive_seconds | INTEGER NULL | `duration_seconds - idle_seconds`, stored for fast reads |

**`system_events`**
| column | type | notes |
|---|---|---|
| id | INTEGER PK AUTOINCREMENT | |
| event_type | TEXT NOT NULL | one of `LOCK`, `UNLOCK`, `BOOT`, `SHUTDOWN`, `SLEEP`, `WAKE` — enforce via Rust enum, not a DB CHECK constraint (keep the DB dumb, keep validation in Rust) |
| timestamp | INTEGER NOT NULL | |
| details | TEXT NULL | free-form JSON blob if needed, don't over-design this |

**`idle_events`**
| column | type | notes |
|---|---|---|
| id | INTEGER PK AUTOINCREMENT | |
| started_at | INTEGER NOT NULL | |
| ended_at | INTEGER NULL | null while still idle |
| duration | INTEGER NULL | computed on close |

Index `sessions(started_at)`, `sessions(app_id)`, and `idle_events(started_at)` — these back every analytics query in Section 8.

---

## 6. Phase 1 Deliverables (produce these *before* writing implementation code)

Before Milestone 1, produce four short markdown docs in `docs/`:
1. `docs/architecture.md` — the pipeline from Section 3, plus a short paragraph per stage on *why* it's a separate module.
2. `docs/database-design.md` — the schema from Section 5, plus your migration strategy.
3. `docs/event-flow.md` — walk through the worked example in Section 6.1 below, end to end, in prose.
4. `docs/coding-standards.md` — copy Section 11 of this document verbatim, it's already final.

### 6.1 Event Flow — worked example (put this in `docs/event-flow.md`)
```
10:00 → Chrome becomes active window
10:25 → still Chrome → nothing written to DB (no change = no write)
10:26 → VS Code becomes active window
      → close Chrome session (ended_at = 10:26, duration = 26 min)
      → open new session for VS Code (started_at = 10:26)
```
The rule that matters: **the collector only writes to the DB on a state change** (app switch, idle start/end, system event). Never write once per second — that would blow the performance budget in Section 7.

**Heartbeat-merge refinement (adapted from ActivityWatch's `aw-core`, a mature prior-art implementation of this exact problem):** instead of a naive "only write on change," treat each 1-second poll as a heartbeat and compare it to the *currently open* session, not just the previous poll. If the heartbeat's app matches the open session's app AND arrives within a small tolerance window (e.g. 2-3 seconds) of the last known activity, extend the open session's duration in memory rather than writing a new row — this absorbs a dropped poll or brief hiccup without fragmenting one real session into several DB rows. Only close the session and open a new one when the app genuinely changes, or the tolerance window is exceeded. Implement this as a pure function you can unit test in isolation (`fn try_merge_heartbeat(open_session, new_sample, tolerance_secs) -> MergeResult`) — this is exactly the kind of small, testable function Milestone 10's edge-case tests should target.

---

## 7. Performance Budget (non-functional requirements — treat as acceptance criteria, not aspirations)

| Metric | Target |
|---|---|
| CPU usage | < 1% |
| RAM usage | < 40 MB |
| Database writes | only on state change, never on a fixed timer |
| Battery impact | minimal |
| Cold launch time | < 2 seconds |

If a design choice in any milestone risks these numbers (e.g. polling faster than 1s, writing every tick, loading the whole `sessions` table into the frontend), flag it and pick the cheaper alternative — this is where the Ponytail ladder and the performance budget point the same direction.

---

## 8. Core Logic Specs

### 8.1 Activity Collector (Milestone 2–3)
Polls once per second: current window handle → executable name → process ID → window title → app icon (cache after first extraction, don't re-extract every poll) → timestamp. Emits an `ActivitySample` only to the Session Manager; does not touch the DB directly.

### 8.2 Idle Detection Engine (Milestone 6)
Checks mouse movement + keyboard input every second via `GetLastInputInfo` (leave a stub for touch/stylus — future roadmap, not now). If inactivity ≥ 5 minutes, emit `IdleStart`; on any input after that, emit `IdleEnd`. Idle time during an open session accumulates into that session's `idle_seconds`.

### 8.3 Session Manager (Milestone 4)
Owns "what's the current open session." On an app-switch event: close current session (`ended_at`, `duration_seconds`, `productive_seconds`), persist it, open a new session for the new app. On `IdleStart`/`IdleEnd`, update the currently open session's `idle_seconds` rather than opening a new session — idle time doesn't end a session, it's tracked *within* one.

### 8.4 System Events (fold into Session Manager + a small handler)
On `LOCK`/`SLEEP`: close the current session immediately (don't let it run indefinitely while the screen is locked). On `UNLOCK`/`WAKE`: start fresh polling as normal. On `BOOT`/`SHUTDOWN`: log the system event; don't try to be clever about reconstructing a session that was interrupted by a crash — closed is closed.

### 8.5 Analytics Engine (Milestone 7)
Compute, from the tables in Section 5, all via SQL aggregation (not by loading rows into Rust and summing in a loop unless the query genuinely can't express it):
- Today's total screen time (sum of `productive_seconds` for today)
- Top apps (group by `app_id`, order by summed duration, today/week/month)
- Average daily time (over the queried range)
- Longest single session
- Most-opened app (count of sessions, not duration)
- Idle percentage (`idle_seconds` / `duration_seconds` across the range)
- Timeline (ordered list of sessions for a given day, for the timeline view)

Expose these as Tauri commands the frontend calls directly — no separate analytics API layer, that would violate the Ponytail ladder (Section 0.2, rung 1: does a separate service need to exist here? No.).

---

## 9. Milestone Execution Order (build in this exact sequence)

| # | Milestone | Done when |
|---|---|---|
| 1 | Create project, install Tauri 2, scaffold folders from Section 4, run a blank desktop window | `npm run tauri dev` opens an empty window |
| 2 | Read the active window via the Windows API layer | Console/log prints the current foreground app's exe name on change |
| 3 | Detect app changes | Log shows `Chrome → VS Code → Discord` as you alt-tab |
| 4 | Session management (in-memory) | Switching apps produces a correct `Session { app, start, end, duration }` object in logs |
| 5 | SQLite integration | Sessions from Milestone 4 are persisted; `sessions` table has correct rows after a manual test run |
| 6 | Idle detection | 5 minutes of no input produces an `idle_events` row; idle time attributes correctly to the open session |
| 7 | Daily statistics (Analytics Engine) | Tauri commands for Section 8.5 return correct aggregates against seeded test data |
| 8 | Dashboard UI — Home + Applications screens | Run UI/UX Pro Max first (brief below), then build Home (today's screen time, current app, current session, most-used apps) and Applications (per-app today/week/month/history) screens against real data from Milestone 7 |
| 9 | Dashboard UI — Timeline + Statistics screens, charts | Timeline view renders the worked example shape from Section 6.1; Statistics screen shows top apps + daily/weekly/monthly trend charts via Recharts |
| 10 | Testing | Unit tests for Session Manager and Idle Detection edge cases (app switch mid-idle, lock during open session, session spanning midnight); manual test pass against the Definition of Success checklist in Section 1.4 |

**UI/UX Pro Max brief to feed the skill at the start of Milestone 8** (use this verbatim as the product brief):
> Product type: privacy-first desktop productivity/wellbeing app. Brand personality: calm, trustworthy, minimal, data-dense but not cluttered — think "Android Digital Wellbeing" crossed with a developer tool, not a consumer gamified app. Target audience: developers and power users who want an honest screen-time picture, not motivational nudging. Core screens: Home dashboard, per-app history, activity timeline, statistics with trend charts. Needs a clean light/dark pairing, a restrained color palette (no gamified "achievement" colors), and a font pairing that reads as precise rather than playful.

---

## 10. Explicitly Out of Scope for This Build (do not implement — reference only)
Website usage tracking, app/website limits, focus mode, Pomodoro timer, notifications/reminders beyond native OS ones already covered, AI-powered insights, productivity score, cross-device sync, plugin system, team dashboards, local REST API, mobile companion app. If you find yourself building toward any of these, stop — you've drifted from the MVP.

---

## 11. Coding Standards

**Rust:** feature-based modules; no global mutable state; strong typing; errors via `Result`, never `unwrap()`/`panic!()` outside of tests and truly unreachable branches; logging via `tracing`.

**React:** functional components only; hooks only, no class components; strict TypeScript (no `any` without a comment explaining why); reusable components live in `packages/ui`; feature-based folder structure, not type-based (i.e. `features/timeline/`, not a giant `components/` dumping ground).

**Database:** every schema change is a migration, never a hand edit; parameterized queries only, no string-built SQL; index every column used in a `WHERE` or `ORDER BY` in the queries from Section 8.5.

---

## 12. Reporting Format (use this after every milestone)

```
## Milestone N complete: <name>
**Built:** <1–2 sentences>
**Files touched:** <list>
**Skipped per Ponytail ladder:** <what you didn't build and which rung stopped you, or "nothing skipped">
**Verified:** <build/test command you ran and its result>
**Next:** Milestone N+1 — <name>
```

Do not move to the next milestone until the current one builds cleanly and matches its "Done when" criterion in Section 9.
