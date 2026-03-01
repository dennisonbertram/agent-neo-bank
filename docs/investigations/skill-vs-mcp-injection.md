# Skill-Based vs MCP-Based Agent Wallet Discovery

**Date**: 2026-03-01
**Status**: Research complete
**Goal**: Find the lightest-weight way to make AI coding agents aware of the Tally Wallet across all major tools

---

## Executive Summary

There are two independent mechanisms for wallet discovery: **instruction injection** (telling the agent the wallet exists and how to use it) and **tool injection** (giving the agent actual MCP tools). The current `auto_discovery.rs` does BOTH — it writes a `.mcp.json` (tool injection) AND appends to `CLAUDE.md` (instruction injection). This is the right architecture, but the instruction layer can be extended to work across many more tools.

**Key finding**: An instruction/skill file CANNOT give an agent access to MCP tools. The MCP server must be pre-configured in the tool's config for the agent to have the tools available. However, an instruction CAN tell an agent to self-install the MCP server (e.g., "run `claude mcp add ...`"), which works for Claude Code but not for most other tools.

---

## 1. How Claude Code Skills Work

### File Structure
```
~/.claude/skills/<skill-name>/
  SKILL.md        # Required — YAML frontmatter + markdown instructions
  scripts/        # Optional helper scripts
  templates/      # Optional templates
```

### Context Loading (Progressive Disclosure)
- **At startup**: Only skill `name` and `description` from frontmatter are loaded (very lightweight)
- **On invocation**: Full `SKILL.md` content loads into context
- **Supporting files**: Only loaded when Claude reads them explicitly

This means a skill's description costs ~100 tokens at all times, and the full content only loads when relevant. The budget for all skill descriptions is 2% of context window (fallback: 16,000 chars).

### Frontmatter Options
```yaml
---
name: tally-wallet
description: Crypto wallet for agent payments. Use when sending payments, checking balances, or trading tokens.
disable-model-invocation: false   # Claude can auto-load when relevant
user-invocable: false             # Hide from /slash menu (background knowledge)
---
```

Setting `user-invocable: false` makes it pure background knowledge — Claude loads it when relevant but users don't see it in the menu. This is ideal for wallet awareness.

### Can a Skill Reference an MCP Server?
**No, not directly.** A skill provides instructions, not tool bindings. The MCP server must be separately configured (in `.mcp.json` or via `claude mcp add`) for the agent to have access to the actual tools.

However, a skill CAN:
1. Tell Claude the wallet exists and how to use it conceptually
2. Instruct Claude to run `claude mcp add tally-wallet --transport http http://127.0.0.1:7403/mcp` if the MCP server isn't configured yet
3. Reference MCP tools by name once they're available

### Current State
The user's `~/.claude/CLAUDE.md` already has a "Tally Agentic Wallet" section (injected by `auto_discovery.rs`). No skill file exists in `~/.claude/skills/` — the wallet instructions are in CLAUDE.md directly. The `.mcp.json` at `~/.claude/.mcp.json` has the MCP server configured.

---

## 2. Claude Code Plugins (.mcp.json)

Plugins are a distribution mechanism that bundles skills + MCP servers together.

### How Plugin MCP Works
- Plugin defines MCP servers in `.mcp.json` at the plugin root or in `plugin.json`
- When a plugin is enabled, its MCP servers start automatically
- Plugin MCP tools appear alongside manually configured tools

### Relevance to Tally
This is the heavier approach — packaging Tally as a Claude Code plugin. It would auto-start the MCP server when enabled. But it requires plugin installation, which is more friction than a skill file.

---

## 3. The Critical Question: Skill Instruction vs MCP Config

### Can an Agent Self-Install an MCP Server?

**Claude Code**: YES, technically. Claude Code has bash access and can run:
```bash
claude mcp add tally-wallet --transport http http://127.0.0.1:7403/mcp
```
But this requires a restart of Claude Code to take effect. So it's a one-time setup, not a per-session discovery.

**All other tools**: NO. Cursor, Windsurf, Copilot, Cline, etc. cannot programmatically add MCP servers at runtime. The config must exist before the session starts.

### The Two-Layer Architecture (Current, Correct)

```
Layer 1: Instruction injection (lightweight, cross-tool)
  → Tells agents the wallet exists
  → Describes how to register, check balances, send payments
  → Points to MCP tools if available, or REST API as fallback

Layer 2: MCP tool injection (heavyweight, Claude-specific for now)
  → Provides actual callable tools (register_agent, send_payment, etc.)
  → Requires pre-configuration in the tool's MCP config
```

The instruction layer is universal. The tool layer is per-tool.

---

## 4. Lightest Injection Per Tool

### Claude Code
**Mechanism**: `~/.claude/CLAUDE.md` section + `~/.claude/.mcp.json` entry
**Context cost**: ~100 tokens (description only) + MCP tool definitions loaded on-demand via Tool Search
**Current state**: Already implemented in `auto_discovery.rs`
**Could improve**: Move to a proper skill at `~/.claude/skills/tally-wallet/SKILL.md` for progressive disclosure and cleaner separation. The skill description loads at startup (~100 tokens), full instructions only load when relevant.

### Claude Desktop
**Mechanism**: `claude_desktop_config.json` MCP entry (only option)
**Context cost**: Full MCP tool definitions always loaded
**Injection**: Must be written to `~/Library/Application Support/Claude/claude_desktop_config.json`
**Notes**: No instruction file mechanism. The MCP server config IS the discovery. Claude Desktop does not read `CLAUDE.md` or skill files.

### Cursor
**Mechanism**: `.cursor/rules/*.mdc` files
**Context cost**: Rules auto-inject into every AI prompt when file glob patterns match
**Lightest approach**: Create `.cursor/rules/tally-wallet.mdc` with:
```yaml
---
description: Tally Wallet integration for agent payments
globs: ["**/*"]
alwaysApply: true
---
A crypto wallet is available via MCP at http://127.0.0.1:7403/mcp...
```
**MCP support**: Cursor supports MCP servers via settings. Write to `.cursor/mcp.json` or Cursor settings.
**Notes**: Rules persist per-project. Global rules go in `~/.cursor/rules/`.

### Windsurf
**Mechanism**: `.windsurf/rules/*.md` files
**Context cost**: Rules auto-included in every Cascade request
**Limits**: 6,000 chars per rule, 12,000 total combined
**Lightest approach**: Create `.windsurf/rules/tally-wallet.md`
**MCP support**: Windsurf supports MCP via its settings/config.

### OpenAI Codex
**Mechanism**: `AGENTS.md` files (hierarchical, root-to-leaf)
**Context cost**: Injected near top of conversation, before user prompt
**Lightest approach**: Append a section to `~/.codex/AGENTS.md` (global) or project `AGENTS.md`
**MCP support**: Codex does not support MCP servers natively. REST API is the only integration path. The `AGENTS.md` instruction would point agents to the REST endpoint.
**Notes**: AGENTS.md is an open standard supported by 20+ tools (Codex, Jules, Copilot, Cursor, Windsurf, Devin, etc.).

### Continue.dev
**Mechanism**: `.continue/rules/*.md` or `.continuerules` file
**Context cost**: Rules joined into system message for chat/agent requests
**Lightest approach**: Create `.continuerules` or `.continue/rules/tally-wallet.md`
**MCP support**: Continue supports MCP in its `config.json`.

### Cline
**Mechanism**: `.clinerules/` directory with `.md` files
**Context cost**: All `.md` files in `.clinerules/` combined into unified rules
**Lightest approach**: Create `.clinerules/tally-wallet.md`
**MCP support**: Cline supports MCP servers via VS Code settings.
**Bonus**: Cline can read/write its own rules files, enabling self-configuration.

### Aider
**Mechanism**: `CONVENTIONS.md` loaded via `--read` or `.aider.conf.yml`
**Context cost**: Loaded as read-only context, cached if prompt caching enabled
**Lightest approach**: Add to project `CONVENTIONS.md` or create dedicated file loaded in `.aider.conf.yml`:
```yaml
read: [CONVENTIONS.md, TALLY-WALLET.md]
```
**MCP support**: Aider does not support MCP. REST API is the only integration path.

### GitHub Copilot
**Mechanism**: `.github/copilot-instructions.md`
**Context cost**: Auto-attached to all Copilot Chat and inline requests
**Lightest approach**: Add section to `.github/copilot-instructions.md`
**MCP support**: Copilot supports MCP servers in VS Code via settings. Also supports multiple `.instructions.md` files with path-specific `applyTo` frontmatter.

---

## 5. Universal Instruction Template

All tools support some form of markdown instruction injection. A single template can be adapted for each:

```markdown
## Tally Agentic Wallet

A crypto wallet is running on this machine. You can send payments,
check balances, trade tokens, and pay for X402 services.

**MCP endpoint**: http://127.0.0.1:{port}/mcp
**REST endpoint**: http://127.0.0.1:{port}/api/v1

**First time?** Call `register_agent` with a descriptive name for yourself.
Save the returned token — you'll need it for all future calls.

**Already registered?** Your token should be in your persistent memory.
All spending is tracked and subject to policies set by the wallet owner.

New agents start with $0 spending limits. The wallet owner will set
your budget after they see you in the app.
```

---

## 6. What `auto_discovery.rs` Currently Does

The existing code in `/Users/dennisonbertram/Develop/apps/agent-neo-bank/src-tauri/src/core/auto_discovery.rs`:

1. **Writes `.mcp.json`** to `~/.claude/.mcp.json` — adds a `tally-wallet` MCP server entry pointing to `http://127.0.0.1:{port}/mcp`
2. **Appends to `CLAUDE.md`** at `~/.claude/CLAUDE.md` — adds the "Tally Agentic Wallet" instruction block
3. **Supports install/uninstall/is_installed** — clean lifecycle management
4. **Merges, not replaces** — preserves existing `.mcp.json` entries and `CLAUDE.md` content
5. **Idempotent** — safe to call multiple times

### What It Should Also Do (Recommendations)

Extend `auto_discovery.rs` to inject into more tools:

| Tool | File to Write | Priority |
|------|--------------|----------|
| Cursor | `~/.cursor/rules/tally-wallet.mdc` | High |
| Windsurf | Global rules or per-project | Medium |
| Codex | `~/.codex/AGENTS.md` | High |
| Cline | Global `.clinerules/tally-wallet.md` | Medium |
| Copilot | Per-project `.github/copilot-instructions.md` | Low (project-specific) |
| Continue | Per-project `.continuerules` | Low |
| Aider | Per-project `CONVENTIONS.md` | Low |

The high-priority targets are tools that support **global** (user-level) instruction files, because the Tally app can inject once and all projects benefit. Project-level files require per-project injection.

---

## 7. Recommendations

### Short Term
1. **Keep the current dual-injection** (`.mcp.json` + `CLAUDE.md`) — it's correct
2. **Add Cursor injection** — write `~/.cursor/rules/tally-wallet.mdc` (global rules)
3. **Add AGENTS.md injection** — write `~/.codex/AGENTS.md` for Codex
4. **Consider a Claude Code skill** — move the `CLAUDE.md` section to `~/.claude/skills/tally-wallet/SKILL.md` for proper progressive disclosure (less startup context cost)

### Medium Term
5. **Add Cline injection** — `.clinerules/` in global config dir
6. **Add Windsurf injection** — global rules directory

### Long Term
7. **Build a universal `AGENTS.md`** section — since AGENTS.md is supported by 20+ tools, a single well-crafted section can serve Codex, Copilot, Cursor, Windsurf, Devin, and more
8. **Detect installed tools** — only inject into tools that are actually installed on the machine

### Architecture Note
The instruction layer and tool layer serve different purposes:
- **Instruction**: "A wallet exists, here's how to think about it" (universal, zero-cost when not relevant)
- **Tools**: "Here are callable functions" (requires MCP config, consumes context)

For tools that don't support MCP (Aider, older Codex), the instruction can point to the REST API as a fallback:
```
curl -X POST http://127.0.0.1:7403/api/v1/agents/register \
  -H "Content-Type: application/json" \
  -d '{"name": "my-agent"}'
```

---

## Sources

- [Claude Code Skills Documentation](https://code.claude.com/docs/en/skills)
- [Claude Code MCP Documentation](https://code.claude.com/docs/en/mcp)
- [Cursor Rules Documentation](https://cursor.com/docs/context/rules)
- [Windsurf Rules Guide](https://uibakery.io/blog/windsurf-ai-rules)
- [OpenAI Codex AGENTS.md](https://developers.openai.com/codex/guides/agents-md/)
- [AGENTS.md Open Standard](https://agents.md/)
- [Continue.dev Rules](https://docs.continue.dev/customize/deep-dives/rules)
- [Cline .clinerules](https://docs.cline.bot/customization/cline-rules)
- [Aider Conventions](https://aider.chat/docs/usage/conventions.html)
- [GitHub Copilot Custom Instructions](https://docs.github.com/copilot/customizing-copilot/adding-custom-instructions-for-github-copilot)
- [Docker Dynamic MCP Discovery](https://docs.docker.com/ai/mcp-catalog-and-toolkit/dynamic-mcp/)
