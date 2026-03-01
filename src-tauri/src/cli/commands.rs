use rust_decimal::Decimal;

/// Commands that can be executed via the awal CLI.
/// Only whitelisted commands are allowed.
#[derive(Debug, Clone)]
pub enum AwalCommand {
    AuthLogin { email: String },
    AuthVerify { flow_id: String, otp: String },
    AuthStatus,
    /// Note: `auth logout` may not be a real awal CLI subcommand — only `login` and `verify` are documented.
    AuthLogout,
    GetBalance { chain: Option<String> },
    GetAddress,
    Send { to: String, amount: Decimal, chain: Option<String> },
    Trade { from: String, to: String, amount: String, slippage: Option<u32> },
    X402Pay { url: String, max_amount: Option<String>, method: Option<String>, data: Option<String>, headers: Option<String> },
    X402BazaarList,
    X402BazaarSearch { query: String },
    X402Details { url: String },
}

impl AwalCommand {
    /// Convert the command to CLI argument strings.
    pub fn to_args(&self) -> Vec<String> {
        match self {
            AwalCommand::AuthLogin { email } => {
                vec!["auth".into(), "login".into(), email.clone(), "--json".into()]
            }
            AwalCommand::AuthVerify { flow_id, otp } => {
                vec!["auth".into(), "verify".into(), flow_id.clone(), otp.clone(), "--json".into()]
            }
            AwalCommand::AuthStatus => {
                vec!["status".into(), "--json".into()]
            }
            AwalCommand::AuthLogout => {
                vec!["auth".into(), "logout".into(), "--json".into()]
            }
            AwalCommand::GetBalance { chain } => {
                let mut args = vec!["balance".into()];
                if let Some(c) = chain {
                    args.push("--chain".into());
                    args.push(c.clone());
                }
                args.push("--json".into());
                args
            }
            AwalCommand::GetAddress => {
                vec!["address".into(), "--json".into()]
            }
            AwalCommand::Send { to, amount, chain } => {
                let mut args = vec!["send".into(), amount.to_string(), to.clone()];
                if let Some(c) = chain {
                    args.push("--chain".into());
                    args.push(c.clone());
                }
                args.push("--json".into());
                args
            }
            AwalCommand::Trade { from, to, amount, slippage } => {
                let mut args = vec!["trade".into(), amount.clone(), from.clone(), to.clone()];
                if let Some(s) = slippage {
                    args.push("--slippage".into());
                    args.push(s.to_string());
                }
                args.push("--json".into());
                args
            }
            AwalCommand::X402Pay { url, max_amount, method, data, headers } => {
                let mut args = vec!["x402".into(), "pay".into(), url.clone()];
                if let Some(m) = max_amount {
                    args.push("--max-amount".into());
                    args.push(m.clone());
                }
                if let Some(mt) = method {
                    args.push("--method".into());
                    args.push(mt.clone());
                }
                if let Some(d) = data {
                    args.push("--data".into());
                    args.push(d.clone());
                }
                if let Some(h) = headers {
                    args.push("--headers".into());
                    args.push(h.clone());
                }
                args.push("--json".into());
                args
            }
            AwalCommand::X402BazaarList => {
                vec!["x402".into(), "bazaar".into(), "list".into(), "--json".into()]
            }
            AwalCommand::X402BazaarSearch { query } => {
                vec!["x402".into(), "bazaar".into(), "search".into(), query.clone(), "--json".into()]
            }
            AwalCommand::X402Details { url } => {
                vec!["x402".into(), "details".into(), url.clone(), "--json".into()]
            }
        }
    }

    /// Key used by MockCliExecutor for response lookup.
    pub fn command_key(&self) -> &str {
        match self {
            Self::AuthLogin { .. } => "auth_login",
            Self::AuthVerify { .. } => "auth_verify",
            Self::AuthStatus => "auth_status",
            Self::AuthLogout => "auth_logout",
            Self::GetBalance { .. } => "get_balance",
            Self::GetAddress => "get_address",
            Self::Send { .. } => "send",
            Self::Trade { .. } => "trade",
            Self::X402Pay { .. } => "x402_pay",
            Self::X402BazaarList => "x402_bazaar_list",
            Self::X402BazaarSearch { .. } => "x402_bazaar_search",
            Self::X402Details { .. } => "x402_details",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_command_to_args_auth_login() {
        let cmd = AwalCommand::AuthLogin { email: "user@example.com".into() };
        let args = cmd.to_args();
        assert_eq!(args, vec!["auth", "login", "user@example.com", "--json"]);
    }

    #[test]
    fn test_cli_command_to_args_auth_verify() {
        let cmd = AwalCommand::AuthVerify {
            flow_id: "flow-user-123".into(),
            otp: "123456".into(),
        };
        let args = cmd.to_args();
        assert_eq!(args, vec!["auth", "verify", "flow-user-123", "123456", "--json"]);
    }

    #[test]
    fn test_cli_command_to_args_auth_status() {
        let args = AwalCommand::AuthStatus.to_args();
        assert_eq!(args, vec!["status", "--json"]);
    }

    #[test]
    fn test_cli_command_to_args_get_balance() {
        let args = AwalCommand::GetBalance { chain: None }.to_args();
        assert_eq!(args, vec!["balance", "--json"]);
    }

    #[test]
    fn test_cli_command_to_args_get_address() {
        let args = AwalCommand::GetAddress.to_args();
        assert_eq!(args, vec!["address", "--json"]);
    }

    #[test]
    fn test_cli_command_to_args_send() {
        let cmd = AwalCommand::Send {
            to: "0xRecipient".into(),
            amount: Decimal::new(500, 2), // 5.00
            chain: None,
        };
        let args = cmd.to_args();
        assert_eq!(args, vec!["send", "5.00", "0xRecipient", "--json"]);
    }

    #[test]
    fn test_command_key_values() {
        assert_eq!(AwalCommand::AuthLogin { email: "a@b.com".into() }.command_key(), "auth_login");
        assert_eq!(AwalCommand::AuthVerify { flow_id: "flow-123".into(), otp: "000000".into() }.command_key(), "auth_verify");
        assert_eq!(AwalCommand::AuthStatus.command_key(), "auth_status");
        assert_eq!(AwalCommand::AuthLogout.command_key(), "auth_logout");
        assert_eq!(AwalCommand::GetBalance { chain: None }.command_key(), "get_balance");
        assert_eq!(AwalCommand::GetAddress.command_key(), "get_address");
        assert_eq!(AwalCommand::Send { to: "0x".into(), amount: Decimal::ONE, chain: None }.command_key(), "send");
    }

    // =====================================================================
    // NEW TDD TESTS: Real CLI format matching
    // =====================================================================

    #[test]
    fn test_cli_command_to_args_get_balance_with_chain() {
        let cmd = AwalCommand::GetBalance { chain: Some("base-sepolia".to_string()) };
        let args = cmd.to_args();
        assert!(args.contains(&"--chain".to_string()));
        assert!(args.contains(&"base-sepolia".to_string()));
    }

    #[test]
    fn test_cli_command_to_args_get_balance_no_chain() {
        let cmd = AwalCommand::GetBalance { chain: None };
        let args = cmd.to_args();
        assert!(!args.contains(&"--chain".to_string()));
        assert_eq!(args, vec!["balance", "--json"]);
    }

    #[test]
    fn test_cli_command_to_args_send_with_chain() {
        let cmd = AwalCommand::Send {
            to: "0xRecipient".into(),
            amount: Decimal::new(500, 2),
            chain: Some("base-sepolia".to_string()),
        };
        let args = cmd.to_args();
        assert!(args.contains(&"--chain".to_string()));
        assert!(args.contains(&"base-sepolia".to_string()));
    }

    #[test]
    fn test_cli_command_to_args_auth_verify_uses_flow_id() {
        let cmd = AwalCommand::AuthVerify {
            flow_id: "flow-abc-123".into(),
            otp: "123456".into(),
        };
        let args = cmd.to_args();
        assert_eq!(args, vec!["auth", "verify", "flow-abc-123", "123456", "--json"]);
    }

    // =====================================================================
    // New command variant tests
    // =====================================================================

    #[test]
    fn test_cli_command_to_args_trade() {
        let cmd = AwalCommand::Trade {
            from: "ETH".into(),
            to: "USDC".into(),
            amount: "1.0".into(),
            slippage: None,
        };
        let args = cmd.to_args();
        assert_eq!(args, vec!["trade", "1.0", "ETH", "USDC", "--json"]);
    }

    #[test]
    fn test_cli_command_to_args_trade_with_slippage() {
        let cmd = AwalCommand::Trade {
            from: "ETH".into(),
            to: "USDC".into(),
            amount: "1.0".into(),
            slippage: Some(50),
        };
        let args = cmd.to_args();
        assert_eq!(args, vec!["trade", "1.0", "ETH", "USDC", "--slippage", "50", "--json"]);
    }

    #[test]
    fn test_cli_command_to_args_x402_pay() {
        let cmd = AwalCommand::X402Pay { url: "https://example.com/api".into(), max_amount: None, method: None, data: None, headers: None };
        let args = cmd.to_args();
        assert_eq!(args, vec!["x402", "pay", "https://example.com/api", "--json"]);
    }

    #[test]
    fn test_cli_command_to_args_x402_pay_with_all_params() {
        let cmd = AwalCommand::X402Pay {
            url: "https://example.com/api".into(),
            max_amount: Some("1.00".into()),
            method: Some("POST".into()),
            data: Some("{\"key\":\"val\"}".into()),
            headers: Some("{\"X-Custom\":\"yes\"}".into()),
        };
        let args = cmd.to_args();
        assert_eq!(args, vec![
            "x402", "pay", "https://example.com/api",
            "--max-amount", "1.00",
            "--method", "POST",
            "--data", "{\"key\":\"val\"}",
            "--headers", "{\"X-Custom\":\"yes\"}",
            "--json"
        ]);
    }

    #[test]
    fn test_cli_command_to_args_x402_bazaar_list() {
        let args = AwalCommand::X402BazaarList.to_args();
        assert_eq!(args, vec!["x402", "bazaar", "list", "--json"]);
    }

    #[test]
    fn test_cli_command_to_args_x402_bazaar_search() {
        let cmd = AwalCommand::X402BazaarSearch { query: "weather".into() };
        let args = cmd.to_args();
        assert_eq!(args, vec!["x402", "bazaar", "search", "weather", "--json"]);
    }

    #[test]
    fn test_cli_command_to_args_x402_details() {
        let cmd = AwalCommand::X402Details { url: "https://weather.x402.org".into() };
        let args = cmd.to_args();
        assert_eq!(args, vec!["x402", "details", "https://weather.x402.org", "--json"]);
    }

    #[test]
    fn test_command_key_new_variants() {
        assert_eq!(AwalCommand::Trade { from: "ETH".into(), to: "USDC".into(), amount: "1".into(), slippage: None }.command_key(), "trade");
        assert_eq!(AwalCommand::X402Pay { url: "url".into(), max_amount: None, method: None, data: None, headers: None }.command_key(), "x402_pay");
        assert_eq!(AwalCommand::X402BazaarList.command_key(), "x402_bazaar_list");
        assert_eq!(AwalCommand::X402BazaarSearch { query: "q".into() }.command_key(), "x402_bazaar_search");
        assert_eq!(AwalCommand::X402Details { url: "url".into() }.command_key(), "x402_details");
    }
}
