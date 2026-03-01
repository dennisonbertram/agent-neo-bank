use std::path::Path;
use crate::provisioning::config_writer;
use crate::provisioning::content;
use crate::provisioning::detection;
use crate::provisioning::error::ProvisioningError;
use crate::provisioning::platform::PlatformPaths;
use crate::provisioning::backup;
use crate::provisioning::tools::ToolProvisioner;
use crate::provisioning::types::*;

pub struct ClineProvisioner { paths: Option<PlatformPaths> }

impl ClineProvisioner {
    pub fn new() -> Self { Self { paths: PlatformPaths::new() } }

    #[cfg(test)]
    pub fn with_paths(paths: PlatformPaths) -> Self {
        Self { paths: Some(paths) }
    }

    fn paths(&self) -> Result<&PlatformPaths, ProvisioningError> { self.paths.as_ref().ok_or(ProvisioningError::HomeDirNotFound) }
}

impl ToolProvisioner for ClineProvisioner {
    fn tool_id(&self) -> ToolId { ToolId::Cline }

    fn detect(&self) -> DetectionResult {
        match &self.paths { Some(paths) => detection::detect_tool(ToolId::Cline, paths), None => DetectionResult { tool: ToolId::Cline, detected: false, methods: vec![], version: None, config_paths: vec![] } }
    }

    fn preview(&self, config: &McpInjectionConfig) -> Result<ProvisionPreview, ProvisioningError> {
        let paths = self.paths()?;
        let mut changes = Vec::new();
        if let Some(mcp_path) = paths.mcp_config_path(ToolId::Cline) {
            let entry = content::mcp_json_entry(config);
            changes.push(FileChange { path: mcp_path, change_type: FileChangeType::MergeJsonKey, description: format!("Add \"{}\" MCP server entry", content::MCP_SERVER_KEY), diff: Some(serde_json::to_string_pretty(&entry).unwrap_or_default()) });
        }
        if let Some(skill_path) = paths.skill_path(ToolId::Cline) {
            changes.push(FileChange { path: skill_path, change_type: FileChangeType::CreateFile, description: "Create wallet instructions file".into(), diff: Some(content::standalone_skill_content()) });
        }
        Ok(ProvisionPreview { tool: ToolId::Cline, changes })
    }

    fn provision(&self, config: &McpInjectionConfig, backup_dir: &Path) -> Result<ProvisionResult, ProvisioningError> {
        let paths = self.paths()?;
        let mut files_modified = Vec::new();
        let backup_mgr = backup::BackupManager::new(&paths.tally_dir());

        if let Some(mcp_path) = paths.mcp_config_path(ToolId::Cline) {
            let sha256_before = if mcp_path.exists() { Some(backup::sha256_hex(&std::fs::read(&mcp_path)?)) } else { None };
            let backup_path = if mcp_path.exists() { Some(backup_mgr.backup_file(backup_dir, "cline", &mcp_path)?.0) } else { None };
            let entry = content::mcp_json_entry(config);
            let (_, modified) = config_writer::atomic_modify(&mcp_path, |existing| { config_writer::json_merge_mcp_server(existing, content::MCP_SERVER_KEY, &entry, "mcpServers") })?;
            let created_new = sha256_before.is_none();
            files_modified.push(ModifiedFile { path: mcp_path, change_type: FileChangeType::MergeJsonKey, backup_path, sha256_before, sha256_after: config_writer::sha256_hex(modified.as_bytes()), created_new });
        }

        if let Some(skill_path) = paths.skill_path(ToolId::Cline) {
            let sha256_before = if skill_path.exists() { Some(backup::sha256_hex(&std::fs::read(&skill_path)?)) } else { None };
            let backup_path = if skill_path.exists() { Some(backup_mgr.backup_file(backup_dir, "cline", &skill_path)?.0) } else { None };
            let skill = content::standalone_skill_content();
            config_writer::create_standalone_file(&skill_path, &skill)?;
            let created_new = sha256_before.is_none();
            files_modified.push(ModifiedFile { path: skill_path, change_type: FileChangeType::CreateFile, backup_path, sha256_before, sha256_after: config_writer::sha256_hex(skill.as_bytes()), created_new });
        }

        Ok(ProvisionResult { tool: ToolId::Cline, success: true, files_modified, error: None, needs_restart: false })
    }

    fn unprovision(&self) -> Result<UnprovisionResult, ProvisioningError> {
        let paths = self.paths()?;
        let mut files_restored = Vec::new();
        let mut files_deleted = Vec::new();
        if let Some(mcp_path) = paths.mcp_config_path(ToolId::Cline) {
            if mcp_path.exists() { config_writer::atomic_modify(&mcp_path, |existing| { config_writer::json_remove_mcp_server(existing, content::MCP_SERVER_KEY, "mcpServers") })?; files_restored.push(mcp_path); }
        }
        if let Some(skill_path) = paths.skill_path(ToolId::Cline) {
            if skill_path.exists() { config_writer::delete_standalone_file(&skill_path)?; files_deleted.push(skill_path); }
        }
        Ok(UnprovisionResult { tool: ToolId::Cline, success: true, files_restored, files_deleted, error: None, strategy_used: RollbackStrategy::SurgicalRemoval })
    }

    fn verify(&self, _expected_version: &str) -> VerificationResult {
        let paths = match &self.paths { Some(p) => p, None => return VerificationResult { tool: ToolId::Cline, status: VerificationStatus::Missing, details: vec![] } };
        let mut details = Vec::new();
        let mut all_intact = true;
        if let Some(mcp_path) = paths.mcp_config_path(ToolId::Cline) {
            if mcp_path.exists() { if let Ok(c) = std::fs::read_to_string(&mcp_path) { if let Ok(doc) = serde_json::from_str::<serde_json::Value>(&c) { let has = doc.get("mcpServers").and_then(|s| s.get(content::MCP_SERVER_KEY)).is_some(); let status = if has { VerificationStatus::Intact } else { all_intact = false; VerificationStatus::Missing }; details.push(VerificationDetail { path: mcp_path, expected_hash: None, actual_hash: None, status }); } } } else { all_intact = false; details.push(VerificationDetail { path: mcp_path, expected_hash: None, actual_hash: None, status: VerificationStatus::Missing }); }
        }
        if let Some(skill_path) = paths.skill_path(ToolId::Cline) { let status = if skill_path.exists() { VerificationStatus::Intact } else { all_intact = false; VerificationStatus::Missing }; details.push(VerificationDetail { path: skill_path, expected_hash: None, actual_hash: None, status }); }
        VerificationResult { tool: ToolId::Cline, status: if all_intact { VerificationStatus::Intact } else { VerificationStatus::Missing }, details }
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

    fn make_provisioner(home: &std::path::Path) -> ClineProvisioner {
        ClineProvisioner::with_paths(PlatformPaths::with_home(home.to_path_buf()))
    }

    #[test]
    fn tool_id_returns_correct_id() {
        let tmp = tempfile::tempdir().unwrap();
        let p = make_provisioner(tmp.path());
        assert_eq!(p.tool_id(), ToolId::Cline);
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
        assert_eq!(preview.tool, ToolId::Cline);
        assert_eq!(preview.changes.len(), 2);
        assert!(preview.changes[0].path.to_string_lossy().contains("cline_mcp_settings.json"));
        assert!(matches!(preview.changes[0].change_type, FileChangeType::MergeJsonKey));
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
        assert_eq!(result.tool, ToolId::Cline);

        let paths = PlatformPaths::with_home(tmp.path().to_path_buf());
        let mcp_path = paths.mcp_config_path(ToolId::Cline).unwrap();
        assert!(mcp_path.exists());
        let contents = std::fs::read_to_string(&mcp_path).unwrap();
        assert!(contents.contains("tally-wallet"));

        let skill_path = paths.skill_path(ToolId::Cline).unwrap();
        assert!(skill_path.exists());
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

        let paths = PlatformPaths::with_home(tmp.path().to_path_buf());
        let skill_path = paths.skill_path(ToolId::Cline).unwrap();
        assert!(!skill_path.exists());
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
