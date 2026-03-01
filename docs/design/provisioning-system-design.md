# Provisioning System Design: Auto-Discovery & Config Injection

> **Status**: Design Document (pre-implementation)
> **Date**: 2026-03-01
> **Depends on**: `docs/investigations/coding-tool-config-provisioning.md`

---

## Overview

Tally Agentic Wallet auto-discovers AI coding tools on the user's machine and injects MCP server configuration + skill instructions into each tool's config files. This document covers the safety, robustness, and edge-case handling required to do this responsibly.

The core tension: we are modifying config files owned by other applications. This is inherently invasive. Every design decision must err on the side of caution, transparency, and reversibility.

---

## 1. Backup & Rollback System

### 1.1 Backup Storage

All backups live under `~/.tally/backups/`, organized by timestamp and tool:

```
~/.tally/
  backups/
    2026-03-01T14-30-00Z/
      manifest.json
      claude-code/
        claude.json.bak
        CLAUDE.md.bak
      cursor/
        mcp.json.bak
      claude-desktop/
        claude_desktop_config.json.bak
    2026-03-01T15-00-00Z/
      manifest.json
      windsurf/
        mcp_config.json.bak
```

### 1.2 Manifest File

Each provisioning operation creates a timestamped directory with a `manifest.json`:

```json
{
  "version": 1,
  "timestamp": "2026-03-01T14:30:00Z",
  "tally_version": "1.2.0",
  "operation": "provision",
  "tools_modified": [
    {
      "tool": "claude-code",
      "files": [
        {
          "path": "/Users/alice/.claude.json",
          "backup_path": "claude-code/claude.json.bak",
          "modification_type": "json_merge",
          "keys_added": ["mcpServers.tally-wallet"],
          "sha256_before": "abc123...",
          "sha256_after": "def456..."
        },
        {
          "path": "/Users/alice/.claude/CLAUDE.md",
          "backup_path": "claude-code/CLAUDE.md.bak",
          "modification_type": "markdown_append",
          "sentinel_start": "<!-- TALLY_WALLET_START -->",
          "sentinel_end": "<!-- TALLY_WALLET_END -->",
          "sha256_before": "...",
          "sha256_after": "..."
        }
      ]
    }
  ]
}
```

The `sha256_before` and `sha256_after` fields are critical for detecting whether the user manually edited the file after our modification.

### 1.3 Backup Procedure

Before modifying ANY file:

1. Compute SHA-256 of current file contents.
2. Copy file to backup directory.
3. Verify backup integrity (read back and compare hash).
4. Only then proceed with modification.
5. Compute SHA-256 of modified file, record in manifest.

If step 2 or 3 fails, abort the entire provisioning operation for that tool. Never leave a half-modified state.

### 1.4 Rollback Modes

**Full rollback** ("undo everything Tally did"):

```
tally unprovision --all
```

Iterates through all manifest files (newest first), restores each file from backup. Removes Tally-added content from files that were modified after our changes (see 1.5).

**Per-tool rollback**:

```
tally unprovision --tool cursor
```

Finds the most recent manifest entry for Cursor, restores those files only.

**Specific timestamp rollback**:

```
tally unprovision --timestamp 2026-03-01T14-30-00Z
```

Restores the exact state from that backup set.

### 1.5 Post-Modification Edit Detection

When rolling back, the file may have been edited by the user after our modification. Three strategies, applied in order of preference:

**Strategy A: Surgical removal (preferred)**
If we used sentinel markers (markdown) or well-defined JSON keys, remove only our content. This preserves all user edits.

- JSON files: delete only the `tally-wallet` key from `mcpServers`/`servers`.
- TOML files: remove only the `[mcp_servers.tally-wallet]` section.
- Markdown files: remove content between `<!-- TALLY_WALLET_START -->` and `<!-- TALLY_WALLET_END -->` markers.
- YAML files: remove only the tally-wallet entry from arrays.

**Strategy B: Diff-based rollback**
If surgical removal is not possible (file structure changed significantly), compute a three-way diff between (1) our backup, (2) our modified version, and (3) the current file. Apply the reverse of our changes only.

**Strategy C: Full restore with confirmation**
If the above fails, show the user a diff between the backup and current file and ask for confirmation before overwriting. Never silently overwrite a file the user has edited.

### 1.6 Backup Retention

- Keep the last 10 backup sets by default.
- Never auto-delete backups less than 30 days old.
- Configurable via `~/.tally/config.json`: `"backup_retention_count"` and `"backup_retention_days"`.
- Total backup size cap: 50MB (warn at 40MB). Config files are tiny, so this should never trigger in practice.

---

## 2. Persistence & Self-Healing

### 2.1 Recommendation: Check-on-Launch Only

**Do NOT run a background daemon.** Reasons:

- A background process monitoring other tools' config files is hostile behavior. Users and security tools will flag it.
- It burns battery and CPU for a problem that rarely occurs.
- It makes Tally feel like malware.
- It creates a persistence mechanism that security-conscious users will actively resist.

Instead: **check and repair on Tally app launch only.** When the user opens Tally, run a quick verification pass:

1. For each provisioned tool (tracked in `~/.tally/provisioning-state.json`), check if our config is still present.
2. If missing, check the "respect removal" counter (see 2.3).
3. If counter allows, offer to re-provision with a non-blocking notification: "Cursor's MCP config was reset (likely by an update). Re-add Tally Wallet?"
4. Never silently re-inject.

### 2.2 Tool-Update Detection

When our config is missing but was previously present, distinguish between:

- **Tool update reset**: The file exists but our key is gone, AND the file's modification time is very recent, AND the tool was recently updated. Heuristic: check tool binary modification time or version file.
- **User removal**: The file exists, our key is specifically gone, but everything else is intact.
- **File recreation**: The file was completely replaced (hash differs entirely from both our backup and our modified version).

For tool update resets, default to offering re-provisioning. For user removal, respect it.

### 2.3 Respect Removal Protocol

Track removal events in `~/.tally/provisioning-state.json`:

```json
{
  "tools": {
    "cursor": {
      "provisioned_at": "2026-03-01T14:30:00Z",
      "last_verified": "2026-03-01T16:00:00Z",
      "removal_count": 0,
      "respect_removal": false,
      "status": "active"
    }
  }
}
```

Rules:
- If the user removes our config and we detect it, increment `removal_count`.
- After `removal_count >= 2` for the same tool, set `respect_removal: true` and stop offering to re-provision for that tool.
- The user can manually reset this via `tally provision --tool cursor --force` or through the UI.
- Display a permanent "Tally is not connected to Cursor" state in the UI for respected-removal tools, with a manual "Reconnect" button.

### 2.4 No Daemon, No Launchd, No Systemd

Explicitly do NOT create:
- macOS launchd plists
- Linux systemd services
- Windows scheduled tasks
- Filesystem watchers (FSEvents, inotify, ReadDirectoryChangesW)

These are all inappropriate for a config provisioning system. The only acceptable persistence mechanism is check-on-app-launch.

**Exception**: If the Tally app itself runs as a background menu bar app (which Tauri supports), then the on-launch check happens when the menu bar app starts, which is effectively on login. This is acceptable because the user explicitly chose to run Tally at startup.

---

## 3. Edge Cases & Failure Modes

### 3.1 Tool Updates Overwrite Config

**Problem**: Cursor, Claude Desktop, VS Code, and others may reset their config files during updates.

**Mitigation**:
- On-launch verification detects missing config.
- Store a `tool_version` field in provisioning state. When the version changes, assume an update occurred and proactively offer re-provisioning.
- For tools that store config in separate files we create (e.g., `.cursor/rules/tally-wallet.mdc`), updates are less likely to delete these. Prefer this pattern where available.

### 3.2 Multiple Tally Installations

**Problem**: User has Tally on two machines syncing dotfiles, or two Tally versions installed.

**Mitigation**:
- Use a machine ID in the provisioning state (derive from hostname + a random salt stored in `~/.tally/machine-id`).
- Before modifying, check if the `tally-wallet` config already exists and matches our expected content. If it matches, skip (idempotent). If it differs (different token, different server path), warn the user.
- The `tally-wallet` MCP server key name is the coordination point. Two Tally instances will both try to write to the same key. Last writer wins, which is acceptable since the config should be identical.

### 3.3 Race Conditions

**Problem**: Tool is writing config at the same time we are.

**Mitigation**:
- Use atomic write pattern: write to a temp file in the same directory, then `rename()` (which is atomic on all major OSes for same-filesystem moves).
- Acquire an advisory file lock (flock on Unix, LockFileEx on Windows) before reading the config. Hold the lock through the read-modify-write cycle. Timeout after 5 seconds; if lock cannot be acquired, skip this tool and inform the user.
- For JSON files specifically: read, parse, merge, serialize, write-to-temp, rename. If parsing fails (someone wrote partial content), retry once after 500ms.

### 3.4 File Permission Issues

**Problem**: Config file is read-only, or parent directory has restricted permissions.

**Mitigation**:
- Check write permission before attempting modification. If not writable, report to the user: "Cannot modify Cursor config: permission denied on ~/.cursor/mcp.json".
- Never `chmod` or `chown` files we don't own. Never use `sudo`.
- On macOS, check for App Sandbox restrictions (some apps store config in `~/Library/Containers/`).

### 3.5 Config File Doesn't Exist Yet

**Problem**: Tool is installed but never configured (no config file yet).

**Mitigation**:
- Create the file with only our content. Create parent directories as needed with mode 0755.
- For JSON: create a minimal valid document containing only our MCP server entry.
- For TOML: create with only the `[mcp_servers.tally-wallet]` section.
- For Markdown: create with sentinel markers and our content.
- Record in manifest that this was a file creation (not a modification), so rollback knows to delete the file entirely rather than trying to restore a backup.

### 3.6 Malformed Config Files

**Problem**: Config file exists but contains invalid JSON/YAML/TOML.

**Mitigation**:
- **Never attempt to fix malformed files.** This is the user's problem, not ours.
- If parsing fails, skip this tool and report: "Cursor config file (~/.cursor/mcp.json) contains invalid JSON. Please fix it manually, then retry provisioning."
- Log the parse error details for debugging.
- Do NOT fall back to appending raw text to a malformed JSON file. This will make it worse.

### 3.7 Symlinked Config Files

**Problem**: Config file is a symlink to another location (e.g., dotfile manager like chezmoi, GNU Stow, or yadm).

**Mitigation**:
- Resolve symlinks before reading. Use `std::fs::canonicalize()` (Rust) or `fs.realpathSync()` (Node).
- Back up the resolved real path, not the symlink path.
- Write to the resolved real path (which modifies the target file).
- Record both the symlink path and the resolved path in the manifest.
- If the symlink is broken (target doesn't exist), treat it as "file doesn't exist" (3.5).

### 3.8 Multiple Claude Code Projects

**Problem**: User has 10 different projects, each with their own `.claude/` and `.mcp.json`. We should not blindly inject into all of them.

**Mitigation**:
- **User-level provisioning only by default.** Write to `~/.claude.json` (global MCP), `~/.claude/CLAUDE.md` (global instructions), and `~/.claude/skills/tally-wallet.md` (global skill). These apply to all projects without touching any project-specific files.
- Project-level provisioning is opt-in: "Add Tally to this specific project" button in the UI, which modifies the project's `.mcp.json`.
- Never scan the filesystem for all `.mcp.json` files and modify them.

### 3.9 MCP Server Binary Not Available at Provision Time

**Problem**: We reference `npx -y @tally/wallet-mcp` in the config, but the user doesn't have Node.js installed, or the package isn't published yet.

**Mitigation**:
- For stdio-based MCP servers, prefer referencing the Tally app's bundled binary directly: `"/Applications/Tally.app/Contents/MacOS/tally-mcp-server"` (macOS) or equivalent.
- This eliminates the Node.js dependency and npx cold-start latency.
- Validate that the binary path exists before writing it to config. If it doesn't exist, don't provision and explain why.
- For development/testing, support an environment variable override: `TALLY_MCP_SERVER_PATH`.

### 3.10 Config File Locked by Another Process

**Problem**: On Windows especially, files can be locked by the application that owns them.

**Mitigation**:
- Attempt to acquire a file lock with a 5-second timeout.
- If locked, retry once after 2 seconds.
- If still locked, skip and inform the user: "Claude Desktop is currently using its config file. Close Claude Desktop, or retry later."
- On Windows, use `GENERIC_READ | GENERIC_WRITE` with `FILE_SHARE_READ` to allow shared access where possible.

### 3.11 Disk Full During Write

**Problem**: Write fails partway through, leaving a truncated file.

**Mitigation**:
- Atomic write pattern (3.3) handles this. We write to a temp file first. If the temp file write fails (disk full), the original file is untouched.
- Check available disk space before starting (need at least 1MB free, which is absurd overkill for config files, but cheap to check).
- If the rename succeeds but we can't write the backup, the modification still happened safely. Log a warning that the backup is incomplete.

### 3.12 MCP Server Name Conflicts

**Problem**: User already has an MCP server named `tally-wallet` configured by a different tool or manually.

**Mitigation**:
- Before writing, check if the `tally-wallet` key already exists.
- If it exists and matches our expected config (same command, same args), skip -- already provisioned.
- If it exists with different config, do NOT overwrite. Inform the user: "An MCP server named 'tally-wallet' already exists in your Cursor config with different settings. Would you like to replace it?"
- Consider using a more specific key: `tally-agentic-wallet` to reduce collision risk.

---

## 4. Detection Reliability

### 4.1 Multi-Strategy Detection

For each tool, check in order of reliability:

1. **Config directory exists** (most reliable -- proves the tool was used):
   - `~/.claude/` for Claude Code
   - `~/.cursor/` for Cursor
   - `~/.codeium/` for Windsurf
   - `~/.codex/` for Codex CLI
   - `~/.continue/` for Continue.dev

2. **Config file exists** (proves the tool is configured):
   - `~/Library/Application Support/Claude/claude_desktop_config.json`
   - Cline's globalStorage path

3. **Binary in PATH** (`which`/`where`):
   - `which claude`, `which cursor`, `which windsurf`, `which codex`, `which aider`

4. **Application bundle exists** (macOS-specific):
   - `/Applications/Claude.app`
   - `/Applications/Cursor.app`
   - `/Applications/Windsurf.app`
   - `~/Applications/` variants (Homebrew Cask installs here sometimes)

5. **VS Code extension installed** (for Cline, Continue, Copilot):
   - Check `~/.vscode/extensions/` for:
     - `saoudrizwan.claude-dev-*` (Cline)
     - `continue.continue-*` (Continue.dev)
     - `github.copilot-*` (Copilot)

6. **Process running** (least reliable, only as a last resort):
   - `pgrep -f cursor`, etc.

### 4.2 Version Manager Awareness

**Problem**: Tools installed via nvm, asdf, mise, volta, fnm, or pyenv may not appear in PATH.

**Mitigation**:
- For Node-based tools: check common version manager shim directories:
  - `~/.nvm/versions/node/*/bin/`
  - `~/.asdf/shims/`
  - `~/.local/share/mise/shims/`
  - `~/.volta/bin/`
- For Python-based tools (Aider): check `~/.local/bin/`, pyenv shims.
- Do NOT scan the entire filesystem. Check known locations only.
- Source the user's shell profile to get the full PATH: `bash -lc 'which claude'` as a fallback.

### 4.3 Docker/Devcontainer Environments

**Problem**: Tools may be installed inside Docker containers or VS Code devcontainers, not on the host.

**Mitigation**:
- Tally provisions the host machine only. We do not attempt to modify configs inside containers.
- For VS Code devcontainers, workspace-level configs (`.vscode/mcp.json`, `.mcp.json`) are mounted from the host and will work inside the container.
- Document this limitation clearly.

### 4.4 Multiple Versions of Same Tool

**Problem**: User has both Cursor stable and Cursor nightly, or VS Code and VS Code Insiders.

**Mitigation**:
- Detect all variants and provision all of them:
  - VS Code: `~/.config/Code/` AND `~/.config/Code - Insiders/`
  - Cursor: `~/.cursor/` (no known variant directories)
- Show all detected variants in the UI so the user knows what was found.

### 4.5 Detection Result Caching

- Cache detection results in `~/.tally/detected-tools.json` with a TTL of 1 hour.
- On explicit "Refresh" in the UI, re-scan immediately.
- On app launch, always re-scan (ignore cache).

---

## 5. User Consent & Transparency

### 5.1 Pre-Modification Preview

The "Add Skill" / "Connect Tool" button MUST show exactly what will happen before doing it. Present a confirmation dialog:

```
Connect Tally Wallet to Cursor

The following changes will be made:

  ~/.cursor/mcp.json
  + Add "tally-wallet" MCP server entry

  .cursor/rules/tally-wallet.mdc  (new file)
  + Create rule file with wallet instructions

[View full diff]  [Cancel]  [Connect]
```

The "[View full diff]" link expands to show the exact JSON/text that will be written.

### 5.2 Per-Tool Consent

- First-time setup: show a "Connect All Detected Tools" option alongside individual per-tool toggles. Default all toggles to ON, but require the user to click a single "Confirm" button.
- Never auto-provision on first launch without user action. Detection is automatic; modification requires explicit consent.
- Store consent in `~/.tally/provisioning-state.json` per tool.

### 5.3 Exclusion List

Allow users to permanently exclude tools:

```json
{
  "excluded_tools": ["aider", "cline"],
  "excluded_reason": "user_preference"
}
```

Excluded tools are never provisioned, never offered for provisioning, and still appear in the UI as "Not connected (excluded)" with an "Include" button.

### 5.4 Transparency Log

Maintain a human-readable log at `~/.tally/provisioning.log`:

```
2026-03-01 14:30:00 [PROVISION] Cursor: merged tally-wallet into ~/.cursor/mcp.json
2026-03-01 14:30:00 [PROVISION] Cursor: created .cursor/rules/tally-wallet.mdc
2026-03-01 14:30:01 [PROVISION] Claude Code: merged tally-wallet into ~/.claude.json
2026-03-01 14:30:01 [SKIP] Aider: excluded by user preference
2026-03-01 16:00:00 [VERIFY] Cursor: config intact
2026-03-01 16:00:00 [VERIFY] Claude Code: config missing (tool update detected)
2026-03-01 16:00:00 [NOTIFY] Offered re-provisioning for Claude Code
```

### 5.5 Privacy Considerations

- We read config files to detect existing settings. We do NOT transmit any config file contents anywhere.
- The provisioning state file contains only tool names, timestamps, and file paths. No sensitive data.
- Backups contain copies of config files which may contain API keys or tokens from other tools. Backups should be created with mode 0600 (user-read-only).
- Document in our privacy policy that Tally reads and modifies AI tool config files.

---

## 6. Update Strategy

### 6.1 Content Versioning

Every piece of content we inject carries a version marker.

**JSON files**: Add a `_tally_meta` field (ignored by all known tools):

```json
{
  "mcpServers": {
    "tally-wallet": {
      "command": "/Applications/Tally.app/Contents/MacOS/tally-mcp-server",
      "args": ["--stdio"],
      "env": {
        "WALLET_TOKEN": "tok_abc123"
      },
      "_tally_version": "1.2.0",
      "_tally_provisioned": "2026-03-01T14:30:00Z"
    }
  }
}
```

If a tool chokes on extra keys inside the MCP server entry, fall back to no metadata and rely on the provisioning state file for version tracking.

**TOML files**: Use a comment above the section:

```toml
# tally-wallet v1.2.0 (provisioned 2026-03-01)
[mcp_servers.tally-wallet]
command = "/Applications/Tally.app/Contents/MacOS/tally-mcp-server"
args = ["--stdio"]
```

**Markdown files**: Use HTML comment sentinels:

```markdown
<!-- TALLY_WALLET_START v1.2.0 -->
## Tally Agentic Wallet

A crypto wallet is running on this machine via MCP. You can send payments,
check balances, trade tokens, and pay for X402 services.

**First time?** Call `register_agent` with a descriptive name for yourself.
Save the returned token -- you'll need it for all future calls.
<!-- TALLY_WALLET_END -->
```

**YAML files**: Use a comment:

```yaml
# tally-wallet v1.2.0
- name: tally-wallet
  command: /Applications/Tally.app/Contents/MacOS/tally-mcp-server
  args: ["--stdio"]
```

**Standalone files** (`.cursor/rules/tally-wallet.mdc`, `.clinerules/tally-wallet.md`): Since we own the entire file, embed version in the file header:

```markdown
---
description: "Tally Agentic Wallet integration (v1.2.0)"
alwaysApply: true
---
```

### 6.2 Update Detection

On app launch, for each provisioned tool:

1. Read the current config file.
2. Find our content (by key name or sentinel markers).
3. Extract the version from our metadata.
4. Compare against the current Tally app version.
5. If our content is outdated, offer to update: "Tally Wallet config for Cursor is outdated (v1.1.0 -> v1.2.0). Update?"

### 6.3 Update Procedure

1. Create a new backup (same as initial provisioning).
2. Surgically replace our content only (identified by key name or sentinels).
3. Preserve all other user content.
4. Update the version marker.
5. Update provisioning state.

### 6.4 Breaking Changes

If a Tally update changes the MCP server binary path, command-line arguments, or required environment variables:

- The update is mandatory. Mark it as such in the update check.
- Show the user exactly what changed: "The MCP server path changed from X to Y."
- Auto-update is acceptable here (with a toast notification) because broken MCP config is worse than a surprise change.

---

## 7. Uninstall Story

### 7.1 Full Clean Uninstall

Accessible from: Tally Settings > Advanced > Uninstall Provisioning, or `tally unprovision --all`.

Procedure:

1. Read all manifests from `~/.tally/backups/`.
2. Build a complete list of files modified across all provisioning operations.
3. Deduplicate (a file may have been provisioned, updated, re-provisioned).
4. For each file, apply surgical removal (Strategy A from section 1.5).
5. For files we created from scratch (recorded in manifest), delete them.
6. Show a summary:

```
Removed Tally Wallet from:
  - Claude Code (/.claude.json, ~/.claude/CLAUDE.md)
  - Cursor (~/.cursor/mcp.json, .cursor/rules/tally-wallet.mdc)
  - Claude Desktop (claude_desktop_config.json)

The following backup files will be retained at ~/.tally/backups/
for 30 days in case you want to investigate issues.

[Delete backups now]  [Keep backups]  [Done]
```

### 7.2 Stale Backup Handling

If backups are outdated (the tool config has been heavily modified since our backup):

- Do NOT restore from backup. Use surgical removal only.
- If surgical removal fails (our sentinels are gone, our JSON key is gone), the tool was already unprovisioned by some other means. Mark it as clean and move on.
- Report any tools where removal failed: "Could not remove Tally from Windsurf -- the config was manually modified. You may need to remove the 'tally-wallet' entry from ~/.codeium/windsurf/mcp_config.json yourself."

### 7.3 Tally App Uninstall Integration

When the user uninstalls the Tally app itself (e.g., drags to Trash on macOS):

- Tauri does not provide an uninstall hook. We cannot intercept this.
- Document in the app: "Before uninstalling Tally, use Settings > Remove Tool Connections to clean up config files."
- As a safety net: the injected MCP server config points to the Tally binary. If the binary is gone, the MCP server will simply fail to start. Tools handle this gracefully (they show "MCP server failed to connect" and continue working). This is ugly but non-destructive.

### 7.4 Cleanup Scope

Full uninstall removes:

- All injected MCP server config entries (JSON keys, TOML sections)
- All injected instruction content (between sentinels)
- All standalone files we created (rule files, skill files)
- `~/.tally/provisioning-state.json`
- `~/.tally/provisioning.log`
- `~/.tally/detected-tools.json`

Full uninstall does NOT remove:

- `~/.tally/backups/` (retained per user choice)
- `~/.tally/config.json` (user preferences)
- Any wallet data, keys, or tokens (those live in the app's data directory)

---

## 8. Cross-Platform Considerations

### 8.1 Config Path Resolution

Use a platform abstraction layer that resolves paths per OS:

```rust
fn config_path(tool: &Tool, scope: Scope) -> PathBuf {
    match (tool, scope, std::env::consts::OS) {
        (Tool::ClaudeDesktop, Scope::User, "macos") =>
            dirs::home_dir().join("Library/Application Support/Claude/claude_desktop_config.json"),
        (Tool::ClaudeDesktop, Scope::User, "windows") =>
            dirs::config_dir().join("Claude/claude_desktop_config.json"),
        (Tool::ClaudeDesktop, Scope::User, "linux") =>
            dirs::config_dir().join("Claude/claude_desktop_config.json"),
        // ... etc
    }
}
```

Use the `dirs` crate (Rust) for `home_dir()`, `config_dir()`, `data_dir()`. Never hardcode `~` expansion -- use the OS-appropriate APIs.

### 8.2 Line Endings

- Read files in binary mode to detect existing line endings.
- When modifying, preserve the file's existing line ending convention.
- When creating new files: LF on macOS/Linux, CRLF on Windows.
- For JSON files, line endings don't matter (JSON ignores whitespace). Use LF everywhere.

### 8.3 File Path Handling

- Use `PathBuf` / `Path` (Rust) or `path.join()` (Node) -- never string concatenation.
- Normalize paths before comparison (resolve `..`, `/./`, trailing slashes).
- On Windows, handle both `\` and `/` in paths.
- Watch out for spaces in paths (common on macOS: `/Application Support/`).

### 8.4 Permissions

- macOS: Files default to 0644, directories to 0755. Config files should be 0600 if they contain secrets.
- Linux: Same as macOS.
- Windows: Use ACLs. For our backup directory, set access to current user only.
- On all platforms: check `fs::metadata().permissions()` before writing.

### 8.5 Application Paths

| Tool | macOS | Windows | Linux |
|------|-------|---------|-------|
| Claude Desktop | `/Applications/Claude.app` | `%LOCALAPPDATA%\Programs\Claude\` | N/A (not available) |
| Cursor | `/Applications/Cursor.app` | `%LOCALAPPDATA%\Programs\Cursor\` | `/usr/share/cursor/` or Snap/Flatpak |
| VS Code | `/Applications/Visual Studio Code.app` | `%LOCALAPPDATA%\Programs\Microsoft VS Code\` | `/usr/share/code/` or Snap/Flatpak |
| Windsurf | `/Applications/Windsurf.app` | TBD | TBD |

---

## 9. Security Considerations

### 9.1 Supply Chain Surface

We are injecting executable configuration into other tools. This means:

- If an attacker can modify `~/.tally/` contents, they can inject arbitrary MCP servers into every AI tool on the machine.
- The provisioning state file and backup manifest are trusted inputs to our rollback system. Tampering with the manifest could cause us to overwrite good configs with malicious ones.

### 9.2 Mitigations

**Backup directory permissions**:
- `~/.tally/` directory: mode 0700 (user only).
- All files within: mode 0600.
- On creation, verify permissions. On read, verify permissions haven't been loosened.

**Manifest integrity**:
- Compute SHA-256 of each backup file and store in the manifest.
- Before restoring from a backup, verify the hash matches.
- This detects tampering with backup files.
- Consider signing the manifest with a key derived from the user's Tally wallet credentials. This prevents an attacker who gains filesystem access from silently modifying backups.

**Content verification**:
- When reading our injected config back (for update checks), verify it matches what we expect. If it's been tampered with (e.g., the MCP server command was changed to something malicious), alert the user.
- Maintain a "golden" copy of what we injected alongside the backup. Compare against this, not against the backup of the pre-modification state.

### 9.3 MCP Server Binary Integrity

- The MCP server binary we reference in configs should be inside the Tally app bundle, which is code-signed on macOS.
- On macOS, tools that launch the MCP server will benefit from Gatekeeper verification.
- Do NOT reference `npx` in production configs. `npx` downloads and runs arbitrary code from npm. Use a bundled binary with a known hash.
- If we must use `npx` (development only), pin to an exact version: `npx @tally/wallet-mcp@1.2.0`.

### 9.4 Token Handling

- MCP server config includes `WALLET_TOKEN`. This is a sensitive credential.
- Prefer environment variable references over inline tokens: `"WALLET_TOKEN": "${env:TALLY_WALLET_TOKEN}"` (supported by Windsurf, can be supported by other tools).
- For tools that don't support env var interpolation, the token must be inline. These config files should be user-readable only (0600).
- Never write tokens to project-level configs (which may be git-tracked). User-level only.
- Warn the user if a project-level config file is in a git repository and contains a token.

### 9.5 Config Injection as Attack Vector

An attacker who compromises the Tally app could:
- Inject malicious MCP servers into all AI tools.
- Modify system instructions to influence AI behavior (prompt injection at the tool level).
- Exfiltrate data via a malicious MCP server that logs all tool interactions.

Mitigations:
- Code-sign the Tally app. Gatekeeper/SmartScreen prevents tampering with the binary.
- Use Tauri's built-in updater with signature verification for updates.
- In the MCP server config, reference only the signed binary path. Don't allow arbitrary commands.
- Log all provisioning operations. If the user suspects compromise, the log shows exactly what was modified and when.

---

## 10. Implementation Architecture

### 10.1 Module Structure

```
src-tauri/src/
  provisioning/
    mod.rs              // Public API: detect, provision, unprovision, verify
    detection.rs        // Tool detection logic
    config_writer.rs    // File modification (JSON, TOML, YAML, Markdown)
    backup.rs           // Backup creation and restoration
    rollback.rs         // Rollback logic (surgical, diff-based, full)
    manifest.rs         // Manifest file reading/writing
    state.rs            // Provisioning state management
    platform.rs         // OS-specific path resolution
    tools/
      mod.rs
      claude_code.rs
      claude_desktop.rs
      cursor.rs
      windsurf.rs
      codex.rs
      continue_dev.rs
      cline.rs
      aider.rs
      copilot.rs
```

### 10.2 Core Trait

```rust
trait ToolProvisioner {
    fn detect(&self) -> DetectionResult;
    fn provision(&self, config: &TallyMcpConfig) -> Result<ProvisionResult>;
    fn unprovision(&self) -> Result<UnprovisionResult>;
    fn verify(&self) -> VerificationResult;
    fn config_paths(&self) -> Vec<ConfigFile>;
}
```

Each tool implements this trait. The orchestrator iterates over all registered provisioners.

### 10.3 Tauri Commands

```rust
#[tauri::command]
async fn detect_tools(state: State<'_, AppState>) -> Result<Vec<DetectedTool>>;

#[tauri::command]
async fn provision_tool(state: State<'_, AppState>, tool: String) -> Result<ProvisionResult>;

#[tauri::command]
async fn provision_all(state: State<'_, AppState>) -> Result<Vec<ProvisionResult>>;

#[tauri::command]
async fn unprovision_tool(state: State<'_, AppState>, tool: String) -> Result<UnprovisionResult>;

#[tauri::command]
async fn unprovision_all(state: State<'_, AppState>) -> Result<Vec<UnprovisionResult>>;

#[tauri::command]
async fn verify_provisioning(state: State<'_, AppState>) -> Result<Vec<VerificationResult>>;

#[tauri::command]
async fn get_provisioning_preview(state: State<'_, AppState>, tool: String) -> Result<ProvisionPreview>;
```

### 10.4 Frontend Flow

1. User navigates to Settings > Connected Tools.
2. App calls `detect_tools` -- shows list of detected tools with status (connected, not connected, excluded, needs update).
3. User clicks "Connect" on a tool.
4. App calls `get_provisioning_preview` -- shows diff dialog.
5. User confirms.
6. App calls `provision_tool` -- performs modification.
7. Success toast with "Restart [tool] to activate" note where applicable.

---

## 11. Testing Strategy

### 11.1 Unit Tests

- JSON merge logic with various edge cases (empty file, missing keys, conflicting keys, malformed input).
- TOML append/remove logic.
- Markdown sentinel insertion and removal.
- YAML array merge logic.
- Path resolution for each OS.
- Backup creation and integrity verification.
- Rollback strategy selection (surgical vs diff vs full).

### 11.2 Integration Tests

- Provision and unprovision roundtrip for each tool (using temp directories).
- Idempotent provisioning (provision twice, verify no duplication).
- Update flow (provision v1, then update to v2).
- Rollback after user edits (modify file between provision and rollback).
- Race condition simulation (concurrent file access).
- Malformed file handling (invalid JSON, truncated YAML).

### 11.3 Platform-Specific Tests

- File permission handling on macOS/Linux.
- Symlink resolution.
- Path handling with spaces and special characters.
- Line ending preservation.

---

## 12. Open Questions

1. **Should we support project-level provisioning at all?** Adding MCP config to project files means it shows up in git diffs. This is great for teams but confusing for individuals. Recommendation: user-level only by default, project-level as an explicit "Share with team" feature.

2. **Token rotation**: When the user rotates their wallet token, all provisioned configs need updating. Should this be automatic or manual? Recommendation: automatic, since we track all provisioned files.

3. **Multi-wallet support**: If a user has multiple Tally wallets, which token goes into the config? Recommendation: use the "default" wallet. Support switching via the Tally UI, which triggers a re-provision with the new token.

4. **Telemetry**: Should we track which tools are detected and provisioned (anonymized) to prioritize development? Recommendation: only with explicit opt-in, and never transmit config file contents.

5. **NPX vs bundled binary**: The reference doc uses `npx` examples. For production, we should bundle the MCP server binary with the Tally app. This eliminates Node.js as a dependency, eliminates cold-start time, and eliminates supply chain risk from npm. Recommendation: bundled binary, with npx as a documented fallback for development.

---

## 13. UX Direction: "Add a Skill" Screen

### 13.1 Default Flow (Simple)

The primary action is a single button: **"Add to All Detected Tools"**. This is the happy path for 90% of users — they want the wallet available everywhere.

- On click, detect all installed tools, show a confirmation with the list of what will be modified, and provision all at once.
- Default scope: **global** (user-level config only). No project files touched.
- Post-provision: success summary showing which tools were connected.

### 13.2 Advanced Section (Collapsed by Default)

Below the primary button, an expandable **"Advanced"** section reveals per-tool controls:

- **Tool list with toggles**: Each detected tool shown with an on/off toggle. Undetected tools shown greyed out (so users know we looked).
- **Per-tool status indicators**: Connected, Not Connected, Needs Update, Excluded, Not Installed.
- **Scope selector** (future — v2): Per-tool choice between Global and Project-level. For v1, global only.
- **Exclusion**: Turning a tool off and confirming adds it to the exclusion list. It won't be re-offered.

### 13.3 Design Principles

- **Detection is automatic, modification requires consent.** The screen shows what was found, the user decides what to connect.
- **Don't overwhelm.** The simple path is one button. Power users dig into Advanced.
- **Show, don't hide.** Even in simple mode, briefly list which tools were detected before the user confirms.
- **Respect the user's machine.** This is their config. We're guests. The tone should reflect that — "We'd like to add Tally Wallet to these tools" not "Connecting your tools..."
