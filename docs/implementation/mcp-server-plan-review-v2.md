# MCP Server Implementation Plan Review (v2)

**Reviewed**: 2026-02-28
**Reviewer**: Claude (Opus 4.6)
**Document**: `docs/implementation/mcp-server-plan.md`
**MCP SDK Reference Version**: `@modelcontextprotocol/sdk@1.27.1` (installed in project)
**Previous Review**: `docs/implementation/mcp-server-plan-review.md`

---

## Previous Review: HIGH Issue Resolution

The v1 review identified 3 HIGH issues. Checking whether the updated plan addresses them:

### HIGH #1: Protocol version was outdated (`2024-11-05`)

**Status: FIXED.** The plan now correctly states protocol version `2025-11-25` and references the Streamable HTTP transport. The `MCP-Protocol-Version` header is documented in the Required Headers table. Version negotiation is mentioned in the Layer 1 test cases (test for unsupported version returning error with supported versions list).

### HIGH #2: `/mcp/register` broke MCP protocol (separate REST endpoint)

**Status: FIXED.** The plan now correctly places `register_agent` as a standard MCP tool call via `tools/call` within the `POST /mcp` flow. Unauthenticated sessions can call `initialize`, `tools/list`, and `register_agent` only. All other tools require `Authorization: Bearer <token>`. This is the correct approach -- no separate endpoints outside the MCP protocol.

### HIGH #3: No testing phase despite TDD requirement

**Status: FIXED.** The plan now includes a comprehensive 4-layer testing strategy with approximately 75 test cases across protocol, handler, integration, and E2E layers. Testing is explicitly woven into each implementation phase (not a separate phase). This satisfies the project's TDD requirement.

---

## 1. Protocol Correctness

### 1.1 Protocol Version

**VERDICT: Correct.** Verified against the installed SDK (`@modelcontextprotocol/sdk@1.27.1`):

```javascript
// From node_modules/@modelcontextprotocol/sdk/dist/esm/types.js
LATEST_PROTOCOL_VERSION = '2025-11-25'
DEFAULT_NEGOTIATED_PROTOCOL_VERSION = '2025-03-26'
SUPPORTED_PROTOCOL_VERSIONS = ['2025-11-25', '2025-06-18', '2025-03-26', '2024-11-05', '2024-10-07']
```

Note: The SDK's `spec.types.js` (auto-generated from the draft schema) contains `LATEST_PROTOCOL_VERSION = "DRAFT-2026-v1"`, but this is marked `@internal` and not exposed as the public protocol version. The plan correctly targets `2025-11-25`.

### 1.2 Streamable HTTP Transport

**VERDICT: Correct.** The plan accurately describes:
- Single endpoint (`/mcp`) handling POST, GET, and DELETE -- matches SDK implementation
- POST for JSON-RPC requests/notifications -- correct
- GET for server-initiated SSE stream -- correct
- DELETE for session termination -- correct
- `202 Accepted` for notifications/responses with no body -- correct

### 1.3 Required Headers

**VERDICT: Mostly correct, one error.**

| Header | Plan Description | SDK Behavior | Match? |
|---|---|---|---|
| `MCP-Session-Id` | Required after initialize | Lowercase `mcp-session-id` in SDK, but HTTP headers are case-insensitive | Yes |
| `MCP-Protocol-Version` | Required after initialize | Validated on all POST/GET/DELETE after init; returns 400 if unsupported | Yes |
| `Accept` | POST: `application/json, text/event-stream`; GET: `text/event-stream` | POST requires both; GET requires `text/event-stream` | Yes |

| Severity | Issue |
|----------|-------|
| **LOW** | Plan says missing Accept header returns `400 Bad Request`; SDK actually returns `406 Not Acceptable` |

In the Layer 1 test case "POST without Accept header -> 400 Bad Request", the expected status code should be `406 Not Acceptable`, not `400`. The SDK source at `webStandardStreamableHttp.js:189` returns: `createJsonErrorResponse(406, -32000, 'Not Acceptable: Client must accept text/event-stream')`.

### 1.4 Session Management

**VERDICT: Correct.** The plan accurately describes:
- Server assigns session ID via `MCP-Session-Id` response header on InitializeResult
- Session ID must be globally unique and cryptographically secure
- Missing session ID on non-init requests returns `400`
- Expired/unknown session ID returns `404`
- Client must re-initialize on `404`

| Severity | Issue |
|----------|-------|
| **MEDIUM** | Plan says "Missing MCP-Protocol-Version after init -> defaults to 2025-03-26" but the actual default is the *negotiated* version |

The SDK's `validateProtocolVersion` accepts a missing `mcp-protocol-version` header (it only rejects *unsupported* values). When the header is absent, the negotiated version from initialization is used. The plan's test case states a default of `2025-03-26` which happens to be the `DEFAULT_NEGOTIATED_PROTOCOL_VERSION`, but the actual default should be whatever version was negotiated during `initialize` -- which could be any version in the supported list depending on what the client requested. The Rust implementation should store the negotiated version per-session and use that as the default, not hardcode `2025-03-26`.

### 1.5 DNS Rebinding Protection

**VERDICT: Correct.** The plan correctly requires:
- Origin header validation (invalid origin returns `403 Forbidden`)
- No Origin header (CLI tools) is allowed
- Server binds to `127.0.0.1` only

This matches the SDK's `enableDnsRebindingProtection` behavior which validates the `Origin` header against an `allowedOrigins` list.

| Severity | Issue |
|----------|-------|
| **MEDIUM** | Plan does not mention `Host` header validation |

The SDK's DNS rebinding protection validates both `Origin` and `Host` headers. The `Host` header check ensures requests are addressed to `localhost`/`127.0.0.1` and not to a DNS-rebound hostname. For a financial application, the Rust implementation should validate the `Host` header as well. Specifically, it should reject requests where the `Host` header does not match `localhost:7403` or `127.0.0.1:7403`.

### 1.6 Resumability

**VERDICT: Correct.** The plan correctly notes resumability as optional and describes the `id` field on SSE events and `Last-Event-ID` header mechanism. Deferring this to a future enhancement is reasonable.

### 1.7 SSE Stream Limit

| Severity | Issue |
|----------|-------|
| **LOW** | Plan does not mention the one-SSE-stream-per-session limit |

The SDK enforces that only one GET SSE stream is allowed per session, returning `409 Conflict` if a second is attempted (`webStandardStreamableHttp.js:213`). The Rust implementation should enforce this same limit. Worth adding as a Layer 1 test case.

---

## 2. Registration Flow

### 2.1 Architecture

**VERDICT: Sound.** The approach of allowing unauthenticated `initialize` + `tools/list` + `register_agent` while requiring `Authorization: Bearer <token>` for all other tools is well-designed. It stays entirely within the MCP protocol -- no custom REST endpoints, no protocol extensions.

The flow is:
1. Agent POSTs `initialize` (no auth needed) -- gets session
2. Agent POSTs `tools/list` (no auth needed) -- discovers `register_agent`
3. Agent POSTs `tools/call` with `register_agent` (no auth needed) -- gets token
4. Agent includes `Authorization: Bearer <token>` on all subsequent tool calls

This is clean and MCP-compliant.

### 2.2 Security of Unauthenticated Registration

| Severity | Issue |
|----------|-------|
| **MEDIUM** | Rate limiting on `register_agent` is mentioned in tests but not in the security section |

The Layer 2 test case includes "Rate limited -> too many registrations from same session," which is good. However, the Security Considerations section only mentions "Per-agent rate limiting (existing: 60 req/min configurable)" and does not explicitly call out registration rate limiting. Since `register_agent` is unauthenticated, rate limiting must be enforced differently -- per-session and/or per-IP, not per-agent (since the agent does not exist yet).

The plan should explicitly specify:
- Maximum registrations per session (recommend: 1 -- an agent should only register once per session)
- Maximum registrations per time window globally (recommend: 10 per hour)
- What happens on rate limit: HTTP 429 or JSON-RPC error?

### 2.3 Token Persistence and Recovery

| Severity | Issue |
|----------|-------|
| **MEDIUM** | No token recovery mechanism for lost tokens |

The plan states agents save tokens to their own persistent memory. If an agent loses its token (memory reset, config corruption), it must re-register as a new agent. This creates orphaned agent records. Consider:
- A user-initiated "revoke and re-issue token" flow from the UI
- Allowing the user to merge duplicate agent records
- At minimum, documenting this limitation so users understand why they might see duplicate agents

### 2.4 Authorization Header on Initialize

| Severity | Issue |
|----------|-------|
| **LOW** | Plan does not specify behavior when a returning agent sends Authorization on initialize |

A returning agent (one that already has a token) will likely send the `Authorization: Bearer <token>` header on all requests including `initialize`. The plan should specify whether the server:
- Ignores the Authorization header during `initialize` (recommended)
- Uses it to associate the session with the agent immediately (alternative)

Either approach works, but it should be documented and tested.

---

## 3. Testing Strategy

### 3.1 Overall Assessment

**VERDICT: Strong.** The 4-layer testing strategy is well-structured and covers the critical paths. The test-first integration into each phase is correct for TDD. The total of ~75 test cases is appropriate for the scope.

### 3.2 Layer 1: Protocol Unit Tests (~20 tests)

**VERDICT: Good coverage.** All major protocol concerns are covered. Missing scenarios:

| Severity | Issue |
|----------|-------|
| **LOW** | Missing test cases for Layer 1 |

Add the following test cases:
- POST with Accept header that includes `application/json` but NOT `text/event-stream` should return `406`
- POST with JSON-RPC batch request (array of messages) -- the spec allows batching
- GET request that returns `409 Conflict` when a stream is already open for the session
- POST with Content-Type other than `application/json` should return `415 Unsupported Media Type`
- Response includes proper `Content-Type` header (`application/json` or `text/event-stream`)
- Fix expected status code: "POST without Accept header" should expect `406`, not `400`

### 3.3 Layer 2: Handler Unit Tests (~30 tests)

**VERDICT: Good coverage.** Comprehensive testing of each tool handler with appropriate edge cases.

| Severity | Issue |
|----------|-------|
| **LOW** | Missing test cases for Layer 2 |

Add:
- `register_agent` with very long name (boundary testing)
- `send_payment` with amount that has too many decimal places
- `send_payment` to the wallet's own address (self-send)
- `trade_tokens` with zero amount
- `pay_x402` with unreachable URL (network error handling)
- `get_transactions` with limit=0 or negative limit

### 3.4 Layer 3: Integration Tests (~15 tests)

**VERDICT: Good.** The integration tests cover the critical multi-step flows. The TOCTOU test ("concurrent transactions respect atomic reservation") is particularly important.

| Severity | Issue |
|----------|-------|
| **LOW** | Missing integration test for session cleanup after idle timeout |

The plan mentions sessions expire after idle timeout but the integration tests do not test this. Add a test that creates a session, waits past the timeout, and verifies the next request returns `404`.

### 3.5 Layer 4: E2E Client Tests (~10 tests)

**VERDICT: Good approach, but consider alternatives.**

The `McpTestClient` struct using `reqwest` is a solid approach for E2E testing. It gives full control over HTTP headers, allows testing protocol edge cases, and runs as a standard Rust test.

| Severity | Issue |
|----------|-------|
| **MEDIUM** | The E2E client should also test SSE event delivery end-to-end |

The E2E test list includes "SSE stream delivers transaction completion notification" which is good, but the `McpTestClient` struct only shows `open_sse_stream` without detailing how SSE events are consumed and verified. The implementation should:
- Use `reqwest-eventsource` or `eventsource-client` crate for proper SSE parsing
- Verify event format matches MCP spec (JSON-RPC messages as `data` fields)
- Test reconnection behavior (disconnect and reconnect with `Last-Event-ID`)

**Alternative approaches considered:**
1. **Using the TypeScript SDK as the test client**: Would test interoperability with real MCP clients but adds Node.js as a test dependency and complicates CI. Not recommended for unit/integration tests but could be valuable as a one-off interop validation.
2. **Using `tower::ServiceExt` for in-process testing**: Axum supports calling handlers directly without an HTTP server. This is faster and avoids port allocation but does not test the full HTTP stack. Already covered by Layer 3.
3. **Using the MCP Inspector**: The official MCP debugging tool could validate protocol compliance. Worth noting as a manual verification step but not suitable for automated CI.

The reqwest-based approach in the plan is the best balance of coverage, speed, and simplicity. Keep it.

### 3.6 Test Infrastructure

| Severity | Issue |
|----------|-------|
| **MEDIUM** | No test fixture specification for the mock awal CLI |

The E2E tests reference "mock awal CLI" but do not specify how CLI responses are mocked. Options:
1. A mock binary that returns hardcoded JSON responses based on arguments
2. Environment variable to switch between real and mock CLI
3. Trait-based abstraction in Rust with a mock implementation

Option 3 (trait-based) is the most testable and is already partially in place via `CliExecutor`. The plan should explicitly state that Layer 2 and Layer 3 tests use trait mocks while Layer 4 can use either trait mocks or a mock binary.

---

## 4. Additional Issues

### 4.1 Session Idle Timeout

| Severity | Issue |
|----------|-------|
| **MEDIUM** | No idle timeout value specified |

The plan references session idle timeout in the lifecycle description but never specifies a value. Recommend:
- Default: 30 minutes (configurable)
- Maximum concurrent sessions per agent: 5
- Cleanup: Background task running every 60 seconds to evict expired sessions

### 4.2 Auto-Discovery Config Format

| Severity | Issue |
|----------|-------|
| **MEDIUM** | The `~/.claude/.mcp.json` config format should be verified |

The plan proposes writing to `~/.claude/.mcp.json`:
```json
{
  "mcpServers": {
    "tally-wallet": {
      "url": "http://localhost:7403/mcp"
    }
  }
}
```

This was flagged in the v1 review. The updated plan still uses this format but has dropped the `type: "streamable-http"` field. For HTTP-based MCP servers in Claude Code, the `url` field (without `command`/`args`) is the correct way to specify a remote/HTTP server. This appears correct based on the MCP specification for Streamable HTTP clients.

However, writing to the user's global `~/.claude/CLAUDE.md` remains invasive. The plan should:
- Check for existing content before appending
- Use clear delimiters (e.g., `<!-- TALLY-WALLET-START -->` / `<!-- TALLY-WALLET-END -->`) for clean uninstall
- Ask user permission via a UI prompt before modifying global files

### 4.3 Phase Ordering

| Severity | Issue |
|----------|-------|
| **LOW** | Phase ordering concern from v1 review not addressed |

The v1 review recommended reordering phases so Auto-Discovery (Phase 3) comes after Wire Real Wallet (Phase 5). The plan still has the original order. While this is not blocking -- agents that discover the server early will simply get "not yet implemented" errors for some tools -- it creates a suboptimal first experience. Consider at least noting this trade-off in the plan.

### 4.4 Error Response Format

| Severity | Issue |
|----------|-------|
| **MEDIUM** | Plan does not specify JSON-RPC error codes for domain errors |

The plan correctly references standard JSON-RPC error codes (`-32700` parse error, `-32601` method not found) but does not define error codes for domain-specific errors like:
- Authentication required (suggest: `-32001`)
- Spending limit exceeded (suggest: `-32002`)
- Kill switch active (suggest: `-32003`)
- Approval pending (suggest: `-32004`)
- Rate limit exceeded (suggest: `-32005`)

These should be documented so agents can programmatically distinguish error types rather than parsing error message strings.

### 4.5 Concurrent Session Handling

| Severity | Issue |
|----------|-------|
| **LOW** | Plan does not address what happens when the same agent has multiple active sessions |

A single agent (same token) might have multiple MCP sessions open -- for example, if the agent restarts without cleanly closing its previous session. The plan should specify:
- Are multiple sessions per agent allowed? (Recommend: yes, with a cap)
- Do they share spending state? (They must -- spending is per-agent, not per-session)
- Is there a maximum? (Recommend: 5 per agent)

---

## Summary

### Issue Counts

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 0 |
| MEDIUM | 8 |
| LOW | 7 |

### Previous HIGH Issues: All 3 Resolved

The plan has been significantly improved since v1. All three HIGH issues (outdated protocol version, non-MCP registration endpoint, missing testing phase) have been properly addressed.

### MEDIUM Issues (should fix before or during implementation)

1. **Default protocol version per-session**: Store negotiated version per-session rather than hardcoding `2025-03-26`.
2. **Host header validation**: Add `Host` header check to DNS rebinding protection.
3. **Registration rate limiting**: Explicitly specify per-session and global rate limits for `register_agent`.
4. **Token recovery**: Document the limitation or add a user-initiated re-issue flow.
5. **SSE event delivery testing**: Ensure E2E tests properly parse and verify SSE events.
6. **Mock CLI specification**: Explicitly state how awal CLI is mocked across test layers.
7. **Session idle timeout value**: Specify a default timeout and maximum concurrent sessions.
8. **Domain error codes**: Define JSON-RPC error codes for authentication, spending limits, and kill switch errors.

### Verdict

**Ready for implementation with MEDIUM fixes.** The plan is protocol-correct, architecturally sound, and has a strong testing strategy. The remaining MEDIUM issues are implementation details that can be resolved during Phase 1 development. No blocking issues remain.
