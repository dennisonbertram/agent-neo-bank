use std::path::Path;

use crate::provisioning::config_writer;
use crate::provisioning::content;
use crate::provisioning::detection;
use crate::provisioning::error::ProvisioningError;
use crate::provisioning::platform::PlatformPaths;
use crate::provisioning::backup;
use crate::provisioning::tools::ToolProvisioner;
use crate::provisioning::types::*;

pub struct CodexProvisioner {
    paths: Option<PlatformPaths>,
}

impl CodexProvisioner {
    pub fn new() -> Self { Self { paths: PlatformPaths::new() } }

    #[cfg(test)]
    pub fn with_paths(paths: PlatformPaths) -> Self {
        Self { paths: Some(paths) }
    }

    fn paths(&self) -> Result<&PlatformPaths, ProvisioningError> {
        self.paths.as_ref().ok_or(ProvisioningError::HomeDirNotFound)
    }
}

impl ToolProvisioner for CodexProvisioner {
    fn tool_id(&self) -> ToolId { ToolId::Codex }

    fn detect(&self) -> DetectionResult {
        match &self.paths {
            Some(paths) => detection::detect_tool(ToolId::Codex, paths),
            None => DetectionResult { tool: ToolId::Codex, detected: false, methods: vec![], version: None, config_paths: vec![] },
        }
    }

    fn preview(&self, config: &McpInjectionConfig) -> Result<ProvisionPreview, ProvisioningError> {
        let paths = self.paths()?;
        let mut changes = Vec::new();

        if let Some(mcp_path) = paths.mcp_config_path(ToolId::Codex) {
            changes.push(FileChange {
                path: mcp_path, change_type: FileChangeType::AppendTomlSection,
                description: format!("Add [mcp_servers.{}] section", content::MCP_SERVER_KEY),
                diff: Some(content::mcp_toml_section(config)),
            });
        }

        if let Some(skill_path) = paths.skill_path(ToolId::Codex) {
            changes.push(FileChange {
                path: skill_path, change_type: FileChangeType::AppendMarkdownSection,
                description: "Append wallet instructions to AGENTS.md".into(),
                diff: Some(content::codex_agents_content()),
            });
        }

        Ok(ProvisionPreview { tool: ToolId::Codex, changes })
    }

    fn provision(&self, config: &McpInjectionConfig, backup_dir: &Path) -> Result<ProvisionResult, ProvisioningError> {
        let paths = self.paths()?;
        let mut files_modified = Vec::new();
        let backup_mgr = backup::BackupManager::new(&paths.tally_dir());

        // TOML config
        if let Some(mcp_path) = paths.mcp_config_path(ToolId::Codex) {
            let sha256_before = if mcp_path.exists() { Some(backup::sha256_hex(&std::fs::read(&mcp_path)?)) } else { None };
            let backup_path = if mcp_path.exists() { Some(backup_mgr.backup_file(backup_dir, "codex", &mcp_path)?.0) } else { None };

            let (_, modified) = config_writer::atomic_modify(&mcp_path, |existing| {
                config_writer::toml_append_mcp_server(existing, content::MCP_SERVER_KEY, &config.server_command, &config.server_args, &config.env, &config.tally_version)
            })?;

            let created_new = sha256_before.is_none();
            files_modified.push(ModifiedFile {
                path: mcp_path, change_type: FileChangeType::AppendTomlSection, backup_path, sha256_before,
                sha256_after: config_writer::sha256_hex(modified.as_bytes()), created_new,
            });
        }

        // AGENTS.md with sentinel markers
        if let Some(skill_path) = paths.skill_path(ToolId::Codex) {
            let sha256_before = if skill_path.exists() { Some(backup::sha256_hex(&std::fs::read(&skill_path)?)) } else { None };
            let backup_path = if skill_path.exists() { Some(backup_mgr.backup_file(backup_dir, "codex", &skill_path)?.0) } else { None };

            let (_, modified) = config_writer::atomic_modify(&skill_path, |existing| {
                config_writer::markdown_upsert_section(existing, content::skill_content_inline(), &config.tally_version)
            })?;

            let created_new = sha256_before.is_none();
            files_modified.push(ModifiedFile {
                path: skill_path, change_type: FileChangeType::AppendMarkdownSection, backup_path, sha256_before,
                sha256_after: config_writer::sha256_hex(modified.as_bytes()), created_new,
            });
        }

        Ok(ProvisionResult { tool: ToolId::Codex, success: true, files_modified, error: None, needs_restart: false })
    }

    fn unprovision(&self) -> Result<UnprovisionResult, ProvisioningError> {
        let paths = self.paths()?;
        let mut files_restored = Vec::new();

        if let Some(mcp_path) = paths.mcp_config_path(ToolId::Codex) {
            if mcp_path.exists() {
                config_writer::atomic_modify(&mcp_path, |existing| {
                    config_writer::toml_remove_mcp_server(existing, content::MCP_SERVER_KEY)
                })?;
                files_restored.push(mcp_path);
            }
        }

        if let Some(skill_path) = paths.skill_path(ToolId::Codex) {
            if skill_path.exists() {
                config_writer::atomic_modify(&skill_path, |existing| {
                    config_writer::markdown_remove_section(existing)
                })?;
                files_restored.push(skill_path);
            }
        }

        Ok(UnprovisionResult { tool: ToolId::Codex, success: true, files_restored, files_deleted: vec![], error: None, strategy_used: RollbackStrategy::SurgicalRemoval })
    }

    fn verify(&self, _expected_version: &str) -> VerificationResult {
        let paths = match &self.paths { Some(p) => p, None => return VerificationResult { tool: ToolId::Codex, status: VerificationStatus::Missing, details: vec![] } };
        let mut details = Vec::new();
        let mut all_intact = true;

        if let Some(mcp_path) = paths.mcp_config_path(ToolId::Codex) {
            if mcp_path.exists() {
                if let Ok(contents) = std::fs::read_to_string(&mcp_path) {
                    let has_section = contents.contains(&format!("[mcp_servers.{}]", content::MCP_SERVER_KEY));
                    let status = if has_section { VerificationStatus::Intact } else { all_intact = false; VerificationStatus::Missing };
                    details.push(VerificationDetail { path: mcp_path, expected_hash: None, actual_hash: None, status });
                }
            } else { all_intact = false; details.push(VerificationDetail { path: mcp_path, expected_hash: None, actual_hash: None, status: VerificationStatus::Missing }); }
        }

        if let Some(skill_path) = paths.skill_path(ToolId::Codex) {
            if skill_path.exists() {
                if let Ok(contents) = std::fs::read_to_string(&skill_path) {
                    let has_section = contents.contains("<!-- TALLY_WALLET_START");
                    let status = if has_section { VerificationStatus::Intact } else { all_intact = false; VerificationStatus::Missing };
                    details.push(VerificationDetail { path: skill_path, expected_hash: None, actual_hash: None, status });
                }
            } else { all_intact = false; details.push(VerificationDetail { path: skill_path, expected_hash: None, actual_hash: None, status: VerificationStatus::Missing }); }
        }

        VerificationResult { tool: ToolId::Codex, status: if all_intact { VerificationStatus::Intact } else { VerificationStatus::Missing }, details }
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

    fn make_provisioner(home: &std::path::Path) -> CodexProvisioner {
        CodexProvisioner::with_paths(PlatformPaths::with_home(home.to_path_buf()))
    }

    #[test]
    fn tool_id_returns_correct_id() {
        let tmp = tempfile::tempdir().unwrap();
        let p = make_provisioner(tmp.path());
        assert_eq!(p.tool_id(), ToolId::Codex);
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
        assert_eq!(preview.tool, ToolId::Codex);
        assert_eq!(preview.changes.len(), 2);
        assert!(preview.changes[0].path.to_string_lossy().contains("config.toml"));
        assert!(matches!(preview.changes[0].change_type, FileChangeType::AppendTomlSection));
        assert!(preview.changes[1].path.to_string_lossy().contains("AGENTS.md"));
        assert!(matches!(preview.changes[1].change_type, FileChangeType::AppendMarkdownSection));
    }

    #[test]
    fn provision_creates_expected_files() {
        let tmp = tempfile::tempdir().unwrap();
        let backup_dir = tempfile::tempdir().unwrap();
        let p = make_provisioner(tmp.path());
        let config = test_config();
        let result = p.provision(&config, backup_dir.path()).unwrap();
        assert!(result.success);
        assert_eq!(result.tool, ToolId::Codex);

        let mcp_path = tmp.path().join(".codex").join("config.toml");
        assert!(mcp_path.exists());
        let contents = std::fs::read_to_string(&mcp_path).unwrap();
        assert!(contents.contains("tally-wallet"));

        let skill_path = tmp.path().join(".codex").join("AGENTS.md");
        assert!(skill_path.exists());
        let skill_contents = std::fs::read_to_string(&skill_path).unwrap();
        assert!(skill_contents.contains("TALLY_WALLET_START"));
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

        // TOML should exist but without our section
        let mcp_path = tmp.path().join(".codex").join("config.toml");
        if mcp_path.exists() {
            let contents = std::fs::read_to_string(&mcp_path).unwrap();
            assert!(!contents.contains("[mcp_servers.tally-wallet]"));
        }

        // AGENTS.md should exist but without sentinel
        let skill_path = tmp.path().join(".codex").join("AGENTS.md");
        if skill_path.exists() {
            let contents = std::fs::read_to_string(&skill_path).unwrap();
            assert!(!contents.contains("TALLY_WALLET_START"));
        }
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
