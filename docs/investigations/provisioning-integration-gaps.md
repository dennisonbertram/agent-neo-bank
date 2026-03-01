# Provisioning System: Integration Gaps Analysis

> **Date**: 2026-03-01
> **Source**: Full codebase exploration of `/Users/dennisonbertram/Develop/apps/agent-neo-bank`
> **Depends on**: `docs/design/provisioning-system-design.md`, `docs/investigations/coding-tool-config-provisioning.md`

---

## Executive Summary

The provisioning design doc (`provisioning-system-design.md`) is thorough on the **file manipulation** side (backups, rollback, edge cases, security) but has significant gaps in how provisioning **integrates with the existing codebase**. The current app already has a working auto-discovery module, an onboarding flow, agent registration, three transport layers, and a settings page -- but the design doc was written as if starting from scratch. This analysis identifies the concrete integration points, data flow gaps, and UX issues that must be resolved before implementation.

---

## 1. Onboarding Flow: Where "Add a Skill" Actually Fits

### Current Flow

The onboarding is a 4-step linear sequence defined in `App.tsx` routes:

```
/onboarding       -> Onboarding.tsx     (4 carousel slides)
/setup/install    -> InstallSkill.tsx   (hardcoded "Research Skill" placeholder)
/setup/connect    -> ConnectCoinbase.tsx (email input -> Coinbase auth)
/setup/verify     -> VerifyOtp.tsx      (6-digit OTP verification)
```

After OTP verification, the user lands on `/home` and is authenticated.

### Gap 1: InstallSkill.tsx is a Non-Functional Placeholder

**`InstallSkill.tsx`** currently shows a hardcoded "Install Research Skill" screen with:
- A fake "What changes?" panel listing `claude.md` and `agents.md` as modified files
- A "Confirm Installation" button that simply transitions to a success state (no backend call)
- A success screen saying "The Research Skill has been configured" (nothing actually happened)

This screen exists in the design but does ZERO actual work. The provisioning system needs to **replace this entirely** with real tool detection and config injection.

### Gap 2: Provisioning Must Happen AFTER Authentication, Not Before

The current route order is: Onboarding carousel -> Install Skill -> Connect Coinbase -> Verify OTP.

**Problem**: The provisioning design doc (Section 13) describes "Add a Skill" as the provisioning step. But in the current flow, this happens BEFORE the user has authenticated with Coinbase. The MCP server config needs a wallet token or at minimum the MCP server must be running -- neither of which is true until after auth completes.

**Options**:
1. Move InstallSkill AFTER VerifyOtp (the user authenticates first, then provisions tools)
2. Split provisioning into two phases: (a) detect tools during onboarding, (b) inject config after auth succeeds
3. Provision without a token initially (agents register later and get their own tokens)

Option 3 is closest to the current auto-discovery behavior -- the `auto_discovery::install()` function in `lib.rs` already runs on app startup and writes MCP config without any agent token. The CLAUDE.md instructions tell agents to call `register_agent` themselves.

**Recommendation**: Keep the current position in the flow but make it real. The "Install Skill" step should:
- Detect installed tools
- Show what will be modified (as the design doc specifies)
- On confirm, run `auto_discovery::install()` (already exists) plus the new multi-tool provisioning
- No token needed at this stage -- agents self-register via invitation codes

### Gap 3: No Route Back to Provisioning After Onboarding

Once the user completes onboarding, there is no way to:
- Re-provision a tool that got reset by an update
- Add a newly installed tool
- View provisioning status

The Settings page (`Settings.tsx`) has NO provisioning section. It shows: Profile, Network/Wallet, Notifications, and Account/Support. **A "Connected Tools" section is completely missing from the Settings page.**

The design doc (Section 10.4) describes this: "User navigates to Settings > Connected Tools." But the current Settings page has no such section and no route for it.

---

## 2. Settings Page: Missing Provisioning Controls

### Current Settings Structure

```
Settings.tsx
  - Profile Card (avatar, name, email)
  - Network section (Base Mainnet, wallet address)
  - Notifications section (5 toggles)
  - Account & Support section (Export, Help, Disconnect)
  - Version footer
```

### What's Missing

1. **"Connected Tools" section** -- needs to show detected tools, their provisioning status, and connect/disconnect actions
2. **No Tauri commands for provisioning** -- the design doc lists `detect_tools`, `provision_tool`, `unprovision_tool`, `verify_provisioning`, `get_provisioning_preview` but none of these exist in the codebase
3. **No provisioning state in any Zustand store** -- there's no `provisioningStore.ts` or similar
4. **No `tauriApi.provisioning.*` methods** in `src/lib/tauri.ts`
5. **No `commands/provisioning.rs`** in the Tauri commands

### Design Implication

The Settings page needs a new section between "Network" and "Notifications":

```
Connected Tools
  [Claude Code]     Connected  [Disconnect]
  [Cursor]          Not Found  [--greyed--]
  [Claude Desktop]  Connected  [Disconnect]
  [Windsurf]        Needs Update [Update]
```

This requires:
- New Tauri commands (as listed in design doc Section 10.3)
- New frontend API methods in `tauri.ts`
- New Zustand store or extension to existing store
- New UI components for tool status display

---

## 3. Agent Registration: No "Connected Via" Tracking

### Current Registration Flow

Agents register via three transports:
1. **MCP** (`register_agent` tool in `mcp_tools.rs`) -- agent calls JSON-RPC over stdio or HTTP
2. **REST API** (`POST /v1/agents/register` in `rest_routes.rs`) -- standard HTTP with invitation code
3. **Tauri commands** -- frontend-initiated (not exposed, but theoretically possible)

### Gap 4: No Transport/Source Tracking on Agent Records

The `Agent` struct in `db/models.rs` stores:
```rust
pub struct Agent {
    pub id: String,
    pub name: String,
    pub description: String,
    pub purpose: String,
    pub agent_type: String,
    pub capabilities: Vec<String>,
    pub status: AgentStatus,
    pub api_token_hash: Option<String>,
    pub token_prefix: Option<String>,
    pub balance_visible: bool,
    pub invitation_code: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub last_active_at: Option<i64>,
    pub metadata: String,    // JSON blob
}
```

**Missing fields**:
- `registered_via`: Which transport was used to register? ("mcp", "rest", "unix_socket", "ui")
- `tool_name`: Which coding tool provisioned this agent? ("claude-code", "cursor", "windsurf", etc.)
- `connection_method`: How does this agent currently connect? ("mcp_stdio", "mcp_http", "rest", "unix_socket")

The `metadata` JSON blob could store this, and the `AgentRegistrationRequest` struct includes `webhook_url` in metadata already. But nothing actually captures which provisioned tool triggered the registration.

### Why This Matters

The AgentDetail page (`AgentDetail.tsx`) shows agent info but can't display "Connected via Claude Code (MCP)" because that data isn't captured. The design doc's vision of showing which tool provisioned an agent is impossible without this.

### Recommendation

Add a `source` field to `AgentRegistrationRequest`:
```rust
pub source: Option<String>,  // "mcp_http", "mcp_stdio", "rest_api", "unix_socket"
```

Each transport handler should set this automatically. Store it in the agent metadata JSON. The frontend can then display it on the AgentDetail page.

---

## 4. Auto-Discovery Module: Already Exists but Narrow

### Current State

`src-tauri/src/core/auto_discovery.rs` already implements:
- `install(mcp_port)` -- writes to `~/.claude/.mcp.json` and `~/.claude/CLAUDE.md`
- `uninstall()` -- removes Tally entries from both files
- `is_installed()` -- checks if config exists

This runs automatically on app startup in `lib.rs`:
```rust
if let Err(e) = crate::core::auto_discovery::install(config.mcp_port) {
    tracing::warn!(error = %e, "Failed to install MCP auto-discovery");
}
```

### Gap 5: Auto-Discovery Only Handles Claude Code

The current `auto_discovery.rs` hardcodes:
- `MCP_CONFIG_FILENAME = ".mcp.json"`
- `CLAUDE_MD_FILENAME = "CLAUDE.md"`
- `CLAUDE_DIR = ".claude"`
- `MCP_SERVER_KEY = "tally-wallet"`

It writes to `~/.claude/.mcp.json` and `~/.claude/CLAUDE.md` only. It does NOT handle Cursor, Windsurf, Claude Desktop, Codex, Continue.dev, Cline, or any other tool.

The provisioning design doc covers 9 tools. The current code covers 1.

### Gap 6: Auto-Discovery Uses Wrong MCP Config Format

The current code writes an HTTP-based MCP config:
```json
{
  "mcpServers": {
    "tally-wallet": {
      "url": "http://127.0.0.1:7403/mcp"
    }
  }
}
```

This uses the Streamable HTTP transport (the MCP HTTP server on port 7403). But the provisioning design doc also discusses stdio-based transport referencing a binary path. The two approaches serve different use cases:
- HTTP: good for the running Tally app (already listening)
- Stdio: good for tools that spawn the MCP server on demand

For tools like Claude Code that support both, HTTP is preferable (the Tally app is already running as a menubar app). But the design doc doesn't clearly reconcile this with the existing HTTP server.

### Gap 7: No CLAUDE.md Sentinel Markers

The current `auto_discovery.rs` uses `## Tally Agentic Wallet` as the section marker:
```rust
const INSTRUCTIONS_MARKER: &str = "## Tally Agentic Wallet";
```

But the provisioning design doc (Section 6.1) specifies HTML comment sentinels:
```markdown
<!-- TALLY_WALLET_START v1.2.0 -->
...
<!-- TALLY_WALLET_END -->
```

These are incompatible. The existing code would need to be migrated to use sentinels, and existing installations would need to be updated (removing the old format and adding the new).

### Gap 8: No Uninstall on App Close/Quit

The `auto_discovery::uninstall()` function exists but is never called. When the user quits Tally, the MCP config still points to `127.0.0.1:7403` which is no longer listening. This causes "MCP server failed to connect" errors in Claude Code.

The design doc (Section 7.3) acknowledges this: "the MCP server will simply fail to start... This is ugly but non-destructive." But it doesn't address that the current HTTP-based approach is worse than stdio: with stdio, the tool tries to spawn the binary (which just doesn't start). With HTTP, the tool hangs trying to connect to a dead port.

---

## 5. MCP Server: What Agents Need to Know

### Current MCP Tool Definitions

The MCP server exposes 13 tools (from `mcp_tools.rs`):
1. `send_payment` -- Send USDC to an address
2. `check_balance` -- Check wallet balance
3. `get_spending_limits` -- Get agent's spending policy
4. `request_limit_increase` -- Request higher limits
5. `get_transactions` -- Get agent's transaction history
6. `register_agent` -- Register with invitation code
7. `get_address` -- Get wallet public address
8. `trade_tokens` -- Swap tokens on Base
9. `pay_x402` -- Pay for X402 services
10. `list_x402_services` -- Browse X402 bazaar
11. `search_x402_services` -- Search X402 bazaar
12. `get_x402_details` -- Get X402 payment details
13. `get_agent_info` -- Get agent profile info

### Gap 9: CLAUDE.md Instructions Are Minimal

The current injected instructions (from `auto_discovery.rs`) are:

```markdown
## Tally Agentic Wallet

A crypto wallet is running on this machine via MCP. You can send payments,
check balances, trade tokens, and pay for X402 services.

**First time?** Call `register_agent` with a descriptive name for yourself
(e.g. "Claude Code - my-project"). Save the returned token in your
persistent memory -- you'll need it for all future calls.

**Already registered?** Your token is in your memory. All spending is
tracked under your agent name and subject to policies set by the user.

New agents start with $0 spending limits. The wallet owner will set
your budget after they see you in the app.
```

**Problems**:
- Does not mention the invitation code requirement. `register_agent` requires `invitation_code` as a mandatory field, but the instructions don't tell the agent how to get one.
- Does not explain the approval flow. After registration, the agent is "pending" until the user approves them. The instructions don't mention this.
- Does not list available tools. Agents discover tools via `tools/list` but a brief summary in instructions helps them decide which tool to call.
- "Save the returned token" is misleading -- `register_agent` returns `agent_id` and `status`, not a token. The token is only available after the user approves the agent and the agent polls `retrieve_token`.
- Does not mention spending limits or what happens when limits are exceeded.

### Gap 10: Token Delivery Flow Not Documented for Agents

The registration flow is complex:
1. Agent calls `register_agent` with name, purpose, invitation_code -> gets back `agent_id` + status "pending"
2. User sees notification in Tally app, approves the agent
3. Backend generates token, stores encrypted in `token_delivery` table (5-min expiry)
4. Agent polls... how? There's no MCP tool for polling registration status or retrieving the token.

Looking at the MCP tools list, there is NO tool for:
- Checking registration status
- Retrieving the issued token
- Polling for approval

The REST API has `GET /v1/agents/register/{id}/status` but the MCP server has no equivalent. An agent using MCP has no way to complete the registration flow.

### Gap 11: MCP HTTP Server Authentication Disconnect

The MCP HTTP server (`mcp_http_server.rs`) authenticates agents via:
1. Bearer token in Authorization header (for existing agents)
2. No auth for `register_agent` (public tool)

But the `register_agent` MCP tool is dispatched through the same router that requires an authenticated agent context. Looking at the code, the HTTP server creates sessions with agent authentication, but `register_agent` needs to work without an existing agent session.

The current implementation handles this by allowing `register_agent` to be called from any authenticated session (agent A can register agent B). But the instructions tell new agents to call `register_agent` as their first action -- when they have no token yet.

---

## 6. REST API and Unix Socket: Provisioning Relationship

### Current REST API Routes

```
Public (no auth):
  POST /v1/agents/register
  GET  /v1/agents/register/{id}/status
  GET  /v1/health

Authenticated (Bearer token):
  POST /v1/send
  GET  /v1/balance
  GET  /v1/transactions
  GET  /v1/transactions/{tx_id}
  POST /v1/limits/request-increase
```

### Gap 12: No REST/Unix Socket Provisioning Equivalent

The provisioning design doc is entirely MCP-centric. But agents can also connect via:
- REST API on port 7402
- Unix socket at `/tmp/tally-agentic-wallet.sock`

For REST API agents, provisioning means setting an environment variable (`TALLY_WALLET_TOKEN`) or configuring an HTTP endpoint URL. For Unix socket agents, it means knowing the socket path.

The provisioning system doesn't address how non-MCP agents discover the wallet. These agents need:
- The REST API URL (http://127.0.0.1:7402)
- The Unix socket path (/tmp/tally-agentic-wallet.sock)
- An invitation code
- Instructions for the registration + token retrieval flow

### Gap 13: Unix Socket Server Not Actually Implemented

The config has `unix_socket_path: "/tmp/tally-agentic-wallet.sock"` but searching the codebase, there is no Unix socket server implementation. The REST server binds to TCP only (`tokio::net::TcpListener`). The Unix socket transport is aspirational but not built.

---

## 7. Existing Provisioning Code vs. Design Doc

### What Already Exists

| Component | Location | Status |
|-----------|----------|--------|
| Auto-discovery (Claude Code only) | `core/auto_discovery.rs` | Working, runs on startup |
| MCP HTTP server | `api/mcp_http_server.rs` | Working, binds to port 7403 |
| MCP stdio server | `api/mcp_server.rs` | Working, used in tests |
| REST API server | `api/rest_server.rs` | Working, binds to port 7402 |
| Agent registration | `core/agent_registry.rs` | Working |
| Invitation codes | `core/invitation.rs` | Working |
| Token delivery | In agent_registry.rs | Working |
| InstallSkill UI | `pages/InstallSkill.tsx` | Placeholder only |
| Settings UI | `pages/Settings.tsx` | No provisioning section |
| Provisioning Tauri commands | -- | Does not exist |
| Multi-tool detection | -- | Does not exist |
| Provisioning state management | -- | Does not exist |
| Backup/rollback system | -- | Does not exist |

### What the Design Doc Proposes That Doesn't Exist Yet

1. `src-tauri/src/provisioning/` module tree (9 files)
2. `ToolProvisioner` trait with per-tool implementations
3. 7 Tauri commands for provisioning operations
4. Provisioning state file (`~/.tally/provisioning-state.json`)
5. Backup system (`~/.tally/backups/`)
6. Manifest files for each provisioning operation
7. Tool detection for 9 different AI tools
8. Config writers for JSON, TOML, YAML, and Markdown formats

---

## 8. Data Flow Gaps

### Gap 14: No Zustand Store for Provisioning

Per the architecture rules, data follows: Backend Service -> Tauri Command -> Zustand Store -> React Components.

The provisioning system needs:
```typescript
// stores/provisioningStore.ts
interface ProvisioningState {
  detectedTools: DetectedTool[]
  provisionedTools: ProvisionedTool[]
  isScanning: boolean
  lastScanned: number | null

  detectTools: () => Promise<void>
  provisionTool: (tool: string) => Promise<ProvisionResult>
  unprovisionTool: (tool: string) => Promise<void>
  verifyAll: () => Promise<VerificationResult[]>
}
```

### Gap 15: No Types for Provisioning Data

`src/types/index.ts` has no provisioning-related types. Needed:

```typescript
interface DetectedTool {
  id: string               // "claude-code", "cursor", etc.
  name: string             // "Claude Code"
  detected: boolean
  installed: boolean
  provisioned: boolean
  status: "connected" | "not_connected" | "needs_update" | "excluded" | "not_installed"
  version?: string
  configPaths: string[]
}

interface ProvisionResult {
  tool: string
  success: boolean
  files_modified: string[]
  error?: string
}
```

### Gap 16: No Tauri API Surface

`src/lib/tauri.ts` needs a new `provisioning` namespace:

```typescript
tauriApi.provisioning = {
  detectTools: () => invoke<DetectedTool[]>('detect_tools'),
  provisionTool: (tool: string) => invoke<ProvisionResult>('provision_tool', { tool }),
  provisionAll: () => invoke<ProvisionResult[]>('provision_all'),
  unprovisionTool: (tool: string) => invoke<void>('unprovision_tool', { tool }),
  getPreview: (tool: string) => invoke<ProvisionPreview>('get_provisioning_preview', { tool }),
  verify: () => invoke<VerificationResult[]>('verify_provisioning'),
}
```

---

## 9. UX Gaps

### Gap 17: InstallSkill Screen Doesn't Match Design Doc Vision

The design doc describes (Section 13):
- "Add to All Detected Tools" primary button
- Advanced section with per-tool toggles
- Detection results shown before confirmation

The current `InstallSkill.tsx` shows:
- A hardcoded "Install Research Skill" screen
- A fake "What changes?" panel
- No tool detection whatsoever

The entire screen needs to be redesigned and rewired.

### Gap 18: No Re-Provisioning Flow

After onboarding, if a tool update resets the config, the user needs to re-provision. The design doc says "check on launch, offer re-provisioning." But:
- There's no mechanism to check on launch (the on-launch check in `lib.rs` just calls `auto_discovery::install()` silently)
- There's no notification system to tell the user "Cursor lost its Tally config"
- There's no "Connected Tools" screen in Settings to manage this

### Gap 19: Agent Detail Page Lacks Provisioning Context

`AgentDetail.tsx` shows: agent name, daily spend, spending controls, transaction history. It does NOT show:
- How the agent connected (MCP, REST, Unix socket)
- Which tool provisioned it
- Whether the agent's tool connection is still active
- Last communication timestamp is available (`last_active_at`) but not displayed

### Gap 20: No Invitation Code Generation in Onboarding

The registration flow requires an invitation code. Currently, invitation codes are managed via Tauri commands (`generate_invitation_code`, `list_invitation_codes`, `revoke_invitation_code`) but there's no UI for generating them during onboarding.

When the provisioning system injects instructions telling agents to call `register_agent`, those agents need a valid invitation code. The provisioning system should either:
1. Auto-generate an invitation code and include it in the injected instructions
2. Set up a "no invitation required" mode for agents connecting via provisioned tools
3. Provide a way for the user to create and share invitation codes from the app

Currently, `config.invitation_code_required` defaults to `true` and there's no UI for managing codes.

---

## 10. Security Gaps Specific to This Codebase

### Gap 21: Static Encryption Key in Agent Registry

`agent_registry.rs` line 45:
```rust
const ENCRYPTION_KEY: &[u8; 32] = b"tally-wallet-token-encrypt-key!!";
```

This hardcoded key encrypts tokens in the delivery cache. If provisioning injects config files that reference tokens, those tokens are only as secure as this static key. The design doc discusses signing manifests with wallet-derived keys (Section 9.2) but doesn't address this existing vulnerability.

### Gap 22: MCP Config Points to HTTP (Not Signed Binary)

The current auto-discovery writes:
```json
{ "url": "http://127.0.0.1:7403/mcp" }
```

The design doc (Section 9.3) recommends against this: "Do NOT reference `npx` in production configs." The same argument applies to HTTP URLs -- an attacker on localhost could bind to port 7403 before Tally starts.

The design doc recommends using a signed binary path for stdio transport. But the current codebase uses HTTP transport exclusively for MCP.

---

## 11. Prioritized Implementation Recommendations

### Must-Do Before First Provisioning PR

1. **Migrate `auto_discovery.rs` to use sentinel markers** -- the existing marker format conflicts with the design doc
2. **Add provisioning Tauri commands** to `commands/` module
3. **Add provisioning types** to `src/types/index.ts`
4. **Add `tauriApi.provisioning` methods** to `src/lib/tauri.ts`
5. **Create `ProvisioningStore`** following the Zustand pattern
6. **Add "Connected Tools" section to Settings.tsx**
7. **Replace InstallSkill.tsx** with real tool detection + provisioning
8. **Fix CLAUDE.md instructions** to accurately describe the registration flow

### Should-Do Before Launch

9. Add `source` field to agent registration to track which transport/tool was used
10. Add `get_registration_status` and `retrieve_token` MCP tools
11. Add invitation code auto-generation during provisioning
12. Add on-launch verification check (extend current `auto_discovery::install()`)
13. Display "connected via" info on AgentDetail page

### Nice-to-Have

14. Implement the full backup/rollback system (can ship v1 without it)
15. Implement per-tool provisioners beyond Claude Code
16. Add provisioning state persistence (`~/.tally/provisioning-state.json`)
17. Add transparency log

---

## 12. Open Questions for the Design

1. **Should provisioning auto-generate invitation codes?** If yes, the provisioned instructions could include a pre-generated code. If no, agents need another way to get a code.

2. **HTTP vs Stdio transport decision**: The existing codebase uses HTTP (port 7403). The design doc discusses stdio with a binary path. Which is the primary transport for provisioned tools? Can we support both?

3. **When does `auto_discovery::install()` become `provisioning::install()`?** Is this a rename and extension, or a parallel system? The current auto-discovery runs silently on startup. The provisioning system requires user consent.

4. **How do we handle the migration from the current auto-discovery format to the new sentinel-based format?** Users who already have Tally installed have the old `## Tally Agentic Wallet` marker in their CLAUDE.md.

5. **Should the MCP HTTP server require authentication for `register_agent`?** Currently it does (you need an existing agent session to call it). This creates a chicken-and-egg problem for new agents.

---

## File References

| File | Relevance |
|------|-----------|
| `src/App.tsx` | Route definitions, onboarding flow order |
| `src/pages/Onboarding.tsx` | Carousel slides, navigates to `/setup/install` |
| `src/pages/InstallSkill.tsx` | Placeholder skill install screen (needs rewrite) |
| `src/pages/ConnectCoinbase.tsx` | Auth email entry |
| `src/pages/VerifyOtp.tsx` | OTP verification, navigates to `/home` |
| `src/pages/Settings.tsx` | Current settings (no provisioning section) |
| `src/pages/AgentDetail.tsx` | Agent info page (no "connected via" display) |
| `src/pages/AgentsList.tsx` | Agent list (no provisioning context) |
| `src/lib/tauri.ts` | Frontend API surface (no provisioning methods) |
| `src/types/index.ts` | TypeScript types (no provisioning types) |
| `src/stores/agentStore.ts` | Agent Zustand store |
| `src-tauri/src/core/auto_discovery.rs` | Existing Claude-only auto-discovery |
| `src-tauri/src/core/agent_registry.rs` | Agent registration + token delivery |
| `src-tauri/src/core/invitation.rs` | Invitation code management |
| `src-tauri/src/api/mcp_tools.rs` | MCP tool definitions (13 tools) |
| `src-tauri/src/api/mcp_server.rs` | MCP stdio server |
| `src-tauri/src/api/mcp_http_server.rs` | MCP HTTP server (port 7403) |
| `src-tauri/src/api/rest_server.rs` | REST API server (port 7402) |
| `src-tauri/src/api/rest_routes.rs` | REST API route handlers |
| `src-tauri/src/state/app_state.rs` | AppState (no provisioning service) |
| `src-tauri/src/config.rs` | AppConfig (has mcp_port, no provisioning config) |
| `src-tauri/src/lib.rs` | App startup, auto-discovery install, MCP server spawn |
| `docs/design/provisioning-system-design.md` | The design doc being analyzed |
| `docs/investigations/coding-tool-config-provisioning.md` | Per-tool config reference |
