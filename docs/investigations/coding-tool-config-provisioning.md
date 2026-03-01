# Coding Tool Configuration & Provisioning Reference

> **Purpose**: Exact file paths and formats for auto-provisioning MCP servers, system instructions, and skills into every major AI coding tool.
>
> **Date**: 2026-03-01

---

## Table of Contents

1. [Claude Code CLI](#1-claude-code-cli)
2. [Claude Desktop App](#2-claude-desktop-app)
3. [Cursor](#3-cursor)
4. [Windsurf (Codeium)](#4-windsurf-codeium)
5. [OpenAI Codex CLI](#5-openai-codex-cli)
6. [Continue.dev](#6-continuedev)
7. [Cline (VS Code)](#7-cline-vs-code)
8. [Aider](#8-aider)
9. [GitHub Copilot](#9-github-copilot)
10. [VS Code Native (Copilot MCP)](#10-vs-code-native-copilot-mcp)
11. [Provisioning Script Strategy](#11-provisioning-script-strategy)

---

## 1. Claude Code CLI

### Config File Locations

| Scope | Path | Git-tracked |
|-------|------|-------------|
| User settings | `~/.claude/settings.json` | N/A |
| User memory | `~/.claude/CLAUDE.md` | N/A |
| User preferences + MCP | `~/.claude.json` | N/A |
| Project settings | `.claude/settings.json` | Yes |
| Project memory | `CLAUDE.md` or `.claude/CLAUDE.md` | Yes |
| Project local memory | `.claude/CLAUDE.local.md` | No (gitignored) |
| Project local settings | `.claude/settings.local.json` | No (gitignored) |
| Project MCP servers | `.mcp.json` (project root) | Yes |

### How to Add MCP Servers

**Project-level** (`.mcp.json` in project root):
```json
{
  "mcpServers": {
    "tally-wallet": {
      "command": "npx",
      "args": ["-y", "@anthropic/tally-wallet-mcp"],
      "env": {
        "WALLET_TOKEN": "your_token"
      }
    }
  }
}
```

**User-level** (`~/.claude.json` — merge into existing):
```json
{
  "mcpServers": {
    "tally-wallet": {
      "command": "node",
      "args": ["/path/to/server.js"],
      "env": {
        "WALLET_TOKEN": "your_token"
      }
    }
  }
}
```

**CLI command**:
```bash
claude mcp add tally-wallet --transport stdio -- node /path/to/server.js
claude mcp add tally-wallet --scope project -- npx -y @tally/wallet-mcp
```

### How to Add System Instructions

Append to `~/.claude/CLAUDE.md` (global) or `CLAUDE.md` / `.claude/CLAUDE.md` (project):

```markdown
## Tally Agentic Wallet

A crypto wallet is available via MCP. Call `register_agent` to get started.
Use your token for all wallet operations.
```

### How to Add Skills/Agents

Place markdown files in:
- **User-level**: `~/.claude/agents/agent-name.md`
- **Project-level**: `.claude/agents/agent-name.md`

Skills directory: `~/.claude/skills/skill-name.md` (invoked via `/skill-name`)

### Auto-Discovery

- `CLAUDE.md` is loaded automatically from project root and `~/.claude/`
- `.mcp.json` is loaded automatically from project root
- `~/.claude.json` mcpServers are loaded globally
- `.claude/agents/*.md` are auto-discovered as subagent types

### Detection

Check for: `which claude` or existence of `~/.claude/` directory.

---

## 2. Claude Desktop App

### Config File Locations

| OS | Path |
|----|------|
| macOS | `~/Library/Application Support/Claude/claude_desktop_config.json` |
| Windows | `%APPDATA%\Claude\claude_desktop_config.json` |
| Windows (MSIX) | `C:\Users\<user>\AppData\Local\Packages\Claude_pzs8sxrjxfjjc\LocalCache\Roaming\Claude\claude_desktop_config.json` |

### How to Add MCP Servers

Edit `claude_desktop_config.json`:
```json
{
  "mcpServers": {
    "tally-wallet": {
      "command": "npx",
      "args": ["-y", "@tally/wallet-mcp"],
      "env": {
        "WALLET_TOKEN": "your_token"
      }
    }
  }
}
```

**Requires app restart** after config changes.

### How to Add System Instructions

Claude Desktop does not support CLAUDE.md-style files. Custom instructions are set in-app via Settings > Profile. No file-based provisioning path exists.

### How to Add Skills/Agents/Plugins

Not supported via config files. MCP servers are the primary extension mechanism.

### Auto-Discovery

- Reads `claude_desktop_config.json` on startup
- No directory scanning

### Detection

Check for config file existence at the OS-specific path above.

---

## 3. Cursor

### Config File Locations

| Scope | Path | Git-tracked |
|-------|------|-------------|
| Project MCP | `.cursor/mcp.json` | Yes |
| Global MCP | `~/.cursor/mcp.json` | N/A |
| Project rules | `.cursor/rules/*.mdc` or `.cursor/rules/*.md` | Yes |
| Legacy rules (deprecated) | `.cursorrules` (project root) | Yes |
| User rules | Cursor Settings > Rules (UI only) | N/A |

### How to Add MCP Servers

**Project-level** (`.cursor/mcp.json`):
```json
{
  "mcpServers": {
    "tally-wallet": {
      "command": "npx",
      "args": ["-y", "@tally/wallet-mcp"],
      "env": {
        "WALLET_TOKEN": "your_token"
      }
    }
  }
}
```

**Global** (`~/.cursor/mcp.json`) — same format.

### How to Add System Instructions

Create `.cursor/rules/tally-wallet.mdc`:
```markdown
---
description: "Tally Agentic Wallet integration rules"
alwaysApply: true
---

## Tally Agentic Wallet

A crypto wallet is available via MCP. Call `register_agent` to get started.
Use your token for all wallet operations.
```

**Frontmatter properties**:
- `description`: Agent uses this to determine relevance
- `alwaysApply`: `true` = always loaded, `false` = on-demand
- `globs`: file patterns like `["*.ts", "src/**"]`

Cursor also reads `AGENTS.md` in project root or subdirectories.

### How to Add Skills/Agents/Plugins

No dedicated skill system. Rules + MCP servers are the extension mechanism.

### Auto-Discovery

- `.cursor/mcp.json` scanned on project open
- `.cursor/rules/` directory scanned for `*.mdc` and `*.md` files
- `.cursorrules` (legacy) read from project root
- `AGENTS.md` read from project root

### Detection

Check for: `which cursor` or `~/.cursor/` directory, or Cursor process running.

---

## 4. Windsurf (Codeium)

### Config File Locations

| Scope | Path | Git-tracked |
|-------|------|-------------|
| Global MCP | `~/.codeium/windsurf/mcp_config.json` | N/A |
| Global rules | `~/.codeium/windsurf/memories/global_rules.md` | N/A |
| Workspace rules | `.windsurf/rules/*.md` (project root) | Yes |
| System rules (macOS) | `/Library/Application Support/Windsurf/rules/*.md` | N/A |
| System rules (Linux) | `/etc/windsurf/rules/*.md` | N/A |
| System rules (Windows) | `C:\ProgramData\Windsurf\rules\*.md` | N/A |

### How to Add MCP Servers

Edit `~/.codeium/windsurf/mcp_config.json`:
```json
{
  "mcpServers": {
    "tally-wallet": {
      "command": "npx",
      "args": ["-y", "@tally/wallet-mcp"],
      "env": {
        "WALLET_TOKEN": "${env:TALLY_WALLET_TOKEN}"
      }
    }
  }
}
```

**Supports env var interpolation** in `command`, `args`, `env`, `serverUrl`, `url`, `headers` via `${env:VAR_NAME}`.

For HTTP/SSE servers:
```json
{
  "mcpServers": {
    "remote-server": {
      "serverUrl": "https://your-server.com/mcp",
      "headers": {
        "Authorization": "Bearer ${env:AUTH_TOKEN}"
      }
    }
  }
}
```

### How to Add System Instructions

**Global** — append to `~/.codeium/windsurf/memories/global_rules.md`:
```markdown
## Tally Agentic Wallet

A crypto wallet is available via MCP. Call `register_agent` to get started.
```

**Project** — create `.windsurf/rules/tally-wallet.md`:
```markdown
## Tally Agentic Wallet

A crypto wallet is available via MCP. Call `register_agent` to get started.
Use your token for all wallet operations.
```

Rules have 4 activation modes:
1. **Always On** — applied to every request
2. **Manual** — `@mention` to activate
3. **Model Decision** — natural language trigger
4. **Glob** — file pattern matching (e.g., `*.ts`)

Max 12,000 characters per rule file.

### How to Add Skills/Agents/Plugins

No dedicated skill system. Rules + MCP servers.

### Auto-Discovery

- `~/.codeium/windsurf/mcp_config.json` loaded on startup
- `.windsurf/rules/*.md` scanned from project root
- `~/.codeium/windsurf/memories/global_rules.md` loaded globally

### Detection

Check for: `which windsurf` or `~/.codeium/` directory.

---

## 5. OpenAI Codex CLI

### Config File Locations

| Scope | Path | Git-tracked |
|-------|------|-------------|
| User config | `~/.codex/config.toml` | N/A |
| User instructions | `~/.codex/AGENTS.md` | N/A |
| User override | `~/.codex/AGENTS.override.md` | N/A |
| Project config | `.codex/config.toml` | Yes |
| Project instructions | `AGENTS.md` (project root + subdirs) | Yes |
| Project override | `AGENTS.override.md` (any dir) | Yes |
| User skills | `~/.agents/skills/` | N/A |
| Project skills | `.agents/skills/` | Yes |
| System skills | `/etc/codex/skills/` | N/A |

### How to Add MCP Servers

Edit `~/.codex/config.toml` or `.codex/config.toml`:

**STDIO server**:
```toml
[mcp_servers.tally-wallet]
command = "npx"
args = ["-y", "@tally/wallet-mcp"]

[mcp_servers.tally-wallet.env]
WALLET_TOKEN = "your_token"
```

**HTTP server**:
```toml
[mcp_servers.tally-wallet]
url = "https://wallet.tally.xyz/mcp"
bearer_token_env_var = "TALLY_WALLET_TOKEN"
```

**Additional options**: `startup_timeout_sec`, `tool_timeout_sec`, `enabled`, `required`, `enabled_tools`, `disabled_tools`.

**CLI command**:
```bash
codex mcp add tally-wallet
```

### How to Add System Instructions

Append to `~/.codex/AGENTS.md` (global) or `AGENTS.md` (project root):

```markdown
## Tally Agentic Wallet

A crypto wallet is available via MCP. Call `register_agent` to get started.
Use your token for all wallet operations.
```

**Merge behavior**: Files concatenated from root down (global -> project root -> cwd). Later files override. Max 32 KiB combined (configurable via `project_doc_max_bytes`).

**Priority**: `AGENTS.override.md` > `AGENTS.md` > fallback filenames.

**Fallback filenames** (configurable in `config.toml`):
```toml
project_doc_fallback_filenames = ["TEAM_GUIDE.md", ".agents.md"]
```

### How to Add Skills

Create a skill directory:
```
~/.agents/skills/tally-wallet/
  SKILL.md          # Required: name + description
  scripts/          # Optional: helper scripts
  references/       # Optional: reference docs
  agents/
    openai.yaml     # Optional: UI/policy config
```

**SKILL.md** (required):
```markdown
# Tally Wallet Skill

**name**: tally-wallet
**description**: Manage crypto wallets, check balances, send payments via Tally Agentic Wallet MCP.
```

**Scan locations** (priority order):
1. `$CWD/.agents/skills/` — folder-specific
2. `$REPO_ROOT/.agents/skills/` — repo-wide
3. `$HOME/.agents/skills/` — user-level
4. `/etc/codex/skills/` — system-level

Skills are auto-detected on startup.

### Auto-Discovery

- `AGENTS.md` / `AGENTS.override.md` scanned from `~/.codex/` and every dir from git root to cwd
- `.agents/skills/` scanned at multiple levels
- `config.toml` loaded from `~/.codex/` and `.codex/`

### Detection

Check for: `which codex` or existence of `~/.codex/` directory.

---

## 6. Continue.dev

### Config File Locations

| Scope | Path | Git-tracked |
|-------|------|-------------|
| Global config (macOS/Linux) | `~/.continue/config.yaml` | N/A |
| Global config (Windows) | `%USERPROFILE%\.continue\config.yaml` | N/A |
| Legacy config | `~/.continue/config.json` (deprecated) | N/A |
| Programmatic config | `~/.continue/config.ts` | N/A |
| Project MCP servers | `.continue/mcpServers/*.yaml` or `*.json` | Yes |
| Workspace overrides | `.continuerc.json` | Yes |

### How to Add MCP Servers

**Project-level** — create `.continue/mcpServers/tally-wallet.yaml`:
```yaml
name: Tally Wallet MCP
version: 0.0.1
schema: v1
mcpServers:
  - name: tally-wallet
    command: npx
    args:
      - "-y"
      - "@tally/wallet-mcp"
    env:
      WALLET_TOKEN: "your_token"
```

**Or** create `.continue/mcpServers/mcp.json` (Claude Desktop format compatible):
```json
{
  "mcpServers": {
    "tally-wallet": {
      "command": "npx",
      "args": ["-y", "@tally/wallet-mcp"],
      "env": {
        "WALLET_TOKEN": "your_token"
      }
    }
  }
}
```

**Global** — add to `~/.continue/config.yaml`:
```yaml
mcpServers:
  - name: tally-wallet
    command: npx
    args:
      - "-y"
      - "@tally/wallet-mcp"
    env:
      WALLET_TOKEN: "your_token"
```

### How to Add System Instructions

Add rules to `~/.continue/config.yaml`:
```yaml
rules:
  - uses: file://~/.continue/rules/tally-wallet.md
```

Or inline:
```yaml
rules:
  - name: Tally Wallet
    rule: "A crypto wallet is available via MCP. Call register_agent to get started."
```

**System prompt override** in model config:
```yaml
models:
  - name: claude-sonnet
    chatOptions:
      baseSystemMessage: "You have access to Tally Wallet MCP..."
      baseAgentSystemMessage: "You are an agent with Tally Wallet access..."
```

### How to Add Skills/Agents/Plugins

No dedicated skill system. Rules + MCP servers + config.ts for programmatic extension.

### Auto-Discovery

- `~/.continue/config.yaml` loaded on startup
- `.continue/mcpServers/` directory scanned for `*.yaml` and `*.json` files
- `.continuerc.json` loaded from workspace root
- Supports Claude Desktop JSON format in `mcpServers/` directory

### Detection

Check for: `~/.continue/` directory.

---

## 7. Cline (VS Code)

### Config File Locations

| Scope | OS | Path |
|-------|----|------|
| MCP settings | macOS | `~/Library/Application Support/Code/User/globalStorage/saoudrizwan.claude-dev/settings/cline_mcp_settings.json` |
| MCP settings | Windows | `%APPDATA%\Code\User\globalStorage\saoudrizwan.claude-dev\settings\cline_mcp_settings.json` |
| MCP settings | Linux | `~/.config/Code/User/globalStorage/saoudrizwan.claude-dev/settings/cline_mcp_settings.json` |
| Project rules (single file) | Any | `.clinerules` (project root) |
| Project rules (directory) | Any | `.clinerules/*.md` (project root) |
| Custom instructions | Any | VS Code Settings > Cline > Custom Instructions (UI only) |

### How to Add MCP Servers

Edit `cline_mcp_settings.json`:

**STDIO server**:
```json
{
  "mcpServers": {
    "tally-wallet": {
      "command": "npx",
      "args": ["-y", "@tally/wallet-mcp"],
      "env": {
        "WALLET_TOKEN": "your_token"
      },
      "alwaysAllow": ["register_agent", "get_balance", "send_payment"],
      "disabled": false
    }
  }
}
```

**SSE/HTTP server**:
```json
{
  "mcpServers": {
    "tally-wallet-remote": {
      "url": "https://wallet.tally.xyz/mcp",
      "headers": {
        "Authorization": "Bearer your_token"
      },
      "alwaysAllow": [],
      "disabled": false
    }
  }
}
```

### How to Add System Instructions

**Single file** — create `.clinerules` at project root:
```markdown
## Tally Agentic Wallet

A crypto wallet is available via MCP. Call `register_agent` to get started.
Use your token for all wallet operations.
```

**Directory** — create `.clinerules/tally-wallet.md`:
```markdown
## Tally Agentic Wallet

A crypto wallet is available via MCP. Call `register_agent` to get started.
```

Files in `.clinerules/` directory are toggleable via the chat input popover.

### How to Add Skills/Agents/Plugins

No dedicated skill system. `.clinerules` + MCP servers.

### Auto-Discovery

- `cline_mcp_settings.json` loaded from VS Code global storage
- `.clinerules` file or `.clinerules/` directory scanned from project root
- Custom instructions from VS Code settings

### Detection

Check for: VS Code extension `saoudrizwan.claude-dev` installed, or the globalStorage path.

---

## 8. Aider

### Config File Locations

| Scope | Path | Git-tracked |
|-------|------|-------------|
| Global config | `~/.aider.conf.yml` | N/A |
| Repo config | `<git-root>/.aider.conf.yml` | Yes |
| Directory config | `.aider.conf.yml` (cwd) | Depends |
| Environment | `~/.aider.env` or `.aider.env` | No |

Files loaded in order: home dir -> git root -> cwd. Later files take priority.

### How to Add MCP Servers

Aider does **not** natively support MCP servers. Third-party `aider-mcp-server` projects exist but MCP is not a built-in Aider feature.

### How to Add System Instructions

**Option 1**: Convention files via config:

Create `CONVENTIONS.md` in your project and add to `.aider.conf.yml`:
```yaml
read:
  - CONVENTIONS.md
```

Or multiple files:
```yaml
read:
  - CONVENTIONS.md
  - .aider-rules.md
  - docs/wallet-instructions.md
```

**Option 2**: Direct in `~/.aider.conf.yml` (global):
```yaml
read:
  - ~/conventions/tally-wallet.md
```

Note: Paths in global config are resolved relative to where `aider` is run, not relative to the config file.

**Option 3**: Use `--read` CLI flag:
```bash
aider --read CONVENTIONS.md
```

### How to Add Skills/Agents/Plugins

No skill system. Convention files (`read:` directive) are the extension mechanism.

### Auto-Discovery

- `.aider.conf.yml` scanned from home dir, git root, and cwd
- `.aider.env` scanned from same locations
- Files referenced in `read:` loaded as read-only context

### Detection

Check for: `which aider` or `pip show aider-chat`.

---

## 9. GitHub Copilot

### Config File Locations

| Scope | Path | Git-tracked |
|-------|------|-------------|
| Repo instructions | `.github/copilot-instructions.md` | Yes |
| Path-specific instructions | `.github/instructions/*.instructions.md` | Yes |
| Agent instructions | `AGENTS.md` (any directory) | Yes |
| Prompt files | `.github/prompts/*.prompt.md` | Yes |
| VS Code MCP (workspace) | `.vscode/mcp.json` | Yes |
| VS Code MCP (global, macOS/Linux) | `~/.config/Code/User/mcp.json` | N/A |
| VS Code MCP (global, Windows) | `%APPDATA%\Code\User\mcp.json` | N/A |

### How to Add MCP Servers

**Workspace** — create `.vscode/mcp.json`:
```json
{
  "servers": {
    "tally-wallet": {
      "command": "npx",
      "args": ["-y", "@tally/wallet-mcp"],
      "env": {
        "WALLET_TOKEN": "your_token"
      }
    }
  }
}
```

Note: VS Code uses `"servers"` key (not `"mcpServers"`).

**Or** in `.vscode/settings.json`:
```json
{
  "mcp": {
    "servers": {
      "tally-wallet": {
        "command": "npx",
        "args": ["-y", "@tally/wallet-mcp"],
        "env": {
          "WALLET_TOKEN": "your_token"
        }
      }
    }
  }
}
```

### How to Add System Instructions

**Repository-wide** — create `.github/copilot-instructions.md`:
```markdown
## Tally Agentic Wallet

A crypto wallet is available via MCP. Call `register_agent` to get started.
Use your token for all wallet operations.
```

**Path-specific** — create `.github/instructions/wallet.instructions.md`:
```yaml
---
applyTo: "**/*.ts,**/*.tsx"
---

When working with wallet-related code, use the Tally Wallet MCP server
for balance checks and transactions.
```

**Agent-specific** — place `AGENTS.md` in any directory:
```markdown
## Tally Agentic Wallet

Instructions for all agents working in this codebase...
```

### How to Add Skills/Agents/Plugins

**Agent files** — create `.github/agents/wallet-agent.agent.md`:
```markdown
---
name: wallet-agent
description: Agent for wallet operations
tools:
  - tally-wallet
---

Instructions for this agent...
```

### Auto-Discovery

- `.github/copilot-instructions.md` auto-loaded for all Copilot interactions
- `.github/instructions/*.instructions.md` auto-matched by `applyTo` globs
- `AGENTS.md` nearest in directory tree is used
- `.vscode/mcp.json` loaded on workspace open
- `.github/prompts/*.prompt.md` available as reusable prompts

### Detection

Check for: VS Code with Copilot extension, or `gh copilot` CLI.

---

## 10. VS Code Native (Copilot MCP)

For VS Code-based tools (Copilot, Cline, Continue) that share the VS Code MCP system:

**Workspace MCP**: `.vscode/mcp.json`
```json
{
  "servers": {
    "tally-wallet": {
      "command": "npx",
      "args": ["-y", "@tally/wallet-mcp"],
      "env": {
        "WALLET_TOKEN": "your_token"
      }
    }
  }
}
```

**User MCP**: `settings.json` under `"mcp"` key.

---

## 11. Provisioning Script Strategy

### Detection Logic

```bash
# Claude Code CLI
[ -d ~/.claude ] || command -v claude >/dev/null

# Claude Desktop
[ -f ~/Library/Application\ Support/Claude/claude_desktop_config.json ]  # macOS
[ -f "$APPDATA/Claude/claude_desktop_config.json" ]                       # Windows

# Cursor
[ -d ~/.cursor ] || command -v cursor >/dev/null

# Windsurf
[ -d ~/.codeium ] || command -v windsurf >/dev/null

# Codex CLI
[ -d ~/.codex ] || command -v codex >/dev/null

# Continue.dev
[ -d ~/.continue ]

# Cline (VS Code extension)
[ -d ~/Library/Application\ Support/Code/User/globalStorage/saoudrizwan.claude-dev ]  # macOS

# Aider
command -v aider >/dev/null

# GitHub Copilot
command -v gh >/dev/null && gh extension list | grep -q copilot
```

### Shared MCP Server Config (Source of Truth)

Since most tools use the same `mcpServers` JSON format, maintain one canonical block:

```json
{
  "tally-wallet": {
    "command": "npx",
    "args": ["-y", "@tally/wallet-mcp"],
    "env": {
      "WALLET_TOKEN": "${TALLY_WALLET_TOKEN}"
    }
  }
}
```

### Non-Destructive Merge Strategy

For JSON files (`claude_desktop_config.json`, `.cursor/mcp.json`, `cline_mcp_settings.json`, `.mcp.json`, `.vscode/mcp.json`):
1. Read existing file (or `{}` if missing)
2. Parse JSON
3. Deep-merge `mcpServers` (or `servers` for VS Code) — never overwrite existing keys
4. Write back with pretty-printing

For TOML files (`~/.codex/config.toml`):
1. Read existing file
2. Check if `[mcp_servers.tally-wallet]` section exists
3. If not, append the TOML block at the end

For Markdown files (`CLAUDE.md`, `AGENTS.md`, `.clinerules`, `.windsurf/rules/`, `.cursorrules`):
1. Read existing file
2. Check if marker (e.g., `## Tally Agentic Wallet`) already exists
3. If not, append the instruction block at the end

For YAML files (`.aider.conf.yml`, `.continue/config.yaml`):
1. Read existing file
2. Parse YAML
3. Merge arrays (e.g., `read:` list, `mcpServers:` list)
4. Write back

### Per-Tool Provisioning Actions

| Tool | MCP Config Action | Instructions Action |
|------|------------------|-------------------|
| Claude Code | Merge into `.mcp.json` | Append to `CLAUDE.md` |
| Claude Desktop | Merge into `claude_desktop_config.json` | N/A (no file-based instructions) |
| Cursor | Merge into `.cursor/mcp.json` | Create `.cursor/rules/tally-wallet.mdc` |
| Windsurf | Merge into `~/.codeium/windsurf/mcp_config.json` | Create `.windsurf/rules/tally-wallet.md` |
| Codex CLI | Append TOML to `~/.codex/config.toml` | Append to `AGENTS.md` |
| Continue.dev | Create `.continue/mcpServers/tally-wallet.json` | Add rule to `config.yaml` |
| Cline | Merge into `cline_mcp_settings.json` | Create `.clinerules/tally-wallet.md` |
| Aider | N/A (no native MCP) | Add `read:` entry to `.aider.conf.yml` + create convention file |
| GitHub Copilot | Merge into `.vscode/mcp.json` | Append to `.github/copilot-instructions.md` |

### Universal Project-Level Files

For any project wanting wallet integration, create these files:

```
.mcp.json                                    # Claude Code
.cursor/mcp.json                             # Cursor
.vscode/mcp.json                             # VS Code / Copilot
.continue/mcpServers/tally-wallet.json       # Continue.dev
.windsurf/rules/tally-wallet.md              # Windsurf rules
.cursor/rules/tally-wallet.mdc              # Cursor rules
.clinerules/tally-wallet.md                  # Cline rules
.github/copilot-instructions.md              # Copilot instructions (append)
.github/instructions/wallet.instructions.md  # Copilot path-specific
CLAUDE.md                                    # Claude Code + Copilot (AGENTS.md)
AGENTS.md                                    # Codex CLI + Copilot + Cursor
```

### Key Format Differences

| Format | Tools |
|--------|-------|
| `{"mcpServers": {...}}` | Claude Code, Claude Desktop, Cursor, Cline, Continue.dev (JSON compat), Windsurf |
| `{"servers": {...}}` | VS Code native (Copilot MCP) |
| TOML `[mcp_servers.name]` | Codex CLI |
| YAML `mcpServers:` list | Continue.dev (native) |
| No MCP support | Aider |

---

## Sources

- [Claude Code Settings](https://code.claude.com/docs/en/settings)
- [Claude Desktop MCP Setup](https://support.claude.com/en/articles/10949351)
- [Cursor Rules](https://cursor.com/docs/context/rules)
- [Cursor MCP](https://cursor.com/docs/context/mcp)
- [Windsurf MCP](https://docs.windsurf.com/windsurf/cascade/mcp)
- [Windsurf Memories](https://docs.windsurf.com/windsurf/cascade/memories)
- [Codex AGENTS.md](https://developers.openai.com/codex/guides/agents-md/)
- [Codex Skills](https://developers.openai.com/codex/skills/)
- [Codex MCP](https://developers.openai.com/codex/mcp/)
- [Codex Advanced Config](https://developers.openai.com/codex/config-advanced/)
- [Continue.dev MCP](https://docs.continue.dev/customize/deep-dives/mcp)
- [Continue.dev Config](https://docs.continue.dev/customize/deep-dives/configuration)
- [Cline MCP Config](https://docs.cline.bot/mcp/configuring-mcp-servers)
- [Cline Rules](https://cline.ghost.io/clinerules-version-controlled-shareable-and-ai-editable-instructions/)
- [Aider Config](https://aider.chat/docs/config/aider_conf.html)
- [Aider Conventions](https://aider.chat/docs/usage/conventions.html)
- [GitHub Copilot Instructions](https://docs.github.com/copilot/customizing-copilot/adding-custom-instructions-for-github-copilot)
- [VS Code MCP Servers](https://code.visualstudio.com/docs/copilot/customization/mcp-servers)
