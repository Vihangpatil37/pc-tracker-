# Frontend Architecture

The **Frontend** (`apps/desktop`) is built with React, Vite, and TailwindCSS. It serves as the primary visual interface for FocusOS.

## Tauri IPC
The frontend has absolutely no direct access to the filesystem or the SQLite database. Instead, it relies on Tauri Inter-Process Communication (IPC).

Whenever the dashboard needs data, it calls `invoke('command_name')` using `@tauri-apps/api/core`.

## UI Components
We use `shadcn/ui` for our component library. This gives us beautifully styled, accessible components without the bloat of a massive UI framework. All components are located in `apps/desktop/src/components/ui/`.
