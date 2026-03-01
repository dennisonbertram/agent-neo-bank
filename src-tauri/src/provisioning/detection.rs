use std::path::{Path, PathBuf};
use std::process::Command;

use crate::provisioning::platform::PlatformPaths;
use crate::provisioning::types::*;

/// Check if a binary exists in PATH or common version manager locations.
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

    // 2. Login shell lookup (catches nvm, asdf, mise, volta, etc.)
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

    // 3. Check known version manager shim directories
    if let Some(paths) = PlatformPaths::new() {
        for dir in paths.version_manager_paths() {
            let candidate = dir.join(name);
            if candidate.exists() {
                return Some(candidate);
            }
        }

        // 4. For nvm: check all installed Node versions
        let nvm_dir = paths.home_dir().join(".nvm").join("versions").join("node");
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
    }

    None
}

/// Check if a directory exists.
pub fn dir_exists(path: &Path) -> bool {
    path.is_dir()
}

/// Check for VS Code extension by prefix.
pub fn find_vscode_extension(ext_dir: &Path, prefix: &str) -> Option<PathBuf> {
    if !ext_dir.is_dir() {
        return None;
    }
    std::fs::read_dir(ext_dir).ok()?.flatten().find_map(|entry| {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with(prefix) {
            Some(entry.path())
        } else {
            None
        }
    })
}

/// Check for macOS app bundles.
pub fn find_app_bundle(candidates: &[PathBuf]) -> Option<PathBuf> {
    candidates.iter().find(|p| p.is_dir()).cloned()
}

/// Try to get tool version by running `tool --version`.
pub fn get_tool_version(binary_path: &Path) -> Option<String> {
    let output = Command::new(binary_path)
        .arg("--version")
        .output()
        .ok()?;

    if output.status.success() {
        let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !version.is_empty() {
            return Some(version);
        }
    }

    None
}

/// Run full detection for a single tool.
pub fn detect_tool(tool: ToolId, paths: &PlatformPaths) -> DetectionResult {
    let mut methods = Vec::new();
    let mut config_paths = Vec::new();
    let mut version = None;

    // Check 1: Config directory
    if let Some(dir) = paths.config_dir(tool) {
        if dir_exists(&dir) {
            methods.push(DetectionMethod::ConfigDirectory);
        }
    }

    // Check 2: Config file (MCP config path)
    if let Some(config_path) = paths.mcp_config_path(tool) {
        let resolved = config_path
            .canonicalize()
            .unwrap_or_else(|_| config_path.clone());
        let is_symlink = config_path.is_symlink();
        let exists = config_path.exists();
        let writable = exists
            && std::fs::metadata(&config_path)
                .map(|m| !m.permissions().readonly())
                .unwrap_or(false);

        if exists {
            methods.push(DetectionMethod::ConfigFile);
        }

        let format = match tool {
            ToolId::Codex => ConfigFormat::Toml,
            ToolId::ContinueDev => ConfigFormat::Yaml,
            ToolId::Aider => ConfigFormat::Yaml,
            ToolId::Copilot => ConfigFormat::JsonWithServersKey,
            _ => ConfigFormat::Json,
        };

        config_paths.push(ConfigFileInfo {
            path: config_path,
            resolved_path: resolved,
            exists,
            writable,
            format,
            purpose: ConfigPurpose::McpServer,
            is_symlink,
        });
    }

    // Check 3: Skill/instruction file
    if let Some(skill_path) = paths.skill_path(tool) {
        let resolved = skill_path
            .canonicalize()
            .unwrap_or_else(|_| skill_path.clone());
        let is_symlink = skill_path.is_symlink();
        let exists = skill_path.exists();
        let writable = if exists {
            std::fs::metadata(&skill_path)
                .map(|m| !m.permissions().readonly())
                .unwrap_or(false)
        } else {
            // Check if parent dir is writable
            skill_path
                .parent()
                .map(|p| p.exists())
                .unwrap_or(false)
        };

        let (format, purpose) = match tool {
            ToolId::ClaudeCode => (ConfigFormat::StandaloneFile, ConfigPurpose::Skill),
            ToolId::Cursor => (ConfigFormat::MarkdownWithFrontmatter, ConfigPurpose::Skill),
            ToolId::Codex | ToolId::Copilot => (ConfigFormat::Markdown, ConfigPurpose::SystemInstructions),
            ToolId::Aider => (ConfigFormat::Yaml, ConfigPurpose::ConventionFile),
            _ => (ConfigFormat::StandaloneFile, ConfigPurpose::Skill),
        };

        config_paths.push(ConfigFileInfo {
            path: skill_path,
            resolved_path: resolved,
            exists,
            writable,
            format,
            purpose,
            is_symlink,
        });
    }

    // Check 4: Binary in PATH
    for binary_name in paths.binary_names(tool) {
        if let Some(bin_path) = find_binary(binary_name) {
            methods.push(DetectionMethod::BinaryInPath);
            // Try to get version
            if version.is_none() {
                version = get_tool_version(&bin_path);
            }
            break;
        }
    }

    // Check 5: App bundle (macOS)
    let bundles = paths.app_bundle_paths(tool);
    if find_app_bundle(&bundles).is_some() {
        methods.push(DetectionMethod::ApplicationBundle);
    }

    // Check 6: VS Code extension
    for (ext_dir, prefix) in paths.vscode_extension_patterns(tool) {
        if find_vscode_extension(&ext_dir, prefix).is_some() {
            methods.push(DetectionMethod::VsCodeExtension);
            break;
        }
    }

    let detected = !methods.is_empty();

    DetectionResult {
        tool,
        detected,
        methods,
        version,
        config_paths,
    }
}

/// Run detection for all known tools.
pub fn detect_all_tools(paths: &PlatformPaths) -> Vec<DetectionResult> {
    ToolId::all()
        .iter()
        .map(|&tool| detect_tool(tool, paths))
        .collect()
}

/// Detection cache persisted to ~/.tally/detected-tools.json.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DetectionCache {
    pub timestamp: String,
    pub ttl_seconds: u64,
    pub results: Vec<DetectionResult>,
}

impl DetectionCache {
    pub fn is_fresh(&self) -> bool {
        if let Ok(cached_time) = chrono::DateTime::parse_from_rfc3339(&self.timestamp) {
            let age = chrono::Utc::now().signed_duration_since(cached_time);
            age.num_seconds() < self.ttl_seconds as i64
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_paths() -> (TempDir, PlatformPaths) {
        let tmp = TempDir::new().unwrap();
        let paths = PlatformPaths::with_home(tmp.path().to_path_buf());
        (tmp, paths)
    }

    // ── dir_exists ──

    #[test]
    fn dir_exists_returns_true_for_existing_dir() {
        let tmp = TempDir::new().unwrap();
        let sub = tmp.path().join("subdir");
        std::fs::create_dir(&sub).unwrap();
        assert!(dir_exists(&sub));
    }

    #[test]
    fn dir_exists_returns_false_for_nonexistent() {
        let tmp = TempDir::new().unwrap();
        assert!(!dir_exists(&tmp.path().join("nope")));
    }

    #[test]
    fn dir_exists_returns_false_for_file() {
        let tmp = TempDir::new().unwrap();
        let file = tmp.path().join("afile.txt");
        std::fs::write(&file, "hello").unwrap();
        assert!(!dir_exists(&file));
    }

    // ── find_vscode_extension ──

    #[test]
    fn find_vscode_extension_found_when_matching_prefix() {
        let tmp = TempDir::new().unwrap();
        let ext_dir = tmp.path().join("extensions");
        std::fs::create_dir(&ext_dir).unwrap();
        std::fs::create_dir(ext_dir.join("saoudrizwan.claude-dev-3.2.1")).unwrap();
        std::fs::create_dir(ext_dir.join("other.extension-1.0.0")).unwrap();

        let result = find_vscode_extension(&ext_dir, "saoudrizwan.claude-dev-");
        assert!(result.is_some());
        assert!(result.unwrap().ends_with("saoudrizwan.claude-dev-3.2.1"));
    }

    #[test]
    fn find_vscode_extension_none_when_dir_absent() {
        let tmp = TempDir::new().unwrap();
        let missing = tmp.path().join("nonexistent");
        assert!(find_vscode_extension(&missing, "anything").is_none());
    }

    #[test]
    fn find_vscode_extension_none_when_no_match() {
        let tmp = TempDir::new().unwrap();
        let ext_dir = tmp.path().join("extensions");
        std::fs::create_dir(&ext_dir).unwrap();
        std::fs::create_dir(ext_dir.join("unrelated.ext-1.0.0")).unwrap();

        assert!(find_vscode_extension(&ext_dir, "saoudrizwan.claude-dev-").is_none());
    }

    // ── find_app_bundle ──

    #[test]
    fn find_app_bundle_returns_first_existing() {
        let tmp = TempDir::new().unwrap();
        let existing = tmp.path().join("App.app");
        std::fs::create_dir(&existing).unwrap();

        let missing = tmp.path().join("Missing.app");
        let candidates = vec![missing, existing.clone()];
        assert_eq!(find_app_bundle(&candidates).unwrap(), existing);
    }

    #[test]
    fn find_app_bundle_none_when_all_absent() {
        let tmp = TempDir::new().unwrap();
        let candidates = vec![
            tmp.path().join("Nope1.app"),
            tmp.path().join("Nope2.app"),
        ];
        assert!(find_app_bundle(&candidates).is_none());
    }

    // ── detect_tool ──

    #[test]
    fn detect_tool_empty_home_no_config_detection() {
        let (_tmp, paths) = make_paths();
        let result = detect_tool(ToolId::ClaudeCode, &paths);
        assert_eq!(result.tool, ToolId::ClaudeCode);
        // Config-based methods should not fire on empty home
        assert!(!result.methods.iter().any(|m| matches!(
            m,
            DetectionMethod::ConfigDirectory | DetectionMethod::ConfigFile
        )));
        // Binary detection may fire on real systems where `claude` is in PATH — that's fine
    }

    #[test]
    fn detect_tool_with_config_dir_detected() {
        let (_tmp, paths) = make_paths();
        // ClaudeCode config_dir is ~/.claude
        let config_dir = _tmp.path().join(".claude");
        std::fs::create_dir_all(&config_dir).unwrap();

        let result = detect_tool(ToolId::ClaudeCode, &paths);
        assert!(result.detected);
        assert!(result.methods.iter().any(|m| matches!(m, DetectionMethod::ConfigDirectory)));
    }

    #[test]
    fn detect_tool_with_config_file_detected() {
        let (_tmp, paths) = make_paths();
        // ClaudeCode mcp_config_path is ~/.claude.json
        let config_file = _tmp.path().join(".claude.json");
        std::fs::write(&config_file, "{}").unwrap();

        let result = detect_tool(ToolId::ClaudeCode, &paths);
        assert!(result.detected);
        assert!(result.methods.iter().any(|m| matches!(m, DetectionMethod::ConfigFile)));
    }

    #[test]
    fn detect_tool_cursor_config_dir() {
        let (_tmp, paths) = make_paths();
        std::fs::create_dir_all(_tmp.path().join(".cursor")).unwrap();

        let result = detect_tool(ToolId::Cursor, &paths);
        assert!(result.detected);
        assert!(result.methods.iter().any(|m| matches!(m, DetectionMethod::ConfigDirectory)));
    }

    #[test]
    fn detect_tool_cline_via_vscode_extension() {
        let (_tmp, paths) = make_paths();
        // Cline is detected via VS Code extension
        let ext_dir = _tmp.path().join(".vscode").join("extensions");
        std::fs::create_dir_all(&ext_dir).unwrap();
        std::fs::create_dir(ext_dir.join("saoudrizwan.claude-dev-3.0.0")).unwrap();

        let result = detect_tool(ToolId::Cline, &paths);
        assert!(result.detected);
        assert!(result.methods.iter().any(|m| matches!(m, DetectionMethod::VsCodeExtension)));
    }

    #[test]
    fn detect_tool_aider_not_detected_in_empty_home() {
        // Aider has no config_dir, detected only via binary — should not be detected
        let (_tmp, paths) = make_paths();
        let result = detect_tool(ToolId::Aider, &paths);
        assert!(!result.detected);
    }

    #[test]
    fn detect_tool_config_paths_populated_even_when_not_detected() {
        let (_tmp, paths) = make_paths();
        // ClaudeCode has mcp_config_path and skill_path — they should appear in
        // config_paths even if the files don't exist
        let result = detect_tool(ToolId::ClaudeCode, &paths);
        // Should have entries for mcp config + skill path
        assert!(result.config_paths.len() >= 2);
        // But they should show exists=false
        assert!(result.config_paths.iter().all(|c| !c.exists));
    }

    // ── detect_all_tools ──

    #[test]
    fn detect_all_tools_returns_nine_results() {
        let (_tmp, paths) = make_paths();
        let results = detect_all_tools(&paths);
        assert_eq!(results.len(), 9);
    }

    #[test]
    fn detect_all_tools_no_config_detections_on_empty_dir() {
        let (_tmp, paths) = make_paths();
        let results = detect_all_tools(&paths);
        for result in &results {
            // Config-based methods should not fire on empty home.
            // ApplicationBundle uses hard-coded /Applications which may exist on the real
            // host (e.g. /Applications/Claude.app), so we exclude it from this assertion.
            // Binary detection may also fire on real systems.
            let has_config_detection = result.methods.iter().any(|m| {
                matches!(
                    m,
                    DetectionMethod::ConfigDirectory
                        | DetectionMethod::ConfigFile
                        | DetectionMethod::VsCodeExtension
                )
            });
            assert!(
                !has_config_detection,
                "Tool {:?} should not have config-based detection in empty home",
                result.tool
            );
        }
    }

    // ── DetectionCache::is_fresh ──

    #[test]
    fn detection_cache_is_fresh_within_ttl() {
        let cache = DetectionCache {
            timestamp: chrono::Utc::now().to_rfc3339(),
            ttl_seconds: 3600,
            results: vec![],
        };
        assert!(cache.is_fresh());
    }

    #[test]
    fn detection_cache_expired_returns_false() {
        // Timestamp 2 hours ago, TTL 1 hour
        let old = chrono::Utc::now() - chrono::Duration::hours(2);
        let cache = DetectionCache {
            timestamp: old.to_rfc3339(),
            ttl_seconds: 3600,
            results: vec![],
        };
        assert!(!cache.is_fresh());
    }

    #[test]
    fn detection_cache_invalid_timestamp_returns_false() {
        let cache = DetectionCache {
            timestamp: "not-a-timestamp".to_string(),
            ttl_seconds: 3600,
            results: vec![],
        };
        assert!(!cache.is_fresh());
    }
}
