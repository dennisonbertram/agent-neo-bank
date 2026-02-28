# MCP Server Implementation Plan Review

**Reviewed**: 2026-02-28
**Reviewer**: Claude (Opus 4.6)
**Document**: `docs/implementation/mcp-server-plan.md`
**MCP SDK Reference Version**: `@modelcontextprotocol/sdk@1.27.1` (installed in project)

---

## 1. Protocol Correctness

### 1.1 Transport Type

**VERDICT: Correct.** Streamable HTTP is the current standard transport. The plan correctly identifies this and correctly notes that the legacy SSE-only transport (`/sse` + `/messages`) is deprecated. Good.

### 1.2 POST/GET/DELETE Endpoints

**VERDICT: Correct.** The three endpoints on a single path (`/mcp`) match the SDK implementation:
- `POST /mcp` for JSON-RPC requests -- correct
- `GET /mcp` for SSE stream (server-initiated messages) -- correct
- `DELETE /mcp` for session termination -- correct

### 1.3 Session Management via `mcp-session-id` Header

**VERDICT: Correct.** The SDK uses `mcp-session-id` (lowercase) as the header name. The plan correctly describes:
- Server generates session ID on `initialize`
- Session ID returned in response headers
- Client includes it on subsequent requests
- Invalid session IDs rejected with 404
- Missing session IDs on non-init requests rejected with 400

### 1.4 Protocol Version

| Severity | Issue |
|----------|-------|
| **HIGH** | Protocol version `2024-11-05` is outdated |

The plan states protocol version `2024-11-05` is "current stable as of 2026". This is **wrong**. According to the SDK (v1.27.1):

```typescript
LATEST_PROTOCOL_VERSION = '2025-11-25'
DEFAULT_NEGOTIATED_PROTOCOL_VERSION = '2025-03-26'
SUPPORTED_PROTOCOL_VERSIONS = ['2025-11-25', '2025-06-18', '2025-03-26', '2024-11-05', '2024-10-07']
```

The latest protocol version is `2025-11-25`. The default negotiated version is `2025-03-26`. While `2024-11-05` is still in the supported list (backward compatible), the plan should:
1. Update the stated protocol version to `2025-11-25` (latest) or at minimum `2025-03-26` (default negotiated)
2. Support version negotiation -- the SDK handles this automatically, but the Rust implementation needs to account for the `MCP-Protocol-Version` header validation

Additionally, the SDK now validates an `MCP-Protocol-Version` header on subsequent requests (not just during initialization). The plan does not mention this header at all.

### 1.5 `~/.claude/.mcp.json` Config Format

| Severity | Issue |
|----------|-------|
| **HIGH** | The `~/.claude/.mcp.json` file does not exist on this machine; Claude Code uses `~/.claude/CLAUDE.md` and project-level `.mcp.json` for MCP server configs |

The plan proposes writing to `~/.claude/.mcp.json`:
```json
{
  "mcpServers": {
    "tally-wallet": {
      "type": "streamable-http",
      "url": "http://localhost:7403/mcp"
    }
  }
}
```

**Issues:**
1. On this machine, there is no `~/.claude/.mcp.json`. The project-level `.mcp.json` exists at the repo root. The actual format observed in `.mcp.json` uses `command` + `args` for stdio-based servers. The `type: "streamable-http"` with `url` field is the correct format for HTTP-based MCP servers in Claude Code, but the plan should verify this is the actual supported config schema, as it may vary by Claude Code version.

2. Writing to a user's global `~/.claude/CLAUDE.md` on first launch is **invasive**. If the user has existing content, appending could corrupt formatting or conflict with other instructions. The plan should use the project-level `.mcp.json` approach or at minimum ask user permission before modifying global files.

### 1.6 Missing: `MCP-Protocol-Version` Header

| Severity | Issue |
|----------|-------|
| **MEDIUM** | Plan does not mention the `MCP-Protocol-Version` header |

The SDK validates this header on subsequent requests after initialization. The plan's Rust implementation needs to:
- Accept and validate this header on POST requests
- Respond with 400 if the version is not in the supported list
- Default to the negotiated version if the header is absent

### 1.7 Missing: Resumability / EventStore

| Severity | Issue |
|----------|-------|
| **LOW** | Plan does not mention resumability support |

The SDK supports an `EventStore` interface for resumability (clients can reconnect and resume via `Last-Event-ID`). This is optional but worth noting as a future enhancement. Not blocking.

### 1.8 Missing: `enableJsonResponse` Option

| Severity | Issue |
|----------|-------|
| **LOW** | Plan does not discuss JSON vs SSE response mode |

The SDK supports `enableJsonResponse: true` to return plain JSON instead of SSE streams for simple request/response. Since this is a local server with low latency, JSON responses would be simpler and more efficient for most tool calls. SSE is only needed for long-running operations. Worth considering.

---

## 2. Architecture Review

### 2.1 Axum HTTP Server Alongside Tauri

| Severity | Issue |
|----------|-------|
| **MEDIUM** | Potential port conflict and lifecycle concerns |

The plan proposes running an Axum HTTP server on port 7403 alongside the existing REST API on 7402. Concerns:

1. **Port availability**: No fallback if 7403 is occupied. Should attempt binding and report a clear error, or try alternative ports.
2. **Lifecycle coupling**: The MCP server starts with the Tauri app and relies on system tray to keep it alive. If the app crashes, agents lose connectivity with no notification. Consider a health-check endpoint or heartbeat mechanism.
3. **Two HTTP servers**: Running two separate Axum instances (REST on 7402, MCP on 7403) is fine architecturally but consider whether they could share a single server with path-based routing (`/api/*` for REST, `/mcp` for MCP). This would reduce port management complexity.

### 2.2 Session Lifecycle and Cleanup

| Severity | Issue |
|----------|-------|
| **MEDIUM** | No idle timeout specified; no maximum session limit |

The plan mentions "session expires after idle timeout" in the session lifecycle description but never specifies the timeout duration or maximum concurrent sessions. Missing details:

1. **Idle timeout**: What value? 30 minutes? 1 hour? Should be configurable.
2. **Max sessions**: A runaway agent could create unlimited sessions. Need a per-agent session limit.
3. **Cleanup mechanism**: No mention of how expired sessions are cleaned up (timer? lazy eviction on access?).

### 2.3 Auth Flow

| Severity | Issue |
|----------|-------|
| **HIGH** | The `/mcp/register` endpoint breaks MCP protocol assumptions |

The plan adds a separate `POST /mcp/register` endpoint outside the MCP protocol. This is architecturally problematic:

1. **Not MCP-compliant**: MCP clients (Claude Code, Cursor) only know how to call MCP tools via `tools/call`. They do not know about arbitrary REST endpoints. The `register_agent` tool is already defined as an MCP tool -- it should work through the standard `POST /mcp` flow.

2. **Chicken-and-egg problem**: The plan says `register_agent` requires no auth, but the `POST /mcp` endpoint requires `Authorization: Bearer <token>`. How does an unregistered agent call any MCP endpoint?

   **Recommended fix**: Allow `POST /mcp` without auth for `initialize` + `tools/call` with `name: "register_agent"` only. All other tools require a valid token. This keeps everything within the MCP protocol.

3. **Token delivery**: The plan says the agent "saves the returned token to its own persistent memory." This relies on the agent being smart enough to do this. The MCP tool description is good about instructing this, but there is no enforcement. If the agent loses the token, there is no recovery mechanism (no "forgot token" flow).

### 2.4 Agent Isolation

**VERDICT: Well-designed.** Each token maps to one agent, agents see only their own data. The spending policy engine provides proper isolation. No cross-agent data leakage vectors identified.

### 2.5 Spending Policy Enforcement

**VERDICT: Well-designed.** The atomic check-and-reserve (TOCTOU protection) pattern is correctly referenced from Phase 2.5. The plan correctly applies the same policy enforcement to `trade_tokens` and `pay_x402` as `send_payment`.

---

## 3. Tool Completeness

### 3.1 Tool Design Quality

Overall the 13 tools are well-designed with clear descriptions, proper input schemas, and appropriate policy enforcement flags. Specific issues:

### 3.2 `register_agent`

| Severity | Issue |
|----------|-------|
| **MEDIUM** | Missing optional metadata fields |

Consider adding:
- `description`: Optional description of what the agent does
- `capabilities`: What the agent intends to use (payments, trading, x402) -- helps the user set appropriate policies

### 3.3 `send_payment`

| Severity | Issue |
|----------|-------|
| **MEDIUM** | Missing `chain` parameter |

The `send_payment` tool assumes Base network but does not make this explicit. If the wallet supports multiple chains in the future, this will need a breaking change. Consider adding `chain` with a default of `"base"`.

### 3.4 `trade_tokens`

| Severity | Issue |
|----------|-------|
| **LOW** | Hardcoded asset enum |

The `enum: ["ETH", "USDC", "WETH"]` is hardcoded. If new tokens are supported, the tool schema must be updated. Consider making this dynamic or at least documenting how to extend it.

### 3.5 `pay_x402`

| Severity | Issue |
|----------|-------|
| **MEDIUM** | No response schema for payment details |

The tool accepts a URL and optional max amount but does not describe what it returns. The agent needs to know: transaction hash, amount paid, asset used, service response. Define the expected response format in the tool description.

### 3.6 `check_balance`

| Severity | Issue |
|----------|-------|
| **LOW** | "May be restricted by the wallet owner" is vague |

The description says balance may be restricted but does not explain what happens when it is restricted. Does it return an error? An empty response? A message saying "restricted"? Clarify the behavior.

### 3.7 `get_transactions`

| Severity | Issue |
|----------|-------|
| **LOW** | No date range filtering |

The tool supports `limit` and `offset` pagination but no date range filtering (`from_date`, `to_date`). This would be useful for agents tracking their own spending over specific periods.

### 3.8 Missing Tools

| Severity | Issue |
|----------|-------|
| **LOW** | No `get_approval_status` tool |

When a transaction requires approval (above auto-approve threshold), the agent has no way to check if it was approved or denied. Consider adding a tool that lets the agent poll for approval status by transaction/request ID.

---

## 4. Implementation Phases

### 4.1 Phase Ordering

**VERDICT: Generally correct but with a dependency issue.**

| Severity | Issue |
|----------|-------|
| **MEDIUM** | Phase 3 (Auto-Discovery) should come after Phase 5 (Wire Real Wallet) |

The auto-discovery phase installs the MCP server into Claude Code's config and writes instructions telling agents they can use the wallet. But if Phases 4 and 5 are not yet complete, the agents will discover a wallet that cannot actually perform trades or x402 payments. This creates a bad first impression.

**Recommended order**: Phase 1 (Transport) -> Phase 4 (New Tools) -> Phase 5 (Wire Wallet) -> Phase 2 (System Tray) -> Phase 3 (Auto-Discovery)

### 4.2 Time Estimates

| Severity | Issue |
|----------|-------|
| **MEDIUM** | Phase 1 estimate of ~2 days is optimistic |

Implementing Streamable HTTP in Rust from scratch (not using the TypeScript SDK) requires:
- JSON-RPC 2.0 parsing and routing
- SSE stream management with proper formatting
- Session management with concurrent access
- Protocol version negotiation
- `MCP-Protocol-Version` header validation
- Content-Type negotiation (JSON vs SSE response)
- Error handling per MCP spec error codes

This is more like 3-5 days for a solid implementation with tests. The TypeScript SDK handles much of this complexity automatically; doing it in Rust means reimplementing all of it.

### 4.3 Missing Phase: Testing

| Severity | Issue |
|----------|-------|
| **HIGH** | No testing phase despite project TDD requirement |

The project `CLAUDE.md` states "All code follows strict TDD. No exceptions." The plan mentions a test file (`src-tauri/tests/mcp_http.rs`) in the file inventory but there is no testing phase in the implementation plan. There should be an explicit phase (or sub-phase within each phase) for:
- Unit tests for JSON-RPC parsing
- Unit tests for session management
- Integration tests for the full Streamable HTTP flow
- Tests for policy enforcement through MCP tools
- Tests for the registration flow
- Tests for error cases (invalid token, expired session, rate limit exceeded)

---

## 5. Security

### 5.1 Localhost-Only Binding

**VERDICT: Appropriate for the threat model.** Binding to `127.0.0.1` prevents network access. No TLS needed for localhost.

| Severity | Issue |
|----------|-------|
| **MEDIUM** | DNS rebinding attack not addressed |

The plan binds to `127.0.0.1` but does not mention DNS rebinding protection. A malicious website could potentially make requests to `localhost:7403` via a DNS rebinding attack. The SDK has built-in support for this (`allowedHosts`, `allowedOrigins`, `enableDnsRebindingProtection`) though these are marked deprecated in favor of external middleware.

For a financial application handling real money, the Rust server should:
1. Validate the `Origin` header (reject non-localhost origins)
2. Validate the `Host` header (only accept `localhost:7403` or `127.0.0.1:7403`)
3. Consider CORS headers that restrict access

### 5.2 Token Generation

**VERDICT: Good.** 256-bit cryptographically random tokens stored as SHA-256 hashes. Shown once at registration. This follows good practice.

| Severity | Issue |
|----------|-------|
| **LOW** | No token rotation mechanism |

If a token is compromised (e.g., agent's memory is leaked), there is no way to rotate it without re-registering. Consider adding a `rotate_token` operation accessible from the UI.

### 5.3 Rate Limiting

| Severity | Issue |
|----------|-------|
| **MEDIUM** | Rate limiting on registration endpoint |

The plan mentions 60 req/min per agent for authenticated endpoints, but the `POST /mcp/register` (or unauthenticated register flow) has no rate limiting mentioned. An attacker could spam registration to:
- Fill the database with garbage agents
- Exhaust agent ID space
- Create noise in the UI

Should add IP-based or global rate limiting on registration (e.g., max 10 registrations per hour).

### 5.4 Kill Switch

**VERDICT: Well-designed.** Instant freeze from UI or tray, clear error message to agents. The 1-second response time target is appropriate.

### 5.5 Token in Authorization Header vs MCP Protocol

| Severity | Issue |
|----------|-------|
| **MEDIUM** | Unclear how the Authorization header interacts with MCP session initialization |

The MCP protocol does not define authentication -- it is transport-layer concern. The plan uses `Authorization: Bearer <token>` which is fine, but the interaction with MCP initialization is unclear:

1. Does the agent send the Bearer token on the `initialize` call?
2. What if an agent initializes without a token (intending to register)?
3. The SDK does not natively handle `Authorization` headers -- this is middleware. The plan should explicitly describe how auth middleware interacts with the MCP transport layer.

---

## Summary

### Issue Counts

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 3 |
| MEDIUM | 9 |
| LOW | 5 |

### HIGH Issues (must fix before implementation)

1. **Protocol version is outdated**: Update from `2024-11-05` to `2025-11-25` (latest) and implement `MCP-Protocol-Version` header validation.
2. **`/mcp/register` breaks MCP protocol**: Registration should happen through the standard MCP `tools/call` flow, not a separate REST endpoint. MCP clients do not know about arbitrary endpoints.
3. **No testing phase**: Violates the project's non-negotiable TDD requirement. Add explicit testing requirements to each phase.

### Key Recommendations

1. **Reorder phases**: Move Auto-Discovery (Phase 3) to after Wire Real Wallet (Phase 5) so agents discover a fully functional server.
2. **Use standard MCP flow for registration**: Allow unauthenticated `initialize` + `register_agent` tool call, then require auth for everything else.
3. **Add DNS rebinding protection**: Validate `Origin` and `Host` headers on all requests.
4. **Rate limit registration**: Add global rate limiting on unauthenticated endpoints.
5. **Revise time estimates**: Phase 1 is likely 3-5 days given the complexity of implementing Streamable HTTP in Rust from scratch.
6. **Consider JSON response mode**: For simple tool calls, returning JSON instead of SSE reduces complexity.
7. **Add `get_approval_status` tool**: Agents need to check if pending transactions were approved.
