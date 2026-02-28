use std::path::{Path, PathBuf};

const MCP_CONFIG_FILENAME: &str = ".mcp.json";
const CLAUDE_MD_FILENAME: &str = "CLAUDE.md";
const CLAUDE_DIR: &str = ".claude";
const MCP_SERVER_KEY: &str = "tally-wallet";
const INSTRUCTIONS_MARKER: &str = "## Tally Agentic Wallet";

const CLAUDE_INSTRUCTIONS: &str = r#"
## Tally Agentic Wallet

A crypto wallet is running on this machine via MCP. You can send payments,
check balances, trade tokens, and pay for X402 services.

**First time?** Call `register_agent` with a descriptive name for yourself
(e.g. "Claude Code - my-project"). Save the returned token in your
persistent memory — you'll need it for all future calls.

**Already registered?** Your token is in your memory. All spending is
tracked under your agent name and subject to policies set by the user.

New agents start with $0 spending limits. The wallet owner will set
your budget after they see you in the app.
"#;

/// Return the path to ~/.claude/
fn claude_dir(home: &Path) -> PathBuf {
    home.join(CLAUDE_DIR)
}

/// Install Tally Wallet MCP discovery files so agents find us automatically.
pub fn install(mcp_port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let home = dirs::home_dir().ok_or("Cannot find home directory")?;
    install_mcp_config(&claude_dir(&home), mcp_port)?;
    install_claude_instructions(&claude_dir(&home))?;
    Ok(())
}

/// Remove Tally Wallet entries from discovery files.
pub fn uninstall() -> Result<(), Box<dyn std::error::Error>> {
    let home = dirs::home_dir().ok_or("Cannot find home directory")?;
    uninstall_from_dir(&claude_dir(&home))
}

/// Check if already installed by looking for our MCP server key in the config.
pub fn is_installed() -> bool {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return false,
    };
    is_installed_in_dir(&claude_dir(&home))
}

// ── Internal helpers (pub(crate) for testing) ──────────────────────────

fn install_mcp_config(claude_dir: &Path, mcp_port: u16) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(claude_dir)?;
    let config_path = claude_dir.join(MCP_CONFIG_FILENAME);

    let mut root: serde_json::Value = if config_path.exists() {
        let contents = std::fs::read_to_string(&config_path)?;
        serde_json::from_str(&contents)?
    } else {
        serde_json::json!({})
    };

    let servers = root
        .as_object_mut()
        .ok_or("MCP config is not a JSON object")?
        .entry("mcpServers")
        .or_insert_with(|| serde_json::json!({}));

    servers
        .as_object_mut()
        .ok_or("mcpServers is not a JSON object")?
        .insert(
            MCP_SERVER_KEY.to_string(),
            serde_json::json!({
                "url": format!("http://localhost:{}/mcp", mcp_port)
            }),
        );

    let formatted = serde_json::to_string_pretty(&root)?;
    std::fs::write(&config_path, formatted)?;
    Ok(())
}

fn install_claude_instructions(claude_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(claude_dir)?;
    let md_path = claude_dir.join(CLAUDE_MD_FILENAME);

    if md_path.exists() {
        let contents = std::fs::read_to_string(&md_path)?;
        if contents.contains(INSTRUCTIONS_MARKER) {
            // Already present — skip
            return Ok(());
        }
        // Append to existing file
        let mut new_contents = contents;
        if !new_contents.ends_with('\n') {
            new_contents.push('\n');
        }
        new_contents.push_str(CLAUDE_INSTRUCTIONS);
        std::fs::write(&md_path, new_contents)?;
    } else {
        std::fs::write(&md_path, CLAUDE_INSTRUCTIONS)?;
    }

    Ok(())
}

fn uninstall_from_dir(claude_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // Remove from MCP config
    let config_path = claude_dir.join(MCP_CONFIG_FILENAME);
    if config_path.exists() {
        let contents = std::fs::read_to_string(&config_path)?;
        let mut root: serde_json::Value = serde_json::from_str(&contents)?;
        if let Some(servers) = root
            .as_object_mut()
            .and_then(|o| o.get_mut("mcpServers"))
            .and_then(|s| s.as_object_mut())
        {
            servers.remove(MCP_SERVER_KEY);
        }
        let formatted = serde_json::to_string_pretty(&root)?;
        std::fs::write(&config_path, formatted)?;
    }

    // Remove from CLAUDE.md
    let md_path = claude_dir.join(CLAUDE_MD_FILENAME);
    if md_path.exists() {
        let contents = std::fs::read_to_string(&md_path)?;
        if let Some(start) = contents.find(INSTRUCTIONS_MARKER) {
            // Remove from marker to end-of-section (next ## or end of file)
            let after_marker = &contents[start + INSTRUCTIONS_MARKER.len()..];
            let section_end = after_marker
                .find("\n## ")
                .map(|pos| start + INSTRUCTIONS_MARKER.len() + pos)
                .unwrap_or(contents.len());
            // Also consume the leading newline before the marker if present
            let actual_start = if start > 0 && contents.as_bytes()[start - 1] == b'\n' {
                start - 1
            } else {
                start
            };
            let mut new_contents = String::new();
            new_contents.push_str(&contents[..actual_start]);
            if section_end < contents.len() {
                new_contents.push_str(&contents[section_end..]);
            }
            std::fs::write(&md_path, new_contents)?;
        }
    }

    Ok(())
}

fn is_installed_in_dir(claude_dir: &Path) -> bool {
    let config_path = claude_dir.join(MCP_CONFIG_FILENAME);
    if !config_path.exists() {
        return false;
    }
    let contents = match std::fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(_) => return false,
    };
    let root: serde_json::Value = match serde_json::from_str(&contents) {
        Ok(v) => v,
        Err(_) => return false,
    };
    root.get("mcpServers")
        .and_then(|s| s.get(MCP_SERVER_KEY))
        .is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_dir() -> TempDir {
        tempfile::tempdir().expect("Failed to create temp dir")
    }

    #[test]
    fn test_install_creates_mcp_config() {
        let tmp = setup_dir();
        let claude_dir = tmp.path().join(CLAUDE_DIR);

        install_mcp_config(&claude_dir, 7403).unwrap();

        let config_path = claude_dir.join(MCP_CONFIG_FILENAME);
        assert!(config_path.exists());

        let contents: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&config_path).unwrap()).unwrap();
        assert_eq!(
            contents["mcpServers"]["tally-wallet"]["url"],
            "http://localhost:7403/mcp"
        );
    }

    #[test]
    fn test_install_merges_with_existing_config() {
        let tmp = setup_dir();
        let claude_dir = tmp.path().join(CLAUDE_DIR);
        std::fs::create_dir_all(&claude_dir).unwrap();

        // Pre-populate with an existing server
        let existing = serde_json::json!({
            "mcpServers": {
                "other-server": { "url": "http://localhost:9999/mcp" }
            }
        });
        std::fs::write(
            claude_dir.join(MCP_CONFIG_FILENAME),
            serde_json::to_string_pretty(&existing).unwrap(),
        )
        .unwrap();

        install_mcp_config(&claude_dir, 7403).unwrap();

        let contents: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(claude_dir.join(MCP_CONFIG_FILENAME)).unwrap(),
        )
        .unwrap();

        // Both servers should be present
        assert_eq!(
            contents["mcpServers"]["other-server"]["url"],
            "http://localhost:9999/mcp"
        );
        assert_eq!(
            contents["mcpServers"]["tally-wallet"]["url"],
            "http://localhost:7403/mcp"
        );
    }

    #[test]
    fn test_install_idempotent() {
        let tmp = setup_dir();
        let claude_dir = tmp.path().join(CLAUDE_DIR);

        install_mcp_config(&claude_dir, 7403).unwrap();
        install_mcp_config(&claude_dir, 7403).unwrap();

        let contents: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(claude_dir.join(MCP_CONFIG_FILENAME)).unwrap(),
        )
        .unwrap();

        let servers = contents["mcpServers"].as_object().unwrap();
        assert_eq!(servers.len(), 1);
        assert_eq!(
            contents["mcpServers"]["tally-wallet"]["url"],
            "http://localhost:7403/mcp"
        );
    }

    #[test]
    fn test_install_appends_claude_instructions() {
        let tmp = setup_dir();
        let claude_dir = tmp.path().join(CLAUDE_DIR);
        std::fs::create_dir_all(&claude_dir).unwrap();

        // Pre-populate with existing content
        std::fs::write(claude_dir.join(CLAUDE_MD_FILENAME), "# My Config\nSome stuff.\n").unwrap();

        install_claude_instructions(&claude_dir).unwrap();

        let contents = std::fs::read_to_string(claude_dir.join(CLAUDE_MD_FILENAME)).unwrap();
        assert!(contents.starts_with("# My Config"));
        assert!(contents.contains(INSTRUCTIONS_MARKER));
        assert!(contents.contains("register_agent"));
    }

    #[test]
    fn test_install_skips_if_instructions_exist() {
        let tmp = setup_dir();
        let claude_dir = tmp.path().join(CLAUDE_DIR);
        std::fs::create_dir_all(&claude_dir).unwrap();

        let original = format!("# Config\n{}\n", CLAUDE_INSTRUCTIONS);
        std::fs::write(claude_dir.join(CLAUDE_MD_FILENAME), &original).unwrap();

        install_claude_instructions(&claude_dir).unwrap();

        let contents = std::fs::read_to_string(claude_dir.join(CLAUDE_MD_FILENAME)).unwrap();
        // Should not be duplicated
        let count = contents.matches(INSTRUCTIONS_MARKER).count();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_install_creates_claude_md_if_missing() {
        let tmp = setup_dir();
        let claude_dir = tmp.path().join(CLAUDE_DIR);

        install_claude_instructions(&claude_dir).unwrap();

        let contents = std::fs::read_to_string(claude_dir.join(CLAUDE_MD_FILENAME)).unwrap();
        assert!(contents.contains(INSTRUCTIONS_MARKER));
    }

    #[test]
    fn test_uninstall_removes_entries() {
        let tmp = setup_dir();
        let claude_dir = tmp.path().join(CLAUDE_DIR);

        // Install first
        install_mcp_config(&claude_dir, 7403).unwrap();
        install_claude_instructions(&claude_dir).unwrap();

        // Verify installed
        assert!(is_installed_in_dir(&claude_dir));

        // Uninstall
        uninstall_from_dir(&claude_dir).unwrap();

        // MCP config should no longer have our key
        assert!(!is_installed_in_dir(&claude_dir));

        // CLAUDE.md should not contain our section
        let md = std::fs::read_to_string(claude_dir.join(CLAUDE_MD_FILENAME)).unwrap();
        assert!(!md.contains(INSTRUCTIONS_MARKER));
    }

    #[test]
    fn test_uninstall_preserves_other_servers() {
        let tmp = setup_dir();
        let claude_dir = tmp.path().join(CLAUDE_DIR);
        std::fs::create_dir_all(&claude_dir).unwrap();

        // Create config with two servers
        let existing = serde_json::json!({
            "mcpServers": {
                "other-server": { "url": "http://localhost:9999/mcp" },
                "tally-wallet": { "url": "http://localhost:7403/mcp" }
            }
        });
        std::fs::write(
            claude_dir.join(MCP_CONFIG_FILENAME),
            serde_json::to_string_pretty(&existing).unwrap(),
        )
        .unwrap();

        uninstall_from_dir(&claude_dir).unwrap();

        let contents: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(claude_dir.join(MCP_CONFIG_FILENAME)).unwrap(),
        )
        .unwrap();

        assert!(contents["mcpServers"].get("tally-wallet").is_none());
        assert_eq!(
            contents["mcpServers"]["other-server"]["url"],
            "http://localhost:9999/mcp"
        );
    }

    #[test]
    fn test_is_installed_returns_false_when_not_installed() {
        let tmp = setup_dir();
        let claude_dir = tmp.path().join(CLAUDE_DIR);
        assert!(!is_installed_in_dir(&claude_dir));
    }

    #[test]
    fn test_is_installed_returns_true_after_install() {
        let tmp = setup_dir();
        let claude_dir = tmp.path().join(CLAUDE_DIR);
        install_mcp_config(&claude_dir, 7403).unwrap();
        assert!(is_installed_in_dir(&claude_dir));
    }

    #[test]
    fn test_install_uses_correct_port() {
        let tmp = setup_dir();
        let claude_dir = tmp.path().join(CLAUDE_DIR);
        install_mcp_config(&claude_dir, 8080).unwrap();

        let contents: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(claude_dir.join(MCP_CONFIG_FILENAME)).unwrap(),
        )
        .unwrap();
        assert_eq!(
            contents["mcpServers"]["tally-wallet"]["url"],
            "http://localhost:8080/mcp"
        );
    }
}
