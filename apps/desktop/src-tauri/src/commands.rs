use chrono::{Timelike, Utc};
use database::SqlitePool;
use tauri::State;

use crate::collector_service::{CurrentSessionInfo, SharedCurrentSession};

fn today_start() -> i64 {
    let now = Utc::now();
    (now - chrono::Duration::hours(now.hour() as i64)
        - chrono::Duration::minutes(now.minute() as i64)
        - chrono::Duration::seconds(now.second() as i64))
    .timestamp()
}

#[tauri::command]
pub async fn get_today_stats(pool: State<'_, SqlitePool>) -> Result<(i64, f64), String> {
    database::get_today_stats(&pool, today_start())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_top_apps(
    pool: State<'_, SqlitePool>,
    days: i64,
) -> Result<Vec<(String, i64, i64)>, String> {
    database::get_top_apps(&pool, today_start(), days)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_longest_session(
    pool: State<'_, SqlitePool>,
) -> Result<Option<(String, i64)>, String> {
    database::get_longest_session(&pool, today_start())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_most_opened_app(
    pool: State<'_, SqlitePool>,
    days: i64,
) -> Result<Option<(String, i64)>, String> {
    database::get_most_opened_app(&pool, today_start(), days)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_idle_percentage(
    pool: State<'_, SqlitePool>,
    days: i64,
) -> Result<f64, String> {
    let (total, idle) = database::get_idle_percentage(&pool, today_start(), days)
        .await
        .map_err(|e| e.to_string())?;
    Ok(if total > 0 {
        idle as f64 / total as f64
    } else {
        0.0
    })
}

#[tauri::command]
pub async fn get_timeline(
    pool: State<'_, SqlitePool>,
) -> Result<Vec<(String, String, i64, i64, i64)>, String> {
    database::get_timeline(&pool, today_start())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_current_session(
    state: State<'_, SharedCurrentSession>,
) -> Result<CurrentSessionInfo, String> {
    match state.lock() {
        Ok(info) => Ok(info.clone()),
        Err(e) => Err(format!("failed to lock current session state: {}", e)),
    }
}

#[tauri::command]
pub async fn get_daily_totals(
    pool: State<'_, SqlitePool>,
    days: i64,
) -> Result<Vec<(String, i64)>, String> {
    database::get_daily_totals(&pool, days)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_app_history(
    pool: State<'_, SqlitePool>,
    exe_name: String,
    days: i64,
) -> Result<Vec<(String, String, i64, i64, i64)>, String> {
    database::get_app_history(&pool, &exe_name, days)
        .await
        .map_err(|e| e.to_string())
}
