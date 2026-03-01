use std::path::Path;

use crate::provisioning::config_writer;
use crate::provisioning::content;
use crate::provisioning::detection;
use crate::provisioning::error::ProvisioningError;
use crate::provisioning::platform::PlatformPaths;
use crate::provisioning::backup;
use crate::provisioning::tools::ToolProvisioner;
use crate::provisioning::types::*;

pub struct ClaudeCodeProvisioner {
    paths: Option<PlatformPaths>,
}

impl ClaudeCodeProvisioner {
    pub fn new() -> Self {
        Self {
            paths: PlatformPaths::new(),
        }
    }

    #[cfg(test)]
    pub fn with_paths(paths: PlatformPaths) -> Self {
        Self { paths: Some(paths) }
    }

    fn paths(&self) -> Result<&PlatformPaths, ProvisioningError> {
        self.paths.as_ref().ok_or(ProvisioningError::HomeDirNotFound)
    }
}

impl ToolProvisioner for ClaudeCodeProvisioner {
    fn tool_id(&self) -> ToolId {
        ToolId::ClaudeCode
    }

    fn detect(&self) -> DetectionResult {
        match &self.paths {
            Some(paths) => detection::detect_tool(ToolId::ClaudeCode, paths),
            None => DetectionResult {
                tool: ToolId::ClaudeCode,
                detected: false,
                methods: vec![],
                version: None,
                config_paths: vec![],
            },
        }
    }

    fn preview(&self, config: &McpInjectionConfig) -> Result<ProvisionPreview, ProvisioningError> {
        let paths = self.paths()?;
        let mut changes = Vec::new();

        // MCP config change
        if let Some(mcp_path) = paths.mcp_config_path(ToolId::ClaudeCode) {
            let entry = content::mcp_json_entry(config);
            changes.push(FileChange {
                path: mcp_path,
                change_type: FileChangeType::MergeJsonKey,
                description: format!("Add \"{}\" MCP server entry", content::MCP_SERVER_KEY),
                diff: Some(serde_json::to_string_pretty(&entry).unwrap_or_default()),
            });
        }

        // Skill file
        if let Some(skill_path) = paths.skill_path(ToolId::ClaudeCode) {
            changes.push(FileChange {
                path: skill_path,
                change_type: FileChangeType::CreateFile,
                description: "Create progressive-disclosure SKILL.md".into(),
                diff: Some(content::claude_code_skill_content()),
            });
        }

        Ok(ProvisionPreview {
            tool: ToolId::ClaudeCode,
            changes,
        })
    }

    fn provision(
        &self,
        config: &McpInjectionConfig,
        backup_dir: &Path,
    ) -> Result<ProvisionResult, ProvisioningError> {
        let paths = self.paths()?;
        let mut files_modified = Vec::new();
        let backup_mgr = backup::BackupManager::new(&paths.tally_dir());

        // 1. MCP config: merge into ~/.claude.json
        if let Some(mcp_path) = paths.mcp_config_path(ToolId::ClaudeCode) {
            let sha256_before = if mcp_path.exists() {
                let contents = std::fs::read(&mcp_path)?;
                Some(backup::sha256_hex(&contents))
            } else {
                None
            };

            // Backup if file exists
            let backup_path = if mcp_path.exists() {
                let (rel, _) = backup_mgr.backup_file(backup_dir, "claude-code", &mcp_path)?;
                Some(rel)
            } else {
                None
            };

            let entry = content::mcp_json_entry(config);
            let (_, modified) = config_writer::atomic_modify(&mcp_path, |existing| {
                config_writer::json_merge_mcp_server(existing, content::MCP_SERVER_KEY, &entry, "mcpServers")
            })?;

            let created_new = sha256_before.is_none();
            files_modified.push(ModifiedFile {
                path: mcp_path,
                change_type: FileChangeType::MergeJsonKey,
                backup_path,
                sha256_before,
                sha256_after: config_writer::sha256_hex(modified.as_bytes()),
                created_new,
            });
        }

        // 2. Skill file: create ~/.claude/skills/tally-wallet/SKILL.md
        if let Some(skill_path) = paths.skill_path(ToolId::ClaudeCode) {
            let sha256_before = if skill_path.exists() {
                let contents = std::fs::read(&skill_path)?;
                Some(backup::sha256_hex(&contents))
            } else {
                None
            };

            let backup_path = if skill_path.exists() {
                let (rel, _) = backup_mgr.backup_file(backup_dir, "claude-code", &skill_path)?;
                Some(rel)
            } else {
                None
            };

            let skill_content = content::claude_code_skill_content();
            config_writer::create_standalone_file(&skill_path, &skill_content)?;

            let created_new = sha256_before.is_none();
            files_modified.push(ModifiedFile {
                path: skill_path,
                change_type: FileChangeType::CreateFile,
                backup_path,
                sha256_before,
                sha256_after: config_writer::sha256_hex(skill_content.as_bytes()),
                created_new,
            });
        }

        Ok(ProvisionResult {
            tool: ToolId::ClaudeCode,
            success: true,
            files_modified,
            error: None,
            needs_restart: false,
        })
    }

    fn unprovision(&self) -> Result<UnprovisionResult, ProvisioningError> {
        let paths = self.paths()?;
        let mut files_restored = Vec::new();
        let mut files_deleted = Vec::new();

        // Remove MCP entry from ~/.claude.json
        if let Some(mcp_path) = paths.mcp_config_path(ToolId::ClaudeCode) {
            if mcp_path.exists() {
                config_writer::atomic_modify(&mcp_path, |existing| {
                    config_writer::json_remove_mcp_server(existing, content::MCP_SERVER_KEY, "mcpServers")
                })?;
                files_restored.push(mcp_path);
            }
        }

        // Delete skill file
        if let Some(skill_path) = paths.skill_path(ToolId::ClaudeCode) {
            if skill_path.exists() {
                config_writer::delete_standalone_file(&skill_path)?;
                files_deleted.push(skill_path);
            }
        }

        Ok(UnprovisionResult {
            tool: ToolId::ClaudeCode,
            success: true,
            files_restored,
            files_deleted,
            error: None,
            strategy_used: RollbackStrategy::SurgicalRemoval,
        })
    }

    fn verify(&self, expected_version: &str) -> VerificationResult {
        let paths = match &self.paths {
            Some(p) => p,
            None => return VerificationResult {
                tool: ToolId::ClaudeCode,
                status: VerificationStatus::Missing,
                details: vec![],
            },
        };

        let mut details = Vec::new();
        let mut all_intact = true;

        // Check MCP config
        if let Some(mcp_path) = paths.mcp_config_path(ToolId::ClaudeCode) {
            if mcp_path.exists() {
                if let Ok(contents) = std::fs::read_to_string(&mcp_path) {
                    if let Ok(doc) = serde_json::from_str::<serde_json::Value>(&contents) {
                        let has_key = doc.get("mcpServers")
                            .and_then(|s| s.get(content::MCP_SERVER_KEY))
                            .is_some();
                        let version_match = doc.get("mcpServers")
                            .and_then(|s| s.get(content::MCP_SERVER_KEY))
                            .and_then(|s| s.get("_tally_version"))
                            .and_then(|v| v.as_str())
                            .map(|v| v == expected_version)
                            .unwrap_or(false);

                        let status = if !has_key {
                            all_intact = false;
                            VerificationStatus::Missing
                        } else if !version_match {
                            VerificationStatus::Modified
                        } else {
                            VerificationStatus::Intact
                        };
                        details.push(VerificationDetail {
                            path: mcp_path,
                            expected_hash: None,
                            actual_hash: None,
                            status,
                        });
                    }
                }
            } else {
                all_intact = false;
                details.push(VerificationDetail {
                    path: mcp_path,
                    expected_hash: None,
                    actual_hash: None,
                    status: VerificationStatus::Missing,
                });
            }
        }

        // Check skill file
        if let Some(skill_path) = paths.skill_path(ToolId::ClaudeCode) {
            let status = if skill_path.exists() {
                VerificationStatus::Intact
            } else {
                all_intact = false;
                VerificationStatus::Missing
            };
            details.push(VerificationDetail {
                path: skill_path,
                expected_hash: None,
                actual_hash: None,
                status,
            });
        }

        VerificationResult {
            tool: ToolId::ClaudeCode,
            status: if all_intact { VerificationStatus::Intact } else { VerificationStatus::Missing },
            details,
        }
    }

    fn needs_restart_after_provision(&self) -> bool {
        false // Claude Code picks up changes immediately
    }
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

    fn make_provisioner(home: &std::path::Path) -> ClaudeCodeProvisioner {
        ClaudeCodeProvisioner::with_paths(PlatformPaths::with_home(home.to_path_buf()))
    }

    #[test]
    fn tool_id_returns_correct_id() {
        let tmp = tempfile::tempdir().unwrap();
        let p = make_provisioner(tmp.path());
        assert_eq!(p.tool_id(), ToolId::ClaudeCode);
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
        assert_eq!(preview.tool, ToolId::ClaudeCode);
        assert_eq!(preview.changes.len(), 2);
        // MCP config
        assert!(preview.changes[0].path.ends_with(".claude.json"));
        assert!(matches!(preview.changes[0].change_type, FileChangeType::MergeJsonKey));
        // Skill file
        assert!(preview.changes[1].path.to_string_lossy().contains("SKILL.md"));
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
        assert_eq!(result.tool, ToolId::ClaudeCode);
        assert_eq!(result.files_modified.len(), 2);

        // Verify MCP config file exists and contains our server
        let mcp_path = tmp.path().join(".claude.json");
        assert!(mcp_path.exists());
        let contents = std::fs::read_to_string(&mcp_path).unwrap();
        assert!(contents.contains("tally-wallet"));

        // Verify skill file exists
        let skill_path = tmp.path().join(".claude").join("skills").join("tally-wallet").join("SKILL.md");
        assert!(skill_path.exists());
        let skill_contents = std::fs::read_to_string(&skill_path).unwrap();
        assert!(skill_contents.contains("Tally Agentic Wallet"));
    }

    #[test]
    fn provision_is_idempotent() {
        let tmp = tempfile::tempdir().unwrap();
        let backup_dir = tempfile::tempdir().unwrap();
        let p = make_provisioner(tmp.path());
        let config = test_config();
        let r1 = p.provision(&config, backup_dir.path()).unwrap();
        assert!(r1.success);
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

        // Skill file should be deleted
        let skill_path = tmp.path().join(".claude").join("skills").join("tally-wallet").join("SKILL.md");
        assert!(!skill_path.exists());

        // MCP config should still exist but without our key
        let mcp_path = tmp.path().join(".claude.json");
        if mcp_path.exists() {
            let contents = std::fs::read_to_string(&mcp_path).unwrap();
            let doc: serde_json::Value = serde_json::from_str(&contents).unwrap();
            let has_key = doc.get("mcpServers")
                .and_then(|s| s.get("tally-wallet"))
                .is_some();
            assert!(!has_key);
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
