mod collector_service;
mod commands;

use tauri::Manager;

pub fn run() {
    std::panic::set_hook(Box::new(|info| {
        tracing::error!("Panic occurred: {}", info);
        #[cfg(target_os = "windows")]
        {
            use windows::core::HSTRING;
            use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_OK, MB_ICONERROR};
            let msg = HSTRING::from(format!("A fatal error occurred:\n{}", info));
            let title = HSTRING::from("FocusOS Fatal Error");
            unsafe {
                let _ = MessageBoxW(None, &msg, &title, MB_OK | MB_ICONERROR);
            }
        }
    }));

    let log_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("FocusOS")
        .join("logs");
    
    std::fs::create_dir_all(&log_dir).expect("Failed to create log directory");

    let file_appender = tracing_appender::rolling::daily(&log_dir, "focusos.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    // To print to stdout AND file, we use a combined writer
    // But since this is a GUI app, file-only is preferred for v1 release reliability.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "focus_os=debug".into()),
        )
        .with_writer(non_blocking)
        .with_ansi(false)
        .init();

    let db_path = log_dir.parent().unwrap().join("focus_os.db");
    let db_url = format!("sqlite:{}", db_path.to_str().unwrap());

    let pool = tauri::async_runtime::block_on(async {
        database::create_pool(&db_url).await
    })
    .expect("failed to create database pool");

    let collector = collector_service::CollectorService::new(pool.clone());
    let current_session_state = collector.current_session_state();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(pool)
        .manage(collector)
        .manage(current_session_state)
        .invoke_handler(tauri::generate_handler![
            commands::get_today_stats,
            commands::get_top_apps,
            commands::get_longest_session,
            commands::get_most_opened_app,
            commands::get_idle_percentage,
            commands::get_timeline,
            commands::get_current_session,
            commands::get_daily_totals,
            commands::get_app_history,
        ])
        .setup(|app| {
            let collector = app.state::<collector_service::CollectorService>();
            collector.start();
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
