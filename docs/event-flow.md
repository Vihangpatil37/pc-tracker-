# FocusOS Event Flow

## Worked Example

```
10:00 → Chrome becomes active window
10:25 → still Chrome → nothing written to DB (no change = no write)
10:26 → VS Code becomes active window
      → close Chrome session (ended_at = 10:26, duration = 26 min)
      → open new session for VS Code (started_at = 10:26)
```

## Key Rule

The collector only writes to the DB on a state change: app switch, idle start/end, system event. Never write once per second. No change = no write.

## Timeline Walkthrough

1. **10:00** — Collector polls `GetForegroundWindow`, sees Chrome. Collector emits `ActivitySample { exe: "chrome.exe", title: "Chrome", pid: 1234, timestamp: 10:00 }`. Session Manager sees no open session, opens a new session for Chrome with `started_at = 10:00`.

2. **10:01–10:25** — Collector polls once per second. Each poll returns Chrome. Session Manager compares: same app → no-op. Nothing written to DB.

3. **10:26** — Collector polls, sees VS Code. Emits sample with `exe: "code.exe"`. Session Manager detects app switch: closes Chrome session (`ended_at = 10:26`, `duration_seconds = 1560`), writes it to DB. Opens new VS Code session (`started_at = 10:26`).

4. **Idle scenario** — At some point user stops moving mouse. Idle Engine detects 5 min of no input, emits `IdleStart`. Session Manager records idle start. On first input after idle, emits `IdleEnd`, adds idle duration to the current session's `idle_seconds`.
