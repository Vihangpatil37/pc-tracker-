<div align="center">
  <h1>FocusOS</h1>
  <p><strong>A free, open-source, privacy-first Windows activity tracker that helps you understand how you use your computer without sending a single byte of data to the cloud.</strong></p>
  
  <p>
    <a href="https://github.com/Vihangpatil37/pc-tracker-/releases/latest"><img alt="Latest Release" src="https://img.shields.io/github/v/release/Vihangpatil37/pc-tracker-?style=flat-square"></a>
    <a href="https://github.com/Vihangpatil37/pc-tracker-/blob/main/LICENSE"><img alt="License" src="https://img.shields.io/github/license/Vihangpatil37/pc-tracker-?style=flat-square"></a>
    <a href="https://github.com/Vihangpatil37/pc-tracker-/actions"><img alt="Build Status" src="https://img.shields.io/github/actions/workflow/status/Vihangpatil37/pc-tracker-/ci.yml?branch=main&style=flat-square"></a>
  </p>
</div>

---

<div align="center">
  <img src="./assets/screenshots/dashboard.png" alt="FocusOS Dashboard" width="800"/>
</div>

## ✨ Features

FocusOS is engineered to be the definitive digital wellbeing tool for Windows.

- **Screen Time Tracking**: Accurately tracks active screen time and logs which applications you use the most.
- **Smart Idle Detection**: Automatically detects when you walk away from your computer (no mouse/keyboard input) and pauses tracking so your data isn't artificially inflated.
- **Interactive Daily Timeline**: View a chronological breakdown of exactly what you were working on throughout the day.
- **100% Local & Private**: Powered by a local SQLite database stored in your AppData folder. Your data never leaves your machine. FocusOS does not even have networking capabilities.
- **Zero-Impact Performance**: Built in Rust utilizing raw Win32 APIs, the background tracking loop takes less than a millisecond to execute, ensuring near-zero CPU and RAM usage.
- **Modern Beautiful UI**: A sleek, dark-mode-first dashboard built with React, TailwindCSS, and shadcn/ui.

---

## 🏗️ Technology Stack

FocusOS utilizes a highly optimized dual-architecture leveraging the Tauri framework.

- **Backend / Core**: Rust (`std`, `tokio`)
- **Native OS Integration**: `windows-sys` (Direct Win32 API calls)
- **Database**: `sqlx` with SQLite
- **Frontend Framework**: React 18 & Vite
- **Styling & UI**: TailwindCSS, Radix UI, `shadcn/ui`, `lucide-react`
- **Application Shell**: Tauri 2.0

---

## 🕵️ How It Works Under The Hood

FocusOS operates entirely on your local machine using a robust background state machine.

1. **The 1-Second Polling Loop**: Every second, a Rust background thread uses `GetForegroundWindow` and `GetWindowThreadProcessId` to determine the exact executable name and window title of the currently focused application.
2. **Idle State Evaluation**: Simultaneously, it checks `GetLastInputInfo` to see how long it has been since the user touched the mouse or keyboard. If this exceeds a threshold (e.g., 5 minutes), the application enters an "Idle" state.
3. **State Machine Finalization**: The `session` manager compares this second's data to the previous second. If the application changed, it finalizes the previous session, saves the exact duration to the local SQLite database, and starts tracking the new application.
4. **Tauri IPC Bridge**: When you open the FocusOS window, the React dashboard sends an IPC message to the Rust backend (e.g., `invoke('get_today_stats')`). The backend runs highly optimized SQL aggregations and returns the payload to React for rendering.

---

## 🔒 The Privacy Guarantee

Most modern activity trackers require you to create an account, upload your deeply personal browsing and app habits to a cloud server, and pay a monthly subscription. 

**FocusOS is fundamentally different.**
We believe your screen time data is *your* business. The application is completely open-source, allowing anyone to audit the code. There is absolutely no telemetry, no cloud syncing, and no account requirements. 

---

## 🚀 Installation (For Users)

You don't need any development tools to use FocusOS.

1. Go to the [Releases page](https://github.com/Vihangpatil37/pc-tracker-/releases).
2. Download the latest `FocusOS_Setup.exe`.
3. Run the installer. FocusOS will automatically launch and begin securely tracking your activity in the background.

---

## 🛠️ Build From Source (For Developers)

FocusOS uses a monorepo structure utilizing NPM workspaces and Cargo workspaces.

### Prerequisites
- **Node.js** (v20+)
- **Rust** (Stable toolchain)
- **C++ Build Tools for Windows** (Required by Tauri to compile native Windows extensions)

### Getting Started

1. **Clone the repository:**
   ```bash
   git clone https://github.com/Vihangpatil37/pc-tracker-.git
   cd FocusOS
   ```

2. **Install Node dependencies:**
   ```bash
   npm install
   ```

3. **Start the Development Server:**
   This command starts the Vite React dev server and compiles the Rust backend, opening the native Tauri window automatically.
   ```bash
   npm run tauri dev
   ```

### Running Tests
We enforce strict testing for both the backend logic and the frontend utilities.
```bash
# Run backend Rust tests
cargo test --workspace

# Run frontend React tests
npm run test
```

---

## 📁 Directory Structure

```text
FocusOS/
├── apps/
│   └── desktop/               # The Tauri application and React frontend
│       ├── src/               # React UI components (Dashboard, Timeline, etc.)
│       └── src-tauri/         # Tauri specific configuration and entry point
├── rust/                      # The core native backend workspaces
│   ├── collector/             # Win32 API integration for active window polling
│   ├── database/              # SQLite database schema, migrations, and queries
│   ├── idle/                  # Win32 API integration for mouse/keyboard idle detection
│   └── session/               # The state machine managing active vs idle time
├── packages/                  # Shared TypeScript packages
│   └── analytics/             # Time parsing and formatting utilities
├── docs/                      # Extensive architecture documentation
└── .github/                   # GitHub Actions for CI/CD and Issue Templates
```

---

## 🗺️ Roadmap

FocusOS v1.0 establishes the core foundation. Future updates will introduce:
- **Data Export** (CSV, JSON, SQLite backup)
- **Categorization** (Tag apps as Development, Gaming, Learning, Work)
- **Weekly Reports** (Generated entirely locally in HTML/PDF formats)
- **Privacy Mode** (A toggle to redact sensitive window titles before saving to the database)

---

## 🤝 Contributing

We welcome contributions of all sizes! Whether you want to fix a bug, improve documentation, or add a new feature, please see our [Contributing Guide](CONTRIBUTING.md) to get started.

- If you find a bug, please [open an issue](https://github.com/Vihangpatil37/pc-tracker-/issues/new).
- Have a feature idea? Start a [Discussion](https://github.com/Vihangpatil37/pc-tracker-/discussions).

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
