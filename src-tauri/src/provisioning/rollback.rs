use std::path::Path;

use crate::provisioning::backup::BackupManager;
use crate::provisioning::config_writer;
use crate::provisioning::content::MCP_SERVER_KEY;
use crate::provisioning::error::ProvisioningError;
use crate::provisioning::types::*;

/// Perform surgical removal of Tally Wallet config from a tool's files.
/// This is the preferred rollback strategy — it removes only our content
/// while preserving all user modifications.
pub fn surgical_remove(
    tool: ToolId,
    files: &[BackupFileEntry],
) -> Result<UnprovisionResult, ProvisioningError> {
    let mut files_restored = Vec::new();
    let mut files_deleted = Vec::new();

    for file in files {
        let path = &file.original_path;

        if !path.exists() {
            // File already gone — nothing to do
            continue;
        }

        if file.created_new {
            // We created this file entirely — just delete it
            std::fs::remove_file(path).map_err(|e| ProvisioningError::RollbackFailed {
                tool: tool.display_name().to_string(),
                reason: format!("Failed to delete {}: {}", path.display(), e),
            })?;
            files_deleted.push(path.clone());
            continue;
        }

        // Surgical removal based on file type
        let current_contents = std::fs::read_to_string(path).map_err(|e| {
            ProvisioningError::RollbackFailed {
                tool: tool.display_name().to_string(),
                reason: format!("Failed to read {}: {}", path.display(), e),
            }
        })?;

        let result = match file.modification_type {
            FileChangeType::MergeJsonKey => {
                let root_key = if !file.keys_added.is_empty() {
                    // Extract root key from "mcpServers.tally-wallet" format
                    file.keys_added[0]
                        .split('.')
                        .next()
                        .unwrap_or("mcpServers")
                } else {
                    "mcpServers"
                };
                config_writer::json_remove_mcp_server(&current_contents, MCP_SERVER_KEY, root_key)
            }
            FileChangeType::AppendTomlSection => {
                config_writer::toml_remove_mcp_server(&current_contents, MCP_SERVER_KEY)
            }
            FileChangeType::AppendMarkdownSection => {
                config_writer::markdown_remove_section(&current_contents)
            }
            FileChangeType::MergeYamlEntry => {
                config_writer::yaml_remove_mcp_server_list(&current_contents)
            }
            FileChangeType::CreateFile => {
                // Shouldn't reach here (handled by created_new above), but just in case
                std::fs::remove_file(path).ok();
                files_deleted.push(path.clone());
                continue;
            }
        };

        match result {
            Ok(cleaned) => {
                // Write the cleaned content back
                config_writer::atomic_modify(path, |_| Ok(cleaned.clone())).map_err(|e| {
                    ProvisioningError::RollbackFailed {
                        tool: tool.display_name().to_string(),
                        reason: format!("Failed to write cleaned {}: {}", path.display(), e),
                    }
                })?;
                files_restored.push(path.clone());
            }
            Err(e) => {
                return Err(ProvisioningError::RollbackFailed {
                    tool: tool.display_name().to_string(),
                    reason: format!(
                        "Surgical removal failed for {}: {}",
                        path.display(),
                        e
                    ),
                });
            }
        }
    }

    Ok(UnprovisionResult {
        tool,
        success: true,
        files_restored,
        files_deleted,
        error: None,
        strategy_used: RollbackStrategy::SurgicalRemoval,
    })
}

/// Full restore from backup — used when surgical removal is not possible.
pub fn full_restore(
    tool: ToolId,
    backup_dir: &Path,
    files: &[BackupFileEntry],
    backup_manager: &BackupManager,
) -> Result<UnprovisionResult, ProvisioningError> {
    let mut files_restored = Vec::new();
    let mut files_deleted = Vec::new();

    for file in files {
        let path = &file.original_path;

        if file.created_new {
            // We created this file — delete it
            if path.exists() {
                std::fs::remove_file(path).map_err(|e| ProvisioningError::RollbackFailed {
                    tool: tool.display_name().to_string(),
                    reason: format!("Failed to delete {}: {}", path.display(), e),
                })?;
                files_deleted.push(path.clone());
            }
            continue;
        }

        // Restore from backup
        let backup_path = backup_manager.get_backup_path(backup_dir, &file.backup_relative_path);
        if !backup_path.exists() {
            return Err(ProvisioningError::RollbackFailed {
                tool: tool.display_name().to_string(),
                reason: format!(
                    "Backup file not found: {}",
                    backup_path.display()
                ),
            });
        }

        // Verify backup integrity before restoring
        if let Some(ref expected_hash) = file.sha256_before {
            let backup_contents = std::fs::read(&backup_path)?;
            let actual_hash = crate::provisioning::backup::sha256_hex(&backup_contents);
            if &actual_hash != expected_hash {
                return Err(ProvisioningError::RollbackFailed {
                    tool: tool.display_name().to_string(),
                    reason: format!(
                        "Backup integrity check failed for {}: expected {}, got {}",
                        backup_path.display(),
                        expected_hash,
                        actual_hash
                    ),
                });
            }
        }

        // Copy backup to original path
        std::fs::copy(&backup_path, path).map_err(|e| ProvisioningError::RollbackFailed {
            tool: tool.display_name().to_string(),
            reason: format!("Failed to restore {}: {}", path.display(), e),
        })?;
        files_restored.push(path.clone());
    }

    Ok(UnprovisionResult {
        tool,
        success: true,
        files_restored,
        files_deleted,
        error: None,
        strategy_used: RollbackStrategy::FullRestore,
    })
}

/// Check if a tool's provisioned config is still intact.
/// Returns true if our config is present and matches expectations.
pub fn check_integrity(path: &Path, file_entry: &BackupFileEntry) -> bool {
    if !path.exists() {
        return false;
    }

    // Compare SHA-256
    if let Ok(contents) = std::fs::read(path) {
        let current_hash = crate::provisioning::backup::sha256_hex(&contents);
        current_hash == file_entry.sha256_after
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provisioning::backup::{sha256_hex, BackupManager};
    use std::path::PathBuf;
    use tempfile::TempDir;

    /// Helper: build a BackupFileEntry with sensible defaults.
    fn make_entry(
        original_path: PathBuf,
        modification_type: FileChangeType,
        created_new: bool,
        sha256_after: &str,
    ) -> BackupFileEntry {
        BackupFileEntry {
            original_path,
            resolved_path: PathBuf::new(),
            backup_relative_path: PathBuf::new(),
            modification_type,
            created_new,
            sha256_before: None,
            sha256_after: sha256_after.to_string(),
            keys_added: vec![],
            sections_added: vec![],
            sentinel_start: None,
            sentinel_end: None,
        }
    }

    // ══════════════════════════════════════════════════════════════
    // surgical_remove — MergeJsonKey
    // ══════════════════════════════════════════════════════════════

    #[test]
    fn surgical_remove_json_removes_tally_wallet_key_preserves_others() {
        let tmp = TempDir::new().unwrap();
        let json_path = tmp.path().join("mcp.json");

        let content = serde_json::json!({
            "mcpServers": {
                "tally-wallet": { "command": "tally-mcp", "args": ["--stdio"] },
                "other-server": { "command": "other", "args": [] }
            }
        });
        std::fs::write(&json_path, serde_json::to_string_pretty(&content).unwrap()).unwrap();

        let entry = BackupFileEntry {
            original_path: json_path.clone(),
            resolved_path: json_path.clone(),
            backup_relative_path: PathBuf::new(),
            modification_type: FileChangeType::MergeJsonKey,
            created_new: false,
            sha256_before: None,
            sha256_after: String::new(),
            keys_added: vec!["mcpServers.tally-wallet".to_string()],
            sections_added: vec![],
            sentinel_start: None,
            sentinel_end: None,
        };

        let result = surgical_remove(ToolId::Cursor, &[entry]).unwrap();
        assert!(result.success);
        assert_eq!(result.files_restored.len(), 1);
        assert_eq!(result.files_deleted.len(), 0);

        // Verify file content: tally-wallet gone, other-server preserved
        let after: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&json_path).unwrap()).unwrap();
        assert!(after["mcpServers"].get("tally-wallet").is_none());
        assert!(after["mcpServers"].get("other-server").is_some());
    }

    #[test]
    fn surgical_remove_json_with_servers_root_key() {
        let tmp = TempDir::new().unwrap();
        let json_path = tmp.path().join("settings.json");

        let content = serde_json::json!({
            "servers": {
                "tally-wallet": { "command": "tally-mcp" },
                "copilot": { "command": "copilot-server" }
            }
        });
        std::fs::write(&json_path, serde_json::to_string_pretty(&content).unwrap()).unwrap();

        let entry = BackupFileEntry {
            original_path: json_path.clone(),
            resolved_path: json_path.clone(),
            backup_relative_path: PathBuf::new(),
            modification_type: FileChangeType::MergeJsonKey,
            created_new: false,
            sha256_before: None,
            sha256_after: String::new(),
            keys_added: vec!["servers.tally-wallet".to_string()],
            sections_added: vec![],
            sentinel_start: None,
            sentinel_end: None,
        };

        let result = surgical_remove(ToolId::Copilot, &[entry]).unwrap();
        assert!(result.success);

        let after: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&json_path).unwrap()).unwrap();
        assert!(after["servers"].get("tally-wallet").is_none());
        assert!(after["servers"].get("copilot").is_some());
    }

    // ══════════════════════════════════════════════════════════════
    // surgical_remove — AppendTomlSection
    // ══════════════════════════════════════════════════════════════

    #[test]
    fn surgical_remove_toml_removes_tally_wallet_section() {
        let tmp = TempDir::new().unwrap();
        let toml_path = tmp.path().join("config.toml");

        let content = r#"
[mcp_servers.tally-wallet]
command = "tally-mcp"
args = ["--stdio"]

[mcp_servers.other-server]
command = "other"
"#;
        std::fs::write(&toml_path, content).unwrap();

        let entry = make_entry(
            toml_path.clone(),
            FileChangeType::AppendTomlSection,
            false,
            "",
        );

        let result = surgical_remove(ToolId::Codex, &[entry]).unwrap();
        assert!(result.success);
        assert_eq!(result.files_restored.len(), 1);

        let after = std::fs::read_to_string(&toml_path).unwrap();
        assert!(!after.contains("tally-wallet"));
        assert!(after.contains("other-server"));
    }

    #[test]
    fn surgical_remove_toml_only_tally_section() {
        let tmp = TempDir::new().unwrap();
        let toml_path = tmp.path().join("config.toml");

        let content = r#"[mcp_servers.tally-wallet]
command = "tally-mcp"
"#;
        std::fs::write(&toml_path, content).unwrap();

        let entry = make_entry(
            toml_path.clone(),
            FileChangeType::AppendTomlSection,
            false,
            "",
        );

        let result = surgical_remove(ToolId::Codex, &[entry]).unwrap();
        assert!(result.success);

        let after = std::fs::read_to_string(&toml_path).unwrap();
        assert!(!after.contains("tally-wallet"));
    }

    // ══════════════════════════════════════════════════════════════
    // surgical_remove — AppendMarkdownSection
    // ══════════════════════════════════════════════════════════════

    #[test]
    fn surgical_remove_markdown_removes_sentinel_block() {
        let tmp = TempDir::new().unwrap();
        let md_path = tmp.path().join("AGENTS.md");

        let content = "# My Agents\n\nSome existing content.\n\n<!-- TALLY_WALLET_START v1.0.0 -->\nTally wallet instructions here.\n<!-- TALLY_WALLET_END -->\n\nMore content after.\n";
        std::fs::write(&md_path, content).unwrap();

        let entry = make_entry(
            md_path.clone(),
            FileChangeType::AppendMarkdownSection,
            false,
            "",
        );

        let result = surgical_remove(ToolId::Codex, &[entry]).unwrap();
        assert!(result.success);
        assert_eq!(result.files_restored.len(), 1);

        let after = std::fs::read_to_string(&md_path).unwrap();
        assert!(!after.contains("TALLY_WALLET_START"));
        assert!(!after.contains("TALLY_WALLET_END"));
        assert!(after.contains("My Agents"));
        assert!(after.contains("More content after"));
    }

    #[test]
    fn surgical_remove_markdown_no_sentinels_is_noop() {
        let tmp = TempDir::new().unwrap();
        let md_path = tmp.path().join("AGENTS.md");

        let content = "# My Agents\n\nNo tally content here.\n";
        std::fs::write(&md_path, content).unwrap();

        let entry = make_entry(
            md_path.clone(),
            FileChangeType::AppendMarkdownSection,
            false,
            "",
        );

        let result = surgical_remove(ToolId::Codex, &[entry]).unwrap();
        assert!(result.success);
        // File is still "restored" (written back) even if content unchanged
        assert_eq!(result.files_restored.len(), 1);
    }

    // ══════════════════════════════════════════════════════════════
    // surgical_remove — MergeYamlEntry
    // ══════════════════════════════════════════════════════════════

    #[test]
    fn surgical_remove_yaml_removes_tally_wallet_entry() {
        let tmp = TempDir::new().unwrap();
        let yaml_path = tmp.path().join("config.yaml");

        let content = r#"mcpServers:
  - name: tally-wallet
    command: tally-mcp
  - name: other-server
    command: other
"#;
        std::fs::write(&yaml_path, content).unwrap();

        let entry = make_entry(
            yaml_path.clone(),
            FileChangeType::MergeYamlEntry,
            false,
            "",
        );

        let result = surgical_remove(ToolId::ContinueDev, &[entry]).unwrap();
        assert!(result.success);
        assert_eq!(result.files_restored.len(), 1);

        let after = std::fs::read_to_string(&yaml_path).unwrap();
        assert!(!after.contains("tally-wallet"));
        assert!(after.contains("other-server"));
    }

    #[test]
    fn surgical_remove_yaml_only_tally_entry() {
        let tmp = TempDir::new().unwrap();
        let yaml_path = tmp.path().join("config.yaml");

        let content = r#"mcpServers:
  - name: tally-wallet
    command: tally-mcp
"#;
        std::fs::write(&yaml_path, content).unwrap();

        let entry = make_entry(
            yaml_path.clone(),
            FileChangeType::MergeYamlEntry,
            false,
            "",
        );

        let result = surgical_remove(ToolId::ContinueDev, &[entry]).unwrap();
        assert!(result.success);

        let after = std::fs::read_to_string(&yaml_path).unwrap();
        assert!(!after.contains("tally-wallet"));
    }

    // ══════════════════════════════════════════════════════════════
    // surgical_remove — CreateFile (created_new = true)
    // ══════════════════════════════════════════════════════════════

    #[test]
    fn surgical_remove_created_new_deletes_file() {
        let tmp = TempDir::new().unwrap();
        let file_path = tmp.path().join("skill.md");
        std::fs::write(&file_path, "tally wallet skill").unwrap();
        assert!(file_path.exists());

        let entry = make_entry(
            file_path.clone(),
            FileChangeType::CreateFile,
            true,
            "",
        );

        let result = surgical_remove(ToolId::ClaudeCode, &[entry]).unwrap();
        assert!(result.success);
        assert_eq!(result.files_deleted.len(), 1);
        assert_eq!(result.files_restored.len(), 0);
        assert!(!file_path.exists());
    }

    #[test]
    fn surgical_remove_created_new_missing_file_is_skipped() {
        let tmp = TempDir::new().unwrap();
        let file_path = tmp.path().join("nonexistent.md");
        assert!(!file_path.exists());

        let entry = make_entry(
            file_path.clone(),
            FileChangeType::CreateFile,
            true,
            "",
        );

        let result = surgical_remove(ToolId::ClaudeCode, &[entry]).unwrap();
        assert!(result.success);
        assert_eq!(result.files_deleted.len(), 0);
        assert_eq!(result.files_restored.len(), 0);
    }

    // ══════════════════════════════════════════════════════════════
    // surgical_remove — missing file is silently skipped
    // ══════════════════════════════════════════════════════════════

    #[test]
    fn surgical_remove_missing_file_skipped_no_error() {
        let tmp = TempDir::new().unwrap();
        let missing = tmp.path().join("gone.json");

        let entry = make_entry(
            missing.clone(),
            FileChangeType::MergeJsonKey,
            false,
            "",
        );

        let result = surgical_remove(ToolId::Cursor, &[entry]).unwrap();
        assert!(result.success);
        assert_eq!(result.files_restored.len(), 0);
        assert_eq!(result.files_deleted.len(), 0);
    }

    // ══════════════════════════════════════════════════════════════
    // surgical_remove — correct counts with mixed entries
    // ══════════════════════════════════════════════════════════════

    #[test]
    fn surgical_remove_returns_correct_counts_mixed() {
        let tmp = TempDir::new().unwrap();

        // A file we created (will be deleted)
        let created_path = tmp.path().join("skill.md");
        std::fs::write(&created_path, "content").unwrap();

        // A JSON file we merged into (will be restored/cleaned)
        let json_path = tmp.path().join("mcp.json");
        let json_content = serde_json::json!({
            "mcpServers": {
                "tally-wallet": { "command": "tally" },
                "keep-me": { "command": "keep" }
            }
        });
        std::fs::write(&json_path, serde_json::to_string_pretty(&json_content).unwrap()).unwrap();

        // A missing file (skipped)
        let missing_path = tmp.path().join("missing.json");

        let entries = vec![
            make_entry(created_path.clone(), FileChangeType::CreateFile, true, ""),
            BackupFileEntry {
                original_path: json_path.clone(),
                resolved_path: json_path.clone(),
                backup_relative_path: PathBuf::new(),
                modification_type: FileChangeType::MergeJsonKey,
                created_new: false,
                sha256_before: None,
                sha256_after: String::new(),
                keys_added: vec!["mcpServers.tally-wallet".to_string()],
                sections_added: vec![],
                sentinel_start: None,
                sentinel_end: None,
            },
            make_entry(missing_path, FileChangeType::MergeJsonKey, false, ""),
        ];

        let result = surgical_remove(ToolId::Cursor, &entries).unwrap();
        assert!(result.success);
        assert_eq!(result.files_deleted.len(), 1);
        assert_eq!(result.files_restored.len(), 1);
    }

    // ══════════════════════════════════════════════════════════════
    // full_restore — created_new file is deleted
    // ══════════════════════════════════════════════════════════════

    #[test]
    fn full_restore_deletes_created_new_file() {
        let tmp = TempDir::new().unwrap();
        let file_path = tmp.path().join("skill.md");
        std::fs::write(&file_path, "tally skill content").unwrap();

        let mgr = BackupManager::new(tmp.path());
        let backup_dir = mgr.create_backup_dir().unwrap();

        let entry = make_entry(
            file_path.clone(),
            FileChangeType::CreateFile,
            true,
            "",
        );

        let result = full_restore(ToolId::ClaudeCode, &backup_dir, &[entry], &mgr).unwrap();
        assert!(result.success);
        assert_eq!(result.files_deleted.len(), 1);
        assert!(!file_path.exists());
        assert_eq!(result.strategy_used, RollbackStrategy::FullRestore);
    }

    // ══════════════════════════════════════════════════════════════
    // full_restore — restores from backup
    // ══════════════════════════════════════════════════════════════

    #[test]
    fn full_restore_restores_file_from_backup() {
        let tmp = TempDir::new().unwrap();
        let mgr = BackupManager::new(tmp.path());
        let backup_dir = mgr.create_backup_dir().unwrap();

        // Create original file with known content
        let original_path = tmp.path().join("mcp.json");
        let original_content = r#"{"mcpServers":{}}"#;
        std::fs::write(&original_path, original_content).unwrap();
        let original_hash = sha256_hex(original_content.as_bytes());

        // Back it up
        let (rel_path, _hash) = mgr.backup_file(&backup_dir, "cursor", &original_path).unwrap();

        // Simulate provisioning: overwrite the original
        std::fs::write(&original_path, r#"{"mcpServers":{"tally-wallet":{}}}"#).unwrap();

        let entry = BackupFileEntry {
            original_path: original_path.clone(),
            resolved_path: original_path.clone(),
            backup_relative_path: rel_path,
            modification_type: FileChangeType::MergeJsonKey,
            created_new: false,
            sha256_before: Some(original_hash),
            sha256_after: String::new(),
            keys_added: vec![],
            sections_added: vec![],
            sentinel_start: None,
            sentinel_end: None,
        };

        let result = full_restore(ToolId::Cursor, &backup_dir, &[entry], &mgr).unwrap();
        assert!(result.success);
        assert_eq!(result.files_restored.len(), 1);

        // File should be restored to original content
        let restored = std::fs::read_to_string(&original_path).unwrap();
        assert_eq!(restored, original_content);
    }

    // ══════════════════════════════════════════════════════════════
    // full_restore — missing backup returns RollbackFailed
    // ══════════════════════════════════════════════════════════════

    #[test]
    fn full_restore_missing_backup_returns_error() {
        let tmp = TempDir::new().unwrap();
        let mgr = BackupManager::new(tmp.path());
        let backup_dir = mgr.create_backup_dir().unwrap();

        let original_path = tmp.path().join("mcp.json");
        std::fs::write(&original_path, "{}").unwrap();

        let entry = BackupFileEntry {
            original_path: original_path.clone(),
            resolved_path: original_path.clone(),
            backup_relative_path: PathBuf::from("cursor/mcp.json.bak"),
            modification_type: FileChangeType::MergeJsonKey,
            created_new: false,
            sha256_before: None,
            sha256_after: String::new(),
            keys_added: vec![],
            sections_added: vec![],
            sentinel_start: None,
            sentinel_end: None,
        };

        let result = full_restore(ToolId::Cursor, &backup_dir, &[entry], &mgr);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Backup file not found"),
            "Expected 'Backup file not found' in: {}",
            err_msg
        );
    }

    #[test]
    fn full_restore_corrupted_backup_returns_error() {
        let tmp = TempDir::new().unwrap();
        let mgr = BackupManager::new(tmp.path());
        let backup_dir = mgr.create_backup_dir().unwrap();

        // Create original file and back it up
        let original_path = tmp.path().join("settings.json");
        let original_content = r#"{"key":"value"}"#;
        std::fs::write(&original_path, original_content).unwrap();

        let (rel_path, _hash) = mgr.backup_file(&backup_dir, "cursor", &original_path).unwrap();

        // Corrupt the backup
        let backup_full = backup_dir.join(&rel_path);
        std::fs::write(&backup_full, "corrupted!").unwrap();

        let entry = BackupFileEntry {
            original_path: original_path.clone(),
            resolved_path: original_path.clone(),
            backup_relative_path: rel_path,
            modification_type: FileChangeType::MergeJsonKey,
            created_new: false,
            sha256_before: Some(sha256_hex(original_content.as_bytes())),
            sha256_after: String::new(),
            keys_added: vec![],
            sections_added: vec![],
            sentinel_start: None,
            sentinel_end: None,
        };

        let result = full_restore(ToolId::Cursor, &backup_dir, &[entry], &mgr);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("integrity check failed"),
            "Expected integrity error in: {}",
            err_msg
        );
    }

    // ══════════════════════════════════════════════════════════════
    // check_integrity — file absent returns false
    // ══════════════════════════════════════════════════════════════

    #[test]
    fn check_integrity_missing_file_returns_false() {
        let tmp = TempDir::new().unwrap();
        let missing = tmp.path().join("nonexistent.json");

        let entry = make_entry(
            missing.clone(),
            FileChangeType::MergeJsonKey,
            false,
            "abcdef1234567890",
        );

        assert!(!check_integrity(&missing, &entry));
    }

    // ══════════════════════════════════════════════════════════════
    // check_integrity — matching hash returns true
    // ══════════════════════════════════════════════════════════════

    #[test]
    fn check_integrity_matching_hash_returns_true() {
        let tmp = TempDir::new().unwrap();
        let file_path = tmp.path().join("mcp.json");
        let content = b"provisioned content here";
        std::fs::write(&file_path, content).unwrap();

        let expected_hash = sha256_hex(content);
        let entry = make_entry(
            file_path.clone(),
            FileChangeType::MergeJsonKey,
            false,
            &expected_hash,
        );

        assert!(check_integrity(&file_path, &entry));
    }

    // ══════════════════════════════════════════════════════════════
    // check_integrity — mismatched hash returns false
    // ══════════════════════════════════════════════════════════════

    #[test]
    fn check_integrity_mismatched_hash_returns_false() {
        let tmp = TempDir::new().unwrap();
        let file_path = tmp.path().join("mcp.json");
        std::fs::write(&file_path, b"actual content").unwrap();

        let entry = make_entry(
            file_path.clone(),
            FileChangeType::MergeJsonKey,
            false,
            "0000000000000000000000000000000000000000000000000000000000000000",
        );

        assert!(!check_integrity(&file_path, &entry));
    }

    #[test]
    fn check_integrity_empty_file_with_correct_hash() {
        let tmp = TempDir::new().unwrap();
        let file_path = tmp.path().join("empty.json");
        std::fs::write(&file_path, b"").unwrap();

        let empty_hash = sha256_hex(b"");
        let entry = make_entry(
            file_path.clone(),
            FileChangeType::CreateFile,
            true,
            &empty_hash,
        );

        assert!(check_integrity(&file_path, &entry));
    }

    // ══════════════════════════════════════════════════════════════
    // surgical_remove — RollbackStrategy is SurgicalRemoval
    // ══════════════════════════════════════════════════════════════

    #[test]
    fn surgical_remove_strategy_is_surgical_removal() {
        let result = surgical_remove(ToolId::Cursor, &[]).unwrap();
        assert_eq!(result.strategy_used, RollbackStrategy::SurgicalRemoval);
    }
}
