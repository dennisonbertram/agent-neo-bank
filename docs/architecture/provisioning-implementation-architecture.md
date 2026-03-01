# Provisioning System: Implementation Architecture

> **Status**: Engineering Blueprint (pre-implementation)
> **Date**: 2026-03-01
> **Depends on**: `docs/design/provisioning-system-design.md`, `docs/investigations/coding-tool-config-provisioning.md`
> **Audience**: Developers implementing the provisioning system

---

## 1. Module Layout

### 1.1 File Structure

```
src-tauri/src/
  provisioning/
    mod.rs                    # Re-exports, ProvisioningService struct, orchestrator logic
    error.rs                  # ProvisioningError enum (domain-specific errors)
    types.rs                  # All shared data types (structs, enums)
    detection.rs              # Tool detection engine (filesystem scanning, binary lookup)
    config_writer.rs          # Format-specific read-modify-write engines (JSON, TOML, YAML, Markdown)
    backup.rs                 # Backup creation, manifest writing, SHA-256 verification
    rollback.rs               # Surgical removal, diff-based rollback, full restore
    state.rs                  # ProvisioningState persistence (~/.tally/provisioning-state.json)
    platform.rs               # OS-specific path resolution, binary locations
    content.rs                # Template rendering, token substitution, content versioning
    logging.rs                # Provisioning log (~/.tally/provisioning.log)
    tools/
      mod.rs                  # ToolProvisioner trait definition, tool registry
      claude_code.rs          # Claude Code CLI provisioner
      claude_desktop.rs       # Claude Desktop app provisioner
      cursor.rs               # Cursor provisioner
      windsurf.rs             # Windsurf (Codeium) provisioner
      codex.rs                # OpenAI Codex CLI provisioner
      continue_dev.rs         # Continue.dev provisioner
      cline.rs                # Cline (VS Code extension) provisioner
      aider.rs                # Aider provisioner
      copilot.rs              # GitHub Copilot / VS Code native provisioner
  commands/
    provisioning.rs           # Tauri commands (new file)
```

### 1.2 Integration Points

**`src-tauri/src/core/mod.rs`** -- add:
```rust
pub mod provisioning; // NOT under core/ -- this is a top-level module
```

Actually, provisioning is a top-level concern (like `cli/`, `api/`, `core/`). It does NOT depend on the CLI executor, database, or auth. It operates purely on the filesystem.

**`src-tauri/src/lib.rs`** -- add:
```rust
pub mod provisioning;
```

And in the `.setup()` hook:
```rust
// After AppState creation, create ProvisioningService and manage it separately
let provisioning_service = Arc::new(
    provisioning::ProvisioningService::new()
        .expect("Failed to create ProvisioningService")
);
app.manage(provisioning_service);
```

And in `invoke_handler`:
```rust
commands::provisioning::detect_tools,
commands::provisioning::get_provisioning_preview,
commands::provisioning::provision_tool,
commands::provisioning::provision_all,
commands::provisioning::unprovision_tool,
commands::provisioning::unprovision_all,
commands::provisioning::verify_provisioning,
commands::provisioning::get_provisioning_state,
commands::provisioning::exclude_tool,
commands::provisioning::include_tool,
commands::provisioning::refresh_detection,
```

**`src-tauri/src/commands/mod.rs`** -- add:
```rust
pub mod provisioning;
```

### 1.3 Why Not on AppState?

ProvisioningService does not need `CliExecutable`, `Database`, or `AuthService`. It reads/writes config files on the local filesystem. Putting it on `AppState` would create a false dependency. Instead, it is managed as a separate Tauri state:

```rust
// In Tauri commands:
state: State<'_, Arc<ProvisioningService>>
```

The only data it needs from the app is the wallet token (passed as a parameter to `provision_tool`), and the MCP server binary path (resolved from the app bundle at runtime).

### 1.4 Crate Dependencies (additions to Cargo.toml)

```toml
# Provisioning-specific
toml_edit = "0.22"         # TOML read-modify-write without losing formatting
sha2 = "0.10"              # SHA-256 (already in the project for auth)
fs2 = "0.4"                # Advisory file locking (flock on Unix, LockFileEx on Windows)
dirs = "5"                 # Cross-platform home/config/data directories
tempfile = "3"             # Atomic writes via NamedTempFile
chrono = "0.4"             # Timestamps (already in project)
serde_yaml = "0.9"         # YAML parsing for Continue.dev and Aider configs
```

---

## 2. Core Data Types (Rust)

All types live in `src-tauri/src/provisioning/types.rs`.

### 2.1 Tool Identity

```rust
use serde::{Deserialize, Serialize};
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
    /// Human-readable display name.
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

    /// Slug used in filesystem paths (backup dirs, state keys).
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

    /// All known tools, in display order.
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
```

### 2.2 Detection Types

```rust
/// How a tool was detected.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DetectionMethod {
    ConfigDirectory,     // ~/.cursor/ exists
    ConfigFile,          // specific config file exists
    BinaryInPath,        // `which cursor` succeeded
    ApplicationBundle,   // /Applications/Cursor.app exists
    VsCodeExtension,     // extension dir found
    ProcessRunning,      // pgrep matched
}

/// Result of scanning for a single tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionResult {
    pub tool: ToolId,
    pub detected: bool,
    pub methods: Vec<DetectionMethod>,
    pub version: Option<String>,
    pub config_paths: Vec<ConfigFileInfo>,
}

/// A config file that we would read or write for a tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigFileInfo {
    pub path: PathBuf,
    pub resolved_path: PathBuf,      // after symlink resolution
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
    JsonWithServersKey,  // VS Code uses "servers" not "mcpServers"
    Toml,
    Yaml,
    Markdown,
    MarkdownWithFrontmatter,  // .mdc files
    StandaloneFile,           // we own the entire file
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfigPurpose {
    McpServer,
    SystemInstructions,
    Skill,
    ConventionFile,
}
```

### 2.3 Provisioning Types

```rust
/// What the user sees before confirming.
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
    /// The exact content diff (for "View full diff" in UI).
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

/// Result of provisioning a single tool.
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

/// Result of unprovisioning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnprovisionResult {
    pub tool: ToolId,
    pub success: bool,
    pub files_restored: Vec<PathBuf>,
    pub files_deleted: Vec<PathBuf>,
    pub error: Option<String>,
    pub strategy_used: RollbackStrategy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RollbackStrategy {
    SurgicalRemoval,
    DiffBased,
    FullRestore,
    AlreadyClean,
}
```

### 2.4 Backup Types

```rust
/// Persisted to ~/.tally/backups/<timestamp>/manifest.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupManifest {
    pub version: u32,                        // always 1 for now
    pub timestamp: String,                   // ISO 8601
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
    pub resolved_path: PathBuf,       // symlink target
    pub backup_relative_path: PathBuf, // relative to backup dir
    pub modification_type: FileChangeType,
    pub created_new: bool,            // if true, rollback = delete
    pub sha256_before: Option<String>, // None if created_new
    pub sha256_after: String,
    /// For JSON: which keys were added (for surgical removal).
    pub keys_added: Vec<String>,
    /// For TOML: which sections were added.
    pub sections_added: Vec<String>,
    /// For Markdown: sentinel markers used.
    pub sentinel_start: Option<String>,
    pub sentinel_end: Option<String>,
}
```

### 2.5 State Types

```rust
/// Persisted to ~/.tally/provisioning-state.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvisioningState {
    pub schema_version: u32,
    pub machine_id: String,
    pub tally_version: String,
    pub tools: HashMap<ToolId, ToolProvisioningState>,
    pub excluded_tools: HashSet<ToolId>,
    pub last_scan: Option<String>,
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
    /// Never scanned or not detected.
    Unknown,
    /// Detected but not provisioned.
    Detected,
    /// Provisioned and verified.
    Provisioned,
    /// Provisioned but our content is outdated.
    NeedsUpdate,
    /// Our config was removed externally (tool update or user).
    Removed,
    /// User explicitly excluded this tool.
    Excluded,
}
```

### 2.6 MCP Config Template

```rust
/// The MCP server configuration to inject. Passed to provisioners.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpInjectionConfig {
    /// Path to the MCP server binary (e.g., /Applications/Tally.app/.../tally-mcp-server).
    pub server_command: String,
    /// Args for the MCP server (e.g., ["--stdio"]).
    pub server_args: Vec<String>,
    /// Environment variables (e.g., {"WALLET_TOKEN": "tok_abc123"}).
    pub env: HashMap<String, String>,
    /// Current Tally version for content versioning markers.
    pub tally_version: String,
    /// Timestamp for provisioning metadata.
    pub provisioned_at: String,
}
```

---

## 3. The ToolProvisioner Trait

### 3.1 Trait Definition

Lives in `src-tauri/src/provisioning/tools/mod.rs`:

```rust
use crate::provisioning::error::ProvisioningError;
use crate::provisioning::types::*;

/// Each supported tool implements this trait. All methods are synchronous
/// (filesystem I/O only) and called from spawn_blocking.
pub trait ToolProvisioner: Send + Sync {
    /// Which tool this provisioner handles.
    fn tool_id(&self) -> ToolId;

    /// Detect if the tool is installed. Checks filesystem paths, binaries, etc.
    /// Never fails -- returns DetectionResult with detected=false on any issue.
    fn detect(&self) -> DetectionResult;

    /// Return the list of config files this tool uses for MCP/instructions.
    /// Called after detection succeeds.
    fn config_targets(&self) -> Vec<ConfigFileInfo>;

    /// Preview what changes would be made without modifying anything.
    fn preview(&self, config: &McpInjectionConfig) -> Result<ProvisionPreview, ProvisioningError>;

    /// Execute provisioning: backup, modify, verify.
    /// The backup_dir is the timestamped directory for this operation.
    fn provision(
        &self,
        config: &McpInjectionConfig,
        backup_dir: &Path,
    ) -> Result<ProvisionResult, ProvisioningError>;

    /// Remove our content from all config files.
    fn unprovision(&self) -> Result<UnprovisionResult, ProvisioningError>;

    /// Verify that our content is still present and matches expected version.
    fn verify(&self, expected_version: &str) -> VerificationResult;

    /// Whether the tool typically requires restart after config changes.
    fn needs_restart_after_provision(&self) -> bool;
}

/// Verification result for a single tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub tool: ToolId,
    pub status: VerificationStatus,
    pub installed_version: Option<String>,
    pub provisioned_version: Option<String>,
    pub files_checked: Vec<FileVerification>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationStatus {
    Intact,
    Missing,
    Outdated,
    Tampered,
    NotProvisioned,
    ToolNotInstalled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileVerification {
    pub path: PathBuf,
    pub status: FileVerificationStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileVerificationStatus {
    Present,
    Missing,
    Modified,
    VersionMismatch,
}
```

### 3.2 Tool Registration

The registry is a simple Vec, constructed once in `ProvisioningService::new()`:

```rust
// In provisioning/mod.rs

use crate::provisioning::tools::{
    claude_code::ClaudeCodeProvisioner,
    claude_desktop::ClaudeDesktopProvisioner,
    cursor::CursorProvisioner,
    windsurf::WindsurfProvisioner,
    codex::CodexProvisioner,
    continue_dev::ContinueDevProvisioner,
    cline::ClineProvisioner,
    aider::AiderProvisioner,
    copilot::CopilotProvisioner,
};

pub struct ProvisioningService {
    tools: Vec<Box<dyn ToolProvisioner>>,
    state: RwLock<ProvisioningState>,
    tally_dir: PathBuf,           // ~/.tally/
}

impl ProvisioningService {
    pub fn new() -> Result<Self, ProvisioningError> {
        let home = dirs::home_dir()
            .ok_or(ProvisioningError::NoHomeDirectory)?;
        let tally_dir = home.join(".tally");

        // Ensure ~/.tally/ exists with 0700 permissions
        if !tally_dir.exists() {
            std::fs::create_dir_all(&tally_dir)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&tally_dir, std::fs::Permissions::from_mode(0o700))?;
            }
        }

        let state = Self::load_or_create_state(&tally_dir)?;

        let tools: Vec<Box<dyn ToolProvisioner>> = vec![
            Box::new(ClaudeCodeProvisioner::new()),
            Box::new(ClaudeDesktopProvisioner::new()),
            Box::new(CursorProvisioner::new()),
            Box::new(WindsurfProvisioner::new()),
            Box::new(CodexProvisioner::new()),
            Box::new(ContinueDevProvisioner::new()),
            Box::new(ClineProvisioner::new()),
            Box::new(AiderProvisioner::new()),
            Box::new(CopilotProvisioner::new()),
        ];

        Ok(Self {
            tools,
            state: RwLock::new(state),
            tally_dir,
        })
    }

    /// Iterate over all tools, skipping excluded ones.
    pub fn active_tools(&self) -> impl Iterator<Item = &dyn ToolProvisioner> {
        self.tools.iter().map(|t| t.as_ref())
    }

    /// Get provisioner for a specific tool.
    pub fn get_tool(&self, id: ToolId) -> Option<&dyn ToolProvisioner> {
        self.tools.iter().find(|t| t.tool_id() == id).map(|t| t.as_ref())
    }
}
```

---

## 4. Detection Algorithm

### 4.1 Per-Tool Detection Matrix

Each tool's `detect()` runs checks in reliability order. The first successful check is enough to set `detected = true`, but we continue to gather all evidence.

| Tool | Check 1 (Config Dir) | Check 2 (Config File) | Check 3 (Binary) | Check 4 (App Bundle) | Check 5 (Extension) |
|------|-----|-----|-----|-----|-----|
| **Claude Code** | `~/.claude/` | `~/.claude.json` | `which claude` | N/A | N/A |
| **Claude Desktop** | N/A | `~/Library/Application Support/Claude/claude_desktop_config.json` (macOS) | N/A | `/Applications/Claude.app` | N/A |
| **Cursor** | `~/.cursor/` | `~/.cursor/mcp.json` | `which cursor` | `/Applications/Cursor.app` | N/A |
| **Windsurf** | `~/.codeium/` | `~/.codeium/windsurf/mcp_config.json` | `which windsurf` | `/Applications/Windsurf.app` | N/A |
| **Codex CLI** | `~/.codex/` | `~/.codex/config.toml` | `which codex` | N/A | N/A |
| **Continue.dev** | `~/.continue/` | `~/.continue/config.yaml` | N/A | N/A | `~/.vscode/extensions/continue.continue-*` |
| **Cline** | N/A | `cline_mcp_settings.json` path (OS-specific) | N/A | N/A | `~/.vscode/extensions/saoudrizwan.claude-dev-*` |
| **Aider** | N/A | `~/.aider.conf.yml` | `which aider` | N/A | N/A |
| **Copilot** | N/A | N/A | N/A | N/A | `~/.vscode/extensions/github.copilot-*` |

### 4.2 Binary Detection with Version Manager Awareness

```rust
// In provisioning/detection.rs

use std::path::{Path, PathBuf};
use std::process::Command;

/// Check if a binary exists in PATH or common version manager locations.
/// Returns the resolved path if found.
pub fn find_binary(name: &str) -> Option<PathBuf> {
    // 1. Standard PATH lookup
    if let Ok(output) = Command::new("which").arg(name).output() {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() {
                return Some(PathBuf::from(path));
            }
        }
    }

    // 2. Shell profile lookup (catches nvm, asdf, etc.)
    //    Run in a login shell to get the full PATH.
    if let Ok(output) = Command::new("bash")
        .args(["-lc", &format!("which {}", name)])
        .output()
    {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() {
                return Some(PathBuf::from(path));
            }
        }
    }

    // 3. Known version manager shim directories
    let home = dirs::home_dir()?;
    let shim_dirs = [
        home.join(".nvm/versions/node"),  // nvm (need to glob deeper)
        home.join(".asdf/shims"),
        home.join(".local/share/mise/shims"),
        home.join(".volta/bin"),
        home.join(".local/bin"),          // pip install --user
    ];

    for dir in &shim_dirs {
        let candidate = dir.join(name);
        if candidate.exists() {
            return Some(candidate);
        }
    }

    // 4. For nvm: check all installed Node versions
    let nvm_dir = home.join(".nvm/versions/node");
    if nvm_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&nvm_dir) {
            for entry in entries.flatten() {
                let bin = entry.path().join("bin").join(name);
                if bin.exists() {
                    return Some(bin);
                }
            }
        }
    }

    None
}

/// Check if a directory exists, resolving ~ to home.
pub fn dir_exists(path: &Path) -> bool {
    path.is_dir()
}

/// Check for VS Code extension by prefix.
pub fn find_vscode_extension(prefix: &str) -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    let ext_dir = home.join(".vscode/extensions");
    if !ext_dir.is_dir() {
        return None;
    }
    std::fs::read_dir(&ext_dir).ok()?.flatten().find_map(|entry| {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with(prefix) {
            Some(entry.path())
        } else {
            None
        }
    })
}

/// Check for macOS app bundles in both /Applications and ~/Applications.
pub fn find_app_bundle(app_name: &str) -> Option<PathBuf> {
    let candidates = [
        PathBuf::from(format!("/Applications/{}.app", app_name)),
        dirs::home_dir()
            .map(|h| h.join(format!("Applications/{}.app", app_name)))
            .unwrap_or_default(),
    ];
    candidates.into_iter().find(|p| p.is_dir())
}
```

### 4.3 Detection Caching

Detection results are cached in `~/.tally/detected-tools.json` with structure:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionCache {
    pub timestamp: String,
    pub ttl_seconds: u64,  // default 3600 (1 hour)
    pub results: Vec<DetectionResult>,
}
```

On app launch, the cache is always ignored (fresh scan). On subsequent `detect_tools` calls within the session, use cached results if within TTL. The "Refresh" button forces a new scan.

### 4.4 Platform-Specific Path Resolution

```rust
// In provisioning/platform.rs

use std::path::PathBuf;

/// Resolve the primary config file path for a tool on the current platform.
pub fn mcp_config_path(tool: ToolId) -> Option<PathBuf> {
    let home = dirs::home_dir()?;

    match tool {
        ToolId::ClaudeCode => Some(home.join(".claude.json")),

        ToolId::ClaudeDesktop => {
            #[cfg(target_os = "macos")]
            { Some(home.join("Library/Application Support/Claude/claude_desktop_config.json")) }
            #[cfg(target_os = "windows")]
            { dirs::config_dir().map(|d| d.join("Claude/claude_desktop_config.json")) }
            #[cfg(target_os = "linux")]
            { dirs::config_dir().map(|d| d.join("Claude/claude_desktop_config.json")) }
        }

        ToolId::Cursor => Some(home.join(".cursor/mcp.json")),

        ToolId::Windsurf => Some(home.join(".codeium/windsurf/mcp_config.json")),

        ToolId::Codex => Some(home.join(".codex/config.toml")),

        ToolId::ContinueDev => Some(home.join(".continue/config.yaml")),

        ToolId::Cline => {
            #[cfg(target_os = "macos")]
            {
                Some(home.join("Library/Application Support/Code/User/globalStorage/saoudrizwan.claude-dev/settings/cline_mcp_settings.json"))
            }
            #[cfg(target_os = "windows")]
            {
                dirs::config_dir().map(|d| d.join("Code/User/globalStorage/saoudrizwan.claude-dev/settings/cline_mcp_settings.json"))
            }
            #[cfg(target_os = "linux")]
            {
                dirs::config_dir().map(|d| d.join("Code/User/globalStorage/saoudrizwan.claude-dev/settings/cline_mcp_settings.json"))
            }
        }

        ToolId::Aider => Some(home.join(".aider.conf.yml")),

        ToolId::Copilot => {
            #[cfg(target_os = "macos")]
            { dirs::config_dir().map(|d| d.join("Code/User/mcp.json")) }
            #[cfg(target_os = "windows")]
            { dirs::config_dir().map(|d| d.join("Code/User/mcp.json")) }
            #[cfg(target_os = "linux")]
            { dirs::config_dir().map(|d| d.join("Code/User/mcp.json")) }
        }
    }
}

/// Resolve the instruction/rules file path for a tool.
pub fn instructions_path(tool: ToolId) -> Option<PathBuf> {
    let home = dirs::home_dir()?;

    match tool {
        ToolId::ClaudeCode => Some(home.join(".claude/CLAUDE.md")),
        ToolId::ClaudeDesktop => None, // No file-based instructions
        ToolId::Cursor => Some(home.join(".cursor/rules/tally-wallet.mdc")),
        ToolId::Windsurf => Some(home.join(".codeium/windsurf/memories/global_rules.md")),
        ToolId::Codex => Some(home.join(".codex/AGENTS.md")),
        ToolId::ContinueDev => Some(home.join(".continue/rules/tally-wallet.md")),
        ToolId::Cline => None, // Instructions are project-level only
        ToolId::Aider => Some(home.join("conventions/tally-wallet.md")),
        ToolId::Copilot => None, // Instructions are project-level only
    }
}

/// Resolve the MCP server binary path from the app bundle.
pub fn mcp_server_binary_path() -> Option<PathBuf> {
    // Check env override first (development)
    if let Ok(path) = std::env::var("TALLY_MCP_SERVER_PATH") {
        let p = PathBuf::from(&path);
        if p.exists() {
            return Some(p);
        }
    }

    #[cfg(target_os = "macos")]
    {
        let bundle_path = PathBuf::from("/Applications/Tally.app/Contents/MacOS/tally-mcp-server");
        if bundle_path.exists() {
            return Some(bundle_path);
        }
        // Fallback for dev: check if running from cargo
        let dev_path = PathBuf::from("target/debug/tally-mcp-server");
        if dev_path.exists() {
            return Some(dev_path.canonicalize().ok()?);
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Some(local_app) = dirs::data_local_dir() {
            let p = local_app.join("Programs/Tally/tally-mcp-server.exe");
            if p.exists() { return Some(p); }
        }
    }

    #[cfg(target_os = "linux")]
    {
        let p = PathBuf::from("/usr/local/bin/tally-mcp-server");
        if p.exists() { return Some(p); }
    }

    None
}
```

---

## 5. Config Modification Engine

### 5.1 The Read-Modify-Write Pipeline

All config modifications go through a common pipeline in `config_writer.rs`:

```rust
use std::path::Path;
use std::io::Write;

use crate::provisioning::error::ProvisioningError;

/// Atomic read-modify-write for any config file.
/// 1. Acquire advisory lock
/// 2. Read current contents
/// 3. Apply modification
/// 4. Write to temp file in same directory
/// 5. Rename temp to target (atomic on same-fs)
/// 6. Release lock
pub fn atomic_modify<F>(
    path: &Path,
    modify: F,
) -> Result<(String, String), ProvisioningError>
where
    F: FnOnce(&str) -> Result<String, ProvisioningError>,
{
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)
                .map_err(|e| ProvisioningError::CreateDir {
                    path: parent.to_path_buf(),
                    source: e,
                })?;
        }
    }

    // Acquire advisory file lock (or create lock file if target doesn't exist)
    let lock_path = path.with_extension("tally-lock");
    let lock_file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(&lock_path)
        .map_err(|e| ProvisioningError::FileLock { path: path.to_path_buf(), source: e })?;

    use fs2::FileExt;
    lock_file.try_lock_exclusive()
        .map_err(|_| ProvisioningError::FileLocked(path.to_path_buf()))?;

    // Read current contents (empty string if file doesn't exist)
    let original = if path.exists() {
        std::fs::read_to_string(path)
            .map_err(|e| ProvisioningError::ReadFile { path: path.to_path_buf(), source: e })?
    } else {
        String::new()
    };

    // Apply modification
    let modified = modify(&original)?;

    // Write to temp file in same directory (for atomic rename)
    let parent = path.parent().unwrap_or(Path::new("."));
    let mut temp = tempfile::NamedTempFile::new_in(parent)
        .map_err(|e| ProvisioningError::TempFile { source: e })?;
    temp.write_all(modified.as_bytes())
        .map_err(|e| ProvisioningError::WriteFile { path: path.to_path_buf(), source: e })?;
    temp.flush()
        .map_err(|e| ProvisioningError::WriteFile { path: path.to_path_buf(), source: e })?;

    // Set permissions (0600 for files that may contain tokens)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = if path.exists() {
            std::fs::metadata(path)?.permissions()
        } else {
            std::fs::Permissions::from_mode(0o600)
        };
        temp.as_file().set_permissions(perms)?;
    }

    // Atomic rename
    temp.persist(path)
        .map_err(|e| ProvisioningError::AtomicRename {
            path: path.to_path_buf(),
            source: e.error,
        })?;

    // Cleanup lock file (best-effort)
    let _ = lock_file.unlock();
    let _ = std::fs::remove_file(&lock_path);

    Ok((original, modified))
}
```

### 5.2 JSON Merge (for Claude Code, Claude Desktop, Cursor, Cline, Windsurf)

```rust
/// Merge an MCP server entry into a JSON config file.
/// Handles both "mcpServers" (most tools) and "servers" (VS Code) keys.
pub fn json_merge_mcp_server(
    existing: &str,
    server_name: &str,
    server_config: &serde_json::Value,
    root_key: &str,  // "mcpServers" or "servers"
) -> Result<String, ProvisioningError> {
    let mut doc: serde_json::Value = if existing.is_empty() {
        serde_json::json!({})
    } else {
        serde_json::from_str(existing)
            .map_err(|e| ProvisioningError::ParseError {
                format: "JSON".into(),
                source: e.to_string(),
            })?
    };

    // Check for existing key with different content
    if let Some(existing_server) = doc
        .get(root_key)
        .and_then(|s| s.get(server_name))
    {
        // Compare command + args (ignoring metadata keys)
        let existing_cmd = existing_server.get("command");
        let new_cmd = server_config.get("command");
        if existing_cmd == new_cmd {
            // Already provisioned with same command -- update in place
        } else {
            return Err(ProvisioningError::ConflictingServer {
                server_name: server_name.to_string(),
                existing: existing_server.clone(),
            });
        }
    }

    // Ensure root key exists
    if !doc.get(root_key).is_some() {
        doc[root_key] = serde_json::json!({});
    }

    // Set our server entry
    doc[root_key][server_name] = server_config.clone();

    // Pretty-print with 2-space indent, trailing newline
    let output = serde_json::to_string_pretty(&doc)
        .map_err(|e| ProvisioningError::SerializeError(e.to_string()))?;

    Ok(format!("{}\n", output))
}

/// Remove an MCP server entry from a JSON config file (surgical rollback).
pub fn json_remove_mcp_server(
    existing: &str,
    server_name: &str,
    root_key: &str,
) -> Result<String, ProvisioningError> {
    let mut doc: serde_json::Value = serde_json::from_str(existing)
        .map_err(|e| ProvisioningError::ParseError {
            format: "JSON".into(),
            source: e.to_string(),
        })?;

    if let Some(servers) = doc.get_mut(root_key).and_then(|s| s.as_object_mut()) {
        servers.remove(server_name);
    }

    let output = serde_json::to_string_pretty(&doc)
        .map_err(|e| ProvisioningError::SerializeError(e.to_string()))?;

    Ok(format!("{}\n", output))
}
```

### 5.3 TOML Append/Remove (for Codex CLI)

```rust
use toml_edit::{DocumentMut, Item, Table};

/// Append an MCP server section to a TOML config file.
pub fn toml_append_mcp_server(
    existing: &str,
    server_name: &str,
    command: &str,
    args: &[String],
    env: &HashMap<String, String>,
    tally_version: &str,
) -> Result<String, ProvisioningError> {
    let mut doc: DocumentMut = if existing.is_empty() {
        DocumentMut::new()
    } else {
        existing.parse::<DocumentMut>()
            .map_err(|e| ProvisioningError::ParseError {
                format: "TOML".into(),
                source: e.to_string(),
            })?
    };

    // Check if section already exists
    if doc.get("mcp_servers")
        .and_then(|s| s.get(server_name))
        .is_some()
    {
        // Already exists -- update in place
        // Remove old section, re-add
    }

    // Build the section
    let section_key = format!("mcp_servers.{}", server_name);

    // Use toml_edit to preserve formatting
    if !doc.contains_key("mcp_servers") {
        doc["mcp_servers"] = toml_edit::Item::Table(Table::new());
    }

    let server_table = &mut doc["mcp_servers"][server_name];
    *server_table = toml_edit::Item::Table(Table::new());

    // Add a version comment
    if let Some(table) = server_table.as_table_mut() {
        table.decor_mut().set_prefix(
            format!("\n# tally-wallet v{} (managed by Tally Agentic Wallet)\n", tally_version)
        );
        table.insert("command", toml_edit::value(command));

        let mut arr = toml_edit::Array::new();
        for arg in args {
            arr.push(arg.as_str());
        }
        table.insert("args", toml_edit::value(arr));

        if !env.is_empty() {
            let mut env_table = Table::new();
            for (k, v) in env {
                env_table.insert(k, toml_edit::value(v.as_str()));
            }
            table.insert("env", toml_edit::Item::Table(env_table));
        }
    }

    Ok(doc.to_string())
}

/// Remove our TOML section (surgical rollback).
pub fn toml_remove_mcp_server(
    existing: &str,
    server_name: &str,
) -> Result<String, ProvisioningError> {
    let mut doc: DocumentMut = existing.parse::<DocumentMut>()
        .map_err(|e| ProvisioningError::ParseError {
            format: "TOML".into(),
            source: e.to_string(),
        })?;

    if let Some(servers) = doc.get_mut("mcp_servers").and_then(|s| s.as_table_mut()) {
        servers.remove(server_name);
    }

    Ok(doc.to_string())
}
```

### 5.4 Markdown Sentinel Markers

```rust
const SENTINEL_START: &str = "<!-- TALLY_WALLET_START";
const SENTINEL_END: &str = "<!-- TALLY_WALLET_END -->";

/// Append or replace content between sentinel markers in a markdown file.
pub fn markdown_upsert_section(
    existing: &str,
    content: &str,
    tally_version: &str,
) -> Result<String, ProvisioningError> {
    let start_marker = format!("{} v{} -->", SENTINEL_START, tally_version);
    let end_marker = SENTINEL_END.to_string();

    let section = format!("{}\n{}\n{}", start_marker, content.trim(), end_marker);

    // Check if sentinel markers already exist (any version)
    if let (Some(start_idx), Some(end_idx)) = (
        existing.find(SENTINEL_START),
        existing.find(SENTINEL_END),
    ) {
        if start_idx < end_idx {
            // Replace existing section
            let before = &existing[..start_idx];
            let after = &existing[end_idx + SENTINEL_END.len()..];
            return Ok(format!("{}{}{}", before.trim_end(), format!("\n\n{}\n", section), after.trim_start()));
        }
    }

    // Append to end
    if existing.is_empty() {
        Ok(format!("{}\n", section))
    } else {
        Ok(format!("{}\n\n{}\n", existing.trim_end(), section))
    }
}

/// Remove content between sentinel markers (surgical rollback).
pub fn markdown_remove_section(existing: &str) -> Result<String, ProvisioningError> {
    if let (Some(start_idx), Some(end_idx)) = (
        existing.find(SENTINEL_START),
        existing.find(SENTINEL_END),
    ) {
        if start_idx < end_idx {
            let before = &existing[..start_idx];
            let after = &existing[end_idx + SENTINEL_END.len()..];
            let result = format!("{}{}", before.trim_end(), after.trim_start());
            // If the file is now empty, return empty string
            let trimmed = result.trim();
            if trimmed.is_empty() {
                return Ok(String::new());
            }
            return Ok(format!("{}\n", trimmed));
        }
    }

    // Sentinels not found -- nothing to remove
    Err(ProvisioningError::SentinelsNotFound)
}
```

### 5.5 YAML Merge (for Continue.dev, Aider)

```rust
/// Merge an MCP server entry into a Continue.dev config.yaml.
/// Continue uses a YAML list under `mcpServers:`.
pub fn yaml_merge_mcp_server_list(
    existing: &str,
    server_entry: &serde_yaml::Value,
) -> Result<String, ProvisioningError> {
    let mut doc: serde_yaml::Value = if existing.is_empty() {
        serde_yaml::Value::Mapping(serde_yaml::Mapping::new())
    } else {
        serde_yaml::from_str(existing)
            .map_err(|e| ProvisioningError::ParseError {
                format: "YAML".into(),
                source: e.to_string(),
            })?
    };

    let mapping = doc.as_mapping_mut()
        .ok_or_else(|| ProvisioningError::ParseError {
            format: "YAML".into(),
            source: "Root is not a mapping".into(),
        })?;

    let key = serde_yaml::Value::String("mcpServers".to_string());

    let servers = mapping.entry(key.clone())
        .or_insert(serde_yaml::Value::Sequence(vec![]));

    if let serde_yaml::Value::Sequence(ref mut seq) = servers {
        // Check if tally-wallet already in list
        let existing_idx = seq.iter().position(|s| {
            s.get("name")
                .and_then(|n| n.as_str())
                .map(|n| n == "tally-wallet")
                .unwrap_or(false)
        });

        if let Some(idx) = existing_idx {
            seq[idx] = server_entry.clone();
        } else {
            seq.push(server_entry.clone());
        }
    }

    serde_yaml::to_string(&doc)
        .map_err(|e| ProvisioningError::SerializeError(e.to_string()))
}

/// Merge a read entry into Aider's .aider.conf.yml.
pub fn yaml_merge_read_entry(
    existing: &str,
    file_path: &str,
) -> Result<String, ProvisioningError> {
    let mut doc: serde_yaml::Value = if existing.is_empty() {
        serde_yaml::Value::Mapping(serde_yaml::Mapping::new())
    } else {
        serde_yaml::from_str(existing)
            .map_err(|e| ProvisioningError::ParseError {
                format: "YAML".into(),
                source: e.to_string(),
            })?
    };

    let mapping = doc.as_mapping_mut()
        .ok_or_else(|| ProvisioningError::ParseError {
            format: "YAML".into(),
            source: "Root is not a mapping".into(),
        })?;

    let key = serde_yaml::Value::String("read".to_string());
    let reads = mapping.entry(key)
        .or_insert(serde_yaml::Value::Sequence(vec![]));

    if let serde_yaml::Value::Sequence(ref mut seq) = reads {
        let entry = serde_yaml::Value::String(file_path.to_string());
        if !seq.contains(&entry) {
            seq.push(entry);
        }
    }

    serde_yaml::to_string(&doc)
        .map_err(|e| ProvisioningError::SerializeError(e.to_string()))
}
```

### 5.6 Standalone File Creation (for Cursor rules, Cline rules, Continue MCP files)

```rust
/// Create a standalone file that we fully own.
/// No merge needed -- we write the entire content.
pub fn create_standalone_file(
    path: &Path,
    content: &str,
) -> Result<(), ProvisioningError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let parent = path.parent().unwrap_or(Path::new("."));
    let mut temp = tempfile::NamedTempFile::new_in(parent)?;
    temp.write_all(content.as_bytes())?;
    temp.flush()?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        temp.as_file().set_permissions(std::fs::Permissions::from_mode(0o644))?;
    }

    temp.persist(path).map_err(|e| ProvisioningError::AtomicRename {
        path: path.to_path_buf(),
        source: e.error,
    })?;

    Ok(())
}
```

---

## 6. Backup System

### 6.1 Directory Structure

```
~/.tally/
  machine-id                              # Random UUID, created once
  config.json                             # User preferences
  provisioning-state.json                 # Tool status tracking
  provisioning.log                        # Human-readable audit log
  detected-tools.json                     # Detection cache
  backups/
    2026-03-01T14-30-00Z/
      manifest.json                       # BackupManifest
      claude-code/
        claude.json.bak                   # Backup of ~/.claude.json
        CLAUDE.md.bak                     # Backup of ~/.claude/CLAUDE.md
      cursor/
        mcp.json.bak
    2026-03-01T15-00-00Z/
      manifest.json
      ...
```

### 6.2 Backup Implementation

```rust
// In provisioning/backup.rs

use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

pub struct BackupManager {
    backups_dir: PathBuf,  // ~/.tally/backups/
}

impl BackupManager {
    pub fn new(tally_dir: &Path) -> Self {
        Self {
            backups_dir: tally_dir.join("backups"),
        }
    }

    /// Create a new timestamped backup directory. Returns the path.
    pub fn create_backup_dir(&self) -> Result<PathBuf, ProvisioningError> {
        let timestamp = chrono::Utc::now().format("%Y-%m-%dT%H-%M-%SZ").to_string();
        let dir = self.backups_dir.join(&timestamp);
        std::fs::create_dir_all(&dir)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700))?;
        }
        Ok(dir)
    }

    /// Back up a single file. Returns the backup-relative path and SHA-256.
    pub fn backup_file(
        &self,
        backup_dir: &Path,
        tool_slug: &str,
        original_path: &Path,
    ) -> Result<(PathBuf, String), ProvisioningError> {
        let tool_dir = backup_dir.join(tool_slug);
        std::fs::create_dir_all(&tool_dir)?;

        let file_name = original_path
            .file_name()
            .ok_or(ProvisioningError::InvalidPath(original_path.to_path_buf()))?;
        let backup_name = format!("{}.bak", file_name.to_string_lossy());
        let backup_path = tool_dir.join(&backup_name);
        let relative_path = PathBuf::from(tool_slug).join(&backup_name);

        // Read and hash original
        let contents = std::fs::read(original_path)
            .map_err(|e| ProvisioningError::ReadFile {
                path: original_path.to_path_buf(),
                source: e,
            })?;
        let hash = sha256_hex(&contents);

        // Write backup
        std::fs::write(&backup_path, &contents)?;

        // Verify backup integrity
        let verify_contents = std::fs::read(&backup_path)?;
        let verify_hash = sha256_hex(&verify_contents);
        if hash != verify_hash {
            return Err(ProvisioningError::BackupVerificationFailed {
                path: backup_path,
            });
        }

        // Set 0600 permissions on backup
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&backup_path, std::fs::Permissions::from_mode(0o600))?;
        }

        Ok((relative_path, hash))
    }

    /// Write the manifest file.
    pub fn write_manifest(
        &self,
        backup_dir: &Path,
        manifest: &BackupManifest,
    ) -> Result<(), ProvisioningError> {
        let path = backup_dir.join("manifest.json");
        let json = serde_json::to_string_pretty(manifest)?;
        std::fs::write(&path, json)?;
        Ok(())
    }

    /// Read the most recent manifest for a given tool.
    pub fn latest_manifest_for_tool(
        &self,
        tool: ToolId,
    ) -> Result<Option<(PathBuf, BackupManifest)>, ProvisioningError> {
        let mut dirs: Vec<_> = std::fs::read_dir(&self.backups_dir)?
            .flatten()
            .filter(|e| e.path().is_dir())
            .collect();

        // Sort by name descending (timestamps sort lexicographically)
        dirs.sort_by(|a, b| b.file_name().cmp(&a.file_name()));

        for entry in dirs {
            let manifest_path = entry.path().join("manifest.json");
            if manifest_path.exists() {
                let content = std::fs::read_to_string(&manifest_path)?;
                let manifest: BackupManifest = serde_json::from_str(&content)?;
                if manifest.tools_modified.iter().any(|t| t.tool == tool) {
                    return Ok(Some((entry.path(), manifest)));
                }
            }
        }

        Ok(None)
    }

    /// Enforce retention policy: keep last N, never delete < 30 days old, cap at 50MB.
    pub fn enforce_retention(
        &self,
        max_count: usize,
        min_age_days: u64,
        max_size_bytes: u64,
    ) -> Result<u32, ProvisioningError> {
        // ... retention logic ...
        Ok(0) // number of directories removed
    }
}

pub fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}
```

### 6.3 Rollback Implementation

```rust
// In provisioning/rollback.rs

/// Execute surgical removal for a tool.
/// This is the preferred rollback strategy.
pub fn surgical_remove(
    tool: ToolId,
    manifest_entry: &BackupToolEntry,
) -> Result<Vec<PathBuf>, ProvisioningError> {
    let mut restored = vec![];

    for file_entry in &manifest_entry.files {
        let path = &file_entry.original_path;

        if file_entry.created_new {
            // We created this file from scratch -- delete it
            if path.exists() {
                std::fs::remove_file(path)?;
                restored.push(path.clone());
            }
            continue;
        }

        if !path.exists() {
            // File was already deleted by something else
            continue;
        }

        let current = std::fs::read_to_string(path)?;

        let modified = match file_entry.modification_type {
            FileChangeType::MergeJsonKey => {
                let root_key = if file_entry.keys_added.first()
                    .map(|k| k.starts_with("servers."))
                    .unwrap_or(false)
                {
                    "servers"
                } else {
                    "mcpServers"
                };
                json_remove_mcp_server(&current, "tally-wallet", root_key)?
            }
            FileChangeType::AppendTomlSection => {
                toml_remove_mcp_server(&current, "tally-wallet")?
            }
            FileChangeType::AppendMarkdownSection => {
                markdown_remove_section(&current)?
            }
            FileChangeType::MergeYamlEntry => {
                // For YAML, remove the tally-wallet entry from arrays
                yaml_remove_mcp_server(&current, "tally-wallet")?
            }
            FileChangeType::CreateFile => {
                // Already handled above
                continue;
            }
        };

        // Atomic write
        atomic_modify(path, |_| Ok(modified))?;
        restored.push(path.clone());
    }

    Ok(restored)
}
```

---

## 7. Provisioning State Machine

### 7.1 State Transitions

```
                    ┌─────────┐
            ┌──────►│ Unknown │◄────── initial state (never scanned)
            │       └────┬────┘
            │            │ detect()
            │            ▼
            │       ┌──────────┐
            │  ┌───►│ Detected │◄─────── tool found, not provisioned
            │  │    └────┬─────┘
            │  │         │ provision()
            │  │         ▼
            │  │  ┌─────────────┐
            │  │  │ Provisioned │◄────── config injected, verified
            │  │  └──┬──────┬───┘
            │  │     │      │
            │  │     │      │ verify() finds outdated version
            │  │     │      ▼
            │  │     │ ┌─────────────┐
            │  │     │ │ NeedsUpdate │
            │  │     │ └──────┬──────┘
            │  │     │        │ provision() (update)
            │  │     │        │
            │  │     │        └──────────► Provisioned
            │  │     │
            │  │     │ verify() finds config missing
            │  │     ▼
            │  │ ┌─────────┐
            │  └─┤ Removed │◄───── config deleted externally
            │    └────┬────┘
            │         │ removal_count >= 2
            │         ▼
            │    ┌──────────┐
            └────┤ Excluded │◄──── user chose to exclude, or auto-excluded
                 └──────────┘

Transitions:
  Unknown    → Detected       on detect() finding the tool
  Unknown    → Unknown        on detect() not finding the tool
  Detected   → Provisioned    on successful provision()
  Detected   → Excluded       on user exclusion
  Provisioned → NeedsUpdate   on verify() finding version mismatch
  Provisioned → Removed       on verify() finding config missing
  Provisioned → Detected      on successful unprovision()
  NeedsUpdate → Provisioned   on successful provision() (update)
  Removed    → Provisioned    on re-provision (if allowed)
  Removed    → Excluded       on removal_count >= 2 (auto) or user choice
  Excluded   → Detected       on user "include" action (re-enables tool)
```

### 7.2 State Persistence

```rust
// In provisioning/state.rs

use std::path::{Path, PathBuf};
use tokio::sync::RwLock;

pub struct StateManager {
    path: PathBuf,  // ~/.tally/provisioning-state.json
}

impl StateManager {
    pub fn new(tally_dir: &Path) -> Self {
        Self {
            path: tally_dir.join("provisioning-state.json"),
        }
    }

    pub fn load(&self) -> Result<ProvisioningState, ProvisioningError> {
        if !self.path.exists() {
            return Ok(ProvisioningState::default());
        }
        let content = std::fs::read_to_string(&self.path)?;
        serde_json::from_str(&content)
            .map_err(|e| ProvisioningError::ParseError {
                format: "JSON".into(),
                source: e.to_string(),
            })
    }

    pub fn save(&self, state: &ProvisioningState) -> Result<(), ProvisioningError> {
        let json = serde_json::to_string_pretty(state)?;
        atomic_modify(&self.path, |_| Ok(json))?;
        Ok(())
    }

    /// Record that a tool was provisioned.
    pub fn mark_provisioned(
        &self,
        state: &mut ProvisioningState,
        tool: ToolId,
        version: &str,
        files: Vec<PathBuf>,
    ) {
        let now = chrono::Utc::now().to_rfc3339();
        let tool_state = state.tools.entry(tool).or_insert_with(|| {
            ToolProvisioningState {
                status: ToolStatus::Unknown,
                provisioned_at: None,
                last_verified: None,
                provisioned_version: None,
                tool_version: None,
                removal_count: 0,
                respect_removal: false,
                files_managed: vec![],
            }
        });
        tool_state.status = ToolStatus::Provisioned;
        tool_state.provisioned_at = Some(now.clone());
        tool_state.last_verified = Some(now);
        tool_state.provisioned_version = Some(version.to_string());
        tool_state.files_managed = files;
    }

    /// Record that config was found missing during verification.
    pub fn mark_removed(
        &self,
        state: &mut ProvisioningState,
        tool: ToolId,
    ) {
        if let Some(tool_state) = state.tools.get_mut(&tool) {
            tool_state.status = ToolStatus::Removed;
            tool_state.removal_count += 1;
            if tool_state.removal_count >= 2 {
                tool_state.respect_removal = true;
                tool_state.status = ToolStatus::Excluded;
            }
        }
    }
}

impl Default for ProvisioningState {
    fn default() -> Self {
        Self {
            schema_version: 1,
            machine_id: uuid::Uuid::new_v4().to_string(),
            tally_version: env!("CARGO_PKG_VERSION").to_string(),
            tools: HashMap::new(),
            excluded_tools: HashSet::new(),
            last_scan: None,
        }
    }
}
```

---

## 8. The Injected Content

### 8.1 Content Templates

Content templates are embedded in the binary (not external files). They live in `provisioning/content.rs`.

```rust
// In provisioning/content.rs

use std::collections::HashMap;

/// Build the MCP server JSON value for tools that use the mcpServers format.
pub fn mcp_server_json(config: &McpInjectionConfig) -> serde_json::Value {
    let mut entry = serde_json::json!({
        "command": config.server_command,
        "args": config.server_args,
    });

    if !config.env.is_empty() {
        entry["env"] = serde_json::to_value(&config.env).unwrap();
    }

    // Add version metadata (tools ignore unknown keys)
    entry["_tally_version"] = serde_json::Value::String(config.tally_version.clone());
    entry["_tally_provisioned"] = serde_json::Value::String(config.provisioned_at.clone());

    entry
}

/// Build the MCP server JSON for Cline (includes alwaysAllow and disabled fields).
pub fn mcp_server_json_cline(config: &McpInjectionConfig) -> serde_json::Value {
    let mut entry = mcp_server_json(config);
    entry["alwaysAllow"] = serde_json::json!([
        "register_agent",
        "get_balance",
        "list_tools",
    ]);
    entry["disabled"] = serde_json::Value::Bool(false);
    entry
}

/// The system instructions markdown content.
pub fn instructions_markdown() -> &'static str {
    r#"## Tally Agentic Wallet

A crypto wallet is running on this machine via MCP. You can send payments,
check balances, trade tokens, and pay for X402 services.

**First time?** Call `register_agent` with a descriptive name for yourself
(e.g. "Claude Code - my-project"). Save the returned token in your
persistent memory -- you'll need it for all future calls.

**Already registered?** Your token is in your memory. All spending is
tracked under your agent name and subject to policies set by the user.

New agents start with $0 spending limits. The wallet owner will set
your budget after they see you in the app."#
}

/// Cursor rule file content (.mdc with frontmatter).
pub fn cursor_rule_content(tally_version: &str) -> String {
    format!(
        r#"---
description: "Tally Agentic Wallet integration (v{})"
alwaysApply: true
---

{}
"#,
        tally_version,
        instructions_markdown()
    )
}

/// Claude Code skill file content.
pub fn claude_code_skill_content(tally_version: &str) -> String {
    format!(
        r#"# Tally Wallet Skill (v{})

{}
"#,
        tally_version,
        instructions_markdown()
    )
}

/// Codex TOML config section values.
pub fn codex_toml_config(config: &McpInjectionConfig) -> (String, Vec<String>, HashMap<String, String>) {
    (
        config.server_command.clone(),
        config.server_args.clone(),
        config.env.clone(),
    )
}

/// Continue.dev standalone MCP JSON file content.
pub fn continue_mcp_json(config: &McpInjectionConfig) -> serde_json::Value {
    serde_json::json!({
        "mcpServers": {
            "tally-wallet": mcp_server_json(config)
        }
    })
}

/// Continue.dev YAML server entry (for config.yaml mcpServers list).
pub fn continue_yaml_entry(config: &McpInjectionConfig) -> serde_yaml::Value {
    serde_yaml::to_value(&serde_json::json!({
        "name": "tally-wallet",
        "command": config.server_command,
        "args": config.server_args,
        "env": config.env,
    })).unwrap()
}

/// Windsurf rule file content.
pub fn windsurf_rule_content(tally_version: &str) -> String {
    format!(
        r#"---
trigger: always_on
---

# Tally Agentic Wallet (v{})

{}
"#,
        tally_version,
        instructions_markdown()
    )
}

/// Aider convention file content.
pub fn aider_convention_content(tally_version: &str) -> String {
    format!(
        r#"# Tally Agentic Wallet (v{})

{}
"#,
        tally_version,
        instructions_markdown()
    )
}
```

### 8.2 Token Substitution

Token substitution happens at provision time, not template time. The `McpInjectionConfig` struct carries the resolved values:

```rust
impl McpInjectionConfig {
    /// Build from the current app state.
    pub fn build(
        wallet_token: &str,
        tally_version: &str,
    ) -> Result<Self, ProvisioningError> {
        let server_command = platform::mcp_server_binary_path()
            .ok_or(ProvisioningError::McpServerNotFound)?
            .to_string_lossy()
            .to_string();

        let mut env = HashMap::new();
        env.insert("WALLET_TOKEN".to_string(), wallet_token.to_string());

        Ok(Self {
            server_command,
            server_args: vec!["--stdio".to_string()],
            env,
            tally_version: tally_version.to_string(),
            provisioned_at: chrono::Utc::now().to_rfc3339(),
        })
    }
}
```

---

## 9. Execution Flow

### 9.1 User Clicks "Add to All Tools"

```
1. Frontend calls `detect_tools` Tauri command
2. ProvisioningService.detect_all() runs on spawn_blocking:
   a. For each ToolId::all(), call provisioner.detect()
   b. Filter to detected=true, not excluded
   c. Cache results
   d. Return Vec<DetectionResult>
3. Frontend shows confirmation dialog with detected tools
4. User confirms
5. Frontend calls `provision_all` with wallet_token parameter
6. ProvisioningService.provision_all(wallet_token) runs on spawn_blocking:
   a. Build McpInjectionConfig from wallet_token + app version
   b. Create timestamped backup directory
   c. For each detected, non-excluded tool:
      i.   Call provisioner.preview(config) -- validate
      ii.  For each config file target:
           - If file exists: backup_file(), compute sha256_before
           - atomic_modify() with appropriate writer (JSON/TOML/YAML/Markdown)
           - Compute sha256_after
      iii. Record in BackupToolEntry
   d. Write manifest.json to backup directory
   e. Update ProvisioningState (mark each tool as Provisioned)
   f. Write provisioning.log entries
   g. Enforce backup retention
   h. Return Vec<ProvisionResult>
7. Frontend shows success toast per tool
```

### 9.2 App Launch Verification Pass

```
1. In lib.rs setup hook, after ProvisioningService creation:
   a. Spawn background task (non-blocking, fire-and-forget)
2. ProvisioningService.verify_all() on spawn_blocking:
   a. Load ProvisioningState
   b. For each tool with status == Provisioned or NeedsUpdate:
      i.   Call provisioner.verify(expected_version)
      ii.  If Intact: update last_verified timestamp
      iii. If Missing:
           - Check if respect_removal is true → skip
           - Increment removal_count
           - If removal_count >= 2 → mark Excluded
           - Else → emit Tauri event "provisioning:config-missing" with tool name
      iv.  If Outdated:
           - Mark as NeedsUpdate
           - Emit Tauri event "provisioning:needs-update" with tool + versions
      v.   If Tampered:
           - Log warning, emit event
   c. Save updated ProvisioningState
3. Frontend listens for events, shows non-blocking notification banner
```

### 9.3 User Clicks "Remove" for a Specific Tool

```
1. Frontend calls `unprovision_tool` with tool_id
2. ProvisioningService.unprovision_tool(tool_id):
   a. Load latest manifest for this tool
   b. Call provisioner.unprovision():
      i.  For each managed file:
          - Try surgical removal first (json_remove, toml_remove, markdown_remove)
          - If file was created by us (created_new=true), delete it
          - If surgical removal fails, try diff-based
          - If that fails, return error (don't full-restore without confirmation)
   c. Update ProvisioningState: mark as Detected (not Excluded)
   d. Write log entry
   e. Return UnprovisionResult
```

### 9.4 App Update Changes MCP Config

```
1. App launches with new version
2. Verification pass detects version mismatch (NeedsUpdate)
3. Frontend receives "provisioning:needs-update" event
4. Shows banner: "Tally config for Cursor is outdated. Update?"
5. On confirm, Frontend calls `provision_tool` with tool_id
6. provision_tool():
   a. Creates new backup (of current state, which includes old Tally config)
   b. Surgical replace: remove old content, insert new content
   c. Update version markers
   d. Update ProvisioningState
```

### 9.5 Error Handling Strategy

Each step has a defined failure mode:

| Step | Failure | Recovery |
|------|---------|----------|
| Detection | Binary not found, dir missing | Return detected=false, no error |
| Backup creation | Disk full, permission denied | Abort provisioning for this tool, return error |
| File read | Permission denied, IO error | Skip tool, return error |
| Parse (JSON/TOML/YAML) | Malformed file | Skip tool, return descriptive error |
| Merge | Conflicting server name | Return ConflictingServer error |
| Atomic write | Temp file fails | Original untouched, return error |
| Atomic rename | Cross-device, permission | Very rare, return error |
| File lock | Locked by another process | Retry once after 2s, then skip with error |
| Backup verify | Hash mismatch after copy | Abort provisioning for this tool |

The key invariant: **if any step after backup fails, the original file is untouched** (because we use atomic_modify). The backup is always created before any modification attempt.

---

## 10. Tauri Command Interface

### 10.1 Commands

```rust
// In src-tauri/src/commands/provisioning.rs

use std::sync::Arc;
use tauri::State;
use crate::provisioning::ProvisioningService;
use crate::provisioning::types::*;
use crate::provisioning::error::ProvisioningError;

/// Detect all installed AI coding tools.
#[tauri::command]
pub async fn detect_tools(
    service: State<'_, Arc<ProvisioningService>>,
) -> Result<Vec<DetectionResult>, ProvisioningError> {
    let svc = service.inner().clone();
    tokio::task::spawn_blocking(move || svc.detect_all())
        .await
        .map_err(|e| ProvisioningError::Internal(e.to_string()))?
}

/// Preview changes that would be made to a specific tool.
#[tauri::command]
pub async fn get_provisioning_preview(
    service: State<'_, Arc<ProvisioningService>>,
    tool: ToolId,
    wallet_token: String,
) -> Result<ProvisionPreview, ProvisioningError> {
    let svc = service.inner().clone();
    tokio::task::spawn_blocking(move || svc.preview_tool(tool, &wallet_token))
        .await
        .map_err(|e| ProvisioningError::Internal(e.to_string()))?
}

/// Provision a single tool.
#[tauri::command]
pub async fn provision_tool(
    service: State<'_, Arc<ProvisioningService>>,
    tool: ToolId,
    wallet_token: String,
) -> Result<ProvisionResult, ProvisioningError> {
    let svc = service.inner().clone();
    tokio::task::spawn_blocking(move || svc.provision_tool(tool, &wallet_token))
        .await
        .map_err(|e| ProvisioningError::Internal(e.to_string()))?
}

/// Provision all detected, non-excluded tools.
#[tauri::command]
pub async fn provision_all(
    service: State<'_, Arc<ProvisioningService>>,
    wallet_token: String,
) -> Result<Vec<ProvisionResult>, ProvisioningError> {
    let svc = service.inner().clone();
    tokio::task::spawn_blocking(move || svc.provision_all(&wallet_token))
        .await
        .map_err(|e| ProvisioningError::Internal(e.to_string()))?
}

/// Remove Tally config from a specific tool.
#[tauri::command]
pub async fn unprovision_tool(
    service: State<'_, Arc<ProvisioningService>>,
    tool: ToolId,
) -> Result<UnprovisionResult, ProvisioningError> {
    let svc = service.inner().clone();
    tokio::task::spawn_blocking(move || svc.unprovision_tool(tool))
        .await
        .map_err(|e| ProvisioningError::Internal(e.to_string()))?
}

/// Remove Tally config from all provisioned tools.
#[tauri::command]
pub async fn unprovision_all(
    service: State<'_, Arc<ProvisioningService>>,
) -> Result<Vec<UnprovisionResult>, ProvisioningError> {
    let svc = service.inner().clone();
    tokio::task::spawn_blocking(move || svc.unprovision_all())
        .await
        .map_err(|e| ProvisioningError::Internal(e.to_string()))?
}

/// Verify all provisioned tools' config integrity.
#[tauri::command]
pub async fn verify_provisioning(
    service: State<'_, Arc<ProvisioningService>>,
) -> Result<Vec<VerificationResult>, ProvisioningError> {
    let svc = service.inner().clone();
    tokio::task::spawn_blocking(move || svc.verify_all())
        .await
        .map_err(|e| ProvisioningError::Internal(e.to_string()))?
}

/// Get the full provisioning state.
#[tauri::command]
pub async fn get_provisioning_state(
    service: State<'_, Arc<ProvisioningService>>,
) -> Result<ProvisioningState, ProvisioningError> {
    let svc = service.inner().clone();
    tokio::task::spawn_blocking(move || svc.get_state())
        .await
        .map_err(|e| ProvisioningError::Internal(e.to_string()))?
}

/// Exclude a tool from provisioning.
#[tauri::command]
pub async fn exclude_tool(
    service: State<'_, Arc<ProvisioningService>>,
    tool: ToolId,
) -> Result<(), ProvisioningError> {
    let svc = service.inner().clone();
    tokio::task::spawn_blocking(move || svc.exclude_tool(tool))
        .await
        .map_err(|e| ProvisioningError::Internal(e.to_string()))?
}

/// Re-include a previously excluded tool.
#[tauri::command]
pub async fn include_tool(
    service: State<'_, Arc<ProvisioningService>>,
    tool: ToolId,
) -> Result<(), ProvisioningError> {
    let svc = service.inner().clone();
    tokio::task::spawn_blocking(move || svc.include_tool(tool))
        .await
        .map_err(|e| ProvisioningError::Internal(e.to_string()))?
}

/// Force a fresh tool detection scan (ignore cache).
#[tauri::command]
pub async fn refresh_detection(
    service: State<'_, Arc<ProvisioningService>>,
) -> Result<Vec<DetectionResult>, ProvisioningError> {
    let svc = service.inner().clone();
    tokio::task::spawn_blocking(move || svc.detect_all_fresh())
        .await
        .map_err(|e| ProvisioningError::Internal(e.to_string()))?
}
```

### 10.2 Error Type

```rust
// In provisioning/error.rs

use serde::Serialize;
use thiserror::Error;
use std::path::PathBuf;

#[derive(Error, Debug)]
pub enum ProvisioningError {
    #[error("No home directory found")]
    NoHomeDirectory,

    #[error("MCP server binary not found. Is Tally installed correctly?")]
    McpServerNotFound,

    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    #[error("Tool not detected: {0}")]
    ToolNotDetected(String),

    #[error("Tool is excluded: {0}")]
    ToolExcluded(String),

    #[error("Config file locked: {}", .0.display())]
    FileLocked(PathBuf),

    #[error("Cannot read file {}: {source}", path.display())]
    ReadFile { path: PathBuf, source: std::io::Error },

    #[error("Cannot write file {}: {source}", path.display())]
    WriteFile { path: PathBuf, source: std::io::Error },

    #[error("Cannot create directory {}: {source}", path.display())]
    CreateDir { path: PathBuf, source: std::io::Error },

    #[error("Cannot lock file {}: {source}", path.display())]
    FileLock { path: PathBuf, source: std::io::Error },

    #[error("Atomic rename failed for {}: {source}", path.display())]
    AtomicRename { path: PathBuf, source: std::io::Error },

    #[error("Temp file creation failed: {source}")]
    TempFile { source: std::io::Error },

    #[error("Parse error ({format}): {source}")]
    ParseError { format: String, source: String },

    #[error("Serialization error: {0}")]
    SerializeError(String),

    #[error("Conflicting MCP server '{server_name}' already exists with different config")]
    ConflictingServer {
        server_name: String,
        existing: serde_json::Value,
    },

    #[error("Backup verification failed for {}", path.display())]
    BackupVerificationFailed { path: PathBuf },

    #[error("Sentinel markers not found in file")]
    SentinelsNotFound,

    #[error("Invalid path: {}", .0.display())]
    InvalidPath(PathBuf),

    #[error("Permission denied: {}", .0.display())]
    PermissionDenied(PathBuf),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl Serialize for ProvisioningError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

impl From<std::io::Error> for ProvisioningError {
    fn from(e: std::io::Error) -> Self {
        Self::Internal(e.to_string())
    }
}

impl From<serde_json::Error> for ProvisioningError {
    fn from(e: serde_json::Error) -> Self {
        Self::SerializeError(e.to_string())
    }
}
```

### 10.3 TypeScript Types (Frontend Mirror)

```typescript
// In src/types/provisioning.ts

export type ToolId =
  | "claude_code"
  | "claude_desktop"
  | "cursor"
  | "windsurf"
  | "codex"
  | "continue_dev"
  | "cline"
  | "aider"
  | "copilot";

export type ToolStatus =
  | "unknown"
  | "detected"
  | "provisioned"
  | "needs_update"
  | "removed"
  | "excluded";

export type DetectionMethod =
  | "config_directory"
  | "config_file"
  | "binary_in_path"
  | "application_bundle"
  | "vs_code_extension"
  | "process_running";

export type ConfigFormat =
  | "json"
  | "json_with_servers_key"
  | "toml"
  | "yaml"
  | "markdown"
  | "markdown_with_frontmatter"
  | "standalone_file";

export type FileChangeType =
  | "create_file"
  | "merge_json_key"
  | "append_toml_section"
  | "append_markdown_section"
  | "merge_yaml_entry";

export type RollbackStrategy =
  | "surgical_removal"
  | "diff_based"
  | "full_restore"
  | "already_clean";

export type VerificationStatus =
  | "intact"
  | "missing"
  | "outdated"
  | "tampered"
  | "not_provisioned"
  | "tool_not_installed";

export interface DetectionResult {
  tool: ToolId;
  detected: boolean;
  methods: DetectionMethod[];
  version: string | null;
  config_paths: ConfigFileInfo[];
}

export interface ConfigFileInfo {
  path: string;
  resolved_path: string;
  exists: boolean;
  writable: boolean;
  format: ConfigFormat;
  purpose: "mcp_server" | "system_instructions" | "skill" | "convention_file";
  is_symlink: boolean;
}

export interface ProvisionPreview {
  tool: ToolId;
  changes: FileChange[];
}

export interface FileChange {
  path: string;
  change_type: FileChangeType;
  description: string;
  diff: string | null;
}

export interface ProvisionResult {
  tool: ToolId;
  success: boolean;
  files_modified: ModifiedFile[];
  error: string | null;
  needs_restart: boolean;
}

export interface ModifiedFile {
  path: string;
  change_type: FileChangeType;
  backup_path: string | null;
  sha256_before: string | null;
  sha256_after: string;
  created_new: boolean;
}

export interface UnprovisionResult {
  tool: ToolId;
  success: boolean;
  files_restored: string[];
  files_deleted: string[];
  error: string | null;
  strategy_used: RollbackStrategy;
}

export interface VerificationResult {
  tool: ToolId;
  status: VerificationStatus;
  installed_version: string | null;
  provisioned_version: string | null;
}

export interface ToolProvisioningState {
  status: ToolStatus;
  provisioned_at: string | null;
  last_verified: string | null;
  provisioned_version: string | null;
  tool_version: string | null;
  removal_count: number;
  respect_removal: boolean;
  files_managed: string[];
}

export interface ProvisioningState {
  schema_version: number;
  machine_id: string;
  tally_version: string;
  tools: Record<ToolId, ToolProvisioningState>;
  excluded_tools: ToolId[];
  last_scan: string | null;
}

// Display name helper
export const TOOL_DISPLAY_NAMES: Record<ToolId, string> = {
  claude_code: "Claude Code",
  claude_desktop: "Claude Desktop",
  cursor: "Cursor",
  windsurf: "Windsurf",
  codex: "Codex CLI",
  continue_dev: "Continue.dev",
  cline: "Cline",
  aider: "Aider",
  copilot: "GitHub Copilot",
};
```

---

## 11. Testing Strategy

### 11.1 Unit Tests (Pure Logic, No Filesystem)

**`config_writer.rs` tests** -- the highest priority:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_merge_into_empty() {
        let result = json_merge_mcp_server("", "tally-wallet", &json!({"command": "tally"}), "mcpServers").unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert!(parsed["mcpServers"]["tally-wallet"]["command"].is_string());
    }

    #[test]
    fn test_json_merge_preserves_existing_servers() {
        let existing = r#"{"mcpServers":{"other":{"command":"other"}}}"#;
        let result = json_merge_mcp_server(existing, "tally-wallet", &json!({"command":"tally"}), "mcpServers").unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert!(parsed["mcpServers"]["other"].is_object());
        assert!(parsed["mcpServers"]["tally-wallet"].is_object());
    }

    #[test]
    fn test_json_merge_conflicting_server_errors() {
        let existing = r#"{"mcpServers":{"tally-wallet":{"command":"different"}}}"#;
        let result = json_merge_mcp_server(existing, "tally-wallet", &json!({"command":"tally"}), "mcpServers");
        assert!(matches!(result, Err(ProvisioningError::ConflictingServer { .. })));
    }

    #[test]
    fn test_json_merge_idempotent() {
        let existing = r#"{"mcpServers":{"tally-wallet":{"command":"tally"}}}"#;
        let result = json_merge_mcp_server(existing, "tally-wallet", &json!({"command":"tally"}), "mcpServers").unwrap();
        // Should succeed without error (same command = update in place)
    }

    #[test]
    fn test_json_remove_server() {
        let existing = r#"{"mcpServers":{"tally-wallet":{"command":"tally"},"other":{"command":"x"}}}"#;
        let result = json_remove_mcp_server(existing, "tally-wallet", "mcpServers").unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert!(parsed["mcpServers"]["tally-wallet"].is_null());
        assert!(parsed["mcpServers"]["other"].is_object());
    }

    #[test]
    fn test_json_merge_with_servers_key() {
        let result = json_merge_mcp_server("", "tally-wallet", &json!({"command":"tally"}), "servers").unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert!(parsed["servers"]["tally-wallet"].is_object());
    }

    #[test]
    fn test_json_merge_malformed_errors() {
        let result = json_merge_mcp_server("{invalid", "tally-wallet", &json!({}), "mcpServers");
        assert!(matches!(result, Err(ProvisioningError::ParseError { .. })));
    }

    // TOML tests
    #[test]
    fn test_toml_append_to_empty() { ... }
    #[test]
    fn test_toml_append_preserves_existing() { ... }
    #[test]
    fn test_toml_remove_section() { ... }

    // Markdown tests
    #[test]
    fn test_markdown_insert_into_empty() { ... }
    #[test]
    fn test_markdown_insert_into_existing() { ... }
    #[test]
    fn test_markdown_replace_existing_section() { ... }
    #[test]
    fn test_markdown_remove_section() { ... }
    #[test]
    fn test_markdown_remove_from_file_with_other_content() { ... }

    // YAML tests
    #[test]
    fn test_yaml_merge_into_empty() { ... }
    #[test]
    fn test_yaml_merge_into_existing_list() { ... }
    #[test]
    fn test_yaml_merge_idempotent() { ... }
    #[test]
    fn test_yaml_merge_read_entry() { ... }
}
```

### 11.2 Integration Tests (With Temp Directories)

```rust
// In src-tauri/tests/provisioning_integration.rs

use tempfile::TempDir;

/// Set up a fake home directory with tool configs for testing.
fn setup_fake_home() -> (TempDir, PathBuf) {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path().to_path_buf();

    // Create fake Claude Code config
    let claude_dir = home.join(".claude");
    std::fs::create_dir_all(&claude_dir).unwrap();
    std::fs::write(home.join(".claude.json"), "{}").unwrap();
    std::fs::write(claude_dir.join("CLAUDE.md"), "# My Config\n").unwrap();

    // Create fake Cursor config
    let cursor_dir = home.join(".cursor");
    std::fs::create_dir_all(&cursor_dir).unwrap();
    std::fs::write(cursor_dir.join("mcp.json"), "{}").unwrap();

    (tmp, home)
}

#[test]
fn test_provision_and_unprovision_roundtrip() {
    let (tmp, home) = setup_fake_home();
    // Override HOME for this test
    // ... provision, verify content, unprovision, verify clean
}

#[test]
fn test_idempotent_provisioning() {
    // Provision twice, verify no duplication
}

#[test]
fn test_update_flow() {
    // Provision with v1, update to v2, verify v2 content
}

#[test]
fn test_rollback_after_user_edits() {
    // Provision, manually edit file, unprovision
    // Verify user edits preserved, only our content removed
}

#[test]
fn test_backup_creation_and_integrity() {
    // Provision, verify backup exists, verify sha256 matches
}

#[test]
fn test_malformed_file_skipped() {
    // Create invalid JSON, attempt provision, verify error returned
}

#[test]
fn test_file_creation_when_missing() {
    // Tool detected but config file doesn't exist
    // Verify file created with our content only
}
```

### 11.3 Mock Strategies

**Filesystem mocking**: Don't mock. Use `tempfile::TempDir` for all tests. This is real filesystem I/O, just in a temp directory. Much more reliable than mocking.

**Platform abstraction for testing**: The `platform.rs` functions take no external state -- they read the OS. For testing, each provisioner accepts an optional `home_override: Option<PathBuf>` constructor parameter:

```rust
impl ClaudeCodeProvisioner {
    pub fn new() -> Self {
        Self { home_override: None }
    }

    /// For testing: override the home directory.
    pub fn with_home(home: PathBuf) -> Self {
        Self { home_override: Some(home) }
    }

    fn home_dir(&self) -> PathBuf {
        self.home_override.clone()
            .or_else(|| dirs::home_dir())
            .expect("No home directory")
    }
}
```

### 11.4 Platform-Specific Testing

Use `#[cfg(target_os = "macos")]` guards on tests that check macOS-specific paths. CI should run on macOS, Linux, and Windows runners.

For file permission tests:
```rust
#[cfg(unix)]
#[test]
fn test_backup_files_are_0600() {
    // ...
}
```

---

## 12. Risks and Open Questions

### 12.1 Hardest Parts to Implement

1. **YAML read-modify-write without losing formatting**: `serde_yaml` re-serializes everything, losing comments and formatting. For Aider's `.aider.conf.yml` and Continue's `config.yaml`, this is destructive. **Mitigation**: For Continue, prefer the standalone `.continue/mcpServers/tally-wallet.json` approach (JSON file we fully own). For Aider, the `read:` array append is the only change -- consider doing regex-based append instead of full YAML parse.

2. **Cline's deeply nested globalStorage path**: This path varies by VS Code version (Code, Code Insiders) and platform. We need to handle multiple variants. **Mitigation**: Check both `Code` and `Code - Insiders` paths.

3. **VS Code "servers" vs everyone else's "mcpServers"**: A subtle difference that's easy to get wrong. **Mitigation**: The `ConfigFormat::JsonWithServersKey` variant explicitly tracks this.

4. **Testing on all platforms**: macOS-specific paths (Application Support) don't exist on Linux CI runners. **Mitigation**: All path resolution goes through `home_override` in tests.

### 12.2 What Might Break in Practice

1. **Tool updates nuking config**: Cursor and Claude Desktop are known to occasionally reset config files during updates. The verification-on-launch system handles this, but there's a UX cost (repeated "re-add?" prompts).

2. **Dotfile managers**: Users with chezmoi, GNU Stow, or yadm may have symlinked configs. Our symlink resolution handles this, but the dotfile manager may overwrite our changes on its next sync. **Mitigation**: Document this limitation.

3. **Concurrent modification**: Two Tally instances (or a tool updating while we write) could cause data loss. The advisory lock + atomic write pattern mitigates this, but advisory locks are not universally respected by other applications. **Mitigation**: The atomic rename ensures no partial writes; at worst, the last writer wins.

4. **Config format changes**: Tools may change their config format (e.g., Cursor adding new required keys). Our merge strategy preserves unknown keys, but a format migration that changes the root structure would break parsing. **Mitigation**: Version-pin our expected format and fail gracefully.

### 12.3 What to Prototype First

**Priority 1**: `config_writer.rs` -- the JSON merge/remove, markdown sentinel, and TOML append/remove functions. These are pure functions with no side effects, easy to test, and form the core of the system. Write these first with comprehensive tests.

**Priority 2**: `ClaudeCodeProvisioner` end-to-end -- it's the simplest (JSON merge + markdown append) and exercises the full pipeline (detect, backup, modify, verify, rollback).

**Priority 3**: `CursorProvisioner` -- adds the standalone file creation pattern (`.cursor/rules/tally-wallet.mdc`).

**Priority 4**: `CodexProvisioner` -- exercises the TOML format.

### 12.4 Open Questions

1. **Should ProvisioningService hold a reference to AppState?** Currently no -- it's independent. But it needs the wallet token (from AuthService) and app version. These are passed as parameters. If we need more state in the future, we could pass an `Arc<AppState>` at construction, but keeping them separate is cleaner.

2. **Should we use Tauri events or return values for verification results?** Both. The verification pass on launch uses events (non-blocking). Explicit `verify_provisioning` command returns values (for the UI to display).

3. **Should we support Windows in v1?** Most AI coding tools run on macOS. Windows support adds complexity (ACLs, different paths, CRLF). **Recommendation**: macOS-first, Linux second, Windows third. Use `#[cfg]` guards now so the structure is ready.

4. **The `uuid` crate dependency for machine-id**: We could use `gethostname` + a random salt instead to avoid adding `uuid`. But `uuid` is lightweight and commonly used. Check if it's already in the dependency tree.

5. **Log rotation for `provisioning.log`**: The log file will grow unbounded. **Recommendation**: Truncate to last 1000 lines on app launch. Or use the `tracing` framework's file appender with rotation.
