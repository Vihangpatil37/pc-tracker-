# Session Manager Architecture

The **Session Manager** (`rust/session`) is the brain of the backend. It receives raw telemetry from the `collector` and the `idle` modules and converts them into structured "Sessions" to be stored in the database.

## State Machine
The Session Manager holds a single `ActiveSession` state in memory. 

- When the `collector` reports a new window that matches the current session, the manager simply extends the duration.
- When the `collector` reports a different window, the manager finalized the previous session, writes it to the database, and starts a new one.
- When the `idle` module reports that the user hasn't moved their mouse or typed in X seconds, the Session Manager pauses the active session to prevent skewing the screen time statistics.
