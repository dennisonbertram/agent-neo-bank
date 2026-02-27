#[derive(Debug, Clone)]
pub enum AwalCommand {
    AuthLogin { email: String },
    AuthVerify { email: String, otp: String },
    AuthStatus,
    GetBalance,
    GetAddress,
    Send { to: String, amount: String, asset: String },
}

impl AwalCommand {
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
            AwalCommand::GetBalance => {
                vec!["balance".into(), "--json".into()]
            }
            AwalCommand::GetAddress => {
                vec!["address".into(), "--json".into()]
            }
            AwalCommand::Send { to, amount, asset } => {
                let _ = asset;
                vec!["send".into(), amount.clone(), to.clone(), "--json".into()]
            }
        }
    }

    pub fn command_key(&self) -> String {
        match self {
            AwalCommand::AuthLogin { .. } => "auth_login".to_string(),
            AwalCommand::AuthVerify { .. } => "auth_verify".to_string(),
            AwalCommand::AuthStatus => "auth_status".to_string(),
            AwalCommand::GetBalance => "get_balance".to_string(),
            AwalCommand::GetAddress => "get_address".to_string(),
            AwalCommand::Send { .. } => "send".to_string(),
        }
    }
}
