use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/// Every supported AI coding tool.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolId {
    ClaudeCode,
    ClaudeDesktop,
    Cursor,
    Windsurf,
    Codex,
    ContinueDev,
    Cline,
    Aider,
    Copilot,
}

impl ToolId {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::ClaudeCode => "Claude Code",
            Self::ClaudeDesktop => "Claude Desktop",
            Self::Cursor => "Cursor",
            Self::Windsurf => "Windsurf",
            Self::Codex => "Codex CLI",
            Self::ContinueDev => "Continue.dev",
            Self::Cline => "Cline",
            Self::Aider => "Aider",
            Self::Copilot => "GitHub Copilot",
        }
    }

    pub fn slug(&self) -> &'static str {
        match self {
            Self::ClaudeCode => "claude-code",
            Self::ClaudeDesktop => "claude-desktop",
            Self::Cursor => "cursor",
            Self::Windsurf => "windsurf",
            Self::Codex => "codex",
            Self::ContinueDev => "continue-dev",
            Self::Cline => "cline",
            Self::Aider => "aider",
            Self::Copilot => "copilot",
        }
    }

    pub fn all() -> &'static [ToolId] {
        &[
            Self::ClaudeCode,
            Self::ClaudeDesktop,
            Self::Cursor,
            Self::Windsurf,
            Self::Codex,
            Self::ContinueDev,
            Self::Cline,
            Self::Aider,
            Self::Copilot,
        ]
    }
}

// ── Detection Types ──

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DetectionMethod {
    ConfigDirectory,
    ConfigFile,
    BinaryInPath,
    ApplicationBundle,
    VsCodeExtension,
    ProcessRunning,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionResult {
    pub tool: ToolId,
    pub detected: bool,
    pub methods: Vec<DetectionMethod>,
    pub version: Option<String>,
    pub config_paths: Vec<ConfigFileInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigFileInfo {
    pub path: PathBuf,
    pub resolved_path: PathBuf,
    pub exists: bool,
    pub writable: bool,
    pub format: ConfigFormat,
    pub purpose: ConfigPurpose,
    pub is_symlink: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfigFormat {
    Json,
    JsonWithServersKey,
    Toml,
    Yaml,
    Markdown,
    MarkdownWithFrontmatter,
    StandaloneFile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfigPurpose {
    McpServer,
    SystemInstructions,
    Skill,
    ConventionFile,
}

// ── Provisioning Types ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvisionPreview {
    pub tool: ToolId,
    pub changes: Vec<FileChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    pub path: PathBuf,
    pub change_type: FileChangeType,
    pub description: String,
    pub diff: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileChangeType {
    CreateFile,
    MergeJsonKey,
    AppendTomlSection,
    AppendMarkdownSection,
    MergeYamlEntry,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvisionResult {
    pub tool: ToolId,
    pub success: bool,
    pub files_modified: Vec<ModifiedFile>,
    pub error: Option<String>,
    pub needs_restart: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModifiedFile {
    pub path: PathBuf,
    pub change_type: FileChangeType,
    pub backup_path: Option<PathBuf>,
    pub sha256_before: Option<String>,
    pub sha256_after: String,
    pub created_new: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnprovisionResult {
    pub tool: ToolId,
    pub success: bool,
    pub files_restored: Vec<PathBuf>,
    pub files_deleted: Vec<PathBuf>,
    pub error: Option<String>,
    pub strategy_used: RollbackStrategy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RollbackStrategy {
    SurgicalRemoval,
    DiffBased,
    FullRestore,
    AlreadyClean,
}

// ── Backup Types ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupManifest {
    pub version: u32,
    pub timestamp: String,
    pub tally_version: String,
    pub operation: BackupOperation,
    pub machine_id: String,
    pub tools_modified: Vec<BackupToolEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BackupOperation {
    Provision,
    Update,
    Unprovision,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupToolEntry {
    pub tool: ToolId,
    pub files: Vec<BackupFileEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupFileEntry {
    pub original_path: PathBuf,
    pub resolved_path: PathBuf,
    pub backup_relative_path: PathBuf,
    pub modification_type: FileChangeType,
    pub created_new: bool,
    pub sha256_before: Option<String>,
    pub sha256_after: String,
    pub keys_added: Vec<String>,
    pub sections_added: Vec<String>,
    pub sentinel_start: Option<String>,
    pub sentinel_end: Option<String>,
}

// ── State Types ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvisioningState {
    pub schema_version: u32,
    pub machine_id: String,
    pub tally_version: String,
    pub tools: HashMap<ToolId, ToolProvisioningState>,
    pub excluded_tools: HashSet<ToolId>,
    pub last_scan: Option<String>,
}

impl Default for ProvisioningState {
    fn default() -> Self {
        Self {
            schema_version: 1,
            machine_id: String::new(),
            tally_version: String::new(),
            tools: HashMap::new(),
            excluded_tools: HashSet::new(),
            last_scan: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolProvisioningState {
    pub status: ToolStatus,
    pub provisioned_at: Option<String>,
    pub last_verified: Option<String>,
    pub provisioned_version: Option<String>,
    pub tool_version: Option<String>,
    pub removal_count: u32,
    pub respect_removal: bool,
    pub files_managed: Vec<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolStatus {
    Unknown,
    Detected,
    Provisioned,
    NeedsUpdate,
    Removed,
    Excluded,
}

// ── MCP Injection Config ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpInjectionConfig {
    pub server_command: String,
    pub server_args: Vec<String>,
    pub env: HashMap<String, String>,
    pub tally_version: String,
    pub provisioned_at: String,
}

// ── Verification ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub tool: ToolId,
    pub status: VerificationStatus,
    pub details: Vec<VerificationDetail>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationStatus {
    Intact,
    Modified,
    Missing,
    Corrupted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationDetail {
    pub path: PathBuf,
    pub expected_hash: Option<String>,
    pub actual_hash: Option<String>,
    pub status: VerificationStatus,
}
