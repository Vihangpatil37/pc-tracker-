use database::SqlitePool;
use idle::IdleState;
use session::MergeResult;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::async_runtime;
use tokio::time::{sleep, Duration};
use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

/// Shared state for the currently active session, exposed to Tauri commands.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct CurrentSessionInfo {
    pub app_name: Option<String>,
    pub window_title: Option<String>,
    pub started_at: Option<i64>,
    pub duration_seconds: i64,
}

pub type SharedCurrentSession = Arc<Mutex<CurrentSessionInfo>>;

// Raw FFI imports for lock/sleep detection — these are Win32 APIs
// that the windows crate doesn't expose in v0.58 with our feature set.
extern "system" {
    fn OpenInputDesktop(dwFlags: u32, fInherit: i32, dwDesiredAccess: u32) -> isize;
    fn CloseDesktop(hDesktop: isize) -> i32;
}

const DESKREAD_OBJECTS: u32 = 0x0001;

/// Detect whether the workstation is locked by trying to open the input desktop.
/// When the workstation is locked (LUID), `OpenInputDesktop` returns NULL.
fn is_workstation_locked() -> bool {
    unsafe {
        let hdesk = OpenInputDesktop(0, 0, DESKREAD_OBJECTS);
        if hdesk == 0 {
            return true;
        }
        let _ = CloseDesktop(hdesk);
        false
    }
}

/// Detect whether the foreground window exists. NULL window handle
/// over 3+ consecutive polls suggests the system may be sleeping.
fn is_foreground_null() -> bool {
    unsafe {
        let hwnd = GetForegroundWindow();
        hwnd.is_invalid()
    }
}

pub struct CollectorService {
    running: Arc<AtomicBool>,
    pool: SqlitePool,
    current_session: SharedCurrentSession,
}

impl CollectorService {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            pool,
            current_session: Arc::new(Mutex::new(CurrentSessionInfo::default())),
        }
    }

    pub fn current_session_state(&self) -> SharedCurrentSession {
        self.current_session.clone()
    }

    pub fn start(&self) {
        self.running.store(true, Ordering::SeqCst);
        let running = self.running.clone();
        let pool = self.pool.clone();
        let current_session = self.current_session.clone();
        async_runtime::spawn(async move {
            let mut session_mgr = session::SessionManager::new();
            let mut idle_detector = idle::IdleDetector::new();
            let mut idle_open: Option<i64> = None;
            let mut was_locked = false;
            let mut sleep_polls: u32 = 0;
            let mut was_sleeping = false;

            // Log BOOT event on first start
            let boot_time = chrono::Utc::now().timestamp();
            if let Err(e) = database::insert_system_event(&pool, "BOOT", boot_time, None).await {
                tracing::error!(error = %e, "failed to persist boot event");
            }
            tracing::info!("collector service started");

            while running.load(Ordering::SeqCst) {
                let now = chrono::Utc::now().timestamp();

                // --- System event detection ---

                // Lock/unlock detection via OpenInputDesktop
                let locked = is_workstation_locked();
                if locked && !was_locked {
                    tracing::info!("system lock detected");
                    if let Some(record) = session_mgr.close_current(now) {
                        tracing::info!(
                            app = %record.exe_name,
                            duration_s = ?record.duration_seconds,
                            "Session closed on lock"
                        );
                        if let Err(e) = persist_session(&pool, &record).await {
                            tracing::error!(error = %e, "failed to persist session on lock");
                        }
                    }
                    if let Err(e) = database::insert_system_event(&pool, "LOCK", now, None).await {
                        tracing::error!(error = %e, "failed to persist lock event");
                    }
                    was_locked = true;
                } else if !locked && was_locked {
                    tracing::info!("system unlock detected");
                    if let Err(e) = database::insert_system_event(&pool, "UNLOCK", now, None).await {
                        tracing::error!(error = %e, "failed to persist unlock event");
                    }
                    // Reset idle state on unlock (user may have been away)
                    idle_detector = idle::IdleDetector::new();
                    idle_open = None;
                    was_locked = false;
                }

                // Sleep/wake detection: 3 consecutive NULL foreground window = sleeping
                // (The foreground window handle is NULL when the desktop is not visible,
                // e.g. during sleep, screen saver, or fast user switch.)
                if is_foreground_null() {
                    sleep_polls += 1;
                } else {
                    sleep_polls = 0;
                }

                let sleeping = sleep_polls >= 3;
                if sleeping && !was_sleeping {
                    tracing::info!("system sleep detected");
                    if let Some(record) = session_mgr.close_current(now) {
                        tracing::info!(
                            app = %record.exe_name,
                            duration_s = ?record.duration_seconds,
                            "Session closed on sleep"
                        );
                        if let Err(e) = persist_session(&pool, &record).await {
                            tracing::error!(error = %e, "failed to persist session on sleep");
                        }
                    }
                    if let Err(e) = database::insert_system_event(&pool, "SLEEP", now, None).await {
                        tracing::error!(error = %e, "failed to persist sleep event");
                    }
                    was_sleeping = true;
                } else if !sleeping && was_sleeping {
                    tracing::info!("system wake detected");
                    if let Err(e) = database::insert_system_event(&pool, "WAKE", now, None).await {
                        tracing::error!(error = %e, "failed to persist wake event");
                    }
                    was_sleeping = false;
                }

                // --- Activity collection (skip if locked or sleeping) ---
                if !locked && !sleeping {
                    if let Some(sample) = collector::sample_current() {
                        match session_mgr.try_merge_heartbeat(&sample) {
                            MergeResult::Switched(record) => {
                                tracing::info!(
                                    app = %record.exe_name,
                                    start = record.started_at,
                                    end = ?record.ended_at,
                                    duration_s = ?record.duration_seconds,
                                    "Session switch: {} for {}s",
                                    record.exe_name,
                                    record.duration_seconds.unwrap_or(0),
                                );
                                if let Err(e) = persist_session(&pool, &record).await {
                                    tracing::error!(error = %e, "failed to persist session");
                                }
                            }
                            MergeResult::Merged => {
                                // Heartbeat merged — no DB write, as designed
                            }
                            MergeResult::NoSession => {
                                tracing::info!(
                                    app = %sample.exe_name,
                                    "New session started"
                                );
                            }
                        }
                    }
                }

                // --- Update current session state for frontend ---
                if let Some(cur) = session_mgr.current_session() {
                    if let Ok(mut state) = current_session.lock() {
                        let duration = now - cur.started_at;
                        state.app_name = Some(cur.exe_name.clone());
                        state.window_title = cur.window_title.clone();
                        state.started_at = Some(cur.started_at);
                        state.duration_seconds = duration;
                    }
                } else {
                    if let Ok(mut state) = current_session.lock() {
                        state.app_name = None;
                        state.window_title = None;
                        state.started_at = None;
                        state.duration_seconds = 0;
                    }
                }

                // --- Idle detection (only when not locked/sleeping) ---
                if !locked && !sleeping {
                    match idle_detector.tick(now) {
                        IdleState::Idle { since } if idle_open.is_none() => {
                            tracing::info!(idle_start = since, "idle started");
                            session_mgr.handle_idle_start(now);
                            idle_open = Some(since);
                        }
                        IdleState::Active if idle_open.is_some() => {
                            let start = idle_open.take().unwrap();
                            let duration = now - start;
                            tracing::info!(idle_duration_s = duration, "idle ended");
                            session_mgr.handle_idle_end(now);
                            if let Err(e) =
                                database::insert_idle_event(&pool, start, now, duration).await
                            {
                                tracing::error!(error = %e, "failed to persist idle event");
                            }
                        }
                        _ => {}
                    }
                } else if idle_open.is_some() {
                    // If we were idle when lock/sleep hit, clear it
                    idle_open = None;
                }

                sleep(Duration::from_secs(1)).await;
            }

            // Log SHUTDOWN event when service stops
            let shutdown_time = chrono::Utc::now().timestamp();
            if let Err(e) = database::insert_system_event(&pool, "SHUTDOWN", shutdown_time, None).await {
                tracing::error!(error = %e, "failed to persist shutdown event");
            }
            tracing::info!("collector service stopped");
        });
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
}

async fn persist_session(pool: &SqlitePool, record: &session::SessionRecord) -> Result<(), sqlx::Error> {
    let app_id = database::upsert_app(pool, &record.exe_name).await?;
    database::insert_session(
        pool,
        app_id,
        record.window_title.as_deref(),
        record.started_at,
        record.ended_at.unwrap_or(0),
        record.duration_seconds.unwrap_or(0),
        record.idle_seconds,
        record.productive_seconds.unwrap_or(0),
    )
    .await?;
    Ok(())
}
