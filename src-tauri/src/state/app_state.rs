use std::sync::Arc;
use std::time::Duration;

use crate::cli::executor::{CliExecutable, MockCliExecutor, RealCliExecutor};
use crate::config::AppConfig;
use crate::core::auth_service::AuthService;
use crate::core::wallet_service::WalletService;
use crate::db::schema::Database;
use crate::error::AppError;

/// AppState wraps the CLI executor, database, auth service, wallet service,
/// and config for Tauri state management.
/// Stored via `app.manage(AppState::new(...))` in the Tauri setup hook.
pub struct AppState {
    pub cli: Arc<dyn CliExecutable>,
    pub auth_service: Arc<AuthService>,
    pub wallet_service: Arc<WalletService>,
    pub db: Arc<Database>,
    pub config: AppConfig,
}

impl AppState {
    /// Create a new AppState based on the provided config.
    /// In mock mode, uses MockCliExecutor with default responses and in-memory DB.
    /// In real mode, uses RealCliExecutor and file-based DB.
    pub fn new(config: AppConfig) -> Result<Self, AppError> {
        let db = if config.mock_mode {
            Database::new_in_memory()?
        } else {
            Database::new(&config.db_path)?
        };
        db.run_migrations()?;
        let db = Arc::new(db);

        let cli: Arc<dyn CliExecutable> = if config.mock_mode {
            Arc::new(MockCliExecutor::with_defaults())
        } else {
            // If using npx fallback, pass "awal" as args prefix; otherwise invoke directly
            let (binary, args_prefix) = if config.awal_binary_path == "npx" {
                ("npx".to_string(), vec!["awal".to_string()])
            } else {
                (config.awal_binary_path.clone(), vec![])
            };
            Arc::new(
                RealCliExecutor::new(
                    &binary,
                    args_prefix,
                    &config.network,
                )
                .map_err(|e| AppError::CliNotFound(e.to_string()))?,
            )
        };

        let cache_ttl = Duration::from_secs(config.token_cache_ttl_seconds);
        let auth_service = Arc::new(AuthService::new(cli.clone(), db.clone(), cache_ttl));
        let wallet_service = Arc::new(WalletService::new(cli.clone(), db.clone(), cache_ttl));

        Ok(Self {
            cli,
            auth_service,
            wallet_service,
            db,
            config,
        })
    }

    /// Create an AppState with mock mode enabled (convenience for tests).
    pub fn new_mock() -> Result<Self, AppError> {
        Self::new(AppConfig::default_test())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::commands::AwalCommand;

    #[test]
    fn test_app_state_with_mock_mode_uses_mock_executor() {
        let config = AppConfig::default_test();
        assert!(config.mock_mode);
        let state = AppState::new(config).unwrap();
        assert!(state.config.mock_mode);
        // Verify the DB was created (tables exist after migrations)
        assert!(state.db.table_exists("agents").unwrap());
    }

    #[tokio::test]
    async fn test_mock_mode_balance_returns_fake_data() {
        let state = AppState::new_mock().unwrap();
        let result = state.cli.run(AwalCommand::GetBalance { chain: None }).await.unwrap();
        assert!(result.success);
        assert_eq!(result.data["balances"]["USDC"]["formatted"], "1247.83");
        assert!(result.data["address"].is_string());
    }

    #[tokio::test]
    async fn test_mock_mode_auth_status_returns_authenticated() {
        let state = AppState::new_mock().unwrap();
        let result = state.cli.run(AwalCommand::AuthStatus).await.unwrap();
        assert!(result.success);
        assert_eq!(result.data["auth"]["authenticated"], true);
        assert_eq!(result.data["auth"]["email"], "test@example.com");
    }

    #[tokio::test]
    async fn test_mock_mode_get_address() {
        let state = AppState::new_mock().unwrap();
        let result = state.cli.run(AwalCommand::GetAddress).await.unwrap();
        assert!(result.success);
        assert!(result.data.is_string());
        assert_eq!(result.data.as_str().unwrap(), "0xMockWalletAddress123");
    }

    #[tokio::test]
    async fn test_mock_mode_send() {
        let state = AppState::new_mock().unwrap();
        let result = state
            .cli
            .run(AwalCommand::Send {
                to: "0xRecipient".into(),
                amount: rust_decimal::Decimal::new(500, 2),
                chain: None,
            })
            .await
            .unwrap();
        assert!(result.success);
        assert_eq!(result.data["tx_hash"], "0xmock_tx_hash_abc123");
    }

    #[test]
    fn test_mock_mode_full_startup() {
        // Full end-to-end: create AppState with mock mode, verify everything works
        let config = AppConfig::default_test();
        let state = AppState::new(config).unwrap();

        // Config is mock
        assert!(state.config.mock_mode);

        // DB has all tables
        let expected_tables = [
            "app_config",
            "agents",
            "spending_policies",
            "global_policy",
            "transactions",
            "approval_requests",
            "invitation_codes",
            "token_delivery",
            "notification_preferences",
            "spending_ledger",
        ];
        for table in &expected_tables {
            assert!(
                state.db.table_exists(table).unwrap(),
                "Table '{}' should exist in mock mode",
                table
            );
        }
    }

    #[tokio::test]
    async fn test_mock_mode_auth_service_check_status() {
        let state = AppState::new_mock().unwrap();
        let status = state.auth_service.check_status().await.unwrap();
        assert!(status.authenticated);
        assert_eq!(status.email, Some("test@example.com".to_string()));
    }

    #[test]
    fn test_new_mock_convenience() {
        let state = AppState::new_mock().unwrap();
        assert!(state.config.mock_mode);
        assert!(state.db.table_exists("agents").unwrap());
    }
}
