use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("CLI error: {0}")]
    CliError(String),

    #[error("CLI not found: {0}")]
    CliNotFound(String),

    #[error("CLI session expired")]
    CliSessionExpired,

    #[error("CLI command timed out")]
    CliTimeout,

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Authentication error: {0}")]
    AuthError(String),

    #[error("Invalid OTP")]
    InvalidOtp,

    #[error("Invalid token")]
    InvalidToken,

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Spending policy violation: {0}")]
    PolicyViolation(String),

    #[error("Kill switch active: {0}")]
    KillSwitchActive(String),

    #[error("Agent suspended: {0}")]
    AgentSuspended(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
