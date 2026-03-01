pub mod backup;
pub mod config_writer;
pub mod content;
pub mod detection;
pub mod error;
pub mod logging;
pub mod platform;
pub mod rollback;
pub mod state;
pub mod tools;
pub mod types;

use std::path::PathBuf;
use std::sync::RwLock;

use crate::provisioning::backup::BackupManager;
use crate::provisioning::error::ProvisioningError;
use crate::provisioning::logging::ProvisioningLogger;
use crate::provisioning::platform::PlatformPaths;
use crate::provisioning::state::StateManager;
use crate::provisioning::tools::ToolProvisioner;
use crate::provisioning::types::*;

use crate::provisioning::tools::{
    aider::AiderProvisioner, cline::ClineProvisioner, claude_code::ClaudeCodeProvisioner,
    claude_desktop::ClaudeDesktopProvisioner, codex::CodexProvisioner,
    continue_dev::ContinueDevProvisioner, copilot::CopilotProvisioner,
    cursor::CursorProvisioner, windsurf::WindsurfProvisioner,
};

/// Central orchestrator for provisioning operations across all supported tools.
/// Managed as separate Tauri state (not on AppState — doesn't need CLI/DB).
pub struct ProvisioningService {
    tool_provisioners: Vec<Box<dyn ToolProvisioner>>,
    state_manager: StateManager,
    backup_manager: BackupManager,
    logger: ProvisioningLogger,
    state: RwLock<ProvisioningState>,
    tally_dir: PathBuf,
}

impl ProvisioningService {
    /// Create a new ProvisioningService. Creates ~/.tally/ if needed.
    pub fn new() -> Result<Self, ProvisioningError> {
        let paths = PlatformPaths::new().ok_or(ProvisioningError::HomeDirNotFound)?;
        let tally_dir = paths.tally_dir();

        // Ensure ~/.tally/ exists with 0700 permissions
        if !tally_dir.exists() {
            std::fs::create_dir_all(&tally_dir)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&tally_dir, std::fs::Permissions::from_mode(0o700))?;
            }
        }

        let state_manager = StateManager::new(&tally_dir);
        let state = state_manager.load()?;
        let backup_manager = BackupManager::new(&tally_dir);
        let logger = ProvisioningLogger::new(&tally_dir)?;

        let tool_provisioners: Vec<Box<dyn ToolProvisioner>> = vec![
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
            tool_provisioners,
            state_manager,
            backup_manager,
            logger,
            state: RwLock::new(state),
            tally_dir,
        })
    }

    /// Detect all installed tools. Returns detection results for all 9 tools.
    pub fn detect_tools(&self) -> Vec<DetectionResult> {
        let paths = match PlatformPaths::new() {
            Some(p) => p,
            None => return vec![],
        };

        let results = detection::detect_all_tools(&paths);

        // Update state with detection results
        if let Ok(mut state) = self.state.write() {
            for result in &results {
                if result.detected {
                    self.state_manager.mark_detected(&mut state, result.tool, result.version.clone());
                    self.logger.log_detect(
                        result.tool,
                        true,
                        &result.methods.iter().map(|m| format!("{:?}", m)).collect::<Vec<_>>(),
                    );
                } else {
                    self.logger.log_detect(result.tool, false, &[]);
                }
            }
            self.state_manager.update_last_scan(&mut state);
            let _ = self.state_manager.save(&state);
        }

        results
    }

    /// Get a preview of what changes would be made for a specific tool.
    pub fn get_preview(
        &self,
        tool: ToolId,
        config: &McpInjectionConfig,
    ) -> Result<ProvisionPreview, ProvisioningError> {
        let provisioner = self.get_provisioner(tool)?;
        provisioner.preview(config)
    }

    /// Provision a single tool.
    pub fn provision_tool(
        &self,
        tool: ToolId,
        config: &McpInjectionConfig,
    ) -> Result<ProvisionResult, ProvisioningError> {
        // Check exclusion
        {
            let state = self.state.read().map_err(|e| ProvisioningError::Internal(e.to_string()))?;
            if state.excluded_tools.contains(&tool) {
                self.logger.log_skip(tool, "excluded by user");
                return Err(ProvisioningError::ToolExcluded(tool.display_name().to_string()));
            }
        }

        let provisioner = self.get_provisioner(tool)?;

        // Create backup directory
        let backup_dir = self.backup_manager.create_backup_dir()?;
        self.logger.log_backup(tool, &backup_dir);

        // Execute provisioning
        let result = provisioner.provision(config, &backup_dir)?;

        // Write backup manifest
        let manifest = BackupManifest {
            version: 1,
            timestamp: chrono::Utc::now().to_rfc3339(),
            tally_version: config.tally_version.clone(),
            operation: BackupOperation::Provision,
            machine_id: self.state.read()
                .map(|s| s.machine_id.clone())
                .unwrap_or_default(),
            tools_modified: vec![BackupToolEntry {
                tool,
                files: result.files_modified.iter().map(|f| BackupFileEntry {
                    original_path: f.path.clone(),
                    resolved_path: f.path.canonicalize().unwrap_or_else(|_| f.path.clone()),
                    backup_relative_path: f.backup_path.clone().unwrap_or_default(),
                    modification_type: f.change_type.clone(),
                    created_new: f.created_new,
                    sha256_before: f.sha256_before.clone(),
                    sha256_after: f.sha256_after.clone(),
                    keys_added: vec![],
                    sections_added: vec![],
                    sentinel_start: None,
                    sentinel_end: None,
                }).collect(),
            }],
        };
        let _ = self.backup_manager.write_manifest(&backup_dir, &manifest);

        // Update state
        if let Ok(mut state) = self.state.write() {
            let files: Vec<PathBuf> = result.files_modified.iter().map(|f| f.path.clone()).collect();
            self.state_manager.mark_provisioned(&mut state, tool, &config.tally_version, files);
            let _ = self.state_manager.save(&state);
        }

        // Log
        for file in &result.files_modified {
            self.logger.log_provision(tool, &file.path, &format!("{:?}", file.change_type));
        }

        // Enforce backup retention (best-effort)
        let _ = self.backup_manager.enforce_retention(10, 30);

        Ok(result)
    }

    /// Provision all detected, non-excluded tools.
    pub fn provision_all(
        &self,
        config: &McpInjectionConfig,
    ) -> Vec<ProvisionResult> {
        let mut results = Vec::new();

        for provisioner in &self.tool_provisioners {
            let tool = provisioner.tool_id();

            // Skip excluded
            if let Ok(state) = self.state.read() {
                if state.excluded_tools.contains(&tool) {
                    self.logger.log_skip(tool, "excluded by user");
                    continue;
                }
            }

            // Skip undetected
            let detection = provisioner.detect();
            if !detection.detected {
                continue;
            }

            match self.provision_tool(tool, config) {
                Ok(result) => results.push(result),
                Err(e) => results.push(ProvisionResult {
                    tool,
                    success: false,
                    files_modified: vec![],
                    error: Some(e.to_string()),
                    needs_restart: false,
                }),
            }
        }

        results
    }

    /// Unprovision a single tool (surgical removal).
    pub fn unprovision_tool(
        &self,
        tool: ToolId,
    ) -> Result<UnprovisionResult, ProvisioningError> {
        let provisioner = self.get_provisioner(tool)?;
        let result = provisioner.unprovision()?;

        // Update state
        if let Ok(mut state) = self.state.write() {
            self.state_manager.mark_unprovisioned(&mut state, tool);
            let _ = self.state_manager.save(&state);
        }

        self.logger.log_unprovision(tool, "surgical removal complete");
        Ok(result)
    }

    /// Unprovision all provisioned tools.
    pub fn unprovision_all(&self) -> Vec<UnprovisionResult> {
        let mut results = Vec::new();

        for provisioner in &self.tool_provisioners {
            let tool = provisioner.tool_id();

            // Only unprovision tools that are actually provisioned
            let is_provisioned = self.state.read()
                .map(|s| s.tools.get(&tool).map(|t| t.status == ToolStatus::Provisioned).unwrap_or(false))
                .unwrap_or(false);

            if !is_provisioned {
                continue;
            }

            match self.unprovision_tool(tool) {
                Ok(result) => results.push(result),
                Err(e) => results.push(UnprovisionResult {
                    tool,
                    success: false,
                    files_restored: vec![],
                    files_deleted: vec![],
                    error: Some(e.to_string()),
                    strategy_used: RollbackStrategy::SurgicalRemoval,
                }),
            }
        }

        results
    }

    /// Verify provisioning status for all provisioned tools.
    pub fn verify_provisioning(&self) -> Vec<VerificationResult> {
        let version = self.state.read()
            .map(|s| s.tally_version.clone())
            .unwrap_or_default();

        let mut results = Vec::new();

        for provisioner in &self.tool_provisioners {
            let tool = provisioner.tool_id();

            let is_provisioned = self.state.read()
                .map(|s| s.tools.get(&tool).map(|t| t.status == ToolStatus::Provisioned).unwrap_or(false))
                .unwrap_or(false);

            if !is_provisioned {
                continue;
            }

            let result = provisioner.verify(&version);
            self.logger.log_verify(tool, &format!("{:?}", result.status));

            // Update state if config was removed
            if result.status == VerificationStatus::Missing {
                if let Ok(mut state) = self.state.write() {
                    self.state_manager.mark_removed(&mut state, tool);
                    let _ = self.state_manager.save(&state);
                }
            } else if let Ok(mut state) = self.state.write() {
                self.state_manager.update_last_verified(&mut state, tool);
                let _ = self.state_manager.save(&state);
            }

            results.push(result);
        }

        results
    }

    /// Get the current provisioning state.
    pub fn get_state(&self) -> Result<ProvisioningState, ProvisioningError> {
        self.state
            .read()
            .map(|s| s.clone())
            .map_err(|e| ProvisioningError::Internal(e.to_string()))
    }

    /// Exclude a tool from provisioning.
    pub fn exclude_tool(&self, tool: ToolId) -> Result<(), ProvisioningError> {
        let mut state = self.state.write().map_err(|e| ProvisioningError::Internal(e.to_string()))?;
        self.state_manager.exclude_tool(&mut state, tool);
        self.state_manager.save(&state)?;
        self.logger.log_skip(tool, "excluded by user");
        Ok(())
    }

    /// Include a previously excluded tool.
    pub fn include_tool(&self, tool: ToolId) -> Result<(), ProvisioningError> {
        let mut state = self.state.write().map_err(|e| ProvisioningError::Internal(e.to_string()))?;
        self.state_manager.include_tool(&mut state, tool);
        self.state_manager.save(&state)?;
        self.logger.log_notify(&format!("Included {} for provisioning", tool.display_name()));
        Ok(())
    }

    /// Force re-detection of all tools (ignores cache).
    pub fn refresh_detection(&self) -> Vec<DetectionResult> {
        self.detect_tools()
    }

    // ── Private helpers ──

    fn get_provisioner(&self, tool: ToolId) -> Result<&dyn ToolProvisioner, ProvisioningError> {
        self.tool_provisioners
            .iter()
            .find(|p| p.tool_id() == tool)
            .map(|p| p.as_ref())
            .ok_or_else(|| ProvisioningError::Internal(format!("No provisioner for {:?}", tool)))
    }
}
