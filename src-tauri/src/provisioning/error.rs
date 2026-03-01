use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProvisioningError {
    #[error("Tool not detected: {0}")]
    ToolNotDetected(String),

    #[error("Tool excluded by user: {0}")]
    ToolExcluded(String),

    #[error("Config file not found: {0}")]
    ConfigNotFound(PathBuf),

    #[error("Config file not writable: {0}")]
    ConfigNotWritable(PathBuf),

    #[error("Config file malformed ({format}): {path}")]
    ConfigMalformed {
        path: PathBuf,
        format: String,
        detail: String,
    },

    #[error("Config file locked: {0}")]
    ConfigLocked(PathBuf),

    #[error("File lock timeout after {timeout_secs}s: {path}")]
    LockTimeout {
        path: PathBuf,
        timeout_secs: u64,
    },

    #[error("Backup failed for {path}: {reason}")]
    BackupFailed {
        path: PathBuf,
        reason: String,
    },

    #[error("Backup integrity check failed: expected {expected}, got {actual}")]
    BackupIntegrityFailed {
        expected: String,
        actual: String,
    },

    #[error("Rollback failed for {tool}: {reason}")]
    RollbackFailed {
        tool: String,
        reason: String,
    },

    #[error("Atomic write failed for {path}: {reason}")]
    AtomicWriteFailed {
        path: PathBuf,
        reason: String,
    },

    #[error("Conflict: MCP server '{key}' already exists in {path} with different config")]
    McpServerConflict {
        path: PathBuf,
        key: String,
    },

    #[error("Symlink resolution failed for {path}: {reason}")]
    SymlinkResolutionFailed {
        path: PathBuf,
        reason: String,
    },

    #[error("State file error: {0}")]
    StateError(String),

    #[error("Insufficient disk space: need {needed_bytes} bytes, have {available_bytes}")]
    InsufficientDiskSpace {
        needed_bytes: u64,
        available_bytes: u64,
    },

    #[error("Home directory not found")]
    HomeDirNotFound,

    #[error("Platform not supported: {0}")]
    PlatformNotSupported(String),

    #[error("Already provisioned: {0}")]
    AlreadyProvisioned(String),

    #[error("Not provisioned: {0}")]
    NotProvisioned(String),

    #[error("Respect removal active for {tool} (removed {count} times)")]
    RespectRemoval {
        tool: String,
        count: u32,
    },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("TOML parse error: {0}")]
    TomlParse(String),

    #[error("TOML edit error: {0}")]
    TomlEdit(String),

    #[error("YAML error: {0}")]
    Yaml(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl serde::Serialize for ProvisioningError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
