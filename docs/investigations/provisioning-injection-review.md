# Provisioning & Injection System: Comprehensive Review

> **Date**: 2026-03-01
> **Scope**: Full read of all provisioning-related docs, Rust source, and GitHub issues
> **Sources**: 6 documentation files, 21 Rust source files, 13 GitHub issues

---

## 1. What the Provisioning System Does

The provisioning system auto-discovers AI coding tools installed on the user's machine and injects two things into each tool's configuration:

1. **MCP server config** -- a JSON/TOML/YAML entry that tells the tool "there is an MCP server at this path/URL that provides wallet tools." This gives the AI agent actual callable functions (send_payment, check_balance, register_agent, etc.).

2. **Instruction/skill content** -- a markdown snippet appended to the tool's instruction/rules file that tells the AI agent "a crypto wallet exists, here's how to register and use it." This is the discovery layer -- it makes the agent aware of the wallet even before it sees the MCP tools.

The system targets 9 AI coding tools: Claude Code, Claude Desktop, Cursor, Windsurf, OpenAI Codex CLI, Continue.dev, Cline, Aider, and GitHub Copilot.

---

## 2. How It Works Mechanically

### 2.1 Architecture Overview

```
ProvisioningService (Tauri state, separate from AppState)
  ├── 9x ToolProvisioner implementations (one per tool)
  ├── StateManager    (persists to ~/.tally/provisioning-state.json)
  ├── BackupManager   (creates ~/.tally/backups/<timestamp>/)
  └── ProvisioningLogger (writes ~/.tally/provisioning.log)
```

The service lives at `src-tauri/src/provisioning/` with 21 files across the module tree. It is managed as separate Tauri state (not on AppState) because it has no dependency on the CLI executor, database, or auth service -- it operates purely on the filesystem.

### 2.2 Detection Phase

**File**: `src-tauri/src/provisioning/detection.rs`

For each of the 9 tools, detection runs a multi-strategy scan (in order of reliability):

1. **Config directory exists** (e.g., `~/.claude/`, `~/.cursor/`, `~/.codeium/`)
2. **Config file exists** (e.g., specific JSON/TOML/YAML config)
3. **Binary in PATH** via `which` + login shell fallback + version manager shim directories
4. **Application bundle** (macOS only, e.g., `/Applications/Cursor.app`)
5. **VS Code extension** (for Cline, Continue.dev, Copilot)
6. **Process running** (least reliable, last resort)

The binary lookup is particularly thorough -- it checks `which`, then `bash -lc 'which ...'` (to pick up nvm/asdf/mise/volta), then scans known version manager shim directories, and finally iterates all nvm Node versions.

Detection results are cached to `~/.tally/detected-tools.json` with a 1-hour TTL. Cache is bypassed on app launch and explicit refresh.

### 2.3 Per-Tool File Writes

**File**: `src-tauri/src/provisioning/platform.rs`

Each tool gets up to two files modified:

| Tool | MCP Config File | Instruction/Skill File |
|------|----------------|----------------------|
| Claude Code | `~/.claude.json` (JSON merge into `mcpServers`) | `~/.claude/skills/tally-wallet/SKILL.md` (new file) |
| Claude Desktop | `~/Library/Application Support/Claude/claude_desktop_config.json` (JSON merge) | None (no instruction file mechanism) |
| Cursor | `~/.cursor/mcp.json` (JSON merge into `mcpServers`) | `~/.cursor/rules/tally-wallet.mdc` (new file with frontmatter) |
| Windsurf | `~/.codeium/windsurf/mcp_config.json` (JSON merge) | `~/.windsurf/rules/tally-wallet.md` (new file) |
| Codex CLI | `~/.codex/config.toml` (TOML section append) | `~/.codex/AGENTS.md` (sentinel-wrapped markdown append) |
| Continue.dev | `~/.continue/config.yaml` (YAML list merge) | `~/.continue/rules/tally-wallet.md` (new file) |
| Cline | `~/Library/.../cline_mcp_settings.json` (JSON merge) | `~/.clinerules/tally-wallet.md` (new file) |
| Aider | None (no MCP support) | `~/.aider.conf.yml` (YAML `read:` entry) + convention file |
| Copilot | `~/Library/.../Code/User/settings.json` (JSON merge into `servers`) | `~/.github/copilot-instructions.md` (sentinel-wrapped append) |

### 2.4 File Modification Mechanics

**File**: `src-tauri/src/provisioning/config_writer.rs`

All file modifications use an **atomic read-modify-write** pattern:

1. Acquire advisory file lock (via sidecar `.tally-lock` file using `fs2::FileExt`)
2. Read current file contents (empty string if file doesn't exist)
3. Apply format-specific modification function
4. Write modified content to a temp file in the same directory (via `tempfile::NamedTempFile`)
5. Atomic `rename()` from temp to target
6. Release lock and clean up lock file

**Format-specific operations**:

- **JSON**: Parse, deep-merge `mcpServers.tally-wallet` (or `servers.tally-wallet` for VS Code), pretty-print back. Detects conflicts if a `tally-wallet` key already exists with a different `command`.
- **TOML**: Parse with `toml_edit` (preserves formatting), append `[mcp_servers.tally-wallet]` section with comment decoration.
- **YAML**: Parse with `serde_yaml`, merge into `mcpServers` list (upsert by name) or `read` list.
- **Markdown**: Sentinel-wrapped sections using `<!-- TALLY_WALLET_START v{version} -->` / `<!-- TALLY_WALLET_END -->`. Supports upsert (replace existing section) and append.
- **Standalone files**: Created fresh with `tempfile` + atomic `persist()`. Owned entirely by Tally.

### 2.5 Content Templates

**File**: `src-tauri/src/provisioning/content.rs`

The MCP server entry injected into JSON configs:
```json
{
  "command": "<server_binary_path>",
  "args": ["--stdio"],
  "env": { "WALLET_TOKEN": "..." },
  "_tally_version": "1.2.0",
  "_tally_provisioned": "2026-03-01T14:30:00Z"
}
```

The instruction content injected into skill/rules files (approximately 100 tokens):
```markdown
## Tally Agentic Wallet

A crypto wallet is available on this machine via MCP. You can send payments,
check balances, trade tokens, and pay for X402 services.

**First time?** Call `register_agent` with a descriptive name for yourself
(e.g. "Claude Code - my-project"). Save the returned token in your
persistent memory -- you'll need it for all future calls.

**Already registered?** Your token is in your memory. All spending is
tracked under your agent name and subject to policies set by the user.

New agents start with $0 spending limits. The wallet owner will set
your budget after they see you in the app.
```

Claude Code gets this wrapped in a `SKILL.md` with YAML frontmatter for progressive disclosure. Cursor gets it in `.mdc` format with `alwaysApply: true`. Codex and Copilot get it sentinel-wrapped in their respective markdown files.

### 2.6 Backup System

**File**: `src-tauri/src/provisioning/backup.rs`

Before every modification:

1. SHA-256 the original file
2. Copy to `~/.tally/backups/<ISO-timestamp>/<tool-slug>/<filename>.bak`
3. Verify backup integrity (read back, compare hash)
4. Proceed with modification
5. SHA-256 the result, record in `manifest.json`

Backup retention: last 10 sets, nothing auto-deleted under 30 days, 50MB cap.

### 2.7 Rollback / Unprovisioning

**File**: `src-tauri/src/provisioning/rollback.rs`

Three strategies (applied in priority order):

1. **Surgical removal** (preferred): Delete only the `tally-wallet` key from JSON, remove TOML section, remove content between sentinel markers, delete standalone files.
2. **Diff-based rollback**: Three-way diff between backup, our modification, and current state. Apply reverse of our changes only.
3. **Full restore with confirmation**: Show diff to user and ask before overwriting.

All provisioners implement `unprovision()` using surgical removal by default.

### 2.8 State Tracking

**File**: `src-tauri/src/provisioning/state.rs`

Persisted to `~/.tally/provisioning-state.json`:

```json
{
  "schema_version": 1,
  "machine_id": "<hostname+salt>",
  "tally_version": "1.2.0",
  "tools": {
    "cursor": {
      "status": "provisioned",
      "provisioned_at": "2026-03-01T14:30:00Z",
      "last_verified": "2026-03-01T16:00:00Z",
      "provisioned_version": "1.2.0",
      "removal_count": 0,
      "respect_removal": false,
      "files_managed": ["~/.cursor/mcp.json", "~/.cursor/rules/tally-wallet.mdc"]
    }
  },
  "excluded_tools": [],
  "last_scan": "2026-03-01T16:00:00Z"
}
```

The "respect removal" protocol: if a user's config is removed twice (removal_count >= 2), the system stops offering to re-provision that tool.

---

## 3. Key Invariants and Assumptions

### 3.1 Invariants

1. **Detection is automatic, modification requires explicit user consent.** The system never silently writes to config files -- it detects, previews, and waits for confirmation.
2. **Non-destructive merge.** Existing config content is always preserved. JSON merge adds keys, never overwrites existing non-Tally keys. Markdown append never replaces non-Tally content.
3. **Idempotent.** Provisioning the same tool twice produces the same result. The system checks for existing entries before writing.
4. **Atomic writes.** All file modifications use temp-file-then-rename. No partial writes.
5. **Backup before modify.** Every modification is preceded by a verified backup.
6. **User-level only by default.** Provisioning writes to global/user config paths (`~/.claude.json`, `~/.cursor/mcp.json`), never to project-level files unless explicitly requested.
7. **No background daemon.** Checks run only on app launch. No launchd/systemd/filesystem watchers.
8. **Respect removal.** After 2 detected removals of Tally config for a tool, stop offering re-provisioning.

### 3.2 Assumptions

1. **Home directory exists and is writable.** The system uses `dirs::home_dir()` and fails gracefully if absent.
2. **Tools store config in well-known paths.** The `PlatformPaths` struct hardcodes all paths per OS. If a tool changes its config location, the provisioner breaks.
3. **JSON/TOML/YAML parsers can round-trip.** The system reads, parses, modifies, and re-serializes. Formatting/comments may be lost for JSON (serde_json) and YAML (serde_yaml). TOML uses `toml_edit` which preserves formatting.
4. **Tools ignore unknown keys.** The MCP entry includes `_tally_version` and `_tally_provisioned` metadata keys. If a tool rejects unknown keys, this will break.
5. **File locking is advisory.** The `fs2` file lock is advisory on Unix. Another process that doesn't respect advisory locks could still race.
6. **The MCP server binary path is stable.** The config points to a bundled binary path (e.g., `/Applications/Tally.app/.../tally-mcp-server`). If the user moves the app, all provisioned configs break.
7. **Claude Code picks up changes immediately.** The provisioner sets `needs_restart: false` for Claude Code, assuming it hot-reloads config. Other tools (Claude Desktop, Cursor) may require restart.

---

## 4. Gaps and Risks Identified

### 4.1 Critical Gaps (from `provisioning-integration-gaps.md`)

**Gap 1: InstallSkill.tsx is a non-functional placeholder.** The onboarding screen shows a fake "Install Research Skill" with hardcoded content and a button that does nothing. It must be completely rewritten to call the real provisioning backend.

**Gap 2: No Tauri commands for provisioning exist.** The design doc specifies 10 Tauri commands (`detect_tools`, `provision_tool`, `provision_all`, `unprovision_tool`, `unprovision_all`, `verify_provisioning`, `get_provisioning_preview`, `get_provisioning_state`, `exclude_tool`, `include_tool`). None of these exist in `src-tauri/src/commands/`. The `ProvisioningService` has the methods, but they are not wired to Tauri.

**Gap 3: No frontend data pipeline.** No `provisioningStore.ts` (Zustand), no provisioning types in `src/types/index.ts`, no `tauriApi.provisioning.*` methods in `src/lib/tauri.ts`.

**Gap 4: No "Connected Tools" section in Settings.** The Settings page has no provisioning management UI. After onboarding, there is no way to view, re-provision, or disconnect tools.

**Gap 5: Auto-discovery (old code) vs. provisioning (new code) are parallel systems.** The existing `core/auto_discovery.rs` writes to `~/.claude/.mcp.json` (HTTP transport, Claude-only). The new provisioning system writes to `~/.claude.json` (stdio transport, all tools). These target different files with different MCP transport types.

**Gap 6: CLAUDE.md instructions are misleading.** The injected instructions tell agents to "save the returned token" from `register_agent`, but `register_agent` returns `agent_id` and `status`, not a token. There is no MCP tool for retrieving the token after approval. This is a broken flow for MCP-connected agents (GitHub issue #12).

### 4.2 Security Risks

**Risk 1: Static encryption key.** `agent_registry.rs` uses a hardcoded 32-byte key (`b"tally-wallet-token-encrypt-key!!"`) for token encryption. If provisioning injects tokens into config files, the chain of trust is only as strong as this static key.

**Risk 2: HTTP MCP transport on localhost.** The existing auto-discovery writes `http://127.0.0.1:7403/mcp`. An attacker on localhost could bind to port 7403 before Tally starts, intercepting all MCP requests. The design doc recommends stdio transport with a signed binary instead.

**Risk 3: Config injection as attack vector.** If `~/.tally/` is compromised, an attacker could modify the provisioning state to inject malicious MCP servers into all detected tools. The design doc recommends signing manifests with wallet-derived keys, but this is not implemented.

**Risk 4: Backup files contain other tools' secrets.** Backups are copies of config files that may contain API keys from other tools. Backups are created with mode 0600, but the design doc notes this should be verified on read as well.

### 4.3 Integration Risks

**Risk 5: Provisioning happens during onboarding before authentication.** The current onboarding flow is: carousel -> Install Skill -> Connect Coinbase -> Verify OTP. The MCP server config may need a wallet token that doesn't exist until after auth. The current design sidesteps this by not including tokens in the initial provisioning (agents self-register later).

**Risk 6: No invitation code in injected instructions.** `register_agent` requires an `invitation_code`, but the injected instructions don't mention this. Without a code, agents cannot register (GitHub issue #12, #16).

**Risk 7: Agent transport/source not tracked.** The `Agent` database record has no field for which tool or transport was used to register. The AgentDetail page cannot show "Connected via Claude Code (MCP)" because that data is never captured (GitHub issue #13).

**Risk 8: No MCP tool for token retrieval.** After `register_agent`, the user approves the agent in-app, and a token is generated. But there is no MCP tool for the agent to poll for approval status or retrieve the token. The REST API has this (`GET /v1/agents/register/{id}/status`) but MCP does not (GitHub issue #12).

### 4.4 Operational Risks

**Risk 9: Tool updates reset config.** Cursor, Claude Desktop, and VS Code may overwrite config files during updates. The system detects this on next launch and offers re-provisioning, but there is no proactive notification between launches.

**Risk 10: JSON formatting loss.** `serde_json` does not preserve comments, trailing commas, or custom formatting. Round-tripping a user's carefully formatted JSON config will normalize it. TOML is handled better by `toml_edit` which preserves formatting.

**Risk 11: Copilot provisioner writes to VS Code settings.json.** The Copilot provisioner merges into `settings.json`, which is a massive file with hundreds of settings. A merge error here could break the user's entire VS Code configuration.

---

## 5. GitHub Issues Related to Provisioning

| # | Title | Type |
|---|-------|------|
| 2 | Backend: Implement install_skill Tauri command | feature |
| 8 | Research skill installation across agents and build skill installer | feature |
| 9 | Rewrite InstallSkill.tsx with real tool detection and provisioning | enhancement |
| 10 | Add Connected Tools section for provisioning management (Settings) | feature |
| 11 | Extend auto_discovery.rs to support all 9 coding tools | feature |
| 12 | Fix agent registration flow -- register_agent doesn't return token, no retrieval tool | bug |
| 13 | Add transport/source tracking to agent records | feature |
| 14 | Build entire provisioning data pipeline -- Tauri commands, types, Zustand store | feature |
| 15 | Reorder onboarding steps -- Welcome -> Add Skill -> Connect Coinbase/OTP -> Fund Wallet | enhancement |
| 16 | Design agent-side first-run experience with guided registration flow | feature |
| 17 | Add Test Connection button after provisioning a tool | enhancement |
| 18 | Handle re-provisioning cascade after token rotation | feature |
| 19 | Multi-wallet support -- which wallet gets provisioned? | feature |

---

## 6. Current State Summary

### What Exists and Works

- **Full `provisioning/` Rust module** with 21 source files -- types, detection, config writers (JSON/TOML/YAML/Markdown), backup, rollback, state, platform paths, content templates, logging, error types, and 9 tool-specific provisioners.
- **ProvisioningService** orchestrator with `detect_tools()`, `provision_tool()`, `provision_all()`, `unprovision_tool()`, `unprovision_all()`, `verify_provisioning()`, `get_preview()`, `get_state()`, `exclude_tool()`, `include_tool()`.
- **Atomic file operations** with advisory locking, temp-file writes, and SHA-256 verification.
- **Backup and manifest system** under `~/.tally/backups/`.
- **State persistence** at `~/.tally/provisioning-state.json`.
- **Detection engine** with 6 strategies and version manager awareness.
- **Old `auto_discovery.rs`** (Claude Code only, HTTP transport) still runs on startup.

### What Does Not Exist Yet

- **Tauri commands** to expose provisioning to the frontend.
- **Frontend API surface** (`tauriApi.provisioning.*`).
- **Zustand store** for provisioning state.
- **TypeScript types** for provisioning data.
- **InstallSkill UI rewrite** with real tool detection.
- **Connected Tools section** in Settings.
- **Integration into `lib.rs`** -- the `ProvisioningService` is not created or managed in the app setup hook.
- **Migration path** from old `auto_discovery.rs` format to new provisioning system.
- **Token retrieval MCP tool** for agents to complete registration.
- **Invitation code generation** during provisioning.
- **Tests** for any of the provisioning code.

---

## 7. Document Sources

| Document | Path | Purpose |
|----------|------|---------|
| Coding tool config reference | `docs/investigations/coding-tool-config-provisioning.md` | Per-tool config paths, formats, and provisioning strategy for all 9 tools |
| OpenClaw config provisioning | `docs/investigations/openclaw-config-provisioning.md` | Research into how OpenClaw/ClawHub handles skill installation and cross-tool config |
| Skill vs MCP injection | `docs/investigations/skill-vs-mcp-injection.md` | Analysis of instruction injection (lightweight, universal) vs MCP tool injection (heavyweight, per-tool) |
| Integration gaps analysis | `docs/investigations/provisioning-integration-gaps.md` | 22 specific gaps between the design doc and the existing codebase |
| System design doc | `docs/design/provisioning-system-design.md` | Full design covering backup, rollback, edge cases, security, UX, testing |
| Implementation architecture | `docs/architecture/provisioning-implementation-architecture.md` | Engineering blueprint with all Rust types, trait definitions, module layout, and per-tool implementation specs |
| Rust source | `src-tauri/src/provisioning/**/*.rs` | 21 files implementing the provisioning system |
