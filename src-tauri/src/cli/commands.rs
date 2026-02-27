use rust_decimal::Decimal;

/// Commands that can be executed via the awal CLI.
/// Only whitelisted commands are allowed.
#[derive(Debug, Clone)]
pub enum AwalCommand {
    AuthLogin { email: String },
    AuthVerify { flow_id: String, otp: String },
    AuthStatus,
    AuthLogout,
    GetBalance { chain: Option<String> },
    GetAddress,
    Send { to: String, amount: Decimal, chain: Option<String> },
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
}
