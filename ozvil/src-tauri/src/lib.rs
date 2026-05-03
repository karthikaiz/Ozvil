pub mod cli;
pub mod commands;
pub mod core;
pub mod db;
pub mod profiles;
pub mod updater;
pub mod windows_adapter;

use tauri::Manager;

pub fn run_gui(safe_mode: bool) {
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .setup(move |app| {
            let db_path = app
                .path()
                .app_data_dir()
                .expect("failed to get app data dir")
                .join("ozvil.db");

            let state = core::AppState::new(db_path, safe_mode)
                .expect("failed to initialize app state");

            app.manage(state);

            // Spawn background update check (non-blocking, non-safe-mode only)
            if !safe_mode {
                let handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    updater::check_for_update(handle).await;
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_app_state_info,
            commands::get_profiles,
            commands::get_profile,
            commands::create_profile,
            commands::update_profile,
            commands::delete_profile,
            commands::start_profile,
            commands::stop_session,
            commands::restore_session,
            commands::dry_run_profile,
            commands::get_active_session,
            commands::get_sessions,
            commands::get_activity_logs,
            commands::export_logs_json,
            commands::export_logs_csv,
            commands::export_profile_json,
            commands::import_profile_json,
            commands::get_system_status,
            commands::get_top_offenders,
            commands::get_settings,
            commands::update_settings,
            commands::toggle_global_pause,
            commands::get_stale_sessions,
            commands::dismiss_stale_session,
            commands::get_approved_apps,
            commands::upsert_approved_app,
            commands::remove_approved_app,
            updater::check_update_command,
            updater::install_update_command,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Ozvil");
}
