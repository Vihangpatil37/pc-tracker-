# Collector Architecture

The **Collector** (`rust/collector`) is responsible for interfacing directly with the Windows API (Win32) to determine what the user is currently looking at.

## How it works

1. It periodically calls `GetForegroundWindow` to find the handle (HWND) of the currently active window.
2. It uses `GetWindowThreadProcessId` and `OpenProcess` to resolve the process ID to an executable name (e.g., `chrome.exe`).
3. It calls `GetWindowTextW` to retrieve the title of the window (e.g., "FocusOS - GitHub - Google Chrome").
4. It passes this data payload over to the `session-manager`.

This module uses `windows-sys` for lightweight, unsafe FFI calls, ensuring absolute minimal CPU overhead during background polling.
