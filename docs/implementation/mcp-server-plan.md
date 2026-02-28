# MCP Server Implementation Plan

**Date**: 2026-02-28
**Status**: Draft — awaiting review

---

## MCP Protocol Standard (2025-11-25)

**Current protocol version: `2025-11-25`**
(Source: https://modelcontextprotocol.io/specification — confirmed Feb 2026)

The MCP spec defines two standard transports:
1. **stdio** — client launches server as subprocess (not applicable for us)
2. **Streamable HTTP** — replaces the deprecated HTTP+SSE transport from 2024-11-05

### Streamable HTTP Transport

A single **MCP endpoint** (e.g. `http://localhost:7403/mcp`) handles all methods:

- **POST** — Client sends JSON-RPC requests/notifications/responses
  - Client MUST include `Accept: application/json, text/event-stream`
  - Server responds with either `application/json` (single response) or
    `text/event-stream` (SSE stream with response + optional server messages)
  - For notifications/responses: server returns `202 Accepted` with no body
- **GET** — Client opens SSE stream for server-initiated messages
  - Server MAY send JSON-RPC requests and notifications to the client
  - Server responds `405 Method Not Allowed` if it doesn't support this
- **DELETE** — Client terminates session
  - Server MAY respond `405` if it doesn't allow client-initiated termination

### Required Headers

| Header | When | Value |
|---|---|---|
| `MCP-Session-Id` | All requests after initialize | Session ID from server's InitializeResult response |
| `MCP-Protocol-Version` | All requests after initialize | Negotiated version (e.g. `2025-11-25`) |
| `Accept` | All POST/GET requests | `application/json, text/event-stream` (POST) or `text/event-stream` (GET) |

### Session Management

- Server MAY assign session ID via `MCP-Session-Id` response header on InitializeResult
- Session ID MUST be globally unique and cryptographically secure
- Server responds `400 Bad Request` to requests missing required session ID
- Server responds `404 Not Found` to requests with expired/terminated session ID
- Client MUST start new session (re-initialize) on receiving 404

### Security Requirements

- Server MUST validate `Origin` header to prevent DNS rebinding attacks
- If `Origin` is present and invalid → `403 Forbidden`
- Local servers SHOULD bind to `127.0.0.1` only (not `0.0.0.0`)
- Server SHOULD implement proper authentication

### Resumability (Optional)

- Server MAY attach `id` field to SSE events for stream resumption
- Client uses `Last-Event-ID` header on GET to resume after disconnection
- Enables reliable delivery across network interruptions

Our server will use **stateful sessions** since each agent has a persistent identity
and we track per-agent spending across requests.

Reference: https://modelcontextprotocol.io/specification/2025-11-25/basic/transports

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                    User's Machine                        │
│                                                          │
│  ┌──────────────────────────────────────────────────┐    │
│  │         Tally Agentic Wallet (Tauri App)          │    │
│  │                                                    │    │
│  │  ┌──────────┐  ┌───────────┐  ┌───────────────┐  │    │
│  │  │ React UI │  │ REST API  │  │  MCP Server   │  │    │
│  │  │ (IPC)    │  │ (:7402)   │  │ (HTTP :7403)  │  │    │
│  │  └────┬─────┘  └─────┬─────┘  └──────┬────────┘  │    │
│  │       │               │               │            │    │
│  │       │               │          Streamable HTTP    │    │
│  │       │               │          POST /mcp          │    │
│  │       │               │          GET  /mcp (SSE)    │    │
│  │       │               │          DELETE /mcp         │    │
│  │       │               │               │            │    │
│  │       └───────┬───────┴───────┬───────┘            │    │
│  │               ▼               ▼                    │    │
│  │  ┌─────────────────────────────────────┐          │    │
│  │  │         Shared App State             │          │    │
│  │  │  ┌─────────┐ ┌──────────┐ ┌──────┐  │          │    │
│  │  │  │ SQLite  │ │ Policy   │ │ Auth │  │          │    │
│  │  │  │   DB    │ │ Engine   │ │Cache │  │          │    │
│  │  │  └─────────┘ └──────────┘ └──────┘  │          │    │
│  │  └──────────────────┬──────────────────┘          │    │
│  │                     ▼                              │    │
│  │  ┌─────────────────────────────────────┐          │    │
│  │  │         awal CLI (subprocess)        │          │    │
│  │  │   balance · send · trade · x402     │          │    │
│  │  └─────────────────────────────────────┘          │    │
│  │                                                    │    │
│  │  ┌──────────┐                                     │    │
│  │  │ System   │  ← App minimizes here               │    │
│  │  │ Tray Icon│     MCP server stays alive           │    │
│  │  └──────────┘                                     │    │
│  └──────────────────────────────────────────────────┘    │
│                                                          │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐         │
│  │ Claude Code│  │  Cursor    │  │ Other Agent│         │
│  │            │  │            │  │            │         │
│  │ Token: abc │  │ Token: xyz │  │ Token: 123 │         │
│  └─────┬──────┘  └─────┬──────┘  └─────┬──────┘         │
│        │               │               │                 │
│        └───────────────┴───────────────┘                 │
│                 All connect to MCP :7403                  │
└─────────────────────────────────────────────────────────┘
```

---

## Agent Registration & Identity Flow

```
First Launch (Agent has no token):

  Agent reads ~/.claude/CLAUDE.md
    │
    ▼
  "I have a wallet available. Do I have a saved token?"
    │
    ▼ No
  Calls MCP: register_agent({ name: "Claude Code - myproject" })
    │
    ▼
  Wallet returns: { agent_id: "ag_7f3a", token: "tok_abc123" }
    │
    ▼
  Agent saves token + name to its own persistent memory
    │
    ▼
  Agent appears in Wallet UI with $0 limits
    │
    ▼
  User sees new agent, sets spending policy ($50/day)
    │
    ▼
  Agent can now spend up to $50/day


Subsequent Sessions (Agent has token):

  Agent reads its memory → finds token "tok_abc123"
    │
    ▼
  Includes token in MCP Authorization header
    │
    ▼
  All requests attributed to "Claude Code - myproject"
    │
    ▼
  Spending tracked against existing policy
```

---

## awal CLI Command Mapping

### What the USER controls (via Tauri UI only)

| awal Command | What It Does | Why User-Only |
|---|---|---|
| `auth login <email>` | Start email OTP login | User authenticates the wallet |
| `auth verify <otp>` | Complete OTP verification | User completes auth |
| `auth logout` | End session | User decides when to log out |
| `auth status` | Check if logged in | Shown in UI status bar |
| `show` | Launch companion window | N/A — we ARE the UI |

These are **never exposed to agents**. The wallet's auth is the user's identity — agents don't log in, they register.

### What AGENTS can do (via MCP)

| awal Command | MCP Tool | Policy Enforced? | Notes |
|---|---|---|---|
| `balance` | `check_balance` | Visibility policy | Agent sees balance only if user allows |
| `address` | `get_address` | No | Agent needs this to receive payments |
| `send <to> <amount> <asset>` | `send_payment` | Yes — full policy check | Per-tx max, daily/weekly/monthly caps, auto-approve threshold |
| `trade <from> <to> <amount>` | `trade_tokens` | Yes — full policy check | NEW: Token swaps (ETH↔USDC↔WETH) |
| `x402 pay <url>` | `pay_x402` | Yes — full policy check | NEW: Pay for X402 services |
| `x402 bazaar list` | `list_x402_services` | No | NEW: Discover available services |
| `x402 bazaar search <q>` | `search_x402_services` | No | NEW: Search service marketplace |
| `x402 details <url>` | `get_x402_details` | No | NEW: Check payment requirements |
| — | `register_agent` | No | Self-registration with chosen name |
| — | `get_spending_limits` | No | Agent sees its own policy |
| — | `request_limit_increase` | No | Creates approval for user to review |
| — | `get_transactions` | No | Agent sees its own tx history |
| — | `get_agent_info` | No | NEW: Agent sees its own profile |

### What the APP manages internally (neither user nor agent)

| Concern | How It Works |
|---|---|
| Spending policy enforcement | Atomic check + reserve before every send/trade/x402 pay |
| Transaction ledger | SQLite records every spend with agent attribution |
| Approval queue | Transactions above auto-approve threshold wait for user |
| Kill switch | User can freeze all agent spending instantly |
| Period tracking | Daily/weekly/monthly caps reset on calendar boundaries |

---

## Implementation Phases

### Phase 1: Streamable HTTP MCP Transport (~2 days)

Implement the modern MCP Streamable HTTP transport as a persistent Axum server.

**Files to create/modify:**
- `src-tauri/src/api/mcp_http_server.rs` — Streamable HTTP transport
- `src-tauri/src/api/mcp_router.rs` — Shared JSON-RPC request routing
- `src-tauri/src/api/mod.rs` — Register new modules
- `src-tauri/src/lib.rs` — Spawn HTTP server on app startup

**Streamable HTTP Protocol (single MCP endpoint):**

All communication goes through a single path: `POST /mcp`, `GET /mcp`, `DELETE /mcp`.
No separate registration endpoint — registration happens via `tools/call` within MCP.

```
POST /mcp
  ├── Accept: application/json, text/event-stream
  ├── MCP-Session-Id: <session-id>           (required after initialize)
  ├── MCP-Protocol-Version: 2025-11-25       (required after initialize)
  ├── Authorization: Bearer <token>          (required after register_agent)
  ├── Content-Type: application/json
  └── Body: JSON-RPC message
      ├── { method: "initialize", ... }      → creates session, returns MCP-Session-Id
      ├── { method: "tools/list", ... }      → returns available tools
      ├── { method: "tools/call", params: { name: "register_agent", ... } }
      │     ↑ allowed without Authorization header (returns token)
      └── { method: "tools/call", params: { name: "send_payment", ... } }
            ↑ requires Authorization: Bearer <token>

GET /mcp
  ├── Accept: text/event-stream
  ├── MCP-Session-Id: <session-id>
  └── Response: SSE event stream
      └── Server pushes notifications (tx completed, approval resolved, etc.)

DELETE /mcp
  ├── MCP-Session-Id: <session-id>
  └── Terminates the session, cleans up resources
```

**Design:**
```rust
pub struct McpHttpServer {
    app_state: Arc<AppState>,
    port: u16,  // default 7403
    sessions: Arc<DashMap<String, McpSession>>,
}

struct McpSession {
    agent_id: Option<String>,           // None until authenticated
    token_hash: Option<String>,         // None until authenticated
    created_at: Instant,
    protocol_version: String,
    tx_sender: broadcast::Sender<String>, // SSE event channel
}

impl McpHttpServer {
    pub async fn start(self) -> Result<(), Error> {
        let app = Router::new()
            .route("/mcp", post(handle_post).get(handle_sse).delete(handle_delete))
            .layer(ValidateOriginLayer::new());  // DNS rebinding protection

        let listener = TcpListener::bind("127.0.0.1:7403").await?;
        axum::serve(listener, app).await
    }
}
```

**Session lifecycle:**

```
1. Agent POSTs { method: "initialize" } to /mcp
   → Server creates session, responds with:
     - MCP-Session-Id header (cryptographically secure UUID)
     - InitializeResult with server capabilities + protocol version

2. Agent POSTs { method: "notifications/initialized" }
   → Server returns 202 Accepted

3. Agent POSTs { method: "tools/list" }
   → Server returns all 13 tools (register_agent is visible to everyone)

4. Agent POSTs { method: "tools/call", name: "register_agent" }
   → No Authorization header needed for this one tool
   → Server creates agent, returns { agent_id, token }
   → Agent saves token to its own persistent memory

5. All subsequent tool calls include Authorization: Bearer <token>
   → Server resolves token → agent_id, enforces policies

6. Agent MAY open GET /mcp for server-pushed notifications
   (e.g., "your pending approval was approved", "transaction confirmed")

7. Agent DELETEs /mcp to end session (or session expires after idle)
```

**Auth model:**
- Session starts unauthenticated (no Authorization header)
- `initialize`, `tools/list`, and `register_agent` work without auth
- All other tools require `Authorization: Bearer <token>`
- Token resolves to agent_id → all actions attributed to that agent
- This stays within standard MCP protocol — no custom endpoints

### Phase 2: System Tray (~1 day)

Keep the app alive when the window is closed.

**Files to modify:**
- `src-tauri/tauri.conf.json` — Enable tray icon
- `src-tauri/src/lib.rs` — Add tray setup
- `src-tauri/icons/tray-icon.png` — Tray icon asset

**Behavior:**
- Window close → minimize to tray (not quit)
- Tray icon shows wallet status (green = connected, yellow = no auth)
- Tray menu: "Open Wallet", "Pause All Agents", "Quit"
- MCP server keeps running in background
- "Pause All Agents" = toggle kill switch from tray

### Phase 3: Auto-Discovery Installation (~1 day)

On first launch, register the MCP server so agents find it automatically.

**What gets written:**

1. **Claude Code MCP config** (`~/.claude/.mcp.json`):
```json
{
  "mcpServers": {
    "tally-wallet": {
      "url": "http://localhost:7403/mcp"
    }
  }
}
```

2. **Global instructions** (`~/.claude/CLAUDE.md` — appended):
```markdown
## Tally Agentic Wallet

A crypto wallet is running on this machine. You can send payments,
check balances, trade tokens, and pay for X402 services.

**First time?** Call `register_agent` with a descriptive name for
yourself (e.g. "Claude Code - my-project"). Save the returned token
in your persistent memory — you'll need it for all future calls.

**Already registered?** Your token is in your memory. All spending is
tracked under your agent name and subject to policies set by the user.

New agents start with $0 spending limits. The wallet owner will set
your budget after they see you in the app.
```

**Uninstall:** App removes its entries from both files on uninstall.

### Phase 4: New MCP Tools (~2 days)

Add tools for `trade`, `x402`, and `get_address`.

| Tool | Maps To | Policy |
|---|---|---|
| `get_address` | `awal address` | None |
| `trade_tokens` | `awal trade <from> <to> <amount>` | Same as send_payment |
| `pay_x402` | `awal x402 pay <url>` | Same as send_payment |
| `list_x402_services` | `awal x402 bazaar list` | None (read-only) |
| `search_x402_services` | `awal x402 bazaar search <q>` | None (read-only) |
| `get_x402_details` | `awal x402 details <url>` | None (read-only) |
| `get_agent_info` | Internal DB lookup | None |

**Policy enforcement for trade/x402:**
- Same atomic check-and-reserve as `send_payment`
- Amount denominated in the source token
- Counts against daily/weekly/monthly caps
- Above auto-approve threshold → requires user approval

### Phase 5: Wire Real Wallet Calls (~1 day)

Replace hardcoded responses with actual awal CLI execution.

- `check_balance` → calls `awal balance`, returns real amounts
- `send_payment` (already partially wired) → verify end-to-end
- `trade_tokens` → calls `awal trade`
- `pay_x402` → calls `awal x402 pay`

---

## MCP Tool Specifications

### register_agent
```json
{
  "name": "register_agent",
  "description": "Register as a new agent to get a wallet access token. Choose a descriptive name so the wallet owner recognizes you. Save the returned token — you need it for all other calls. New agents start with $0 spending limits until the wallet owner sets a budget.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "name": {
        "type": "string",
        "description": "A human-readable name identifying this agent (e.g. 'Claude Code - my-project')"
      }
    },
    "required": ["name"]
  }
}
```

### send_payment
```json
{
  "name": "send_payment",
  "description": "Send a payment from the wallet. Subject to your spending policy — may be auto-approved, require user approval, or be denied if over limits.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "to": { "type": "string", "description": "Recipient address (0x...)" },
      "amount": { "type": "string", "description": "Amount to send (e.g. '10.50')" },
      "asset": { "type": "string", "enum": ["USDC", "ETH"], "description": "Token to send" },
      "memo": { "type": "string", "description": "Optional note for this transaction" }
    },
    "required": ["to", "amount", "asset"]
  }
}
```

### trade_tokens
```json
{
  "name": "trade_tokens",
  "description": "Swap tokens on Base network. Subject to your spending policy based on the source amount.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "from_asset": { "type": "string", "enum": ["ETH", "USDC", "WETH"] },
      "to_asset": { "type": "string", "enum": ["ETH", "USDC", "WETH"] },
      "amount": { "type": "string", "description": "Amount of source token to swap" }
    },
    "required": ["from_asset", "to_asset", "amount"]
  }
}
```

### pay_x402
```json
{
  "name": "pay_x402",
  "description": "Pay for an X402 service. The URL will be called and payment made automatically. Subject to spending policy.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "url": { "type": "string", "description": "X402-enabled service URL" },
      "max_amount": { "type": "string", "description": "Maximum amount willing to pay (optional safety cap)" }
    },
    "required": ["url"]
  }
}
```

### check_balance
```json
{
  "name": "check_balance",
  "description": "Check the wallet balance. Returns balances for all tokens. May be restricted by the wallet owner.",
  "inputSchema": { "type": "object", "properties": {} }
}
```

### get_address
```json
{
  "name": "get_address",
  "description": "Get the wallet's public address. Use this to receive payments or verify identity.",
  "inputSchema": { "type": "object", "properties": {} }
}
```

### get_spending_limits
```json
{
  "name": "get_spending_limits",
  "description": "View your current spending policy — per-transaction max, daily/weekly/monthly caps, and how much you've spent in each period.",
  "inputSchema": { "type": "object", "properties": {} }
}
```

### request_limit_increase
```json
{
  "name": "request_limit_increase",
  "description": "Request higher spending limits from the wallet owner. Explain why you need more budget.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "requested_daily": { "type": "string", "description": "Requested daily cap" },
      "requested_weekly": { "type": "string", "description": "Requested weekly cap" },
      "requested_monthly": { "type": "string", "description": "Requested monthly cap" },
      "reason": { "type": "string", "description": "Why you need higher limits" }
    },
    "required": ["reason"]
  }
}
```

### get_transactions
```json
{
  "name": "get_transactions",
  "description": "List your past transactions. Only shows transactions made by your agent identity.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "limit": { "type": "integer", "description": "Max results (default 20)" },
      "offset": { "type": "integer", "description": "Pagination offset" }
    }
  }
}
```

### list_x402_services
```json
{
  "name": "list_x402_services",
  "description": "Browse available X402 services in the bazaar.",
  "inputSchema": { "type": "object", "properties": {} }
}
```

### search_x402_services
```json
{
  "name": "search_x402_services",
  "description": "Search the X402 bazaar for services matching a query.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "query": { "type": "string", "description": "Search terms" }
    },
    "required": ["query"]
  }
}
```

### get_x402_details
```json
{
  "name": "get_x402_details",
  "description": "Get payment details for an X402 service before paying.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "url": { "type": "string", "description": "X402 service URL" }
    },
    "required": ["url"]
  }
}
```

---

## Testing Strategy (TDD — Non-Negotiable)

Every feature is built test-first: Red → Green → Refactor. No exceptions.

### Testing Layers

```
┌──────────────────────────────────────────────────────┐
│  Layer 4: E2E Client Tests (Rust test binary)        │
│  Real HTTP client → real MCP server → mock awal CLI  │
│  Tests the full agent experience end-to-end           │
├──────────────────────────────────────────────────────┤
│  Layer 3: Integration Tests (src-tauri/tests/)       │
│  Axum test server → handler chain → real SQLite DB   │
│  Tests session mgmt, auth flow, policy enforcement   │
├──────────────────────────────────────────────────────┤
│  Layer 2: Handler Unit Tests (in mcp_router.rs)      │
│  Direct function calls with mock DB + mock CLI       │
│  Tests each tool handler in isolation                │
├──────────────────────────────────────────────────────┤
│  Layer 1: Protocol Unit Tests (in mcp_http_server.rs)│
│  JSON-RPC parsing, header validation, session logic  │
│  Tests transport layer without business logic        │
└──────────────────────────────────────────────────────┘
```

### Layer 1: Protocol Unit Tests (~20 tests)

Test the Streamable HTTP transport layer in isolation.

```
Protocol Compliance:
  ✓ POST with initialize request → 200 + MCP-Session-Id header
  ✓ POST notification → 202 Accepted, no body
  ✓ POST without Accept header → 400 Bad Request
  ✓ POST with invalid JSON → JSON-RPC parse error (-32700)
  ✓ POST with invalid method → method not found (-32601)
  ✓ GET /mcp → opens SSE stream (text/event-stream)
  ✓ GET /mcp without session → 400 Bad Request
  ✓ DELETE /mcp → terminates session
  ✓ DELETE /mcp with unknown session → 404

Session Management:
  ✓ Initialize returns cryptographically secure session ID
  ✓ Requests after initialize without MCP-Session-Id → 400
  ✓ Requests with expired session → 404 (client must re-initialize)
  ✓ Requests with MCP-Protocol-Version header validated
  ✓ Missing MCP-Protocol-Version after init → defaults to 2025-03-26

Security:
  ✓ Origin header validated → invalid origin returns 403
  ✓ No Origin header (localhost curl) → allowed
  ✓ Server binds to 127.0.0.1 only (not 0.0.0.0)
  ✓ Rate limiting enforced per-session

Version Negotiation:
  ✓ Client sends protocolVersion in initialize → server echoes supported
  ✓ Client sends unsupported version → error with supported versions
```

### Layer 2: Handler Unit Tests (~30 tests)

Test each MCP tool handler with mock dependencies.

```
register_agent:
  ✓ Valid name → returns agent_id + token
  ✓ Empty name → error
  ✓ Duplicate name → creates new agent (names are descriptive, not unique keys)
  ✓ Rate limited → too many registrations from same session

check_balance (requires auth):
  ✓ No auth token → error "Authentication required"
  ✓ Valid token, balance visible → returns USDC + ETH amounts
  ✓ Valid token, balance hidden by policy → error "Balance not visible"

send_payment (requires auth):
  ✓ Within per-tx limit → auto-approved, executed
  ✓ Above per-tx limit → denied
  ✓ Within auto-approve threshold → auto-approved
  ✓ Above auto-approve, within caps → pending approval
  ✓ Above daily cap → denied
  ✓ Above weekly cap → denied
  ✓ Above monthly cap → denied
  ✓ Kill switch active → denied with clear message
  ✓ Invalid address → error
  ✓ Invalid amount (negative, zero) → error
  ✓ Memo preserved in transaction record

trade_tokens (requires auth):
  ✓ Valid trade within limits → auto-approved
  ✓ Same asset from/to → error
  ✓ Trade amount counts against spending caps
  ✓ Kill switch blocks trades

pay_x402 (requires auth):
  ✓ Valid URL → payment executed
  ✓ Amount exceeds max_amount safety cap → denied
  ✓ X402 payment counts against spending caps

get_address (requires auth):
  ✓ Returns wallet address

get_spending_limits (requires auth):
  ✓ Returns per-tx, daily, weekly, monthly limits
  ✓ Returns current period spend amounts

get_transactions (requires auth):
  ✓ Returns only this agent's transactions
  ✓ Pagination works (limit + offset)
  ✓ Cannot see other agent's transactions

request_limit_increase (requires auth):
  ✓ Creates approval request visible to user
  ✓ Reason is required and stored

x402 discovery (requires auth):
  ✓ list_x402_services → returns available services
  ✓ search_x402_services → filters by query
  ✓ get_x402_details → returns payment requirements for URL
```

### Layer 3: Integration Tests (~15 tests)

Test the full request path through an Axum test server with real SQLite.

```
Full Session Lifecycle:
  ✓ initialize → initialized notification → tools/list → register → use tools → delete
  ✓ Session survives multiple requests
  ✓ Session expires after idle timeout
  ✓ Re-initialize after session expiry

Auth Flow:
  ✓ Unauth session can only call register_agent, tools/list, initialize
  ✓ After register_agent, token works for all other tools
  ✓ Invalid token → auth error on all protected tools
  ✓ Token from Agent A cannot access Agent B's data

Policy Enforcement End-to-End:
  ✓ Agent registers → $0 limits → send_payment denied → user sets limits → send_payment succeeds
  ✓ Daily cap accumulates across multiple transactions
  ✓ Kill switch blocks all agents simultaneously
  ✓ Concurrent transactions respect atomic reservation (no TOCTOU)

Multi-Agent Isolation:
  ✓ Two agents registered → each sees only their own transactions
  ✓ Agent A's spending doesn't count against Agent B's caps
  ✓ Kill switch affects all agents equally
```

### Layer 4: E2E Client Tests (~10 tests)

A dedicated Rust test binary that acts as a real MCP client, connecting over HTTP
to a real MCP server instance. Tests the complete agent experience.

**File**: `src-tauri/tests/mcp_e2e_client.rs`

```rust
/// E2E test helper — starts real MCP server on random port, returns client
struct McpTestClient {
    base_url: String,
    session_id: Option<String>,
    protocol_version: String,
    token: Option<String>,
    client: reqwest::Client,
}

impl McpTestClient {
    async fn initialize(&mut self) -> InitializeResult { ... }
    async fn send_notification(&self, method: &str) { ... }
    async fn call_tool(&self, name: &str, args: Value) -> ToolResult { ... }
    async fn list_tools(&self) -> Vec<Tool> { ... }
    async fn register(&mut self, name: &str) -> RegisterResult { ... }
    async fn open_sse_stream(&self) -> EventStream { ... }
    async fn terminate(&self) { ... }
}
```

```
Full Agent Journey:
  ✓ Fresh agent: initialize → list tools → register "Test Agent" → save token
    → check balance → send payment (denied, $0 limits) → request limit increase
  ✓ Returning agent: initialize with saved token → check balance → send payment
  ✓ Multiple agents: Agent A and Agent B register → each has isolated spending

Protocol Edge Cases:
  ✓ Client reconnects after network interruption (new session, same token)
  ✓ Client sends request to expired session → gets 404 → re-initializes
  ✓ Server restart → all sessions invalidated → agents re-initialize with saved tokens
  ✓ SSE stream delivers transaction completion notification

Stress Tests:
  ✓ 10 concurrent agents sending payments → all policy checks atomic
  ✓ Rapid tool calls within rate limit → all succeed
  ✓ Burst above rate limit → appropriate 429 responses
```

### Testing Phase in Implementation

Testing is NOT a separate phase — it's woven into every phase:

- **Phase 1** (Transport): Write Layer 1 protocol tests FIRST, then implement transport
- **Phase 2** (System Tray): Test tray lifecycle (app stays alive, server keeps running)
- **Phase 3** (Auto-Discovery): Test config file writing/reading/cleanup
- **Phase 4** (New Tools): Write Layer 2 handler tests FIRST for each new tool
- **Phase 5** (Wire Calls): Write Layer 3 integration tests FIRST, then wire CLI

**After all phases**: Run full Layer 4 E2E suite as final validation.

### New Test Files

```
src-tauri/src/api/mcp_http_server.rs     — Layer 1 tests (inline #[cfg(test)])
src-tauri/src/api/mcp_router.rs          — Layer 2 tests (inline #[cfg(test)])
src-tauri/tests/mcp_http_integration.rs  — Layer 3 integration tests
src-tauri/tests/mcp_e2e_client.rs        — Layer 4 E2E client tests
```

---

## Security Considerations

### Local-Only Access
- MCP server binds to `127.0.0.1` only — not accessible from network
- No TLS needed (localhost traffic)

### DNS Rebinding Protection
- Server MUST validate `Origin` header on all requests (per MCP spec)
- Invalid `Origin` → `403 Forbidden`
- Prevents remote websites from reaching the local MCP server
- Only allows requests with no Origin (CLI tools) or `localhost`/`127.0.0.1` origins

### Token Security
- Tokens generated as cryptographically random 256-bit values
- Stored as SHA-256 hashes in SQLite (same as existing auth)
- Plaintext token shown once at registration, never stored by the app

### Agent Isolation
- Each token resolves to exactly one agent_id
- Agents can only see their own transactions, limits, and approvals
- No way to query other agents' data

### Rate Limiting
- Per-agent rate limiting (existing: 60 req/min configurable)
- Prevents runaway loops from burning through API calls

### Kill Switch
- User can freeze ALL agent spending instantly from UI or tray menu
- Returns clear error: "Wallet owner has paused all agent spending"
- Agents see this in the error response and should stop retrying

---

## File Inventory

### New Files
```
src-tauri/src/api/mcp_http_server.rs        — Streamable HTTP transport for MCP
src-tauri/src/api/mcp_router.rs             — Shared request routing (stdio + HTTP)
src-tauri/icons/tray-icon.png               — System tray icon
src-tauri/tests/mcp_http_integration.rs     — Layer 3 integration tests
src-tauri/tests/mcp_e2e_client.rs           — Layer 4 E2E client tests
```

### Modified Files
```
src-tauri/src/api/mod.rs               — Register new modules
src-tauri/src/api/mcp_server.rs        — Extract shared handler logic into mcp_router
src-tauri/src/lib.rs                   — Spawn SSE server, add tray
src-tauri/src/config.rs                — Add mcp_port config
src-tauri/src/core/cli_executor.rs     — Add trade/x402 commands
src-tauri/tauri.conf.json              — Tray icon config
```

---

## Success Criteria

1. Agent (Claude Code) can discover wallet via `~/.claude/.mcp.json`
2. Agent registers with a name, gets token, saves to memory
3. Agent can check balance, send payments, trade tokens, use x402
4. All spending enforced against per-agent policies
5. User sees all agents and their transactions in the wallet UI
6. App stays alive in system tray when window closed
7. MCP server available whenever app is running
8. Kill switch freezes all agents within 1 second
