use std::collections::HashMap;

use crate::provisioning::types::McpInjectionConfig;

/// Sentinel markers for markdown content injection.
pub const SENTINEL_START: &str = "<!-- TALLY_WALLET_START -->";
pub const SENTINEL_END: &str = "<!-- TALLY_WALLET_END -->";

/// The MCP server key name used across all tool configs.
pub const MCP_SERVER_KEY: &str = "tally-wallet";

// ── MCP Config Templates ──

/// Generate the JSON value for an MCP server entry (Claude Code, Claude Desktop, Cursor, Windsurf, Cline).
pub fn mcp_json_entry(config: &McpInjectionConfig) -> serde_json::Value {
    let mut entry = serde_json::json!({
        "command": config.server_command,
        "args": config.server_args,
    });

    if !config.env.is_empty() {
        entry["env"] = serde_json::to_value(&config.env).unwrap_or_default();
    }

    // Metadata for version tracking (tools ignore unknown keys)
    entry["_tally_version"] = serde_json::Value::String(config.tally_version.clone());
    entry["_tally_provisioned"] = serde_json::Value::String(config.provisioned_at.clone());

    entry
}

/// Generate TOML content for Codex CLI's config.toml [mcp_servers.tally-wallet] section.
pub fn mcp_toml_section(config: &McpInjectionConfig) -> String {
    let mut lines = vec![
        format!("# tally-wallet v{} (provisioned {})", config.tally_version, config.provisioned_at),
        format!("[mcp_servers.{}]", MCP_SERVER_KEY),
        format!("command = \"{}\"", config.server_command),
    ];

    if !config.server_args.is_empty() {
        let args: Vec<String> = config.server_args.iter().map(|a| format!("\"{}\"", a)).collect();
        lines.push(format!("args = [{}]", args.join(", ")));
    }

    for (key, value) in &config.env {
        lines.push(format!("[mcp_servers.{}.env]", MCP_SERVER_KEY));
        lines.push(format!("{} = \"{}\"", key, value));
        break; // Only one env section needed
    }

    lines.join("\n")
}

/// Generate YAML content for Continue.dev's config.yaml mcpServers entry.
pub fn mcp_yaml_entry(config: &McpInjectionConfig) -> String {
    let mut lines = vec![
        format!("  # tally-wallet v{} (provisioned {})", config.tally_version, config.provisioned_at),
        format!("  - name: {}", MCP_SERVER_KEY),
        format!("    command: \"{}\"", config.server_command),
    ];

    if !config.server_args.is_empty() {
        lines.push("    args:".to_string());
        for arg in &config.server_args {
            lines.push(format!("      - \"{}\"", arg));
        }
    }

    if !config.env.is_empty() {
        lines.push("    env:".to_string());
        for (key, value) in &config.env {
            lines.push(format!("      {}: \"{}\"", key, value));
        }
    }

    lines.join("\n")
}

// ── Skill/Instruction Templates ──

/// The lightweight skill content injected into tool instruction files.
/// This is intentionally tiny (~100 tokens) for discovery purposes.
const SKILL_CONTENT_INLINE: &str = r#"## Tally Agentic Wallet

A crypto wallet is available on this machine via MCP. You can send payments,
check balances, trade tokens, and pay for X402 services.

**First time?** Call `register_agent` with a descriptive name for yourself
(e.g. "Claude Code - my-project"). Save the returned token in your
persistent memory — you'll need it for all future calls.

**Already registered?** Your token is in your memory. All spending is
tracked under your agent name and subject to policies set by the user.

New agents start with $0 spending limits. The wallet owner will set
your budget after they see you in the app."#;

/// Claude Code SKILL.md uses progressive disclosure (description field loaded at startup,
/// full content only when the agent engages with wallet tasks).
pub fn claude_code_skill_content() -> String {
    format!(
        r#"---
description: "Tally Agentic Wallet — send payments, check balances, trade tokens via MCP"
---

{}
"#,
        SKILL_CONTENT_INLINE
    )
}

/// Cursor .mdc file with frontmatter.
pub fn cursor_rule_content() -> String {
    format!(
        r#"---
description: "Tally Agentic Wallet MCP integration"
globs: []
alwaysApply: true
---

{}
"#,
        SKILL_CONTENT_INLINE
    )
}

/// Standalone markdown for Windsurf, Continue.dev, Cline.
pub fn standalone_skill_content() -> String {
    SKILL_CONTENT_INLINE.to_string()
}

/// Codex AGENTS.md sentinel-wrapped content (appended/upserted into existing file).
pub fn codex_agents_content() -> String {
    format!(
        "{}\n{}\n{}",
        SENTINEL_START, SKILL_CONTENT_INLINE, SENTINEL_END
    )
}

/// Copilot copilot-instructions.md sentinel-wrapped content.
pub fn copilot_instructions_content() -> String {
    format!(
        "{}\n{}\n{}",
        SENTINEL_START, SKILL_CONTENT_INLINE, SENTINEL_END
    )
}

/// Aider conventions file content (referenced from .aider.conf.yml read list).
pub fn aider_conventions_content() -> String {
    SKILL_CONTENT_INLINE.to_string()
}

/// The YAML entry to add to Aider's .aider.conf.yml read list.
pub fn aider_read_entry() -> String {
    "- .aider/tally-wallet.md".to_string()
}

/// Perform token substitution in content strings.
/// Replaces {{TOKEN}}, {{VERSION}}, {{MCP_COMMAND}} etc.
pub fn substitute_tokens(template: &str, vars: &HashMap<String, String>) -> String {
    let mut result = template.to_string();
    for (key, value) in vars {
        result = result.replace(&format!("{{{{{}}}}}", key), value);
    }
    result
}

/// Get the inline skill content (for use by provisioners).
pub fn skill_content_inline() -> &'static str {
    SKILL_CONTENT_INLINE
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_config(env: HashMap<String, String>) -> McpInjectionConfig {
        McpInjectionConfig {
            server_command: "/usr/bin/tally-mcp".to_string(),
            server_args: vec!["--stdio".to_string()],
            env,
            tally_version: "1.2.3".to_string(),
            provisioned_at: "2026-03-01T00:00:00Z".to_string(),
        }
    }

    fn empty_config() -> McpInjectionConfig {
        make_config(HashMap::new())
    }

    fn config_with_env() -> McpInjectionConfig {
        let mut env = HashMap::new();
        env.insert("TALLY_TOKEN".to_string(), "abc123".to_string());
        make_config(env)
    }

    // ── mcp_json_entry ──

    #[test]
    fn mcp_json_entry_contains_required_fields() {
        let cfg = empty_config();
        let val = mcp_json_entry(&cfg);
        assert_eq!(val["command"], "/usr/bin/tally-mcp");
        assert_eq!(val["args"][0], "--stdio");
        assert_eq!(val["_tally_version"], "1.2.3");
        assert_eq!(val["_tally_provisioned"], "2026-03-01T00:00:00Z");
    }

    #[test]
    fn mcp_json_entry_no_env_key_when_empty() {
        let cfg = empty_config();
        let val = mcp_json_entry(&cfg);
        assert!(val.get("env").is_none(), "env key should be absent when env is empty");
    }

    #[test]
    fn mcp_json_entry_includes_env_when_present() {
        let cfg = config_with_env();
        let val = mcp_json_entry(&cfg);
        let env = val.get("env").expect("env key should be present");
        assert_eq!(env["TALLY_TOKEN"], "abc123");
    }

    // ── mcp_toml_section ──

    #[test]
    fn mcp_toml_section_contains_server_name_and_args() {
        let cfg = empty_config();
        let toml = mcp_toml_section(&cfg);
        assert!(toml.contains("[mcp_servers.tally-wallet]"));
        assert!(toml.contains("command = \"/usr/bin/tally-mcp\""));
        assert!(toml.contains("args = [\"--stdio\"]"));
    }

    #[test]
    fn mcp_toml_section_contains_version_comment() {
        let cfg = empty_config();
        let toml = mcp_toml_section(&cfg);
        assert!(toml.contains("# tally-wallet v1.2.3 (provisioned 2026-03-01T00:00:00Z)"));
    }

    #[test]
    fn mcp_toml_section_includes_env_when_present() {
        let cfg = config_with_env();
        let toml = mcp_toml_section(&cfg);
        assert!(toml.contains("[mcp_servers.tally-wallet.env]"));
        assert!(toml.contains("TALLY_TOKEN = \"abc123\""));
    }

    #[test]
    fn mcp_toml_section_no_env_when_empty() {
        let cfg = empty_config();
        let toml = mcp_toml_section(&cfg);
        assert!(!toml.contains(".env]"));
    }

    // ── mcp_yaml_entry ──

    #[test]
    fn mcp_yaml_entry_contains_name_and_args() {
        let cfg = empty_config();
        let yaml = mcp_yaml_entry(&cfg);
        assert!(yaml.contains("name: tally-wallet"));
        assert!(yaml.contains("command: \"/usr/bin/tally-mcp\""));
        assert!(yaml.contains("- \"--stdio\""));
    }

    #[test]
    fn mcp_yaml_entry_includes_env_when_present() {
        let cfg = config_with_env();
        let yaml = mcp_yaml_entry(&cfg);
        assert!(yaml.contains("env:"));
        assert!(yaml.contains("TALLY_TOKEN: \"abc123\""));
    }

    #[test]
    fn mcp_yaml_entry_no_env_when_empty() {
        let cfg = empty_config();
        let yaml = mcp_yaml_entry(&cfg);
        assert!(!yaml.contains("env:"));
    }

    // ── claude_code_skill_content ──

    #[test]
    fn claude_code_skill_has_yaml_frontmatter() {
        let content = claude_code_skill_content();
        assert!(content.starts_with("---\n"));
        assert!(content.contains("description:"));
        assert!(content.contains("---\n\n"));
    }

    #[test]
    fn claude_code_skill_contains_inline_content() {
        let content = claude_code_skill_content();
        assert!(content.contains("Tally Agentic Wallet"));
        assert!(content.contains("register_agent"));
    }

    // ── cursor_rule_content ──

    #[test]
    fn cursor_rule_has_always_apply_frontmatter() {
        let content = cursor_rule_content();
        assert!(content.starts_with("---\n"));
        assert!(content.contains("alwaysApply: true"));
    }

    #[test]
    fn cursor_rule_contains_inline_content() {
        let content = cursor_rule_content();
        assert!(content.contains("Tally Agentic Wallet"));
    }

    // ── standalone_skill_content / skill_content_inline ──

    #[test]
    fn standalone_skill_content_returns_inline() {
        assert_eq!(standalone_skill_content(), skill_content_inline());
    }

    #[test]
    fn skill_content_inline_is_non_empty_and_contains_tally() {
        let content = skill_content_inline();
        assert!(!content.is_empty());
        assert!(content.contains("Tally"));
    }

    // ── codex_agents_content ──

    #[test]
    fn codex_agents_content_wrapped_in_sentinels() {
        let content = codex_agents_content();
        assert!(content.starts_with(SENTINEL_START));
        assert!(content.ends_with(SENTINEL_END));
    }

    #[test]
    fn codex_agents_content_contains_skill_body() {
        let content = codex_agents_content();
        assert!(content.contains("Tally Agentic Wallet"));
    }

    // ── copilot_instructions_content ──

    #[test]
    fn copilot_instructions_wrapped_in_sentinels() {
        let content = copilot_instructions_content();
        assert!(content.starts_with(SENTINEL_START));
        assert!(content.ends_with(SENTINEL_END));
    }

    #[test]
    fn copilot_instructions_contains_skill_body() {
        let content = copilot_instructions_content();
        assert!(content.contains("register_agent"));
    }

    // ── aider_conventions_content ──

    #[test]
    fn aider_conventions_equals_inline() {
        assert_eq!(aider_conventions_content(), skill_content_inline());
    }

    #[test]
    fn aider_conventions_non_empty() {
        assert!(!aider_conventions_content().is_empty());
    }

    // ── aider_read_entry ──

    #[test]
    fn aider_read_entry_correct_format() {
        assert_eq!(aider_read_entry(), "- .aider/tally-wallet.md");
    }

    #[test]
    fn aider_read_entry_contains_tally_wallet() {
        assert!(aider_read_entry().contains("tally-wallet"));
    }

    // ── substitute_tokens ──

    #[test]
    fn substitute_tokens_replaces_known_keys() {
        let mut vars = HashMap::new();
        vars.insert("VERSION".to_string(), "2.0.0".to_string());
        let result = substitute_tokens("v{{VERSION}}", &vars);
        assert_eq!(result, "v2.0.0");
    }

    #[test]
    fn substitute_tokens_unknown_keys_unchanged() {
        let vars = HashMap::new();
        let result = substitute_tokens("{{UNKNOWN}}", &vars);
        assert_eq!(result, "{{UNKNOWN}}");
    }

    #[test]
    fn substitute_tokens_multiple_substitutions() {
        let mut vars = HashMap::new();
        vars.insert("A".to_string(), "1".to_string());
        vars.insert("B".to_string(), "2".to_string());
        let result = substitute_tokens("{{A}}-{{B}}-{{A}}", &vars);
        assert_eq!(result, "1-2-1");
    }

    #[test]
    fn substitute_tokens_empty_vars_noop() {
        let vars = HashMap::new();
        let template = "hello world";
        assert_eq!(substitute_tokens(template, &vars), template);
    }
}
