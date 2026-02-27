use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use super::commands::AwalCommand;

// -------------------------------------------------------------------------
// CliOutput
// -------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliOutput {
    pub success: bool,
    pub data: serde_json::Value,
    pub raw: String,
    pub stderr: String,
}

// -------------------------------------------------------------------------
// CliError
// -------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("Command failed (exit {exit_code:?}): {stderr}")]
    CommandFailed {
        stderr: String,
        exit_code: Option<i32>,
    },

    #[error("Command timed out")]
    Timeout,

    #[error("CLI binary not found: {0}")]
    NotFound(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Session expired")]
    SessionExpired,
}

// -------------------------------------------------------------------------
// CliExecutable trait
// -------------------------------------------------------------------------

#[async_trait]
pub trait CliExecutable: Send + Sync {
    async fn run(&self, cmd: AwalCommand) -> Result<CliOutput, CliError>;
}

// -------------------------------------------------------------------------
// RealCliExecutor
// -------------------------------------------------------------------------

/// Executes CLI commands by spawning real processes.
#[derive(Debug)]
pub struct RealCliExecutor {
    pub binary: String,
    pub args_prefix: Vec<String>,
    pub timeout: Duration,
    pub network: String,
}

impl RealCliExecutor {
    pub fn new(binary: &str, args_prefix: Vec<String>, network: &str) -> Result<Self, CliError> {
        // Check that the binary exists via `which`
        let which_result = std::process::Command::new("which")
            .arg(binary)
            .output();

        match which_result {
            Ok(output) if output.status.success() => {}
            _ => {
                return Err(CliError::NotFound(format!(
                    "Binary '{}' not found in PATH",
                    binary
                )));
            }
        }

        Ok(Self {
            binary: binary.to_string(),
            args_prefix,
            timeout: Duration::from_secs(30),
            network: network.to_string(),
        })
    }

    /// Create without checking binary existence (useful for testing paths).
    pub fn new_unchecked(binary: &str, args_prefix: Vec<String>, network: &str) -> Self {
        Self {
            binary: binary.to_string(),
            args_prefix,
            timeout: Duration::from_secs(30),
            network: network.to_string(),
        }
    }
}

#[async_trait]
impl CliExecutable for RealCliExecutor {
    async fn run(&self, cmd: AwalCommand) -> Result<CliOutput, CliError> {
        let args = cmd.to_args();

        let mut command = tokio::process::Command::new(&self.binary);
        command
            .args(&self.args_prefix)
            .args(&args)
            .env("AWAL_NETWORK", &self.network)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        let child = command.spawn().map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                CliError::NotFound(format!("Binary '{}' not found", self.binary))
            } else {
                CliError::CommandFailed {
                    stderr: e.to_string(),
                    exit_code: None,
                }
            }
        })?;

        let result = tokio::time::timeout(self.timeout, child.wait_with_output()).await;

        match result {
            Err(_) => Err(CliError::Timeout),
            Ok(Err(e)) => Err(CliError::CommandFailed {
                stderr: e.to_string(),
                exit_code: None,
            }),
            Ok(Ok(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                let exit_code = output.status.code();
                let success = output.status.success();

                if !success {
                    return Err(CliError::CommandFailed {
                        stderr: if stderr.is_empty() { stdout.clone() } else { stderr },
                        exit_code,
                    });
                }

                let data: serde_json::Value = serde_json::from_str(&stdout)
                    .unwrap_or_else(|_| serde_json::json!({ "raw": stdout.trim() }));

                Ok(CliOutput {
                    success: true,
                    data,
                    raw: stdout,
                    stderr,
                })
            }
        }
    }
}

// -------------------------------------------------------------------------
// MockCliExecutor
// -------------------------------------------------------------------------

/// Returns canned responses for testing. Activated via ANB_MOCK=true.
pub struct MockCliExecutor {
    responses: std::sync::RwLock<HashMap<String, CliOutput>>,
}

impl MockCliExecutor {
    pub fn new() -> Self {
        Self {
            responses: std::sync::RwLock::new(HashMap::new()),
        }
    }

    pub fn set_response(&self, command: &str, output: CliOutput) {
        self.responses
            .write()
            .expect("RwLock poisoned")
            .insert(command.to_string(), output);
    }
}

impl Default for MockCliExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CliExecutable for MockCliExecutor {
    async fn run(&self, cmd: AwalCommand) -> Result<CliOutput, CliError> {
        let key = cmd.command_key();

        let responses = self.responses.read().expect("RwLock poisoned");
        match responses.get(key) {
            Some(output) => Ok(output.clone()),
            None => Ok(CliOutput {
                success: true,
                data: serde_json::json!({}),
                raw: "{}".to_string(),
                stderr: String::new(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;

    #[tokio::test]
    async fn test_mock_executor_returns_canned_balance() {
        let mock = MockCliExecutor::new();
        mock.set_response(
            "get_balance",
            CliOutput {
                success: true,
                data: serde_json::json!({"balance": "1247.83", "asset": "USDC"}),
                raw: r#"{"balance": "1247.83", "asset": "USDC"}"#.to_string(),
                stderr: String::new(),
            },
        );

        let result = mock.run(AwalCommand::GetBalance).await.unwrap();
        assert!(result.success);
        assert_eq!(result.data["balance"], "1247.83");
        assert_eq!(result.data["asset"], "USDC");
    }

    #[tokio::test]
    async fn test_mock_executor_returns_canned_send() {
        let mock = MockCliExecutor::new();
        mock.set_response(
            "send",
            CliOutput {
                success: true,
                data: serde_json::json!({"tx_hash": "0xabc123"}),
                raw: r#"{"tx_hash": "0xabc123"}"#.to_string(),
                stderr: String::new(),
            },
        );

        let cmd = AwalCommand::Send {
            to: "0xRecipient".into(),
            amount: Decimal::new(500, 2),
            asset: "USDC".into(),
        };
        let result = mock.run(cmd).await.unwrap();
        assert!(result.success);
        assert_eq!(result.data["tx_hash"], "0xabc123");
    }

    #[tokio::test]
    async fn test_mock_executor_returns_default_for_unknown_command() {
        let mock = MockCliExecutor::new();
        // No responses set -- should return default
        let result = mock.run(AwalCommand::GetAddress).await.unwrap();
        assert!(result.success);
        assert_eq!(result.data, serde_json::json!({}));
        assert_eq!(result.raw, "{}");
    }

    #[tokio::test]
    async fn test_real_executor_binary_not_found() {
        let result = RealCliExecutor::new(
            "nonexistent_binary_that_does_not_exist_12345",
            vec![],
            "base-sepolia",
        );
        assert!(result.is_err());
        match result.unwrap_err() {
            CliError::NotFound(msg) => {
                assert!(msg.contains("nonexistent_binary_that_does_not_exist_12345"));
            }
            other => panic!("Expected CliError::NotFound, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_real_executor_spawn_nonexistent_binary() {
        // Use new_unchecked to skip the which check, then run should fail
        let executor = RealCliExecutor::new_unchecked(
            "/nonexistent/path/to/binary",
            vec![],
            "base-sepolia",
        );
        let result = executor.run(AwalCommand::AuthStatus).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            CliError::NotFound(_) => {}
            CliError::CommandFailed { .. } => {}
            other => panic!("Expected NotFound or CommandFailed, got: {:?}", other),
        }
    }
}
