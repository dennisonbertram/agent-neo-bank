use std::path::{Path, PathBuf};

use crate::provisioning::types::ToolId;

/// Resolve all relevant paths for a given tool on the current platform.
pub struct PlatformPaths {
    home: PathBuf,
}

impl PlatformPaths {
    pub fn new() -> Option<Self> {
        dirs::home_dir().map(|home| Self { home })
    }

    /// Create with an explicit home directory (for testing).
    #[cfg(test)]
    pub fn with_home(home: PathBuf) -> Self {
        Self { home }
    }

    /// Create with an explicit home directory (for integration tests in other crates).
    pub fn with_home_dir(home: PathBuf) -> Self {
        Self { home }
    }

    /// The ~/.tally/ directory for state, backups, logs.
    pub fn tally_dir(&self) -> PathBuf {
        self.home.join(".tally")
    }

    pub fn home_dir(&self) -> &Path {
        &self.home
    }

    /// Config file paths for MCP server injection per tool.
    pub fn mcp_config_path(&self, tool: ToolId) -> Option<PathBuf> {
        match tool {
            ToolId::ClaudeCode => Some(self.home.join(".claude.json")),
            ToolId::ClaudeDesktop => Some(self.claude_desktop_config_path()),
            ToolId::Cursor => Some(self.home.join(".cursor").join("mcp.json")),
            ToolId::Windsurf => Some(
                self.home
                    .join(".codeium")
                    .join("windsurf")
                    .join("mcp_config.json"),
            ),
            ToolId::Codex => Some(self.home.join(".codex").join("config.toml")),
            ToolId::ContinueDev => Some(self.home.join(".continue").join("config.yaml")),
            ToolId::Cline => Some(self.cline_mcp_settings_path()),
            ToolId::Aider => None, // Aider has no MCP support
            ToolId::Copilot => Some(self.copilot_mcp_config_path()),
        }
    }

    /// Skill/instruction file paths per tool.
    pub fn skill_path(&self, tool: ToolId) -> Option<PathBuf> {
        match tool {
            ToolId::ClaudeCode => Some(
                self.home
                    .join(".claude")
                    .join("skills")
                    .join("tally-wallet")
                    .join("SKILL.md"),
            ),
            ToolId::ClaudeDesktop => None, // No instruction files
            ToolId::Cursor => Some(
                self.home
                    .join(".cursor")
                    .join("rules")
                    .join("tally-wallet.mdc"),
            ),
            ToolId::Windsurf => Some(
                self.home
                    .join(".windsurf")
                    .join("rules")
                    .join("tally-wallet.md"),
            ),
            ToolId::Codex => Some(self.home.join(".codex").join("AGENTS.md")),
            ToolId::ContinueDev => Some(
                self.home
                    .join(".continue")
                    .join("rules")
                    .join("tally-wallet.md"),
            ),
            ToolId::Cline => Some(
                self.home
                    .join(".clinerules")
                    .join("tally-wallet.md"),
            ),
            ToolId::Aider => Some(self.home.join(".aider.conf.yml")),
            ToolId::Copilot => Some(
                self.home
                    .join(".github")
                    .join("copilot-instructions.md"),
            ),
        }
    }

    /// Detection directories — their existence indicates the tool is/was used.
    pub fn config_dir(&self, tool: ToolId) -> Option<PathBuf> {
        match tool {
            ToolId::ClaudeCode => Some(self.home.join(".claude")),
            ToolId::ClaudeDesktop => Some(self.claude_desktop_support_dir()),
            ToolId::Cursor => Some(self.home.join(".cursor")),
            ToolId::Windsurf => Some(self.home.join(".codeium")),
            ToolId::Codex => Some(self.home.join(".codex")),
            ToolId::ContinueDev => Some(self.home.join(".continue")),
            ToolId::Cline => None, // Detected via VS Code extension
            ToolId::Aider => None, // Detected via binary
            ToolId::Copilot => None, // Detected via VS Code extension
        }
    }

    /// Binary names to search for in PATH.
    pub fn binary_names(&self, tool: ToolId) -> Vec<&'static str> {
        match tool {
            ToolId::ClaudeCode => vec!["claude"],
            ToolId::ClaudeDesktop => vec![],
            ToolId::Cursor => vec!["cursor"],
            ToolId::Windsurf => vec!["windsurf"],
            ToolId::Codex => vec!["codex"],
            ToolId::ContinueDev => vec![],
            ToolId::Cline => vec![],
            ToolId::Aider => vec!["aider"],
            ToolId::Copilot => vec![],
        }
    }

    /// Application bundle paths (macOS only, returns empty on other OSes).
    pub fn app_bundle_paths(&self, tool: ToolId) -> Vec<PathBuf> {
        #[cfg(target_os = "macos")]
        {
            let apps = PathBuf::from("/Applications");
            let user_apps = self.home.join("Applications");
            match tool {
                ToolId::ClaudeCode => vec![], // CLI only
                ToolId::ClaudeDesktop => vec![
                    apps.join("Claude.app"),
                    user_apps.join("Claude.app"),
                ],
                ToolId::Cursor => vec![
                    apps.join("Cursor.app"),
                    user_apps.join("Cursor.app"),
                ],
                ToolId::Windsurf => vec![
                    apps.join("Windsurf.app"),
                    user_apps.join("Windsurf.app"),
                ],
                ToolId::Codex => vec![], // CLI only
                ToolId::ContinueDev => vec![], // VS Code extension
                ToolId::Cline => vec![], // VS Code extension
                ToolId::Aider => vec![], // CLI only
                ToolId::Copilot => vec![], // VS Code extension
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = tool;
            vec![]
        }
    }

    /// VS Code extension directory patterns to check.
    pub fn vscode_extension_patterns(&self, tool: ToolId) -> Vec<(PathBuf, &'static str)> {
        let vscode_ext_dir = self.home.join(".vscode").join("extensions");
        match tool {
            ToolId::Cline => vec![(vscode_ext_dir, "saoudrizwan.claude-dev-")],
            ToolId::ContinueDev => vec![(vscode_ext_dir, "continue.continue-")],
            ToolId::Copilot => vec![(vscode_ext_dir, "github.copilot-")],
            _ => vec![],
        }
    }

    /// Version manager shim directories for binary lookup fallback.
    pub fn version_manager_paths(&self) -> Vec<PathBuf> {
        vec![
            // Node version managers
            self.home.join(".nvm").join("versions").join("node"),
            self.home.join(".asdf").join("shims"),
            self.home.join(".local").join("share").join("mise").join("shims"),
            self.home.join(".volta").join("bin"),
            // Python (for Aider)
            self.home.join(".local").join("bin"),
            self.home.join(".pyenv").join("shims"),
        ]
    }

    // ── Private OS-specific helpers ──

    fn claude_desktop_config_path(&self) -> PathBuf {
        #[cfg(target_os = "macos")]
        {
            self.home
                .join("Library")
                .join("Application Support")
                .join("Claude")
                .join("claude_desktop_config.json")
        }
        #[cfg(target_os = "linux")]
        {
            self.home
                .join(".config")
                .join("Claude")
                .join("claude_desktop_config.json")
        }
        #[cfg(target_os = "windows")]
        {
            self.home
                .join("AppData")
                .join("Roaming")
                .join("Claude")
                .join("claude_desktop_config.json")
        }
    }

    fn claude_desktop_support_dir(&self) -> PathBuf {
        #[cfg(target_os = "macos")]
        {
            self.home
                .join("Library")
                .join("Application Support")
                .join("Claude")
        }
        #[cfg(target_os = "linux")]
        {
            self.home.join(".config").join("Claude")
        }
        #[cfg(target_os = "windows")]
        {
            self.home
                .join("AppData")
                .join("Roaming")
                .join("Claude")
        }
    }

    fn cline_mcp_settings_path(&self) -> PathBuf {
        #[cfg(target_os = "macos")]
        {
            self.home
                .join("Library")
                .join("Application Support")
                .join("Code")
                .join("User")
                .join("globalStorage")
                .join("saoudrizwan.claude-dev")
                .join("settings")
                .join("cline_mcp_settings.json")
        }
        #[cfg(target_os = "linux")]
        {
            self.home
                .join(".config")
                .join("Code")
                .join("User")
                .join("globalStorage")
                .join("saoudrizwan.claude-dev")
                .join("settings")
                .join("cline_mcp_settings.json")
        }
        #[cfg(target_os = "windows")]
        {
            self.home
                .join("AppData")
                .join("Roaming")
                .join("Code")
                .join("User")
                .join("globalStorage")
                .join("saoudrizwan.claude-dev")
                .join("settings")
                .join("cline_mcp_settings.json")
        }
    }

    fn copilot_mcp_config_path(&self) -> PathBuf {
        // VS Code's workspace-level mcp.json — but for global, use user settings
        #[cfg(target_os = "macos")]
        {
            self.home
                .join("Library")
                .join("Application Support")
                .join("Code")
                .join("User")
                .join("settings.json")
        }
        #[cfg(target_os = "linux")]
        {
            self.home
                .join(".config")
                .join("Code")
                .join("User")
                .join("settings.json")
        }
        #[cfg(target_os = "windows")]
        {
            self.home
                .join("AppData")
                .join("Roaming")
                .join("Code")
                .join("User")
                .join("settings.json")
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

    // ── home_dir ──

    #[test]
    fn home_dir_returns_injected_home() {
        let (_tmp, paths) = make_paths();
        assert_eq!(paths.home_dir(), _tmp.path());
    }

    // ── tally_dir ──

    #[test]
    fn tally_dir_returns_dot_tally_under_home() {
        let (_tmp, paths) = make_paths();
        assert_eq!(paths.tally_dir(), _tmp.path().join(".tally"));
    }

    // ── mcp_config_path ──

    #[test]
    fn mcp_config_path_claude_code_returns_claude_json() {
        let (_tmp, paths) = make_paths();
        assert_eq!(
            paths.mcp_config_path(ToolId::ClaudeCode).unwrap(),
            _tmp.path().join(".claude.json")
        );
    }

    #[test]
    fn mcp_config_path_claude_desktop_returns_platform_path() {
        let (_tmp, paths) = make_paths();
        let p = paths.mcp_config_path(ToolId::ClaudeDesktop).unwrap();
        assert!(p.ends_with("claude_desktop_config.json"));
    }

    #[test]
    fn mcp_config_path_cursor_returns_mcp_json() {
        let (_tmp, paths) = make_paths();
        assert_eq!(
            paths.mcp_config_path(ToolId::Cursor).unwrap(),
            _tmp.path().join(".cursor").join("mcp.json")
        );
    }

    #[test]
    fn mcp_config_path_windsurf_returns_mcp_config_json() {
        let (_tmp, paths) = make_paths();
        assert_eq!(
            paths.mcp_config_path(ToolId::Windsurf).unwrap(),
            _tmp.path().join(".codeium").join("windsurf").join("mcp_config.json")
        );
    }

    #[test]
    fn mcp_config_path_codex_returns_config_toml() {
        let (_tmp, paths) = make_paths();
        assert_eq!(
            paths.mcp_config_path(ToolId::Codex).unwrap(),
            _tmp.path().join(".codex").join("config.toml")
        );
    }

    #[test]
    fn mcp_config_path_continue_dev_returns_config_yaml() {
        let (_tmp, paths) = make_paths();
        assert_eq!(
            paths.mcp_config_path(ToolId::ContinueDev).unwrap(),
            _tmp.path().join(".continue").join("config.yaml")
        );
    }

    #[test]
    fn mcp_config_path_cline_returns_platform_specific() {
        let (_tmp, paths) = make_paths();
        let p = paths.mcp_config_path(ToolId::Cline).unwrap();
        assert!(p.ends_with("cline_mcp_settings.json"));
    }

    #[test]
    fn mcp_config_path_aider_returns_none() {
        let (_tmp, paths) = make_paths();
        assert!(paths.mcp_config_path(ToolId::Aider).is_none());
    }

    #[test]
    fn mcp_config_path_copilot_returns_settings_json() {
        let (_tmp, paths) = make_paths();
        let p = paths.mcp_config_path(ToolId::Copilot).unwrap();
        assert!(p.ends_with("settings.json"));
    }

    // ── skill_path ──

    #[test]
    fn skill_path_claude_code_returns_skill_md() {
        let (_tmp, paths) = make_paths();
        assert_eq!(
            paths.skill_path(ToolId::ClaudeCode).unwrap(),
            _tmp.path()
                .join(".claude")
                .join("skills")
                .join("tally-wallet")
                .join("SKILL.md")
        );
    }

    #[test]
    fn skill_path_claude_desktop_returns_none() {
        let (_tmp, paths) = make_paths();
        assert!(paths.skill_path(ToolId::ClaudeDesktop).is_none());
    }

    #[test]
    fn skill_path_cursor_returns_mdc_file() {
        let (_tmp, paths) = make_paths();
        let p = paths.skill_path(ToolId::Cursor).unwrap();
        assert!(p.ends_with("tally-wallet.mdc"));
        assert!(p.to_string_lossy().contains(".cursor"));
    }

    #[test]
    fn skill_path_windsurf_returns_md_in_windsurf_rules() {
        let (_tmp, paths) = make_paths();
        assert_eq!(
            paths.skill_path(ToolId::Windsurf).unwrap(),
            _tmp.path().join(".windsurf").join("rules").join("tally-wallet.md")
        );
    }

    #[test]
    fn skill_path_codex_returns_agents_md() {
        let (_tmp, paths) = make_paths();
        assert_eq!(
            paths.skill_path(ToolId::Codex).unwrap(),
            _tmp.path().join(".codex").join("AGENTS.md")
        );
    }

    #[test]
    fn skill_path_continue_dev_returns_rules_md() {
        let (_tmp, paths) = make_paths();
        assert_eq!(
            paths.skill_path(ToolId::ContinueDev).unwrap(),
            _tmp.path().join(".continue").join("rules").join("tally-wallet.md")
        );
    }

    #[test]
    fn skill_path_cline_returns_clinerules_md() {
        let (_tmp, paths) = make_paths();
        assert_eq!(
            paths.skill_path(ToolId::Cline).unwrap(),
            _tmp.path().join(".clinerules").join("tally-wallet.md")
        );
    }

    #[test]
    fn skill_path_aider_returns_conf_yml() {
        let (_tmp, paths) = make_paths();
        assert_eq!(
            paths.skill_path(ToolId::Aider).unwrap(),
            _tmp.path().join(".aider.conf.yml")
        );
    }

    #[test]
    fn skill_path_copilot_returns_copilot_instructions() {
        let (_tmp, paths) = make_paths();
        assert_eq!(
            paths.skill_path(ToolId::Copilot).unwrap(),
            _tmp.path().join(".github").join("copilot-instructions.md")
        );
    }

    // ── config_dir ──

    #[test]
    fn config_dir_claude_code_returns_dot_claude() {
        let (_tmp, paths) = make_paths();
        assert_eq!(
            paths.config_dir(ToolId::ClaudeCode).unwrap(),
            _tmp.path().join(".claude")
        );
    }

    #[test]
    fn config_dir_cursor_returns_dot_cursor() {
        let (_tmp, paths) = make_paths();
        assert_eq!(
            paths.config_dir(ToolId::Cursor).unwrap(),
            _tmp.path().join(".cursor")
        );
    }

    #[test]
    fn config_dir_windsurf_returns_dot_codeium() {
        let (_tmp, paths) = make_paths();
        assert_eq!(
            paths.config_dir(ToolId::Windsurf).unwrap(),
            _tmp.path().join(".codeium")
        );
    }

    #[test]
    fn config_dir_codex_returns_dot_codex() {
        let (_tmp, paths) = make_paths();
        assert_eq!(
            paths.config_dir(ToolId::Codex).unwrap(),
            _tmp.path().join(".codex")
        );
    }

    #[test]
    fn config_dir_continue_dev_returns_dot_continue() {
        let (_tmp, paths) = make_paths();
        assert_eq!(
            paths.config_dir(ToolId::ContinueDev).unwrap(),
            _tmp.path().join(".continue")
        );
    }

    #[test]
    fn config_dir_cline_returns_none() {
        let (_tmp, paths) = make_paths();
        assert!(paths.config_dir(ToolId::Cline).is_none());
    }

    #[test]
    fn config_dir_aider_returns_none() {
        let (_tmp, paths) = make_paths();
        assert!(paths.config_dir(ToolId::Aider).is_none());
    }

    #[test]
    fn config_dir_copilot_returns_none() {
        let (_tmp, paths) = make_paths();
        assert!(paths.config_dir(ToolId::Copilot).is_none());
    }

    // ── binary_names ──

    #[test]
    fn binary_names_claude_code_returns_claude() {
        let (_tmp, paths) = make_paths();
        assert_eq!(paths.binary_names(ToolId::ClaudeCode), vec!["claude"]);
    }

    #[test]
    fn binary_names_claude_desktop_returns_empty() {
        let (_tmp, paths) = make_paths();
        assert!(paths.binary_names(ToolId::ClaudeDesktop).is_empty());
    }

    #[test]
    fn binary_names_cursor_returns_cursor() {
        let (_tmp, paths) = make_paths();
        assert_eq!(paths.binary_names(ToolId::Cursor), vec!["cursor"]);
    }

    #[test]
    fn binary_names_aider_returns_aider() {
        let (_tmp, paths) = make_paths();
        assert_eq!(paths.binary_names(ToolId::Aider), vec!["aider"]);
    }

    #[test]
    fn binary_names_cline_returns_empty() {
        let (_tmp, paths) = make_paths();
        assert!(paths.binary_names(ToolId::Cline).is_empty());
    }

    #[test]
    fn binary_names_copilot_returns_empty() {
        let (_tmp, paths) = make_paths();
        assert!(paths.binary_names(ToolId::Copilot).is_empty());
    }

    // ── vscode_extension_patterns ──

    #[test]
    fn vscode_extension_patterns_cline_has_saoudrizwan_prefix() {
        let (_tmp, paths) = make_paths();
        let patterns = paths.vscode_extension_patterns(ToolId::Cline);
        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0].1, "saoudrizwan.claude-dev-");
        assert!(patterns[0].0.ends_with("extensions"));
    }

    #[test]
    fn vscode_extension_patterns_continue_dev_has_prefix() {
        let (_tmp, paths) = make_paths();
        let patterns = paths.vscode_extension_patterns(ToolId::ContinueDev);
        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0].1, "continue.continue-");
    }

    #[test]
    fn vscode_extension_patterns_copilot_has_prefix() {
        let (_tmp, paths) = make_paths();
        let patterns = paths.vscode_extension_patterns(ToolId::Copilot);
        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0].1, "github.copilot-");
    }

    #[test]
    fn vscode_extension_patterns_claude_code_returns_empty() {
        let (_tmp, paths) = make_paths();
        assert!(paths.vscode_extension_patterns(ToolId::ClaudeCode).is_empty());
    }

    // ── version_manager_paths ──

    #[test]
    fn version_manager_paths_contains_expected_dirs() {
        let (_tmp, paths) = make_paths();
        let vm_paths = paths.version_manager_paths();
        let path_strs: Vec<String> = vm_paths.iter().map(|p| p.to_string_lossy().to_string()).collect();

        // Should contain nvm, asdf, mise, volta, local/bin, pyenv
        assert!(path_strs.iter().any(|p| p.contains(".nvm")));
        assert!(path_strs.iter().any(|p| p.contains(".volta")));
        assert!(path_strs.iter().any(|p| p.contains(".pyenv")));
        assert!(path_strs.iter().any(|p| p.contains(".asdf")));
        assert!(path_strs.iter().any(|p| p.contains("mise")));
        assert!(path_strs.iter().any(|p| p.contains(".local")));
    }

    // ── app_bundle_paths ──

    #[test]
    fn app_bundle_paths_claude_code_is_empty() {
        let (_tmp, paths) = make_paths();
        assert!(paths.app_bundle_paths(ToolId::ClaudeCode).is_empty());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn app_bundle_paths_claude_desktop_has_entries_on_macos() {
        let (_tmp, paths) = make_paths();
        let bundles = paths.app_bundle_paths(ToolId::ClaudeDesktop);
        assert!(!bundles.is_empty());
        assert!(bundles.iter().any(|p| p.ends_with("Claude.app")));
    }
}
