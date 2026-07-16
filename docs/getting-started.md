# Getting Started

Welcome to FocusOS! This guide will help you set up your local development environment so you can start contributing.

## Prerequisites

FocusOS is a Tauri application. You will need:
- [Rust](https://www.rust-lang.org/tools/install) (The backend language)
- [Node.js](https://nodejs.org/) (The frontend runtime for building React)
- C++ Build Tools for Windows (Required by Rust/Tauri to compile native extensions)

## Installation Steps

1. **Clone the repository:**
   ```bash
   git clone https://github.com/yourusername/FocusOS.git
   cd FocusOS
   ```

2. **Install Node dependencies:**
   ```bash
   npm install
   ```

3. **Start the Development Server:**
   FocusOS uses Tauri's CLI. You can start both the Vite React dev server and the Rust native window with one command:
   ```bash
   npm run tauri dev
   ```

## Repository Structure

- `apps/desktop`: The main Tauri application containing the React frontend.
- `rust/`: The native Rust backend workspace.
  - `rust/collector`: Win32 API polling.
  - `rust/session`: State machine managing the current active window.
  - `rust/idle`: Global input hooking for idle detection.
  - `rust/database`: SQLite database interactions.
- `packages/`: Shared frontend utilities.

## Testing
- Run Rust tests: `cargo test --workspace`
- Run React tests: `npm run test`
