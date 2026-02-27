use rust_decimal::Decimal;

/// Commands that can be executed via the awal CLI.
/// Only whitelisted commands are allowed.
#[derive(Debug, Clone)]
pub enum AwalCommand {
    AuthLogin { email: String },
    AuthVerify { email: String, otp: String },
    AuthStatus,
    AuthLogout,
    GetBalance,
    GetAddress,
    Send { to: String, amount: Decimal, asset: String },
}

impl AwalCommand {
    /// Convert the command to CLI argument strings.
    pub fn to_args(&self) -> Vec<String> {
        match self {
            AwalCommand::AuthLogin { email } => {
                vec!["auth".into(), "login".into(), email.clone(), "--json".into()]
            }
            AwalCommand::AuthVerify { email, otp } => {
                vec!["auth".into(), "verify".into(), email.clone(), otp.clone(), "--json".into()]
            }
            AwalCommand::AuthStatus => {
                vec!["status".into(), "--json".into()]
            }
            AwalCommand::AuthLogout => {
                vec!["auth".into(), "logout".into(), "--json".into()]
            }
            AwalCommand::GetBalance => {
                vec!["balance".into(), "--json".into()]
            }
            AwalCommand::GetAddress => {
                vec!["address".into(), "--json".into()]
            }
            AwalCommand::Send { to, amount, asset } => {
                let _ = asset; // asset is always USDC for now
                vec!["send".into(), amount.to_string(), to.clone(), "--json".into()]
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
            Self::GetBalance => "get_balance",
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
            email: "user@example.com".into(),
            otp: "123456".into(),
        };
        let args = cmd.to_args();
        assert_eq!(args, vec!["auth", "verify", "user@example.com", "123456", "--json"]);
    }

    #[test]
    fn test_cli_command_to_args_auth_status() {
        let args = AwalCommand::AuthStatus.to_args();
        assert_eq!(args, vec!["status", "--json"]);
    }

    #[test]
    fn test_cli_command_to_args_get_balance() {
        let args = AwalCommand::GetBalance.to_args();
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
            asset: "USDC".into(),
        };
        let args = cmd.to_args();
        assert_eq!(args, vec!["send", "5.00", "0xRecipient", "--json"]);
    }

    #[test]
    fn test_command_key_values() {
        assert_eq!(AwalCommand::AuthLogin { email: "a@b.com".into() }.command_key(), "auth_login");
        assert_eq!(AwalCommand::AuthVerify { email: "a@b.com".into(), otp: "000000".into() }.command_key(), "auth_verify");
        assert_eq!(AwalCommand::AuthStatus.command_key(), "auth_status");
        assert_eq!(AwalCommand::AuthLogout.command_key(), "auth_logout");
        assert_eq!(AwalCommand::GetBalance.command_key(), "get_balance");
        assert_eq!(AwalCommand::GetAddress.command_key(), "get_address");
        assert_eq!(AwalCommand::Send { to: "0x".into(), amount: Decimal::ONE, asset: "USDC".into() }.command_key(), "send");
    }
}
