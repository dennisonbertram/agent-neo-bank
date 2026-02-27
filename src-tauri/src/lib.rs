pub mod api;
mod commands;
pub mod cli;
pub mod config;
pub mod core;
pub mod db;
pub mod error;
mod state;
pub mod test_helpers;

use std::sync::Arc;

use tauri::Manager;

use crate::config::AppConfig;
use crate::core::approval_manager::ApprovalManager;
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

            // Spawn periodic cleanup of expired approvals (every 5 minutes)
            let cleanup_db = app_state.db.clone();
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(300));
                loop {
                    interval.tick().await;
                    let db = cleanup_db.clone();
                    let result = tokio::task::spawn_blocking(move || {
                        let manager = ApprovalManager::new(db);
                        manager.cleanup_expired()
                    })
                    .await;
                    match result {
                        Ok(Ok(count)) => {
                            if count > 0 {
                                tracing::info!(count, "Cleaned up expired approvals");
                            }
                        }
                        Ok(Err(e)) => {
                            tracing::error!(error = %e, "Failed to cleanup expired approvals");
                        }
                        Err(e) => {
                            tracing::error!(error = %e, "Cleanup task panicked");
                        }
                    }
                }
            });

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
            commands::transactions::list_transactions,
            commands::transactions::get_transaction,
            commands::agents::list_agents,
            commands::agents::get_agent,
            commands::agents::get_agent_spending_policy,
            commands::agents::update_agent_spending_policy,
            commands::agents::suspend_agent,
            commands::agents::revoke_agent,
            commands::agents::get_agent_transactions,
            commands::approvals::list_approvals,
            commands::approvals::resolve_approval,
            commands::approvals::get_approval,
            commands::invitation_codes::list_invitation_codes,
            commands::invitation_codes::generate_invitation_code,
            commands::invitation_codes::revoke_invitation_code,
            commands::notifications::get_notification_preferences,
            commands::notifications::update_notification_preferences,
            commands::settings::get_global_policy,
            commands::settings::update_global_policy,
            commands::settings::toggle_kill_switch,
            commands::budget::get_agent_budget_summaries,
            commands::budget::get_global_budget_summary,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
