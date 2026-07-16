# FocusOS Database Design

## Schema

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
| app_id | INTEGER NOT NULL REFERENCES apps(id) | |
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
| event_type | TEXT NOT NULL | `LOCK`, `UNLOCK`, `BOOT`, `SHUTDOWN`, `SLEEP`, `WAKE` |
| timestamp | INTEGER NOT NULL | |
| details | TEXT NULL | Free-form JSON if needed |

### `idle_events`

| Column | Type | Notes |
|--------|------|-------|
| id | INTEGER PK AUTOINCREMENT | |
| started_at | INTEGER NOT NULL | |
| ended_at | INTEGER NULL | Null while still idle |
| duration | INTEGER NULL | Computed on close |

## Indexes

- `sessions(started_at)`
- `sessions(app_id)`
- `idle_events(started_at)`

## Migration Strategy

Use SQLx migrations (`migrate!`). Each schema change is a new numbered `.sql` file in `rust/database/migrations/`. Never hand-edit the schema. Migrations run automatically on app startup.
