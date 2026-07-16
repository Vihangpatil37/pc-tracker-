# FocusOS Architecture

## Pipeline

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
Analytics Service             (rust/database — aggregates for the UI)
   │
   ▼
React Dashboard               (apps/desktop frontend — reads via Tauri commands)
```

## Components

FocusOS is divided into two primary contexts:

1. **Rust Backend (`rust/`)**: The native system integration.
   - [Collector (Win32 API)](collector.md)
   - [Session Manager](session-manager.md)
   - [Database Manager](database-design.md)
   - [Event Flow](event-flow.md)
2. **React Frontend (`apps/desktop`)**: The UI layer.
   - [Frontend Architecture](frontend.md) The frontend never computes durations or aggregates — it only renders what the Rust backend returns via Tauri commands. This keeps the UI thin and the logic testable.
