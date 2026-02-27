mod commands;
mod cli;
mod config;
mod core;
mod db;
mod error;
mod state;
#[cfg(test)]
mod test_helpers;

use tauri::Manager;

use crate::config::AppConfig;
use crate::state::app_state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app| {
            let mut config = AppConfig::from_env();

            // In non-mock mode, resolve db_path relative to app data dir
            if !config.mock_mode {
                let app_data_dir = app
                    .path()
                    .app_data_dir()
                    .expect("Failed to get app data dir");
                std::fs::create_dir_all(&app_data_dir).expect("Failed to create app data dir");
                config.db_path = app_data_dir
                    .join("agent-neo-bank.db")
                    .to_string_lossy()
                    .to_string();
            }

            let app_state = AppState::new(config)
                .expect("Failed to create AppState");

            app.manage(app_state);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::auth::auth_login,
            commands::auth::auth_verify,
            commands::auth::auth_status,
            commands::auth::auth_logout,
            commands::wallet::get_balance,
            commands::wallet::get_address,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
