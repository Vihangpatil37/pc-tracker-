CREATE TABLE IF NOT EXISTS apps (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    exe_name TEXT NOT NULL UNIQUE,
    icon_path TEXT NULL,
    category TEXT NULL
);

CREATE TABLE IF NOT EXISTS sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    app_id INTEGER NOT NULL REFERENCES apps(id),
    window_title TEXT NULL,
    started_at INTEGER NOT NULL,
    ended_at INTEGER NULL,
    duration_seconds INTEGER NULL,
    idle_seconds INTEGER NOT NULL DEFAULT 0,
    productive_seconds INTEGER NULL
);

CREATE TABLE IF NOT EXISTS system_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_type TEXT NOT NULL,
    timestamp INTEGER NOT NULL,
    details TEXT NULL
);

CREATE TABLE IF NOT EXISTS idle_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    started_at INTEGER NOT NULL,
    ended_at INTEGER NULL,
    duration INTEGER NULL
);

CREATE INDEX IF NOT EXISTS idx_sessions_started_at ON sessions(started_at);
CREATE INDEX IF NOT EXISTS idx_sessions_app_id ON sessions(app_id);
CREATE INDEX IF NOT EXISTS idx_idle_events_started_at ON idle_events(started_at);
