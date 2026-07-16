pub use sqlx::SqlitePool;
use sqlx::sqlite::{SqlitePoolOptions, SqliteConnectOptions};
use std::str::FromStr;

pub async fn create_pool(path: &str) -> Result<SqlitePool, sqlx::Error> {
    let connect_options = SqliteConnectOptions::from_str(path)?
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(connect_options)
        .await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}

pub async fn upsert_app(pool: &SqlitePool, exe_name: &str) -> Result<i64, sqlx::Error> {
    let row = sqlx::query_scalar::<_, i64>(
        "SELECT id FROM apps WHERE exe_name = ?1",
    )
    .bind(exe_name)
    .fetch_optional(pool)
    .await?;

    if let Some(id) = row {
        Ok(id)
    } else {
        let result = sqlx::query(
            "INSERT INTO apps (name, exe_name) VALUES (?1, ?2)",
        )
        .bind(exe_name)
        .bind(exe_name)
        .execute(pool)
        .await?;
        Ok(result.last_insert_rowid())
    }
}

pub async fn insert_session(
    pool: &SqlitePool,
    app_id: i64,
    window_title: Option<&str>,
    started_at: i64,
    ended_at: i64,
    duration: i64,
    idle: i64,
    productive: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO sessions (app_id, window_title, started_at, ended_at, duration_seconds, idle_seconds, productive_seconds) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
    )
    .bind(app_id)
    .bind(window_title)
    .bind(started_at)
    .bind(ended_at)
    .bind(duration)
    .bind(idle)
    .bind(productive)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn insert_system_event(
    pool: &SqlitePool,
    event_type: &str,
    timestamp: i64,
    details: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO system_events (event_type, timestamp, details) VALUES (?1, ?2, ?3)",
    )
    .bind(event_type)
    .bind(timestamp)
    .bind(details)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn insert_idle_event(
    pool: &SqlitePool,
    started_at: i64,
    ended_at: i64,
    duration: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO idle_events (started_at, ended_at, duration) VALUES (?1, ?2, ?3)",
    )
    .bind(started_at)
    .bind(ended_at)
    .bind(duration)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_today_stats(pool: &SqlitePool, date: i64) -> Result<(i64, f64), sqlx::Error> {
    let day_start = date;
    let day_end = date + 86400;
    let row = sqlx::query_as::<_, (Option<i64>, Option<f64>)>(
        "SELECT COALESCE(SUM(productive_seconds), 0), COALESCE(AVG(productive_seconds), 0.0)
         FROM sessions WHERE started_at >= ?1 AND started_at < ?2",
    )
    .bind(day_start)
    .bind(day_end)
    .fetch_one(pool)
    .await?;
    Ok((row.0.unwrap_or(0), row.1.unwrap_or(0.0)))
}

pub async fn get_top_apps(
    pool: &SqlitePool,
    date: i64,
    range_days: i64,
) -> Result<Vec<(String, i64, i64)>, sqlx::Error> {
    let start = date - range_days * 86400;
    let end = date + 86400;
    sqlx::query_as::<_, (String, i64, i64)>(
        "SELECT a.name, COALESCE(SUM(s.productive_seconds), 0), COUNT(s.id)
         FROM sessions s JOIN apps a ON s.app_id = a.id
         WHERE s.started_at >= ?1 AND s.started_at < ?2
         GROUP BY a.id ORDER BY 2 DESC LIMIT 20",
    )
    .bind(start)
    .bind(end)
    .fetch_all(pool)
    .await
}

pub async fn get_longest_session(pool: &SqlitePool, date: i64) -> Result<Option<(String, i64)>, sqlx::Error> {
    let day_start = date;
    let day_end = date + 86400;
    sqlx::query_as::<_, (String, i64)>(
        "SELECT a.name, s.duration_seconds
         FROM sessions s JOIN apps a ON s.app_id = a.id
         WHERE s.started_at >= ?1 AND s.started_at < ?2 AND s.duration_seconds IS NOT NULL
         ORDER BY s.duration_seconds DESC LIMIT 1",
    )
    .bind(day_start)
    .bind(day_end)
    .fetch_optional(pool)
    .await
}

pub async fn get_most_opened_app(pool: &SqlitePool, date: i64, range_days: i64) -> Result<Option<(String, i64)>, sqlx::Error> {
    let start = date - range_days * 86400;
    let end = date + 86400;
    sqlx::query_as::<_, (String, i64)>(
        "SELECT a.name, COUNT(s.id)
         FROM sessions s JOIN apps a ON s.app_id = a.id
         WHERE s.started_at >= ?1 AND s.started_at < ?2
         GROUP BY a.id ORDER BY 2 DESC LIMIT 1",
    )
    .bind(start)
    .bind(end)
    .fetch_optional(pool)
    .await
}

pub async fn get_idle_percentage(pool: &SqlitePool, date: i64, range_days: i64) -> Result<(i64, i64), sqlx::Error> {
    let start = date - range_days * 86400;
    let end = date + 86400;
    sqlx::query_as::<_, (Option<i64>, Option<i64>)>(
        "SELECT COALESCE(SUM(duration_seconds), 0), COALESCE(SUM(idle_seconds), 0)
         FROM sessions WHERE started_at >= ?1 AND started_at < ?2",
    )
    .bind(start)
    .bind(end)
    .fetch_one(pool)
    .await
    .map(|r| (r.0.unwrap_or(0), r.1.unwrap_or(0)))
}

pub async fn get_timeline(pool: &SqlitePool, date: i64) -> Result<Vec<(String, String, i64, i64, i64)>, sqlx::Error> {
    let day_start = date;
    let day_end = date + 86400;
    sqlx::query_as::<_, (String, String, i64, i64, i64)>(
        "SELECT a.name, COALESCE(s.window_title, ''), s.started_at, COALESCE(s.ended_at, s.started_at), COALESCE(s.duration_seconds, 0)
         FROM sessions s JOIN apps a ON s.app_id = a.id
         WHERE s.started_at >= ?1 AND s.started_at < ?2
         ORDER BY s.started_at ASC",
    )
    .bind(day_start)
    .bind(day_end)
    .fetch_all(pool)
    .await
}

pub async fn get_daily_totals(pool: &SqlitePool, days: i64) -> Result<Vec<(String, i64)>, sqlx::Error> {
    let now = chrono::Utc::now().timestamp();
    // Day-aligned boundaries: start of today = (now / 86400) * 86400
    let start_of_today = (now / 86400) * 86400;
    let start = start_of_today - (days - 1) * 86400;
    let end = start_of_today + 86400;
    sqlx::query_as::<_, (String, i64)>(
        "SELECT DATE(started_at, 'unixepoch') as day, COALESCE(SUM(productive_seconds), 0)
         FROM sessions WHERE started_at >= ?1 AND started_at < ?2
         GROUP BY day ORDER BY day ASC",
    )
    .bind(start)
    .bind(end)
    .fetch_all(pool)
    .await
}

pub async fn get_app_history(
    pool: &SqlitePool,
    exe_name: &str,
    range_days: i64,
) -> Result<Vec<(String, String, i64, i64, i64)>, sqlx::Error> {
    let now = chrono::Utc::now().timestamp();
    let start = now - range_days * 86400;
    sqlx::query_as::<_, (String, String, i64, i64, i64)>(
        "SELECT a.name, COALESCE(s.window_title, ''), s.started_at, COALESCE(s.ended_at, s.started_at), COALESCE(s.duration_seconds, 0)
         FROM sessions s JOIN apps a ON s.app_id = a.id
         WHERE a.exe_name = ?1 AND s.started_at >= ?2
         ORDER BY s.started_at DESC",
    )
    .bind(exe_name)
    .bind(start)
    .fetch_all(pool)
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn setup_test_db() -> Result<SqlitePool, sqlx::Error> {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await?;
        sqlx::migrate!("./migrations").run(&pool).await?;
        Ok(pool)
    }

    #[tokio::test]
    async fn test_upsert_app() {
        let pool = setup_test_db().await.unwrap();
        
        let id1 = upsert_app(&pool, "test.exe").await.unwrap();
        let id2 = upsert_app(&pool, "test.exe").await.unwrap();
        
        assert_eq!(id1, id2);
    }

    #[tokio::test]
    async fn test_insert_session() {
        let pool = setup_test_db().await.unwrap();
        let app_id = upsert_app(&pool, "test.exe").await.unwrap();
        
        insert_session(&pool, app_id, Some("Title"), 1000, 1100, 100, 10, 90).await.unwrap();
        
        let (total_time, avg_time) = get_today_stats(&pool, 1000).await.unwrap();
        assert_eq!(total_time, 90);
        assert_eq!(avg_time, 90.0);
    }

    #[tokio::test]
    async fn test_insert_system_event() {
        let pool = setup_test_db().await.unwrap();
        insert_system_event(&pool, "LOCK", 1000, None).await.unwrap();
        
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM system_events")
            .fetch_one(&pool).await.unwrap();
        assert_eq!(row.0, 1);
    }

    #[tokio::test]
    async fn test_insert_idle_event() {
        let pool = setup_test_db().await.unwrap();
        insert_idle_event(&pool, 1000, 1300, 300).await.unwrap();
        
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM idle_events")
            .fetch_one(&pool).await.unwrap();
        assert_eq!(row.0, 1);
    }

    #[tokio::test]
    async fn test_analytics_queries() {
        let pool = setup_test_db().await.unwrap();
        let app_id = upsert_app(&pool, "test.exe").await.unwrap();
        
        let date = 100000;
        insert_session(&pool, app_id, Some("T1"), date, date + 100, 100, 0, 100).await.unwrap();
        insert_session(&pool, app_id, Some("T2"), date + 200, date + 400, 200, 50, 150).await.unwrap();

        let top_apps = get_top_apps(&pool, date, 7).await.unwrap();
        assert_eq!(top_apps.len(), 1);
        assert_eq!(top_apps[0].1, 250);

        let longest = get_longest_session(&pool, date).await.unwrap().unwrap();
        assert_eq!(longest.1, 200);

        let most_opened = get_most_opened_app(&pool, date, 7).await.unwrap().unwrap();
        assert_eq!(most_opened.1, 2);

        let (total, idle) = get_idle_percentage(&pool, date, 7).await.unwrap();
        assert_eq!(total, 300);
        assert_eq!(idle, 50);

        let timeline = get_timeline(&pool, date).await.unwrap();
        assert_eq!(timeline.len(), 2);
    }
}
