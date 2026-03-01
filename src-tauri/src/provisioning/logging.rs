use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use chrono::Local;

use crate::provisioning::error::ProvisioningError;
use crate::provisioning::types::ToolId;

/// Append-only log for provisioning operations.
pub struct ProvisioningLogger {
    log_path: PathBuf,
}

impl ProvisioningLogger {
    pub fn new(tally_dir: &Path) -> Result<Self, ProvisioningError> {
        fs::create_dir_all(tally_dir)?;
        Ok(Self {
            log_path: tally_dir.join("provisioning.log"),
        })
    }

    pub fn log_provision(&self, tool: ToolId, file_path: &Path, detail: &str) {
        self.append(&format!(
            "[PROVISION] {}: {} ({})",
            tool.display_name(),
            detail,
            file_path.display()
        ));
    }

    pub fn log_unprovision(&self, tool: ToolId, detail: &str) {
        self.append(&format!(
            "[UNPROVISION] {}: {}",
            tool.display_name(),
            detail
        ));
    }

    pub fn log_verify(&self, tool: ToolId, detail: &str) {
        self.append(&format!(
            "[VERIFY] {}: {}",
            tool.display_name(),
            detail
        ));
    }

    pub fn log_skip(&self, tool: ToolId, reason: &str) {
        self.append(&format!(
            "[SKIP] {}: {}",
            tool.display_name(),
            reason
        ));
    }

    pub fn log_error(&self, tool: ToolId, error: &str) {
        self.append(&format!(
            "[ERROR] {}: {}",
            tool.display_name(),
            error
        ));
    }

    pub fn log_detect(&self, tool: ToolId, detected: bool, methods: &[String]) {
        if detected {
            self.append(&format!(
                "[DETECT] {}: found via {}",
                tool.display_name(),
                methods.join(", ")
            ));
        } else {
            self.append(&format!(
                "[DETECT] {}: not found",
                tool.display_name()
            ));
        }
    }

    pub fn log_backup(&self, tool: ToolId, backup_dir: &Path) {
        self.append(&format!(
            "[BACKUP] {}: backed up to {}",
            tool.display_name(),
            backup_dir.display()
        ));
    }

    pub fn log_notify(&self, message: &str) {
        self.append(&format!("[NOTIFY] {}", message));
    }

    fn append(&self, message: &str) {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
        let line = format!("{} {}\n", timestamp, message);

        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)
        {
            let _ = file.write_all(line.as_bytes());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_logger() -> (TempDir, ProvisioningLogger) {
        let tmp = TempDir::new().expect("failed to create temp dir");
        let logger = ProvisioningLogger::new(tmp.path()).expect("failed to create logger");
        (tmp, logger)
    }

    fn read_log(tmp: &TempDir) -> String {
        fs::read_to_string(tmp.path().join("provisioning.log")).unwrap_or_default()
    }

    // ── new ──

    #[test]
    fn new_creates_log_path_under_tally_dir() {
        let (tmp, logger) = make_logger();
        assert_eq!(logger.log_path, tmp.path().join("provisioning.log"));
    }

    #[test]
    fn new_creates_directory_if_missing() {
        let tmp = TempDir::new().unwrap();
        let nested = tmp.path().join("deep").join("nested");
        let logger = ProvisioningLogger::new(&nested).expect("should create nested dirs");
        assert!(nested.exists());
        assert_eq!(logger.log_path, nested.join("provisioning.log"));
    }

    // ── log_provision ──

    #[test]
    fn log_provision_writes_provision_prefix_and_tool_name() {
        let (tmp, logger) = make_logger();
        logger.log_provision(ToolId::ClaudeCode, Path::new("/tmp/config.json"), "injected MCP");
        let contents = read_log(&tmp);
        assert!(contents.contains("[PROVISION] Claude Code: injected MCP (/tmp/config.json)"));
    }

    #[test]
    fn log_provision_includes_file_path() {
        let (tmp, logger) = make_logger();
        let p = Path::new("/home/user/.cursor/mcp.json");
        logger.log_provision(ToolId::Cursor, p, "wrote config");
        let contents = read_log(&tmp);
        assert!(contents.contains("/home/user/.cursor/mcp.json"));
    }

    // ── log_unprovision ──

    #[test]
    fn log_unprovision_writes_unprovision_prefix() {
        let (tmp, logger) = make_logger();
        logger.log_unprovision(ToolId::Windsurf, "removed MCP block");
        let contents = read_log(&tmp);
        assert!(contents.contains("[UNPROVISION] Windsurf: removed MCP block"));
    }

    #[test]
    fn log_unprovision_uses_correct_tool_display_name() {
        let (tmp, logger) = make_logger();
        logger.log_unprovision(ToolId::ContinueDev, "cleaned up");
        let contents = read_log(&tmp);
        assert!(contents.contains("Continue.dev"));
    }

    // ── log_verify ──

    #[test]
    fn log_verify_writes_verify_prefix() {
        let (tmp, logger) = make_logger();
        logger.log_verify(ToolId::Cline, "config intact");
        let contents = read_log(&tmp);
        assert!(contents.contains("[VERIFY] Cline: config intact"));
    }

    #[test]
    fn log_verify_with_different_tool() {
        let (tmp, logger) = make_logger();
        logger.log_verify(ToolId::Copilot, "version mismatch");
        let contents = read_log(&tmp);
        assert!(contents.contains("[VERIFY] GitHub Copilot: version mismatch"));
    }

    // ── log_skip ──

    #[test]
    fn log_skip_writes_skip_prefix() {
        let (tmp, logger) = make_logger();
        logger.log_skip(ToolId::Aider, "excluded by user");
        let contents = read_log(&tmp);
        assert!(contents.contains("[SKIP] Aider: excluded by user"));
    }

    #[test]
    fn log_skip_with_different_reason() {
        let (tmp, logger) = make_logger();
        logger.log_skip(ToolId::Codex, "not detected");
        let contents = read_log(&tmp);
        assert!(contents.contains("[SKIP] Codex CLI: not detected"));
    }

    // ── log_error ──

    #[test]
    fn log_error_writes_error_prefix() {
        let (tmp, logger) = make_logger();
        logger.log_error(ToolId::ClaudeDesktop, "permission denied");
        let contents = read_log(&tmp);
        assert!(contents.contains("[ERROR] Claude Desktop: permission denied"));
    }

    #[test]
    fn log_error_with_different_tool() {
        let (tmp, logger) = make_logger();
        logger.log_error(ToolId::Cursor, "file locked");
        let contents = read_log(&tmp);
        assert!(contents.contains("[ERROR] Cursor: file locked"));
    }

    // ── log_detect ──

    #[test]
    fn log_detect_found_with_methods() {
        let (tmp, logger) = make_logger();
        logger.log_detect(ToolId::ClaudeCode, true, &["binary".to_string(), "config".to_string()]);
        let contents = read_log(&tmp);
        assert!(contents.contains("[DETECT] Claude Code: found via binary, config"));
    }

    #[test]
    fn log_detect_not_found() {
        let (tmp, logger) = make_logger();
        logger.log_detect(ToolId::Windsurf, false, &[]);
        let contents = read_log(&tmp);
        assert!(contents.contains("[DETECT] Windsurf: not found"));
    }

    // ── log_backup ──

    #[test]
    fn log_backup_writes_backup_prefix_and_path() {
        let (tmp, logger) = make_logger();
        let backup_dir = Path::new("/home/user/.tally/backups/20260301_120000");
        logger.log_backup(ToolId::Cline, backup_dir);
        let contents = read_log(&tmp);
        assert!(contents.contains("[BACKUP] Cline: backed up to /home/user/.tally/backups/20260301_120000"));
    }

    #[test]
    fn log_backup_with_different_tool() {
        let (tmp, logger) = make_logger();
        logger.log_backup(ToolId::Aider, Path::new("/tmp/backup"));
        let contents = read_log(&tmp);
        assert!(contents.contains("[BACKUP] Aider: backed up to /tmp/backup"));
    }

    // ── log_notify ──

    #[test]
    fn log_notify_writes_notify_prefix() {
        let (tmp, logger) = make_logger();
        logger.log_notify("Provisioning complete for 3 tools");
        let contents = read_log(&tmp);
        assert!(contents.contains("[NOTIFY] Provisioning complete for 3 tools"));
    }

    #[test]
    fn log_notify_with_different_message() {
        let (tmp, logger) = make_logger();
        logger.log_notify("Included Cursor for provisioning");
        let contents = read_log(&tmp);
        assert!(contents.contains("[NOTIFY] Included Cursor for provisioning"));
    }

    // ── timestamp ──

    #[test]
    fn all_log_lines_start_with_timestamp() {
        let (tmp, logger) = make_logger();
        logger.log_provision(ToolId::ClaudeCode, Path::new("/x"), "test");
        logger.log_error(ToolId::Cursor, "boom");
        logger.log_notify("hello");

        let contents = read_log(&tmp);
        for line in contents.lines() {
            // Timestamp format: YYYY-MM-DD HH:MM:SS
            assert!(
                line.len() >= 19,
                "line too short to contain timestamp: {line}"
            );
            let ts = &line[..19];
            assert!(
                ts.chars().nth(4) == Some('-')
                    && ts.chars().nth(7) == Some('-')
                    && ts.chars().nth(10) == Some(' ')
                    && ts.chars().nth(13) == Some(':')
                    && ts.chars().nth(16) == Some(':'),
                "bad timestamp format: {ts}"
            );
        }
    }

    // ── append behaviour ──

    #[test]
    fn multiple_calls_append_not_overwrite() {
        let (tmp, logger) = make_logger();
        logger.log_notify("first");
        logger.log_notify("second");
        logger.log_notify("third");

        let contents = read_log(&tmp);
        let lines: Vec<&str> = contents.lines().collect();
        assert_eq!(lines.len(), 3, "expected 3 lines, got {}", lines.len());
        assert!(lines[0].contains("[NOTIFY] first"));
        assert!(lines[1].contains("[NOTIFY] second"));
        assert!(lines[2].contains("[NOTIFY] third"));
    }

    #[test]
    fn mixed_log_types_all_appended() {
        let (tmp, logger) = make_logger();
        logger.log_provision(ToolId::ClaudeCode, Path::new("/a"), "detail");
        logger.log_error(ToolId::Cursor, "fail");
        logger.log_skip(ToolId::Aider, "excluded");
        logger.log_detect(ToolId::Windsurf, true, &["binary".to_string()]);

        let contents = read_log(&tmp);
        let lines: Vec<&str> = contents.lines().collect();
        assert_eq!(lines.len(), 4);
        assert!(lines[0].contains("[PROVISION]"));
        assert!(lines[1].contains("[ERROR]"));
        assert!(lines[2].contains("[SKIP]"));
        assert!(lines[3].contains("[DETECT]"));
    }
}
