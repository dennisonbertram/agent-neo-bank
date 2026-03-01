use std::path::Path;
use crate::provisioning::config_writer;
use crate::provisioning::content;
use crate::provisioning::detection;
use crate::provisioning::error::ProvisioningError;
use crate::provisioning::platform::PlatformPaths;
use crate::provisioning::backup;
use crate::provisioning::tools::ToolProvisioner;
use crate::provisioning::types::*;

pub struct AiderProvisioner { paths: Option<PlatformPaths> }

impl AiderProvisioner {
    pub fn new() -> Self { Self { paths: PlatformPaths::new() } }

    #[cfg(test)]
    pub fn with_paths(paths: PlatformPaths) -> Self {
        Self { paths: Some(paths) }
    }

    fn paths(&self) -> Result<&PlatformPaths, ProvisioningError> { self.paths.as_ref().ok_or(ProvisioningError::HomeDirNotFound) }

    /// The conventions file that gets referenced from .aider.conf.yml
    fn conventions_path(&self) -> Option<std::path::PathBuf> {
        self.paths.as_ref().map(|p| p.home_dir().join(".aider").join("tally-wallet.md"))
    }
}

impl ToolProvisioner for AiderProvisioner {
    fn tool_id(&self) -> ToolId { ToolId::Aider }

    fn detect(&self) -> DetectionResult {
        match &self.paths { Some(paths) => detection::detect_tool(ToolId::Aider, paths), None => DetectionResult { tool: ToolId::Aider, detected: false, methods: vec![], version: None, config_paths: vec![] } }
    }

    fn preview(&self, _config: &McpInjectionConfig) -> Result<ProvisionPreview, ProvisioningError> {
        let paths = self.paths()?;
        let mut changes = Vec::new();

        // Aider has no MCP. We add a read entry to .aider.conf.yml pointing to a conventions file.
        if let Some(conf_path) = paths.skill_path(ToolId::Aider) {
            changes.push(FileChange {
                path: conf_path, change_type: FileChangeType::MergeYamlEntry,
                description: "Add tally-wallet.md to read list".into(),
                diff: Some(content::aider_read_entry()),
            });
        }

        if let Some(conv_path) = self.conventions_path() {
            changes.push(FileChange {
                path: conv_path, change_type: FileChangeType::CreateFile,
                description: "Create wallet conventions file".into(),
                diff: Some(content::aider_conventions_content()),
            });
        }

        Ok(ProvisionPreview { tool: ToolId::Aider, changes })
    }

    fn provision(&self, _config: &McpInjectionConfig, backup_dir: &Path) -> Result<ProvisionResult, ProvisioningError> {
        let paths = self.paths()?;
        let mut files_modified = Vec::new();
        let backup_mgr = backup::BackupManager::new(&paths.tally_dir());

        // Create conventions file
        if let Some(conv_path) = self.conventions_path() {
            let sha256_before = if conv_path.exists() { Some(backup::sha256_hex(&std::fs::read(&conv_path)?)) } else { None };
            let backup_path = if conv_path.exists() { Some(backup_mgr.backup_file(backup_dir, "aider", &conv_path)?.0) } else { None };

            let conv_content = content::aider_conventions_content();
            config_writer::create_standalone_file(&conv_path, &conv_content)?;

            files_modified.push(ModifiedFile {
                path: conv_path.clone(), change_type: FileChangeType::CreateFile, backup_path, sha256_before,
                sha256_after: config_writer::sha256_hex(conv_content.as_bytes()), created_new: true,
            });

            // Add read entry to .aider.conf.yml
            if let Some(conf_path) = paths.skill_path(ToolId::Aider) {
                let conf_sha_before = if conf_path.exists() { Some(backup::sha256_hex(&std::fs::read(&conf_path)?)) } else { None };
                let conf_backup = if conf_path.exists() { Some(backup_mgr.backup_file(backup_dir, "aider", &conf_path)?.0) } else { None };

                let (_, modified) = config_writer::atomic_modify(&conf_path, |existing| {
                    config_writer::yaml_merge_read_entry(existing, &conv_path.to_string_lossy())
                })?;

                let created_new = conf_sha_before.is_none();
                files_modified.push(ModifiedFile {
                    path: conf_path, change_type: FileChangeType::MergeYamlEntry, backup_path: conf_backup, sha256_before: conf_sha_before,
                    sha256_after: config_writer::sha256_hex(modified.as_bytes()), created_new,
                });
            }
        }

        Ok(ProvisionResult { tool: ToolId::Aider, success: true, files_modified, error: None, needs_restart: false })
    }

    fn unprovision(&self) -> Result<UnprovisionResult, ProvisioningError> {
        let paths = self.paths()?;
        let mut files_restored = Vec::new();
        let mut files_deleted = Vec::new();

        // Remove conventions file
        if let Some(conv_path) = self.conventions_path() {
            if conv_path.exists() { config_writer::delete_standalone_file(&conv_path)?; files_deleted.push(conv_path.clone()); }

            // Remove read entry from .aider.conf.yml
            if let Some(conf_path) = paths.skill_path(ToolId::Aider) {
                if conf_path.exists() {
                    config_writer::atomic_modify(&conf_path, |existing| {
                        config_writer::yaml_remove_read_entry(existing, &conv_path.to_string_lossy())
                    })?;
                    files_restored.push(conf_path);
                }
            }
        }

        Ok(UnprovisionResult { tool: ToolId::Aider, success: true, files_restored, files_deleted, error: None, strategy_used: RollbackStrategy::SurgicalRemoval })
    }

    fn verify(&self, _expected_version: &str) -> VerificationResult {
        let mut details = Vec::new();
        let mut all_intact = true;

        if let Some(conv_path) = self.conventions_path() {
            let status = if conv_path.exists() { VerificationStatus::Intact } else { all_intact = false; VerificationStatus::Missing };
            details.push(VerificationDetail { path: conv_path, expected_hash: None, actual_hash: None, status });
        }

        VerificationResult { tool: ToolId::Aider, status: if all_intact { VerificationStatus::Intact } else { VerificationStatus::Missing }, details }
    }

    fn needs_restart_after_provision(&self) -> bool { false }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provisioning::platform::PlatformPaths;
    use crate::provisioning::tools::ToolProvisioner;
    use std::collections::HashMap;

    fn test_config() -> McpInjectionConfig {
        McpInjectionConfig {
            server_command: "tally-mcp".to_string(),
            server_args: vec!["--stdio".to_string()],
            env: HashMap::new(),
            tally_version: "1.0.0-test".to_string(),
            provisioned_at: "2026-01-01T00:00:00Z".to_string(),
        }
    }

    fn make_provisioner(home: &std::path::Path) -> AiderProvisioner {
        AiderProvisioner::with_paths(PlatformPaths::with_home(home.to_path_buf()))
    }

    #[test]
    fn tool_id_returns_correct_id() {
        let tmp = tempfile::tempdir().unwrap();
        let p = make_provisioner(tmp.path());
        assert_eq!(p.tool_id(), ToolId::Aider);
    }

    #[test]
    fn needs_restart() {
        let tmp = tempfile::tempdir().unwrap();
        let p = make_provisioner(tmp.path());
        assert!(!p.needs_restart_after_provision());
    }

    #[test]
    fn preview_returns_expected_files() {
        let tmp = tempfile::tempdir().unwrap();
        let p = make_provisioner(tmp.path());
        let config = test_config();
        let preview = p.preview(&config).unwrap();
        assert_eq!(preview.tool, ToolId::Aider);
        assert_eq!(preview.changes.len(), 2);
        // .aider.conf.yml (skill_path)
        assert!(preview.changes[0].path.to_string_lossy().contains(".aider.conf.yml"));
        assert!(matches!(preview.changes[0].change_type, FileChangeType::MergeYamlEntry));
        // conventions file
        assert!(preview.changes[1].path.to_string_lossy().contains("tally-wallet.md"));
        assert!(matches!(preview.changes[1].change_type, FileChangeType::CreateFile));
    }

    #[test]
    fn provision_creates_expected_files() {
        let tmp = tempfile::tempdir().unwrap();
        let backup_dir = tempfile::tempdir().unwrap();
        let p = make_provisioner(tmp.path());
        let config = test_config();
        let result = p.provision(&config, backup_dir.path()).unwrap();
        assert!(result.success);
        assert_eq!(result.tool, ToolId::Aider);

        // Conventions file
        let conv_path = tmp.path().join(".aider").join("tally-wallet.md");
        assert!(conv_path.exists());
        let contents = std::fs::read_to_string(&conv_path).unwrap();
        assert!(contents.contains("Tally Agentic Wallet"));

        // .aider.conf.yml should reference the conventions file
        let conf_path = tmp.path().join(".aider.conf.yml");
        assert!(conf_path.exists());
        let conf_contents = std::fs::read_to_string(&conf_path).unwrap();
        assert!(conf_contents.contains("tally-wallet.md"));
    }

    #[test]
    fn provision_is_idempotent() {
        let tmp = tempfile::tempdir().unwrap();
        let backup_dir = tempfile::tempdir().unwrap();
        let p = make_provisioner(tmp.path());
        let config = test_config();
        p.provision(&config, backup_dir.path()).unwrap();
        let r2 = p.provision(&config, backup_dir.path()).unwrap();
        assert!(r2.success);
    }

    #[test]
    fn verify_after_provision_returns_intact() {
        let tmp = tempfile::tempdir().unwrap();
        let backup_dir = tempfile::tempdir().unwrap();
        let p = make_provisioner(tmp.path());
        let config = test_config();
        p.provision(&config, backup_dir.path()).unwrap();
        let v = p.verify(&config.tally_version);
        assert_eq!(v.status, VerificationStatus::Intact);
    }

    #[test]
    fn unprovision_removes_traces() {
        let tmp = tempfile::tempdir().unwrap();
        let backup_dir = tempfile::tempdir().unwrap();
        let p = make_provisioner(tmp.path());
        let config = test_config();
        p.provision(&config, backup_dir.path()).unwrap();
        let result = p.unprovision().unwrap();
        assert!(result.success);

        // Conventions file should be deleted
        let conv_path = tmp.path().join(".aider").join("tally-wallet.md");
        assert!(!conv_path.exists());
    }

    #[test]
    fn verify_after_unprovision_returns_missing() {
        let tmp = tempfile::tempdir().unwrap();
        let backup_dir = tempfile::tempdir().unwrap();
        let p = make_provisioner(tmp.path());
        let config = test_config();
        p.provision(&config, backup_dir.path()).unwrap();
        p.unprovision().unwrap();
        let v = p.verify(&config.tally_version);
        assert_eq!(v.status, VerificationStatus::Missing);
    }
}
