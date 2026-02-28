use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub mock_mode: bool,
    pub network: String,
    pub rest_port: u16,
    pub rest_host: String,
    pub unix_socket_path: String,
    pub mcp_enabled: bool,
    pub token_hash_algorithm: String,
    pub token_cache_ttl_seconds: u64,
    pub rate_limit_requests_per_minute: u32,
    pub invitation_code_required: bool,
    pub new_agent_per_tx_max: String,
    pub new_agent_daily_cap: String,
    pub new_agent_weekly_cap: String,
    pub new_agent_monthly_cap: String,
    pub new_agent_auto_approve_max: String,
    pub new_agent_balance_visible: bool,
    pub global_daily_cap: String,
    pub global_weekly_cap: String,
    pub global_monthly_cap: String,
    pub global_min_reserve_balance: String,
    pub global_kill_switch_active: bool,
    pub balance_ttl_seconds: u64,
    pub approval_expiry_check_interval: u64,
    pub approval_default_expiry_hours: u64,
    pub awal_binary_path: String,
    pub db_path: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            mock_mode: false,
            network: "base-sepolia".to_string(),
            rest_port: 7402,
            rest_host: "127.0.0.1".to_string(),
            unix_socket_path: "/tmp/tally-agentic-wallet.sock".to_string(),
            mcp_enabled: true,
            token_hash_algorithm: "argon2id".to_string(),
            token_cache_ttl_seconds: 300,
            rate_limit_requests_per_minute: 60,
            invitation_code_required: true,
            new_agent_per_tx_max: "0".to_string(),
            new_agent_daily_cap: "0".to_string(),
            new_agent_weekly_cap: "0".to_string(),
            new_agent_monthly_cap: "0".to_string(),
            new_agent_auto_approve_max: "0".to_string(),
            new_agent_balance_visible: true,
            global_daily_cap: "0".to_string(),
            global_weekly_cap: "0".to_string(),
            global_monthly_cap: "0".to_string(),
            global_min_reserve_balance: "0".to_string(),
            global_kill_switch_active: false,
            balance_ttl_seconds: 30,
            approval_expiry_check_interval: 300,
            approval_default_expiry_hours: 24,
            awal_binary_path: "npx".to_string(),
            db_path: "tally-agentic-wallet.db".to_string(),
        }
    }
}

impl AppConfig {
    /// Build config from environment variables, falling back to defaults.
    /// Reads ANB_MOCK env var to enable mock mode.
    pub fn from_env() -> Self {
        let mock_mode = std::env::var("ANB_MOCK")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);

        // Resolve local awal binary from node_modules
        let awal_binary_path = Self::resolve_awal_path();

        Self {
            mock_mode,
            awal_binary_path,
            ..Self::default()
        }
    }

    /// Find the local awal binary installed via npm.
    /// Falls back to "npx" if the local binary isn't found.
    fn resolve_awal_path() -> String {
        // In dev mode, cwd is src-tauri/, so node_modules is one level up
        let candidates = [
            "../node_modules/.bin/awal",
            "node_modules/.bin/awal",
        ];
        for candidate in &candidates {
            let path = std::path::Path::new(candidate);
            if path.exists() {
                return path.canonicalize()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|_| candidate.to_string());
            }
        }
        // Fallback: check if awal is globally available
        "awal".to_string()
    }

    pub fn default_test() -> Self {
        Self {
            mock_mode: true,
            network: "base-sepolia".to_string(),
            rest_port: 0,
            rest_host: "127.0.0.1".to_string(),
            unix_socket_path: "/tmp/tally-agentic-wallet-test.sock".to_string(),
            mcp_enabled: false,
            token_hash_algorithm: "argon2id".to_string(),
            token_cache_ttl_seconds: 300,
            rate_limit_requests_per_minute: 1000,
            invitation_code_required: false,
            new_agent_per_tx_max: "100".to_string(),
            new_agent_daily_cap: "1000".to_string(),
            new_agent_weekly_cap: "5000".to_string(),
            new_agent_monthly_cap: "20000".to_string(),
            new_agent_auto_approve_max: "10".to_string(),
            new_agent_balance_visible: true,
            global_daily_cap: "0".to_string(),
            global_weekly_cap: "0".to_string(),
            global_monthly_cap: "0".to_string(),
            global_min_reserve_balance: "0".to_string(),
            global_kill_switch_active: false,
            balance_ttl_seconds: 0,
            approval_expiry_check_interval: 60,
            approval_default_expiry_hours: 1,
            awal_binary_path: "npx".to_string(),
            db_path: ":memory:".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Env var tests must be serialized since they share process-global state
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn test_mock_mode_from_env_var() {
        let _lock = ENV_LOCK.lock().unwrap();
        unsafe { std::env::set_var("ANB_MOCK", "true"); }
        let config = AppConfig::from_env();
        assert!(config.mock_mode, "ANB_MOCK=true should enable mock_mode");
        unsafe { std::env::remove_var("ANB_MOCK"); }
    }

    #[test]
    fn test_mock_mode_from_env_var_numeric() {
        let _lock = ENV_LOCK.lock().unwrap();
        unsafe { std::env::set_var("ANB_MOCK", "1"); }
        let config = AppConfig::from_env();
        assert!(config.mock_mode, "ANB_MOCK=1 should enable mock_mode");
        unsafe { std::env::remove_var("ANB_MOCK"); }
    }

    #[test]
    fn test_mock_mode_from_env_var_false() {
        let _lock = ENV_LOCK.lock().unwrap();
        unsafe { std::env::set_var("ANB_MOCK", "false"); }
        let config = AppConfig::from_env();
        assert!(!config.mock_mode, "ANB_MOCK=false should not enable mock_mode");
        unsafe { std::env::remove_var("ANB_MOCK"); }
    }

    #[test]
    fn test_mock_mode_unset() {
        let _lock = ENV_LOCK.lock().unwrap();
        unsafe { std::env::remove_var("ANB_MOCK"); }
        let config = AppConfig::from_env();
        assert!(!config.mock_mode, "Unset ANB_MOCK should default to false");
    }

    #[test]
    fn test_default_config_mock_mode_false() {
        let config = AppConfig::default();
        assert!(!config.mock_mode, "Default config should have mock_mode=false");
    }

    #[test]
    fn test_default_test_config_has_mock_mode() {
        let config = AppConfig::default_test();
        assert!(config.mock_mode, "default_test() should have mock_mode=true");
    }

    #[test]
    fn test_default_config_has_db_path() {
        let config = AppConfig::default();
        assert_eq!(config.db_path, "tally-agentic-wallet.db");
    }

    #[test]
    fn test_default_test_config_has_memory_db_path() {
        let config = AppConfig::default_test();
        assert_eq!(config.db_path, ":memory:");
    }
}
