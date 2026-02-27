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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_clipboard_manager::init())
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
