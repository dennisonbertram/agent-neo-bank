use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::provisioning::error::ProvisioningError;
use crate::provisioning::types::*;

/// Manages backup creation, verification, and retention.
pub struct BackupManager {
    backups_dir: PathBuf,
}

impl BackupManager {
    pub fn new(tally_dir: &Path) -> Self {
        Self {
            backups_dir: tally_dir.join("backups"),
        }
    }

    /// Create a new timestamped backup directory. Returns the path.
    pub fn create_backup_dir(&self) -> Result<PathBuf, ProvisioningError> {
        let timestamp = chrono::Utc::now()
            .format("%Y-%m-%dT%H-%M-%SZ")
            .to_string();
        let dir = self.backups_dir.join(&timestamp);
        std::fs::create_dir_all(&dir)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700))?;
        }

        Ok(dir)
    }

    /// Back up a single file. Returns the backup-relative path and SHA-256 of the original.
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
            .ok_or_else(|| ProvisioningError::BackupFailed {
                path: original_path.to_path_buf(),
                reason: "No file name in path".into(),
            })?;
        let backup_name = format!("{}.bak", file_name.to_string_lossy());
        let backup_path = tool_dir.join(&backup_name);
        let relative_path = PathBuf::from(tool_slug).join(&backup_name);

        // Read and hash original
        let contents = std::fs::read(original_path).map_err(|e| ProvisioningError::BackupFailed {
            path: original_path.to_path_buf(),
            reason: format!("Failed to read original: {}", e),
        })?;
        let hash = sha256_hex(&contents);

        // Write backup
        std::fs::write(&backup_path, &contents).map_err(|e| ProvisioningError::BackupFailed {
            path: backup_path.clone(),
            reason: format!("Failed to write backup: {}", e),
        })?;

        // Verify backup integrity
        let verify_contents =
            std::fs::read(&backup_path).map_err(|e| ProvisioningError::BackupFailed {
                path: backup_path.clone(),
                reason: format!("Failed to read back backup for verification: {}", e),
            })?;
        let verify_hash = sha256_hex(&verify_contents);
        if hash != verify_hash {
            return Err(ProvisioningError::BackupIntegrityFailed {
                expected: hash,
                actual: verify_hash,
            });
        }

        // Set 0600 permissions on backup
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(
                &backup_path,
                std::fs::Permissions::from_mode(0o600),
            )?;
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

    /// Read the manifest from a backup directory.
    pub fn read_manifest(&self, backup_dir: &Path) -> Result<BackupManifest, ProvisioningError> {
        let path = backup_dir.join("manifest.json");
        let contents = std::fs::read_to_string(&path)?;
        let manifest: BackupManifest = serde_json::from_str(&contents)?;
        Ok(manifest)
    }

    /// Get the backup file path for restoring.
    pub fn get_backup_path(
        &self,
        backup_dir: &Path,
        relative_path: &Path,
    ) -> PathBuf {
        backup_dir.join(relative_path)
    }

    /// List all backup directories, sorted newest first.
    pub fn list_backups(&self) -> Result<Vec<PathBuf>, ProvisioningError> {
        if !self.backups_dir.exists() {
            return Ok(vec![]);
        }

        let mut dirs: Vec<PathBuf> = std::fs::read_dir(&self.backups_dir)?
            .flatten()
            .filter(|e| e.path().is_dir())
            .map(|e| e.path())
            .collect();

        // Sort by name descending (timestamp-based names sort chronologically)
        dirs.sort_by(|a, b| b.cmp(a));
        Ok(dirs)
    }

    /// Find the most recent backup for a specific tool.
    pub fn find_latest_backup_for_tool(
        &self,
        tool: ToolId,
    ) -> Result<Option<(PathBuf, BackupManifest)>, ProvisioningError> {
        for backup_dir in self.list_backups()? {
            if let Ok(manifest) = self.read_manifest(&backup_dir) {
                if manifest
                    .tools_modified
                    .iter()
                    .any(|t| t.tool == tool)
                {
                    return Ok(Some((backup_dir, manifest)));
                }
            }
        }
        Ok(None)
    }

    /// Enforce backup retention policy.
    /// Keeps at most `max_count` backups, never deletes backups newer than `min_age_days`.
    pub fn enforce_retention(
        &self,
        max_count: usize,
        min_age_days: u64,
    ) -> Result<u32, ProvisioningError> {
        let dirs = self.list_backups()?;
        if dirs.len() <= max_count {
            return Ok(0);
        }

        let min_age = chrono::Duration::days(min_age_days as i64);
        let cutoff = chrono::Utc::now() - min_age;
        let mut removed = 0u32;

        // dirs are sorted newest first, skip the first `max_count`
        for dir in dirs.iter().skip(max_count) {
            // Parse timestamp from directory name
            let dir_name = dir
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            // Convert dir name format (2026-03-01T14-30-00Z) to RFC3339
            let rfc3339 = dir_name
                .replacen('T', "T", 1)
                .replace("-", ":")
                .replacen(":", "-", 2); // only restore first two : back to -

            // Simple check: if we can't parse, skip (don't delete unknown dirs)
            if let Ok(timestamp) = chrono::DateTime::parse_from_rfc3339(&format!("{}+00:00", rfc3339.trim_end_matches('Z')))
            {
                if timestamp < cutoff {
                    if std::fs::remove_dir_all(dir).is_ok() {
                        removed += 1;
                    }
                }
            }
        }

        Ok(removed)
    }
}

/// Compute SHA-256 hex digest of bytes.
pub fn sha256_hex(data: &[u8]) -> String {
    let hash = Sha256::digest(data);
    format!("{:x}", hash)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // ── sha256_hex ──

    #[test]
    fn sha256_hex_known_vector() {
        // NIST / well-known test vector for "hello"
        let expected = "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824";
        assert_eq!(sha256_hex(b"hello"), expected);
    }

    #[test]
    fn sha256_hex_empty_input() {
        let expected = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        assert_eq!(sha256_hex(b""), expected);
    }

    #[test]
    fn sha256_hex_deterministic() {
        let a = sha256_hex(b"deterministic");
        let b = sha256_hex(b"deterministic");
        assert_eq!(a, b);
    }

    // ── BackupManager::new ──

    #[test]
    fn new_sets_backups_dir() {
        let tmp = TempDir::new().unwrap();
        let mgr = BackupManager::new(tmp.path());
        assert_eq!(mgr.backups_dir, tmp.path().join("backups"));
    }

    #[test]
    fn new_does_not_create_dir() {
        let tmp = TempDir::new().unwrap();
        let _mgr = BackupManager::new(tmp.path());
        assert!(!tmp.path().join("backups").exists());
    }

    // ── create_backup_dir ──

    #[test]
    fn create_backup_dir_creates_timestamped_dir() {
        let tmp = TempDir::new().unwrap();
        let mgr = BackupManager::new(tmp.path());
        let dir = mgr.create_backup_dir().unwrap();
        assert!(dir.exists());
        assert!(dir.is_dir());
        // The dir should be inside backups/
        assert_eq!(dir.parent().unwrap(), tmp.path().join("backups"));
        // Name should look like a timestamp: YYYY-MM-DDTHH-MM-SSZ
        let name = dir.file_name().unwrap().to_string_lossy();
        assert!(name.contains('T'));
        assert!(name.ends_with('Z'));
    }

    #[test]
    fn create_backup_dir_two_rapid_calls_distinct() {
        let tmp = TempDir::new().unwrap();
        let mgr = BackupManager::new(tmp.path());
        let d1 = mgr.create_backup_dir().unwrap();
        // If timestamps collide (same second), the second call should still succeed
        // because create_dir_all is idempotent. We at least verify no error.
        let d2 = mgr.create_backup_dir().unwrap();
        // Both exist
        assert!(d1.exists());
        assert!(d2.exists());
        // Note: they may be identical if called within the same second, which is fine --
        // create_dir_all succeeds even if dir exists.
    }

    // ── backup_file ──

    #[test]
    fn backup_file_copies_content_and_returns_hash() {
        let tmp = TempDir::new().unwrap();
        let mgr = BackupManager::new(tmp.path());
        let backup_dir = mgr.create_backup_dir().unwrap();

        // Create an original file
        let original = tmp.path().join("original.json");
        std::fs::write(&original, b"original content").unwrap();

        let (rel_path, hash) = mgr
            .backup_file(&backup_dir, "cursor", &original)
            .unwrap();

        // Relative path should be tool_slug/filename.bak
        assert_eq!(rel_path, PathBuf::from("cursor/original.json.bak"));

        // Hash should match sha256 of content
        assert_eq!(hash, sha256_hex(b"original content"));

        // Backup file content should match original
        let backup_full = backup_dir.join(&rel_path);
        assert_eq!(
            std::fs::read(&backup_full).unwrap(),
            b"original content"
        );
    }

    #[test]
    fn backup_file_missing_original_returns_error() {
        let tmp = TempDir::new().unwrap();
        let mgr = BackupManager::new(tmp.path());
        let backup_dir = mgr.create_backup_dir().unwrap();

        let nonexistent = tmp.path().join("does_not_exist.json");
        let result = mgr.backup_file(&backup_dir, "cursor", &nonexistent);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Failed to read original"));
    }

    // ── write_manifest / read_manifest ──

    fn sample_manifest() -> BackupManifest {
        BackupManifest {
            version: 1,
            timestamp: "2026-03-01T00:00:00Z".into(),
            tally_version: "0.1.0".into(),
            operation: BackupOperation::Provision,
            machine_id: "test-machine".into(),
            tools_modified: vec![BackupToolEntry {
                tool: ToolId::Cursor,
                files: vec![],
            }],
        }
    }

    #[test]
    fn write_and_read_manifest_round_trip() {
        let tmp = TempDir::new().unwrap();
        let mgr = BackupManager::new(tmp.path());
        let backup_dir = mgr.create_backup_dir().unwrap();

        let manifest = sample_manifest();
        mgr.write_manifest(&backup_dir, &manifest).unwrap();

        let loaded = mgr.read_manifest(&backup_dir).unwrap();
        assert_eq!(loaded.version, manifest.version);
        assert_eq!(loaded.timestamp, manifest.timestamp);
        assert_eq!(loaded.tally_version, manifest.tally_version);
        assert_eq!(loaded.machine_id, manifest.machine_id);
        assert_eq!(loaded.tools_modified.len(), 1);
        assert_eq!(loaded.tools_modified[0].tool, ToolId::Cursor);
    }

    #[test]
    fn read_manifest_missing_file_returns_error() {
        let tmp = TempDir::new().unwrap();
        let mgr = BackupManager::new(tmp.path());
        let result = mgr.read_manifest(tmp.path());
        assert!(result.is_err());
    }

    // ── get_backup_path ──

    #[test]
    fn get_backup_path_joins_correctly() {
        let tmp = TempDir::new().unwrap();
        let mgr = BackupManager::new(tmp.path());
        let backup_dir = PathBuf::from("/backups/2026-03-01T00-00-00Z");
        let rel = PathBuf::from("cursor/settings.json.bak");
        let result = mgr.get_backup_path(&backup_dir, &rel);
        assert_eq!(
            result,
            PathBuf::from("/backups/2026-03-01T00-00-00Z/cursor/settings.json.bak")
        );
    }

    #[test]
    fn get_backup_path_with_nested_relative() {
        let tmp = TempDir::new().unwrap();
        let mgr = BackupManager::new(tmp.path());
        let backup_dir = tmp.path().join("backups").join("snapshot");
        let rel = PathBuf::from("claude-code/mcp.json.bak");
        assert_eq!(
            mgr.get_backup_path(&backup_dir, &rel),
            backup_dir.join("claude-code/mcp.json.bak")
        );
    }

    // ── list_backups ──

    #[test]
    fn list_backups_empty_dir_returns_empty() {
        let tmp = TempDir::new().unwrap();
        let mgr = BackupManager::new(tmp.path());
        // backups dir doesn't even exist yet
        let result = mgr.list_backups().unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn list_backups_sorted_newest_first() {
        let tmp = TempDir::new().unwrap();
        let mgr = BackupManager::new(tmp.path());
        let backups_dir = tmp.path().join("backups");
        std::fs::create_dir_all(&backups_dir).unwrap();

        // Create dirs with timestamp names (older first)
        let names = vec![
            "2026-01-01T00-00-00Z",
            "2026-03-01T00-00-00Z",
            "2026-02-01T00-00-00Z",
        ];
        for name in &names {
            std::fs::create_dir_all(backups_dir.join(name)).unwrap();
        }

        let result = mgr.list_backups().unwrap();
        assert_eq!(result.len(), 3);
        // Should be sorted newest first (descending string order)
        assert!(result[0].file_name().unwrap().to_string_lossy() == "2026-03-01T00-00-00Z");
        assert!(result[1].file_name().unwrap().to_string_lossy() == "2026-02-01T00-00-00Z");
        assert!(result[2].file_name().unwrap().to_string_lossy() == "2026-01-01T00-00-00Z");
    }

    #[test]
    fn list_backups_excludes_files() {
        let tmp = TempDir::new().unwrap();
        let mgr = BackupManager::new(tmp.path());
        let backups_dir = tmp.path().join("backups");
        std::fs::create_dir_all(&backups_dir).unwrap();

        // Create one dir and one file
        std::fs::create_dir_all(backups_dir.join("2026-01-01T00-00-00Z")).unwrap();
        std::fs::write(backups_dir.join("stray-file.txt"), "oops").unwrap();

        let result = mgr.list_backups().unwrap();
        assert_eq!(result.len(), 1);
    }

    // ── find_latest_backup_for_tool ──

    fn create_backup_with_manifest(
        mgr: &BackupManager,
        dir_name: &str,
        tool: ToolId,
    ) -> PathBuf {
        let backups_dir = mgr.backups_dir.clone();
        let dir = backups_dir.join(dir_name);
        std::fs::create_dir_all(&dir).unwrap();

        let manifest = BackupManifest {
            version: 1,
            timestamp: dir_name.to_string(),
            tally_version: "0.1.0".into(),
            operation: BackupOperation::Provision,
            machine_id: "test".into(),
            tools_modified: vec![BackupToolEntry {
                tool,
                files: vec![],
            }],
        };
        mgr.write_manifest(&dir, &manifest).unwrap();
        dir
    }

    #[test]
    fn find_latest_backup_for_tool_returns_matching() {
        let tmp = TempDir::new().unwrap();
        let mgr = BackupManager::new(tmp.path());

        create_backup_with_manifest(&mgr, "2026-01-01T00-00-00Z", ToolId::Cursor);

        let result = mgr.find_latest_backup_for_tool(ToolId::Cursor).unwrap();
        assert!(result.is_some());
        let (_, manifest) = result.unwrap();
        assert_eq!(manifest.tools_modified[0].tool, ToolId::Cursor);
    }

    #[test]
    fn find_latest_backup_for_tool_returns_none_when_no_match() {
        let tmp = TempDir::new().unwrap();
        let mgr = BackupManager::new(tmp.path());

        create_backup_with_manifest(&mgr, "2026-01-01T00-00-00Z", ToolId::Cursor);

        let result = mgr.find_latest_backup_for_tool(ToolId::Aider).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn find_latest_backup_for_tool_returns_latest_when_multiple() {
        let tmp = TempDir::new().unwrap();
        let mgr = BackupManager::new(tmp.path());

        create_backup_with_manifest(&mgr, "2026-01-01T00-00-00Z", ToolId::Cursor);
        create_backup_with_manifest(&mgr, "2026-03-01T00-00-00Z", ToolId::Cursor);
        create_backup_with_manifest(&mgr, "2026-02-01T00-00-00Z", ToolId::Cursor);

        let result = mgr.find_latest_backup_for_tool(ToolId::Cursor).unwrap();
        let (dir, manifest) = result.unwrap();
        // list_backups sorts newest first, so the first match should be the newest
        assert!(dir.to_string_lossy().contains("2026-03-01"));
        assert_eq!(manifest.timestamp, "2026-03-01T00-00-00Z");
    }

    // ── enforce_retention ──

    #[test]
    fn enforce_retention_does_nothing_when_count_le_max() {
        let tmp = TempDir::new().unwrap();
        let mgr = BackupManager::new(tmp.path());

        create_backup_with_manifest(&mgr, "2026-01-01T00-00-00Z", ToolId::Cursor);
        create_backup_with_manifest(&mgr, "2026-02-01T00-00-00Z", ToolId::Cursor);

        let removed = mgr.enforce_retention(5, 0).unwrap();
        assert_eq!(removed, 0);
        assert_eq!(mgr.list_backups().unwrap().len(), 2);
    }

    #[test]
    fn enforce_retention_deletes_oldest_beyond_max() {
        let tmp = TempDir::new().unwrap();
        let mgr = BackupManager::new(tmp.path());

        // Create 4 backups with old timestamps (well past min_age_days=0)
        create_backup_with_manifest(&mgr, "2024-01-01T00-00-00Z", ToolId::Cursor);
        create_backup_with_manifest(&mgr, "2024-02-01T00-00-00Z", ToolId::Cursor);
        create_backup_with_manifest(&mgr, "2024-03-01T00-00-00Z", ToolId::Cursor);
        create_backup_with_manifest(&mgr, "2024-04-01T00-00-00Z", ToolId::Cursor);

        // Keep at most 2, min_age 0 days (delete anything older than now)
        let removed = mgr.enforce_retention(2, 0).unwrap();
        assert_eq!(removed, 2);
        let remaining = mgr.list_backups().unwrap();
        assert_eq!(remaining.len(), 2);
        // The two newest should remain
        let names: Vec<String> = remaining
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        assert!(names.contains(&"2024-04-01T00-00-00Z".to_string()));
        assert!(names.contains(&"2024-03-01T00-00-00Z".to_string()));
    }

    #[test]
    fn enforce_retention_respects_min_age() {
        let tmp = TempDir::new().unwrap();
        let mgr = BackupManager::new(tmp.path());

        // Create backups: one very old, rest recent (use future dates to ensure they're "new")
        create_backup_with_manifest(&mgr, "2020-01-01T00-00-00Z", ToolId::Cursor);
        create_backup_with_manifest(&mgr, "2026-02-28T00-00-00Z", ToolId::Cursor);
        create_backup_with_manifest(&mgr, "2026-03-01T00-00-00Z", ToolId::Cursor);

        // max 1, but min_age 9999 days -- nothing is older than that
        let removed = mgr.enforce_retention(1, 9999).unwrap();
        // The old one (2020) might be > 9999 days old, but 2026 ones won't be
        // Only dirs beyond max_count AND older than min_age get deleted
        // 2020-01-01 is ~2251 days ago (as of 2026-03-01), so with min_age=9999 it stays
        assert_eq!(removed, 0);
    }
}
