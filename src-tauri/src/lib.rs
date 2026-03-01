pub mod api;
mod commands;
pub mod cli;
pub mod config;
pub mod core;
pub mod provisioning;
pub mod db;
pub mod error;
mod state;
pub mod test_helpers;

use tauri::Manager;
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::TrayIconBuilder;

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
                    .join("tally-agentic-wallet.db")
                    .to_string_lossy()
                    .to_string();
            }

            let app_state = AppState::new(config.clone())
                .expect("Failed to create AppState");

            // Spawn periodic cleanup of expired approvals (every 5 minutes)
            let cleanup_db = app_state.db.clone();
            tauri::async_runtime::spawn(async move {
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

            // Initialize provisioning service (non-fatal if it fails)
            match crate::provisioning::ProvisioningService::new() {
                Ok(service) => {
                    let service = std::sync::Arc::new(service);
                    app.manage(service);
                    tracing::info!("Provisioning service initialized");
                }
                Err(e) => {
                    tracing::warn!(error = %e, "Failed to initialize provisioning service");
                }
            }

            // Spawn MCP HTTP server
            if config.mcp_enabled {
                let mcp_db = app_state.db.clone();
                let mcp_cli = app_state.cli.clone();
                let mcp_port = config.mcp_port;
                tauri::async_runtime::spawn(async move {
                    // Build MCP state with CLI executor so financial operations are actually executed
                    let mcp_state = crate::api::mcp_http_server::McpHttpState::new_with_cli(mcp_db, mcp_cli);
                    let router = crate::api::mcp_http_server::build_router(mcp_state);
                    let listener = match tokio::net::TcpListener::bind(
                        format!("127.0.0.1:{}", mcp_port),
                    )
                    .await
                    {
                        Ok(l) => l,
                        Err(e) => {
                            tracing::error!(error = %e, "Failed to bind MCP HTTP server");
                            return;
                        }
                    };
                    tracing::info!("MCP HTTP server listening on 127.0.0.1:{}", mcp_port);
                    if let Err(e) = axum::serve(listener, router).await {
                        tracing::error!(error = %e, "MCP HTTP server failed");
                    }
                });
            }

            app.manage(app_state);

            // System tray
            let open = MenuItemBuilder::with_id("open", "Open Wallet").build(app)?;
            let pause = MenuItemBuilder::with_id("pause", "Pause All Agents").build(app)?;
            let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
            let menu = MenuBuilder::new(app)
                .items(&[&open, &pause, &quit])
                .build()?;

            let tray_icon = tauri::image::Image::from_bytes(
                include_bytes!("../icons/32x32.png"),
            )?;

            TrayIconBuilder::new()
                .icon(tray_icon)
                .menu(&menu)
                .tooltip("Tally Agentic Wallet")
                .on_menu_event(|app, event| {
                    match event.id().as_ref() {
                        "open" => {
                            if let Some(window) = app.get_webview_window("main") {
                                window.show().ok();
                                window.set_focus().ok();
                            }
                        }
                        "pause" => {
                            // TODO: Toggle kill switch via app state
                            tracing::info!("Pause All Agents toggled from tray");
                        }
                        "quit" => {
                            std::process::exit(0);
                        }
                        _ => {}
                    }
                })
                .build(app)?;

            // Minimize to tray on window close instead of quitting
            if let Some(window) = app.get_webview_window("main") {
                let win = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        win.hide().ok();
                    }
                });
            }

            // Open devtools in debug builds
            #[cfg(debug_assertions)]
            if let Some(window) = app.get_webview_window("main") {
                window.open_devtools();
            }

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
            commands::provisioning::detect_tools,
            commands::provisioning::get_provisioning_preview,
            commands::provisioning::provision_tool,
            commands::provisioning::provision_all,
            commands::provisioning::unprovision_tool,
            commands::provisioning::unprovision_all,
            commands::provisioning::verify_provisioning,
            commands::provisioning::get_provisioning_state,
            commands::provisioning::exclude_tool,
            commands::provisioning::include_tool,
            commands::provisioning::refresh_detection,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
