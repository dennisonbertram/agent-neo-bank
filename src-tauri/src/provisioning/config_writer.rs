use std::io::Write;
use std::path::Path;

use fs2::FileExt;

use crate::provisioning::error::ProvisioningError;

/// Sentinel markers for markdown content injection.
const SENTINEL_START: &str = "<!-- TALLY_WALLET_START";
const SENTINEL_END: &str = "<!-- TALLY_WALLET_END -->";

// ── Core: Atomic Read-Modify-Write ──

/// Atomic read-modify-write for any config file.
/// 1. Acquire advisory file lock
/// 2. Read current contents (empty string if file doesn't exist)
/// 3. Apply modification function
/// 4. Write to temp file in same directory
/// 5. Rename temp to target (atomic on same-fs)
/// 6. Release lock
///
/// Returns (original_contents, modified_contents).
pub fn atomic_modify<F>(
    path: &Path,
    modify: F,
) -> Result<(String, String), ProvisioningError>
where
    F: FnOnce(&str) -> Result<String, ProvisioningError>,
{
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }

    // Acquire advisory file lock via a sidecar lock file
    let lock_path = path.with_extension(
        format!(
            "{}.tally-lock",
            path.extension()
                .map(|e| e.to_string_lossy().to_string())
                .unwrap_or_default()
        ),
    );
    let lock_file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(&lock_path)?;

    lock_file
        .try_lock_exclusive()
        .map_err(|_| ProvisioningError::ConfigLocked(path.to_path_buf()))?;

    // Read current contents (empty string if file doesn't exist)
    let original = if path.exists() {
        std::fs::read_to_string(path)?
    } else {
        String::new()
    };

    // Apply modification
    let modified = modify(&original)?;

    // Write to temp file in same directory (for atomic rename)
    let parent = path.parent().unwrap_or(Path::new("."));
    let mut temp = tempfile::NamedTempFile::new_in(parent)?;
    temp.write_all(modified.as_bytes())?;
    temp.flush()?;

    // Preserve permissions if file already exists
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = if path.exists() {
            std::fs::metadata(path)?.permissions()
        } else {
            std::fs::Permissions::from_mode(0o644)
        };
        temp.as_file().set_permissions(perms)?;
    }

    // Atomic rename
    temp.persist(path).map_err(|e| ProvisioningError::AtomicWriteFailed {
        path: path.to_path_buf(),
        reason: e.error.to_string(),
    })?;

    // Cleanup lock file (best-effort)
    let _ = lock_file.unlock();
    let _ = std::fs::remove_file(&lock_path);

    Ok((original, modified))
}

// ── JSON Operations ──

/// Merge an MCP server entry into a JSON config file.
/// Handles both "mcpServers" (most tools) and "servers" (VS Code/Copilot) root keys.
pub fn json_merge_mcp_server(
    existing: &str,
    server_name: &str,
    server_config: &serde_json::Value,
    root_key: &str,
) -> Result<String, ProvisioningError> {
    let mut doc: serde_json::Value = if existing.is_empty() {
        serde_json::json!({})
    } else {
        serde_json::from_str(existing)?
    };

    // Check for existing key with different content
    if let Some(existing_server) = doc.get(root_key).and_then(|s| s.get(server_name)) {
        let existing_cmd = existing_server.get("command");
        let new_cmd = server_config.get("command");
        if existing_cmd != new_cmd {
            return Err(ProvisioningError::McpServerConflict {
                path: std::path::PathBuf::new(), // caller sets this
                key: server_name.to_string(),
            });
        }
    }

    // Ensure root key exists
    if doc.get(root_key).is_none() {
        doc[root_key] = serde_json::json!({});
    }

    // Set our server entry
    doc[root_key][server_name] = server_config.clone();

    // Pretty-print with trailing newline
    let output = serde_json::to_string_pretty(&doc)?;
    Ok(format!("{}\n", output))
}

/// Remove an MCP server entry from a JSON config file (surgical rollback).
pub fn json_remove_mcp_server(
    existing: &str,
    server_name: &str,
    root_key: &str,
) -> Result<String, ProvisioningError> {
    let mut doc: serde_json::Value = serde_json::from_str(existing)?;

    if let Some(servers) = doc.get_mut(root_key).and_then(|s| s.as_object_mut()) {
        servers.remove(server_name);
    }

    let output = serde_json::to_string_pretty(&doc)?;
    Ok(format!("{}\n", output))
}

// ── TOML Operations ──

/// Append or update an MCP server section in a TOML config file (Codex CLI).
pub fn toml_append_mcp_server(
    existing: &str,
    server_name: &str,
    command: &str,
    args: &[String],
    env: &std::collections::HashMap<String, String>,
    tally_version: &str,
) -> Result<String, ProvisioningError> {
    use toml_edit::{Array, DocumentMut, Item, Table};

    let mut doc: DocumentMut = if existing.is_empty() {
        DocumentMut::new()
    } else {
        existing
            .parse::<DocumentMut>()
            .map_err(|e| ProvisioningError::TomlParse(e.to_string()))?
    };

    // Ensure [mcp_servers] table exists
    if !doc.contains_key("mcp_servers") {
        doc["mcp_servers"] = Item::Table(Table::new());
    }

    // Build server entry
    let server_table = &mut doc["mcp_servers"][server_name];
    *server_table = Item::Table(Table::new());

    if let Some(table) = server_table.as_table_mut() {
        table.decor_mut().set_prefix(format!(
            "\n# tally-wallet v{} (managed by Tally Agentic Wallet)\n",
            tally_version
        ));
        table.insert("command", toml_edit::value(command));

        let mut arr = Array::new();
        for arg in args {
            arr.push(arg.as_str());
        }
        table.insert("args", toml_edit::value(arr));

        if !env.is_empty() {
            let mut env_table = Table::new();
            for (k, v) in env {
                env_table.insert(k, toml_edit::value(v.as_str()));
            }
            table.insert("env", Item::Table(env_table));
        }
    }

    Ok(doc.to_string())
}

/// Remove our TOML section (surgical rollback).
pub fn toml_remove_mcp_server(
    existing: &str,
    server_name: &str,
) -> Result<String, ProvisioningError> {
    use toml_edit::DocumentMut;

    let mut doc: DocumentMut = existing
        .parse::<DocumentMut>()
        .map_err(|e| ProvisioningError::TomlParse(e.to_string()))?;

    if let Some(servers) = doc.get_mut("mcp_servers").and_then(|s| s.as_table_mut()) {
        servers.remove(server_name);
    }

    Ok(doc.to_string())
}

// ── Markdown Sentinel Operations ──

/// Append or replace content between sentinel markers in a markdown file.
pub fn markdown_upsert_section(
    existing: &str,
    content: &str,
    tally_version: &str,
) -> Result<String, ProvisioningError> {
    let start_marker = format!("{} v{} -->", SENTINEL_START, tally_version);
    let end_marker = SENTINEL_END.to_string();

    let section = format!("{}\n{}\n{}", start_marker, content.trim(), end_marker);

    // Check if sentinel markers already exist (any version)
    if let (Some(start_idx), Some(end_idx)) = (
        existing.find(SENTINEL_START),
        existing.find(SENTINEL_END),
    ) {
        if start_idx < end_idx {
            // Replace existing section
            let before = &existing[..start_idx];
            let after = &existing[end_idx + SENTINEL_END.len()..];
            return Ok(format!(
                "{}\n\n{}\n{}",
                before.trim_end(),
                section,
                after.trim_start()
            ));
        }
    }

    // Append to end
    if existing.is_empty() {
        Ok(format!("{}\n", section))
    } else {
        Ok(format!("{}\n\n{}\n", existing.trim_end(), section))
    }
}

/// Remove content between sentinel markers (surgical rollback).
pub fn markdown_remove_section(existing: &str) -> Result<String, ProvisioningError> {
    if let (Some(start_idx), Some(end_idx)) = (
        existing.find(SENTINEL_START),
        existing.find(SENTINEL_END),
    ) {
        if start_idx < end_idx {
            let before = &existing[..start_idx];
            let after = &existing[end_idx + SENTINEL_END.len()..];
            let result = format!("{}{}", before.trim_end(), after.trim_start());
            let trimmed = result.trim();
            if trimmed.is_empty() {
                return Ok(String::new());
            }
            return Ok(format!("{}\n", trimmed));
        }
    }

    // Sentinels not found — nothing to remove, not an error
    Ok(existing.to_string())
}

// ── YAML Operations ──

/// Merge an MCP server entry into a Continue.dev config.yaml list.
pub fn yaml_merge_mcp_server_list(
    existing: &str,
    server_entry: &serde_yaml::Value,
) -> Result<String, ProvisioningError> {
    let mut doc: serde_yaml::Value = if existing.is_empty() {
        serde_yaml::Value::Mapping(serde_yaml::Mapping::new())
    } else {
        serde_yaml::from_str(existing).map_err(|e| ProvisioningError::Yaml(e.to_string()))?
    };

    let mapping = doc
        .as_mapping_mut()
        .ok_or_else(|| ProvisioningError::Yaml("Root is not a YAML mapping".into()))?;

    let key = serde_yaml::Value::String("mcpServers".to_string());
    let servers = mapping
        .entry(key)
        .or_insert(serde_yaml::Value::Sequence(vec![]));

    if let serde_yaml::Value::Sequence(ref mut seq) = servers {
        // Check if tally-wallet already in list
        let existing_idx = seq.iter().position(|s| {
            s.get("name")
                .and_then(|n| n.as_str())
                .map(|n| n == "tally-wallet")
                .unwrap_or(false)
        });

        if let Some(idx) = existing_idx {
            seq[idx] = server_entry.clone();
        } else {
            seq.push(server_entry.clone());
        }
    }

    serde_yaml::to_string(&doc).map_err(|e| ProvisioningError::Yaml(e.to_string()))
}

/// Merge a read entry into Aider's .aider.conf.yml.
pub fn yaml_merge_read_entry(
    existing: &str,
    file_path: &str,
) -> Result<String, ProvisioningError> {
    let mut doc: serde_yaml::Value = if existing.is_empty() {
        serde_yaml::Value::Mapping(serde_yaml::Mapping::new())
    } else {
        serde_yaml::from_str(existing).map_err(|e| ProvisioningError::Yaml(e.to_string()))?
    };

    let mapping = doc
        .as_mapping_mut()
        .ok_or_else(|| ProvisioningError::Yaml("Root is not a YAML mapping".into()))?;

    let key = serde_yaml::Value::String("read".to_string());
    let reads = mapping
        .entry(key)
        .or_insert(serde_yaml::Value::Sequence(vec![]));

    if let serde_yaml::Value::Sequence(ref mut seq) = reads {
        let entry = serde_yaml::Value::String(file_path.to_string());
        if !seq.contains(&entry) {
            seq.push(entry);
        }
    }

    serde_yaml::to_string(&doc).map_err(|e| ProvisioningError::Yaml(e.to_string()))
}

/// Remove tally-wallet entry from YAML list (surgical rollback for Continue.dev).
pub fn yaml_remove_mcp_server_list(existing: &str) -> Result<String, ProvisioningError> {
    let mut doc: serde_yaml::Value =
        serde_yaml::from_str(existing).map_err(|e| ProvisioningError::Yaml(e.to_string()))?;

    if let Some(mapping) = doc.as_mapping_mut() {
        let key = serde_yaml::Value::String("mcpServers".to_string());
        if let Some(serde_yaml::Value::Sequence(ref mut seq)) = mapping.get_mut(&key) {
            seq.retain(|s| {
                s.get("name")
                    .and_then(|n| n.as_str())
                    .map(|n| n != "tally-wallet")
                    .unwrap_or(true)
            });
        }
    }

    serde_yaml::to_string(&doc).map_err(|e| ProvisioningError::Yaml(e.to_string()))
}

/// Remove tally-wallet read entry from Aider config (surgical rollback).
pub fn yaml_remove_read_entry(
    existing: &str,
    file_path: &str,
) -> Result<String, ProvisioningError> {
    let mut doc: serde_yaml::Value =
        serde_yaml::from_str(existing).map_err(|e| ProvisioningError::Yaml(e.to_string()))?;

    if let Some(mapping) = doc.as_mapping_mut() {
        let key = serde_yaml::Value::String("read".to_string());
        if let Some(serde_yaml::Value::Sequence(ref mut seq)) = mapping.get_mut(&key) {
            seq.retain(|s| s.as_str().map(|s| s != file_path).unwrap_or(true));
        }
    }

    serde_yaml::to_string(&doc).map_err(|e| ProvisioningError::Yaml(e.to_string()))
}

// ── Standalone File Operations ──

/// Create a standalone file that we fully own (no merge needed).
pub fn create_standalone_file(path: &Path, content: &str) -> Result<(), ProvisioningError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let parent = path.parent().unwrap_or(Path::new("."));
    let mut temp = tempfile::NamedTempFile::new_in(parent)?;
    temp.write_all(content.as_bytes())?;
    temp.flush()?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        temp.as_file()
            .set_permissions(std::fs::Permissions::from_mode(0o644))?;
    }

    temp.persist(path).map_err(|e| ProvisioningError::AtomicWriteFailed {
        path: path.to_path_buf(),
        reason: e.error.to_string(),
    })?;

    Ok(())
}

/// Delete a standalone file that we fully own.
pub fn delete_standalone_file(path: &Path) -> Result<(), ProvisioningError> {
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

// ── Helpers ──

/// Compute SHA-256 hex digest of bytes.
pub fn sha256_hex(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let hash = Sha256::digest(data);
    format!("{:x}", hash)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tempfile::TempDir;

    // ── atomic_modify ──

    #[test]
    fn atomic_modify_creates_new_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("new_config.json");

        let (original, modified) =
            atomic_modify(&path, |contents| Ok(format!("{{\"added\": true, \"was\": \"{}\"}}", contents)))
                .unwrap();

        assert_eq!(original, "");
        assert!(modified.contains("\"added\": true"));
        assert!(path.exists());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), modified);
    }

    #[test]
    fn atomic_modify_modifies_existing_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("existing.txt");
        std::fs::write(&path, "original content").unwrap();

        let (original, modified) =
            atomic_modify(&path, |contents| Ok(contents.to_uppercase())).unwrap();

        assert_eq!(original, "original content");
        assert_eq!(modified, "ORIGINAL CONTENT");
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "ORIGINAL CONTENT");
    }

    #[test]
    fn atomic_modify_creates_parent_dirs() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("deep").join("nested").join("dir").join("file.txt");

        let (original, _modified) =
            atomic_modify(&path, |_| Ok("hello".to_string())).unwrap();

        assert_eq!(original, "");
        assert!(path.exists());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "hello");
    }

    #[test]
    fn atomic_modify_propagates_closure_error() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("fail.txt");

        let result = atomic_modify(&path, |_| {
            Err(ProvisioningError::Internal("test error".into()))
        });

        assert!(result.is_err());
        assert!(!path.exists());
    }

    // ── json_merge_mcp_server ──

    #[test]
    fn json_merge_empty_input_creates_structure() {
        let config = serde_json::json!({"command": "tally-mcp", "args": ["serve"]});
        let result = json_merge_mcp_server("", "tally-wallet", &config, "mcpServers").unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["mcpServers"]["tally-wallet"]["command"], "tally-mcp");
        assert_eq!(parsed["mcpServers"]["tally-wallet"]["args"][0], "serve");
    }

    #[test]
    fn json_merge_adds_to_existing() {
        let existing = r#"{"mcpServers": {"other-tool": {"command": "other"}}}"#;
        let config = serde_json::json!({"command": "tally-mcp"});
        let result =
            json_merge_mcp_server(existing, "tally-wallet", &config, "mcpServers").unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["mcpServers"]["other-tool"]["command"], "other");
        assert_eq!(parsed["mcpServers"]["tally-wallet"]["command"], "tally-mcp");
    }

    #[test]
    fn json_merge_idempotent_same_command() {
        let config = serde_json::json!({"command": "tally-mcp", "args": ["serve"]});
        let first = json_merge_mcp_server("", "tally-wallet", &config, "mcpServers").unwrap();
        let second =
            json_merge_mcp_server(&first, "tally-wallet", &config, "mcpServers").unwrap();

        let p1: serde_json::Value = serde_json::from_str(&first).unwrap();
        let p2: serde_json::Value = serde_json::from_str(&second).unwrap();
        assert_eq!(p1, p2);
    }

    #[test]
    fn json_merge_conflict_different_command() {
        let existing =
            r#"{"mcpServers": {"tally-wallet": {"command": "old-binary"}}}"#;
        let config = serde_json::json!({"command": "new-binary"});
        let result =
            json_merge_mcp_server(existing, "tally-wallet", &config, "mcpServers");

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("tally-wallet"));
    }

    #[test]
    fn json_merge_uses_custom_root_key() {
        let config = serde_json::json!({"command": "tally-mcp"});
        let result = json_merge_mcp_server("", "tally-wallet", &config, "servers").unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["servers"]["tally-wallet"]["command"], "tally-mcp");
        assert!(parsed.get("mcpServers").is_none());
    }

    #[test]
    fn json_merge_trailing_newline() {
        let config = serde_json::json!({"command": "x"});
        let result = json_merge_mcp_server("", "s", &config, "mcpServers").unwrap();
        assert!(result.ends_with('\n'));
    }

    // ── json_remove_mcp_server ──

    #[test]
    fn json_remove_existing_server() {
        let existing = r#"{
  "mcpServers": {
    "tally-wallet": {"command": "tally-mcp"},
    "other": {"command": "other"}
  }
}"#;
        let result = json_remove_mcp_server(existing, "tally-wallet", "mcpServers").unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert!(parsed["mcpServers"].get("tally-wallet").is_none());
        assert_eq!(parsed["mcpServers"]["other"]["command"], "other");
    }

    #[test]
    fn json_remove_noop_if_absent() {
        let existing = r#"{"mcpServers": {"other": {"command": "other"}}}"#;
        let result = json_remove_mcp_server(existing, "tally-wallet", "mcpServers").unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["mcpServers"]["other"]["command"], "other");
    }

    #[test]
    fn json_remove_preserves_other_keys() {
        let existing = r#"{"mcpServers": {"tally-wallet": {}}, "otherConfig": true}"#;
        let result = json_remove_mcp_server(existing, "tally-wallet", "mcpServers").unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["otherConfig"], true);
    }

    // ── toml_append_mcp_server ──

    #[test]
    fn toml_append_empty_input() {
        let env = HashMap::new();
        let args = vec!["serve".to_string()];
        let result =
            toml_append_mcp_server("", "tally-wallet", "tally-mcp", &args, &env, "1.0.0")
                .unwrap();

        assert!(result.contains("[mcp_servers.tally-wallet]"));
        assert!(result.contains("command = \"tally-mcp\""));
        assert!(result.contains("\"serve\""));
    }

    #[test]
    fn toml_append_to_existing() {
        let existing = r#"
[mcp_servers.other-tool]
command = "other"
args = []
"#;
        let env = HashMap::new();
        let args = vec!["serve".to_string()];
        let result = toml_append_mcp_server(
            existing,
            "tally-wallet",
            "tally-mcp",
            &args,
            &env,
            "1.0.0",
        )
        .unwrap();

        assert!(result.contains("[mcp_servers.other-tool]"));
        assert!(result.contains("[mcp_servers.tally-wallet]"));
        assert!(result.contains("command = \"tally-mcp\""));
    }

    #[test]
    fn toml_append_with_env_table() {
        let mut env = HashMap::new();
        env.insert("API_KEY".to_string(), "secret123".to_string());
        let args = vec![];
        let result =
            toml_append_mcp_server("", "tally-wallet", "cmd", &args, &env, "2.0.0").unwrap();

        assert!(result.contains("[mcp_servers.tally-wallet.env]"));
        assert!(result.contains("API_KEY = \"secret123\""));
    }

    #[test]
    fn toml_append_idempotent_upsert() {
        let env = HashMap::new();
        let args = vec!["serve".to_string()];
        let first =
            toml_append_mcp_server("", "tally-wallet", "tally-mcp", &args, &env, "1.0.0")
                .unwrap();
        let second = toml_append_mcp_server(
            &first,
            "tally-wallet",
            "tally-mcp-v2",
            &args,
            &env,
            "2.0.0",
        )
        .unwrap();

        // Should have updated, not duplicated
        assert!(second.contains("tally-mcp-v2"));
        // Count occurrences of [mcp_servers.tally-wallet] -- should be exactly 1
        let count = second.matches("[mcp_servers.tally-wallet]").count();
        assert_eq!(count, 1, "Should have exactly one tally-wallet section");
    }

    #[test]
    fn toml_append_includes_version_comment() {
        let env = HashMap::new();
        let args = vec![];
        let result =
            toml_append_mcp_server("", "tw", "cmd", &args, &env, "3.2.1").unwrap();

        assert!(result.contains("tally-wallet v3.2.1"));
    }

    // ── toml_remove_mcp_server ──

    #[test]
    fn toml_remove_existing() {
        let existing = r#"
[mcp_servers.tally-wallet]
command = "tally-mcp"
args = ["serve"]

[mcp_servers.other-tool]
command = "other"
args = []
"#;
        let result = toml_remove_mcp_server(existing, "tally-wallet").unwrap();

        assert!(!result.contains("tally-wallet"));
        assert!(result.contains("[mcp_servers.other-tool]"));
        assert!(result.contains("command = \"other\""));
    }

    #[test]
    fn toml_remove_noop_if_absent() {
        let existing = r#"
[mcp_servers.other-tool]
command = "other"
"#;
        let result = toml_remove_mcp_server(existing, "tally-wallet").unwrap();

        assert!(result.contains("[mcp_servers.other-tool]"));
    }

    #[test]
    fn toml_remove_preserves_other_sections() {
        let existing = r#"
[general]
debug = true

[mcp_servers.tally-wallet]
command = "tally-mcp"
"#;
        let result = toml_remove_mcp_server(existing, "tally-wallet").unwrap();

        assert!(!result.contains("tally-wallet"));
        assert!(result.contains("[general]"));
        assert!(result.contains("debug = true"));
    }

    // ── markdown_upsert_section ──

    #[test]
    fn markdown_upsert_empty_file() {
        let result = markdown_upsert_section("", "# Tally Wallet\nUse this tool.", "1.0.0").unwrap();

        assert!(result.contains("<!-- TALLY_WALLET_START v1.0.0 -->"));
        assert!(result.contains("# Tally Wallet"));
        assert!(result.contains("Use this tool."));
        assert!(result.contains("<!-- TALLY_WALLET_END -->"));
        assert!(result.ends_with('\n'));
    }

    #[test]
    fn markdown_upsert_appends_to_nonempty() {
        let existing = "# My Skills\n\nSome existing content.";
        let result =
            markdown_upsert_section(existing, "Tally info here", "1.0.0").unwrap();

        assert!(result.starts_with("# My Skills"));
        assert!(result.contains("Some existing content."));
        assert!(result.contains("<!-- TALLY_WALLET_START v1.0.0 -->"));
        assert!(result.contains("Tally info here"));
    }

    #[test]
    fn markdown_upsert_replaces_existing_sentinel() {
        let existing = "# Header\n\n<!-- TALLY_WALLET_START v0.9.0 -->\nOld content\n<!-- TALLY_WALLET_END -->\n\n# Footer";
        let result =
            markdown_upsert_section(existing, "New content", "1.0.0").unwrap();

        assert!(result.contains("<!-- TALLY_WALLET_START v1.0.0 -->"));
        assert!(result.contains("New content"));
        assert!(!result.contains("Old content"));
        assert!(!result.contains("v0.9.0"));
        assert!(result.contains("# Header"));
        assert!(result.contains("# Footer"));
    }

    #[test]
    fn markdown_upsert_trims_content() {
        let result = markdown_upsert_section("", "  \n  padded  \n  ", "1.0.0").unwrap();

        // Content between sentinels should be trimmed
        assert!(result.contains("padded"));
    }

    // ── markdown_remove_section ──

    #[test]
    fn markdown_remove_sentinel_block() {
        let existing = "# Header\n\n<!-- TALLY_WALLET_START v1.0.0 -->\nTally stuff\n<!-- TALLY_WALLET_END -->\n\n# Footer";
        let result = markdown_remove_section(existing).unwrap();

        assert!(!result.contains("TALLY_WALLET"));
        assert!(!result.contains("Tally stuff"));
        assert!(result.contains("# Header"));
        assert!(result.contains("# Footer"));
    }

    #[test]
    fn markdown_remove_noop_if_no_sentinels() {
        let existing = "# Just a normal markdown file\n\nNo tally here.";
        let result = markdown_remove_section(existing).unwrap();

        assert_eq!(result, existing);
    }

    #[test]
    fn markdown_remove_only_sentinel_content_returns_empty() {
        let existing =
            "<!-- TALLY_WALLET_START v1.0.0 -->\nOnly tally\n<!-- TALLY_WALLET_END -->";
        let result = markdown_remove_section(existing).unwrap();

        assert_eq!(result, "");
    }

    // ── yaml_merge_mcp_server_list ──

    #[test]
    fn yaml_merge_mcp_server_list_empty_input() {
        let entry = serde_yaml::to_value(serde_json::json!({
            "name": "tally-wallet",
            "command": "tally-mcp",
            "args": ["serve"]
        }))
        .unwrap();

        let result = yaml_merge_mcp_server_list("", &entry).unwrap();
        let parsed: serde_yaml::Value = serde_yaml::from_str(&result).unwrap();

        let servers = parsed["mcpServers"].as_sequence().unwrap();
        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0]["name"].as_str().unwrap(), "tally-wallet");
    }

    #[test]
    fn yaml_merge_mcp_server_list_adds_to_existing() {
        let existing = "mcpServers:\n  - name: other-tool\n    command: other\n";
        let entry = serde_yaml::to_value(serde_json::json!({
            "name": "tally-wallet",
            "command": "tally-mcp"
        }))
        .unwrap();

        let result = yaml_merge_mcp_server_list(existing, &entry).unwrap();
        let parsed: serde_yaml::Value = serde_yaml::from_str(&result).unwrap();

        let servers = parsed["mcpServers"].as_sequence().unwrap();
        assert_eq!(servers.len(), 2);
    }

    #[test]
    fn yaml_merge_mcp_server_list_replaces_existing_tally() {
        let existing =
            "mcpServers:\n  - name: tally-wallet\n    command: old-cmd\n  - name: other\n    command: other\n";
        let entry = serde_yaml::to_value(serde_json::json!({
            "name": "tally-wallet",
            "command": "new-cmd"
        }))
        .unwrap();

        let result = yaml_merge_mcp_server_list(existing, &entry).unwrap();
        let parsed: serde_yaml::Value = serde_yaml::from_str(&result).unwrap();

        let servers = parsed["mcpServers"].as_sequence().unwrap();
        assert_eq!(servers.len(), 2);
        let tally = servers.iter().find(|s| {
            s["name"].as_str() == Some("tally-wallet")
        }).unwrap();
        assert_eq!(tally["command"].as_str().unwrap(), "new-cmd");
    }

    // ── yaml_merge_read_entry ──

    #[test]
    fn yaml_merge_read_entry_adds_to_empty() {
        let result = yaml_merge_read_entry("", "/path/to/skill.md").unwrap();
        let parsed: serde_yaml::Value = serde_yaml::from_str(&result).unwrap();

        let reads = parsed["read"].as_sequence().unwrap();
        assert_eq!(reads.len(), 1);
        assert_eq!(reads[0].as_str().unwrap(), "/path/to/skill.md");
    }

    #[test]
    fn yaml_merge_read_entry_noop_if_present() {
        let existing = "read:\n  - /path/to/skill.md\n  - /other/file.md\n";
        let result = yaml_merge_read_entry(existing, "/path/to/skill.md").unwrap();
        let parsed: serde_yaml::Value = serde_yaml::from_str(&result).unwrap();

        let reads = parsed["read"].as_sequence().unwrap();
        assert_eq!(reads.len(), 2);
    }

    #[test]
    fn yaml_merge_read_entry_adds_new_path() {
        let existing = "read:\n  - /existing/file.md\n";
        let result = yaml_merge_read_entry(existing, "/new/file.md").unwrap();
        let parsed: serde_yaml::Value = serde_yaml::from_str(&result).unwrap();

        let reads = parsed["read"].as_sequence().unwrap();
        assert_eq!(reads.len(), 2);
    }

    // ── yaml_remove_mcp_server_list ──

    #[test]
    fn yaml_remove_mcp_server_list_removes_tally() {
        let existing =
            "mcpServers:\n  - name: tally-wallet\n    command: tally-mcp\n  - name: other\n    command: other\n";
        let result = yaml_remove_mcp_server_list(existing).unwrap();
        let parsed: serde_yaml::Value = serde_yaml::from_str(&result).unwrap();

        let servers = parsed["mcpServers"].as_sequence().unwrap();
        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0]["name"].as_str().unwrap(), "other");
    }

    #[test]
    fn yaml_remove_mcp_server_list_noop_if_absent() {
        let existing = "mcpServers:\n  - name: other\n    command: other\n";
        let result = yaml_remove_mcp_server_list(existing).unwrap();
        let parsed: serde_yaml::Value = serde_yaml::from_str(&result).unwrap();

        let servers = parsed["mcpServers"].as_sequence().unwrap();
        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0]["name"].as_str().unwrap(), "other");
    }

    // ── yaml_remove_read_entry ──

    #[test]
    fn yaml_remove_read_entry_removes_path() {
        let existing = "read:\n  - /path/to/skill.md\n  - /other/file.md\n";
        let result = yaml_remove_read_entry(existing, "/path/to/skill.md").unwrap();
        let parsed: serde_yaml::Value = serde_yaml::from_str(&result).unwrap();

        let reads = parsed["read"].as_sequence().unwrap();
        assert_eq!(reads.len(), 1);
        assert_eq!(reads[0].as_str().unwrap(), "/other/file.md");
    }

    #[test]
    fn yaml_remove_read_entry_noop_if_absent() {
        let existing = "read:\n  - /other/file.md\n";
        let result = yaml_remove_read_entry(existing, "/nonexistent.md").unwrap();
        let parsed: serde_yaml::Value = serde_yaml::from_str(&result).unwrap();

        let reads = parsed["read"].as_sequence().unwrap();
        assert_eq!(reads.len(), 1);
    }

    // ── create_standalone_file ──

    #[test]
    fn create_standalone_file_writes_content() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("skill.md");

        create_standalone_file(&path, "# Tally Wallet Skill").unwrap();

        assert!(path.exists());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "# Tally Wallet Skill");
    }

    #[test]
    fn create_standalone_file_creates_parent_dirs() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("deep").join("nested").join("skill.md");

        create_standalone_file(&path, "content").unwrap();

        assert!(path.exists());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "content");
    }

    // ── delete_standalone_file ──

    #[test]
    fn delete_standalone_file_removes_existing() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("to_delete.txt");
        std::fs::write(&path, "data").unwrap();
        assert!(path.exists());

        delete_standalone_file(&path).unwrap();

        assert!(!path.exists());
    }

    #[test]
    fn delete_standalone_file_noop_if_absent() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("nonexistent.txt");

        // Should not error
        delete_standalone_file(&path).unwrap();
    }

    // ── sha256_hex ──

    #[test]
    fn sha256_hex_known_vector() {
        // SHA-256 of "hello" is well-known
        let hash = sha256_hex(b"hello");
        assert_eq!(
            hash,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn sha256_hex_empty_input() {
        // SHA-256 of empty string
        let hash = sha256_hex(b"");
        assert_eq!(
            hash,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn sha256_hex_deterministic() {
        let a = sha256_hex(b"test data");
        let b = sha256_hex(b"test data");
        assert_eq!(a, b);
    }
}
