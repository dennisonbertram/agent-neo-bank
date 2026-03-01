# OpenClaw Configuration Provisioning Research

## Date: 2026-03-01

## Executive Summary

OpenClaw is the dominant open-source AI coding assistant (220k+ GitHub stars). It uses a skill-based extensibility model with a public registry (ClawHub, 13,700+ skills). This document investigates how OpenClaw discovers installations, provisions skills/MCP configs, and the mechanisms behind "Add a skill" flows -- with an eye toward building similar provisioning for Tally Agentic Wallet's MCP server.

---

## 1. How OpenClaw Finds Existing Installations

### OpenClaw's Own Config Locations

OpenClaw stores its configuration at:
- **Main config**: `~/.openclaw/openclaw.json` (formerly `~/.clawdbot/clawdbot.json`)
- **Skills directory**: `~/.openclaw/skills/` (user-level, global)
- **Workspace skills**: `<workspace>/skills/` (project-level, highest priority)
- **Bundled skills**: Shipped inside the `openclaw` npm package (lowest priority)

### Binary Detection for Skills

OpenClaw skills declare binary dependencies in their `SKILL.md` frontmatter under `metadata.openclaw.requires.bins`. At session start, OpenClaw checks if required binaries exist on the system PATH before making a skill eligible:

```yaml
metadata:
  openclaw:
    requires:
      bins: ["claude", "node"]    # ALL must be installed
      anyBins: ["bun", "node"]    # At least ONE must exist
      env: ["API_KEY"]            # Required env vars
      config: ["path/to/config"]  # Required config files
    os: ["darwin", "linux"]       # OS restrictions
```

### How OpenClaw Detects Claude Code

The `openclaw-claude-code` skill detects Claude Code by:
1. Checking `~/.claude/local/claude` (the standard local installation path)
2. Falling back to `claude` on the system PATH
3. Optionally connecting via MCP to a Claude Code API endpoint (configurable via `CLAUDE_CODE_API_URL` env var, defaults to `http://127.0.0.1:18795`)

---

## 2. How OpenClaw Adds Skills / MCP Server Configurations

### Skill Installation Flow

**Via ClawHub CLI:**
```bash
# Install from registry
clawhub install <skill-slug>

# Install to specific directory
clawhub install <skill-slug> --workdir /path/to/dir

# Update all installed skills
clawhub update --all

# Sync local skills with registry
clawhub sync --all
```

**What happens on disk:**
1. Skill folder is downloaded to `~/.openclaw/skills/<author>/<skill-name>/`
2. A `SKILL.md` file (with YAML frontmatter) is the primary artifact
3. Optional supporting files (configs, scripts) are included
4. A `.clawhub/lock.json` lockfile tracks installed skills locally
5. Per-install metadata is stored in `<skill>/.clawhub/origin.json`

**Via npm (for MCP-based skills):**
```bash
npm install openclaw-claude-code-skill
# or
npx playbooks add skill openclaw/skills --skill openclaw-claude-code
```

### Per-Skill Configuration

Individual skill settings live in `~/.openclaw/openclaw.json` under `skills.entries.<skillName>`:

```json
{
  "skills": {
    "entries": {
      "my-skill": {
        "enabled": true,
        "apiKey": { "source": "env", "provider": "default", "id": "MY_API_KEY" },
        "env": { "MY_API_KEY": "value" },
        "config": { "endpoint": "https://example.com" }
      }
    },
    "load": {
      "extraDirs": ["/custom/skills/path"],
      "watch": true,
      "watchDebounceMs": 500
    }
  }
}
```

### Skill Priority (Highest to Lowest)

1. `<workspace>/skills/` -- project-specific
2. `~/.openclaw/skills/` -- user-level (installed via CLI/ClawHub)
3. Bundled skills -- shipped with OpenClaw package

---

## 3. File Paths and Config Formats

### OpenClaw Config Files

| File | Purpose |
|------|---------|
| `~/.openclaw/openclaw.json` | Main config (skills, API keys, preferences) |
| `~/.openclaw/skills/` | Installed skills directory |
| `<workspace>/skills/` | Project-level skills |
| `.clawhub/lock.json` | Skill installation state |
| `<skill>/.clawhub/origin.json` | Per-skill install metadata |

### SKILL.md Format (AgentSkills-Compatible)

```yaml
---
name: my-skill
description: What this skill does
version: 1.2.0
metadata:
  openclaw:
    requires:
      env:
        - MY_API_KEY
      bins:
        - node
        - curl
      anyBins:
        - bun
        - node
    primaryEnv: MY_API_KEY
    emoji: "icon"
    homepage: https://example.com
    os: ["darwin", "linux"]
    install:
      - kind: brew
        formula: jq
        bins: [jq]
      - kind: node
        package: typescript
        bins: [tsc]
---

# Skill Instructions

Step-by-step instructions loaded into the agent's context...
```

### Claude Code Config Files (Target for Provisioning)

| File | Purpose | Scope |
|------|---------|-------|
| `~/.claude.json` | User preferences, OAuth, MCP servers | User + Local |
| `~/.claude/settings.json` | Permissions, model, env, hooks | User |
| `.mcp.json` (project root) | Project-scoped MCP servers | Project (git-committed) |
| `.claude/settings.json` | Project-level settings | Project |
| `.claude/settings.local.json` | Personal project overrides | Local (gitignored) |
| `/Library/Application Support/ClaudeCode/managed-settings.json` | Org-wide enforcement (macOS) | Managed |
| `/Library/Application Support/ClaudeCode/managed-mcp.json` | Managed MCP servers (macOS) | Managed |

### Claude Code MCP Server Config Format

**In `~/.claude.json` (user-level):**
```json
{
  "mcpServers": {
    "my-server": {
      "command": "npx",
      "args": ["-y", "my-mcp-server"],
      "env": {
        "API_KEY": "${MY_API_KEY}"
      }
    }
  }
}
```

**In `.mcp.json` (project-level, committed to git):**
```json
{
  "mcpServers": {
    "shared-tool": {
      "command": "/path/to/server",
      "args": [],
      "env": {}
    }
  }
}
```

**HTTP/Remote servers:**
```json
{
  "mcpServers": {
    "remote-api": {
      "type": "http",
      "url": "https://api.example.com/mcp",
      "headers": {
        "Authorization": "Bearer ${API_KEY}"
      }
    }
  }
}
```

---

## 4. Mechanism for "Add a Skill" Auto-Discovery and Provisioning

### How OpenClaw Does It (Skill Installation)

OpenClaw's approach is **agent-centric**: skills are installed into OpenClaw's own directory structure and loaded into the agent's context at session start. The flow is:

1. **Registry browsing**: User visits [clawhub.ai](https://clawhub.ai) or searches via `clawhub search`
2. **Installation**: `clawhub install <slug>` downloads the skill folder
3. **Auto-discovery**: OpenClaw snapshots eligible skills at session start, checking:
   - Binary requirements (`requires.bins`)
   - Environment variables (`requires.env`)
   - OS compatibility (`os`)
   - Config file presence (`requires.config`)
4. **Skill injection**: Eligible skills are loaded into the system prompt

### How Claude Code Does It (MCP Provisioning)

Claude Code's approach is **config-file-centric**: MCP servers are registered by modifying JSON config files. The primary mechanisms are:

1. **CLI command**: `claude mcp add <name> --transport stdio -- <command> [args...]`
   - This writes to `~/.claude.json` (local/user scope) or `.mcp.json` (project scope)

2. **JSON import**: `claude mcp add-json <name> '<json>'`
   - Directly adds a JSON config block

3. **Desktop import**: `claude mcp add-from-claude-desktop`
   - Reads Claude Desktop's config and imports MCP servers

4. **Plugin system**: Plugins define MCP servers in `.mcp.json` at plugin root or inline in `plugin.json`
   - Auto-starts when plugin is enabled

5. **Managed deployment**: IT admins deploy `managed-mcp.json` to system directories

### Building an "Add to Claude Code" Button

Based on this research, there are several approaches to auto-provision an MCP server config into Claude Code:

#### Approach A: CLI-Based (Recommended)

The most reliable approach uses the `claude mcp add` CLI command:

```bash
# Stdio transport (local server)
claude mcp add tally-wallet --transport stdio --scope user \
  -- npx -y @tally/wallet-mcp-server

# HTTP transport (remote server)
claude mcp add tally-wallet --transport http --scope user \
  https://localhost:7402/mcp

# JSON config (most control)
claude mcp add-json tally-wallet '{
  "type": "stdio",
  "command": "node",
  "args": ["/path/to/wallet-mcp-server.js"],
  "env": { "WALLET_API_URL": "http://localhost:7402" }
}' --scope user
```

**Pros**: Uses Claude's official API, handles config merging, respects scopes.
**Cons**: Requires `claude` CLI to be in PATH.

#### Approach B: Direct File Manipulation

Directly modify `~/.claude.json` to add an `mcpServers` entry:

```javascript
const fs = require('fs');
const path = require('path');

const claudeConfigPath = path.join(os.homedir(), '.claude.json');

// Read existing config (or create empty)
let config = {};
if (fs.existsSync(claudeConfigPath)) {
  config = JSON.parse(fs.readFileSync(claudeConfigPath, 'utf8'));
}

// Add MCP server
config.mcpServers = config.mcpServers || {};
config.mcpServers['tally-wallet'] = {
  command: 'node',
  args: ['/path/to/wallet-mcp-server.js'],
  env: {
    WALLET_API_URL: 'http://localhost:7402'
  }
};

// Write back (Claude Code auto-backs up config files)
fs.writeFileSync(claudeConfigPath, JSON.stringify(config, null, 2));
```

**Pros**: No dependency on `claude` CLI. Works even if Claude Code isn't running.
**Cons**: Must handle JSON merge carefully. No validation. Bypasses Claude's own config management.

#### Approach C: Project-Level `.mcp.json`

Ship a `.mcp.json` file in your project repository:

```json
{
  "mcpServers": {
    "tally-wallet": {
      "command": "${TALLY_WALLET_PATH:-npx}",
      "args": ["-y", "@tally/wallet-mcp-server"],
      "env": {
        "WALLET_API_URL": "${WALLET_API_URL:-http://localhost:7402}"
      }
    }
  }
}
```

**Pros**: Team-shareable. Git-committed. Env var expansion supported.
**Cons**: Only works within that project. Users must approve on first use.

#### Approach D: Plugin System

Bundle the MCP server as a Claude Code plugin:

```json
// plugin.json
{
  "name": "tally-wallet",
  "mcpServers": {
    "tally-wallet": {
      "command": "${CLAUDE_PLUGIN_ROOT}/servers/wallet-mcp-server",
      "args": ["--config", "${CLAUDE_PLUGIN_ROOT}/config.json"]
    }
  }
}
```

**Pros**: Official plugin mechanism. Auto-lifecycle management.
**Cons**: Plugin ecosystem is newer. Requires plugin marketplace registration.

### Detecting Claude Code Installation

To check if Claude Code is installed before provisioning:

```bash
# Check standard installation path
test -f ~/.claude/local/claude && echo "Found at ~/.claude/local/claude"

# Check PATH
which claude 2>/dev/null && echo "Found in PATH"

# Check if running
pgrep -f "claude" && echo "Claude Code is running"
```

For Claude Desktop (separate product):
- macOS: `~/Library/Application Support/Claude/claude_desktop_config.json`
- Windows: `%APPDATA%\Claude\claude_desktop_config.json`
- Linux: `~/.config/Claude/claude_desktop_config.json`

---

## 5. Cross-Pollination: How OpenClaw and Claude Code Share Config

### Issue #5363: Import Claude Code Plugins into OpenClaw

A GitHub issue proposed importing Claude Code plugins into OpenClaw by:
1. Extracting `.mcp.json` from Claude Code plugin directories
2. Registering them with OpenClaw via `mcporter config` or native MCP support
3. Mapping Claude Code skill definitions to OpenClaw format

The issue was closed as "not planned" (Feb 2026), but the proposed plugin structure was:
```
.claude-plugin/
  plugin.json     # Plugin metadata
  .mcp.json       # MCP server config (import target)
  commands/       # Slash commands (skip)
  agents/         # Agent definitions (skip)
  skills/         # Skill definitions (potential adaptation)
```

### mcporter: Universal MCP Config Manager

The `mcporter` skill/CLI tool provides cross-platform MCP server management:
- Reads config from `./config/mcporter.json` (default)
- Can import Claude Code configs from project root and `~/.config/`
- Supports `mcporter config import` for pulling in MCP configs from other tools
- Provides daemon mode for persistent MCP server management

---

## 6. Recommendations for Tally Agentic Wallet

### For Adding Tally's MCP Server to Claude Code

**Best approach**: Use the `claude mcp add` CLI command from the Tauri app:

```bash
claude mcp add tally-wallet --transport stdio --scope user \
  -- /path/to/bundled/tally-mcp-server
```

Or for HTTP transport (since Tally already runs a REST API on localhost:7402):

```bash
claude mcp add tally-wallet --transport http --scope user \
  http://localhost:7402/mcp
```

**Fallback**: Direct file manipulation of `~/.claude.json` if the CLI isn't available.

### For Detecting Claude Code

```rust
// In Tauri backend
fn detect_claude_code() -> Option<PathBuf> {
    let home = dirs::home_dir()?;

    // Check local installation
    let local_path = home.join(".claude/local/claude");
    if local_path.exists() {
        return Some(local_path);
    }

    // Check PATH
    if let Ok(output) = Command::new("which").arg("claude").output() {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            return Some(PathBuf::from(path));
        }
    }

    None
}
```

### For an "Add to Claude Code" Button in the UI

1. Detect Claude Code installation (binary check)
2. Show "Add to Claude Code" button if found
3. On click: run `claude mcp add-json tally-wallet '<config>'` via Tauri shell command
4. Verify: run `claude mcp get tally-wallet` to confirm installation
5. Show success/failure state in UI

---

## Sources

- [OpenClaw Skills Documentation](https://docs.openclaw.ai/tools/skills)
- [ClawHub Skill Format Specification](https://github.com/openclaw/clawhub/blob/main/docs/skill-format.md)
- [Claude Code MCP Documentation](https://code.claude.com/docs/en/mcp)
- [Claude Code Settings Documentation](https://code.claude.com/docs/en/settings)
- [OpenClaw Claude Code Skill](https://github.com/Enderfga/openclaw-claude-code-skill)
- [GitHub Issue #5363: Import Claude Code Plugins](https://github.com/openclaw/openclaw/issues/5363)
- [mcporter Skill](https://github.com/openclaw/skills/blob/main/skills/steipete/mcporter/SKILL.md)
- [ClawHub Registry](https://github.com/openclaw/clawhub)
