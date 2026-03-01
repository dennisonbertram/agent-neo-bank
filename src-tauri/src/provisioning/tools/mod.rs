use std::path::Path;

use crate::provisioning::error::ProvisioningError;
use crate::provisioning::types::*;

pub mod claude_code;
pub mod claude_desktop;
pub mod cursor;
pub mod windsurf;
pub mod codex;
pub mod continue_dev;
pub mod cline;
pub mod aider;
pub mod copilot;

/// Each supported tool implements this trait. All methods are synchronous
/// (filesystem I/O only) and called from spawn_blocking.
pub trait ToolProvisioner: Send + Sync {
    /// Which tool this provisioner handles.
    fn tool_id(&self) -> ToolId;

    /// Detect if the tool is installed. Checks filesystem paths, binaries, etc.
    /// Never fails — returns DetectionResult with detected=false on any issue.
    fn detect(&self) -> DetectionResult;

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
