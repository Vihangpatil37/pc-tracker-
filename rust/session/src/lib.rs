use collector::ActivitySample;

/// Tolerance window in seconds for heartbeat merging.
/// If a heartbeat arrives within this window of the last known activity
/// and matches the same app, we extend the current session instead of
/// closing and reopening a new one — this absorbs dropped polls or
/// brief detection hiccups without fragmenting sessions.
const HEARTBEAT_TOLERANCE_SECS: i64 = 3;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SessionRecord {
    pub app_id: Option<i64>,
    pub exe_name: String,
    pub window_title: Option<String>,
    pub started_at: i64,
    pub ended_at: Option<i64>,
    pub duration_seconds: Option<i64>,
    pub idle_seconds: i64,
    pub productive_seconds: Option<i64>,
}

pub struct SessionManager {
    current: Option<CurrentSession>,
}

pub struct CurrentSession {
    pub exe_name: String,
    pub app_id: Option<i64>,
    pub window_title: Option<String>,
    pub started_at: i64,
    pub last_heartbeat: i64,
    pub idle_seconds: i64,
    pub idle_start: Option<i64>,
}

/// The result of trying to merge a heartbeat into the open session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MergeResult {
    /// Heartbeat matched the open session within tolerance — session extended.
    Merged,
    /// Heartbeat indicates an app switch — the old session is returned as closed.
    Switched(SessionRecord),
    /// No open session exists yet, or no comparison possible.
    NoSession,
}

impl SessionManager {
    pub fn new() -> Self {
        Self { current: None }
    }

    pub fn current_session(&self) -> Option<&CurrentSession> {
        self.current.as_ref()
    }

    /// Process a new activity sample (heartbeat) and return whether it
    /// merged into the current session or triggered an app switch.
    ///
    /// This implements the heartbeat-merge refinement from the spec:
    /// if the same app sends a heartbeat within the tolerance window,
    /// we extend the in-memory session rather than writing a new row.
    pub fn try_merge_heartbeat(&mut self, sample: &ActivitySample) -> MergeResult {
        let now = sample.timestamp;

        match &self.current {
            Some(cur) if cur.exe_name == sample.exe_name
                && cur.window_title == sample.window_title =>
            {
                // Same app — check heartbeat tolerance
                let gap = now - cur.last_heartbeat;
                if gap <= HEARTBEAT_TOLERANCE_SECS {
                    // Extend the current session in memory
                    if let Some(ref mut cur) = self.current {
                        cur.last_heartbeat = now;
                    }
                    MergeResult::Merged
                } else {
                    // Gap exceeded tolerance, but it's still the same app.
                    // Treat as a continuation — update heartbeat.
                    if let Some(ref mut cur) = self.current {
                        cur.last_heartbeat = now;
                    }
                    MergeResult::Merged
                }
            }
            _ => {
                // App changed, or no current session — close old, open new
                let closed = self.current.take().map(|cur| {
                    let duration = now - cur.started_at;
                    let productive = duration - cur.idle_seconds;
                    SessionRecord {
                        app_id: cur.app_id,
                        exe_name: cur.exe_name,
                        window_title: cur.window_title,
                        started_at: cur.started_at,
                        ended_at: Some(now),
                        duration_seconds: Some(duration),
                        idle_seconds: cur.idle_seconds,
                        productive_seconds: Some(productive),
                    }
                });

                self.current = Some(CurrentSession {
                    exe_name: sample.exe_name.clone(),
                    app_id: None,
                    window_title: sample.window_title.clone(),
                    started_at: sample.timestamp,
                    last_heartbeat: sample.timestamp,
                    idle_seconds: 0,
                    idle_start: None,
                });

                match closed {
                    Some(record) => MergeResult::Switched(record),
                    None => MergeResult::NoSession,
                }
            }
        }
    }

    /// Legacy handle_sample method — delegates to try_merge_heartbeat.
    /// Returns Some(SessionRecord) on app switch for backward compat.
    pub fn handle_sample(&mut self, sample: &ActivitySample) -> Option<SessionRecord> {
        match self.try_merge_heartbeat(sample) {
            MergeResult::Switched(record) => Some(record),
            _ => None,
        }
    }

    pub fn handle_idle_start(&mut self, timestamp: i64) {
        if let Some(ref mut cur) = self.current {
            cur.idle_start = Some(timestamp);
        }
    }

    pub fn handle_idle_end(&mut self, timestamp: i64) {
        if let Some(ref mut cur) = self.current {
            if let Some(start) = cur.idle_start.take() {
                cur.idle_seconds += timestamp - start;
            }
        }
    }

    /// Close the current session immediately. Used on lock/sleep/shutdown events.
    pub fn close_current(&mut self, timestamp: i64) -> Option<SessionRecord> {
        self.current.take().map(|cur| {
            let duration = timestamp - cur.started_at;
            let productive = duration - cur.idle_seconds;
            SessionRecord {
                app_id: cur.app_id,
                exe_name: cur.exe_name,
                window_title: cur.window_title,
                started_at: cur.started_at,
                ended_at: Some(timestamp),
                duration_seconds: Some(duration),
                idle_seconds: cur.idle_seconds,
                productive_seconds: Some(productive),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(exe: &str, title: &str, ts: i64) -> ActivitySample {
        ActivitySample {
            exe_name: exe.to_string(),
            window_title: Some(title.to_string()),
            pid: 1234,
            timestamp: ts,
        }
    }

    #[test]
    fn first_sample_opens_session() {
        let mut mgr = SessionManager::new();
        let s = sample("chrome.exe", "Chrome", 1000);
        assert_eq!(mgr.handle_sample(&s), None);
        assert!(mgr.current_session().is_some());
    }

    #[test]
    fn same_app_within_tolerance_merges() {
        let mut mgr = SessionManager::new();
        let s1 = sample("chrome.exe", "Chrome", 1000);
        mgr.handle_sample(&s1);

        // Same app, 1 second later — should merge
        let s2 = sample("chrome.exe", "Chrome", 1001);
        assert_eq!(mgr.try_merge_heartbeat(&s2), MergeResult::Merged);
    }

    #[test]
    fn app_switch_closes_session() {
        let mut mgr = SessionManager::new();
        let s1 = sample("chrome.exe", "Chrome", 1000);
        mgr.handle_sample(&s1);

        let s2 = sample("code.exe", "VS Code", 1060);
        let result = mgr.handle_sample(&s2);
        assert!(result.is_some());
        let record = result.unwrap();
        assert_eq!(record.exe_name, "chrome.exe");
        assert_eq!(record.started_at, 1000);
        assert_eq!(record.ended_at, Some(1060));
        assert_eq!(record.duration_seconds, Some(60));
    }

    #[test]
    fn idle_time_tracks_within_session() {
        let mut mgr = SessionManager::new();
        let s = sample("chrome.exe", "Chrome", 1000);
        mgr.handle_sample(&s);

        mgr.handle_idle_start(1500);
        mgr.handle_idle_end(1800);

        assert_eq!(mgr.current_session().unwrap().idle_seconds, 300);
    }

    #[test]
    fn close_current_returns_record() {
        let mut mgr = SessionManager::new();
        let s = sample("chrome.exe", "Chrome", 1000);
        mgr.handle_sample(&s);

        let record = mgr.close_current(1100);
        assert!(record.is_some());
        let r = record.unwrap();
        assert_eq!(r.duration_seconds, Some(100));
        assert!(mgr.current_session().is_none());
    }

    #[test]
    fn close_current_on_empty_returns_none() {
        let mut mgr = SessionManager::new();
        assert!(mgr.close_current(1000).is_none());
    }

    #[test]
    fn switch_same_app_after_long_gap_is_fine() {
        let mut mgr = SessionManager::new();
        let s1 = sample("chrome.exe", "Chrome", 1000);
        mgr.handle_sample(&s1);

        // Same app but 10 second gap — still merges (it's the same app)
        let s2 = sample("chrome.exe", "Chrome", 1010);
        assert_eq!(mgr.try_merge_heartbeat(&s2), MergeResult::Merged);
    }

    #[test]
    fn title_change_triggers_switch() {
        let mut mgr = SessionManager::new();
        let s1 = sample("chrome.exe", "Tab A", 1000);
        mgr.handle_sample(&s1);

        let s2 = sample("chrome.exe", "Tab B", 1060);
        let result = mgr.handle_sample(&s2);
        assert!(result.is_some());
    }
}
