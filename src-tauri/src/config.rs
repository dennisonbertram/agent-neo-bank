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
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            mock_mode: false,
            network: "base-sepolia".to_string(),
            rest_port: 7402,
            rest_host: "127.0.0.1".to_string(),
            unix_socket_path: "/tmp/agent-neo-bank.sock".to_string(),
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
        }
    }
}

impl AppConfig {
    pub fn default_test() -> Self {
        Self {
            mock_mode: true,
            network: "base-sepolia".to_string(),
            rest_port: 0,
            rest_host: "127.0.0.1".to_string(),
            unix_socket_path: "/tmp/agent-neo-bank-test.sock".to_string(),
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
        }
    }
}
