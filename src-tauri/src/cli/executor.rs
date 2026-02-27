use async_trait::async_trait;
use serde_json::Value;

use super::commands::AwalCommand;

#[derive(Debug, Clone)]
pub struct CliOutput {
    pub success: bool,
    pub data: Value,
    pub raw: String,
    pub stderr: String,
}

#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("Command failed: {0}")]
    CommandFailed(String),

    #[error("CLI binary not found")]
    NotFound,

    #[error("Command timed out")]
    Timeout,

    #[error("Parse error: {0}")]
    ParseError(String),
}

#[async_trait]
pub trait CliExecutable: Send + Sync {
    async fn run(&self, cmd: AwalCommand) -> Result<CliOutput, CliError>;
}

pub struct RealCliExecutor {
    pub binary_path: std::path::PathBuf,
    pub network: String,
}

impl RealCliExecutor {
    pub fn new(binary_path: &std::path::Path) -> Result<Self, CliError> {
        Ok(Self {
            binary_path: binary_path.to_path_buf(),
            network: "base-sepolia".to_string(),
        })
    }
}

#[async_trait]
impl CliExecutable for RealCliExecutor {
    async fn run(&self, _cmd: AwalCommand) -> Result<CliOutput, CliError> {
        todo!("Real CLI execution will be implemented in the CLI wrapper task")
    }
}

pub struct MockCliExecutor {
    pub responses: std::collections::HashMap<String, CliOutput>,
}

impl MockCliExecutor {
    pub fn new() -> Self {
        Self {
            responses: std::collections::HashMap::new(),
        }
    }

    pub fn set_response(&mut self, command: &str, output: CliOutput) {
        self.responses.insert(command.to_string(), output);
    }
}

#[async_trait]
impl CliExecutable for MockCliExecutor {
    async fn run(&self, cmd: AwalCommand) -> Result<CliOutput, CliError> {
        let key = cmd.command_key();
        self.responses
            .get(&key)
            .cloned()
            .ok_or_else(|| CliError::CommandFailed(format!("No mock response for: {}", key)))
    }
}
