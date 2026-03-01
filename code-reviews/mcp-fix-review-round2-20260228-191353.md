## CRITICAL

None found that directly enables unauthorized tool execution or spending-policy bypass leading to immediate unauthorized transfers.

---

## HIGH

1. **MCP Streamable HTTP non-compliance: SSE (GET) not implemented**
   - **File:** `src/api/mcp_http_server.rs` (`handle_get`)
   - **Issue:** Streamable HTTP transport for MCP 2025-11-25 expects GET to establish an SSE stream for server→client events (and in some clients, required for the transport to be considered “Streamable HTTP”). Returning **405** makes this transport likely incompatible with compliant MCP clients.
   - **Fix:** Implement SSE per spec (session-bound event stream, correct `Content-Type: text/event-stream`, heartbeats, disconnect handling).

2. **Spending reservation + transaction persistence is not atomic; rollback failures are ignored**
   - **File:** `src/api/mcp_router.rs` (`handle_send_payment`, `handle_trade_tokens`, `handle_pay_x402`)
   - **Issues:**
     - After `check_policy_and_reserve_atomic(...)` succeeds, `insert_transaction(...)` can fail and **no rollback** occurs → caps may be consumed without a corresponding transaction record.
     - On CLI failure, rollback is attempted but the result is discarded: `let _ = queries::rollback_reservation(...)`. If rollback fails, caps can remain “stuck”.
   - **Impact:** Spending-policy accounting can permanently diverge (phantom reservations), causing incorrect enforcement (often overly restrictive) and making financial operations non-auditable.
   - **Fix:** Use a DB transaction spanning: policy reserve → CLI outcome recording intent → transaction insert (and approval insert where relevant). If any step fails, rollback reservation and fail the tool call. Also **surface rollback failure** (error + logging), don’t ignore it.

3. **Invitation code consumption / max-use enforcement likely raceable (non-atomic)**
   - **File:** `src/api/mcp_router.rs` (`handle_register_agent`)
   - **Issue:** The flow is:
     1) `get_invitation_code` and check `use_count` vs `max_uses`
     2) `insert_agent`
     3) `use_invitation_code`
     This is not shown to be atomic. Concurrent registrations can both pass the `use_count` check before incrementing. Also if step (3) fails after step (2), an agent/token may be created without consuming the code.
   - **Impact:** Invitation-code policy can be bypassed under concurrency or partial failure, enabling unintended agent creation (and token issuance).
   - **Fix:** Make invitation validation + increment + agent insert a **single DB transaction**, ideally with a conditional update like `UPDATE ... SET use_count=use_count+1 WHERE code=? AND use_count < max_uses` and verify affected rows.

4. **Excessive thread/runtime creation per CLI call (DoS / resource exhaustion risk)**
   - **Files:**
     - `src/api/mcp_http_server.rs` (`spawn_blocking` per tools/call)
     - `src/api/mcp_router.rs` (`run_cli` spawns a new OS thread and creates a new Tokio runtime per call)
   - **Issue:** A single tool call can cause:
     - a Tokio blocking task thread, **plus**
     - an extra OS thread, **plus**
     - a fresh Tokio runtime build.
   - **Impact:** Under load, this becomes an easy local DoS vector (CPU/memory/thread exhaustion), especially because initialize is unauthenticated.
   - **Fix:** Make router tool handlers async and call `cli.run(cmd).await` directly under the existing runtime; or keep sync router but use a shared runtime / dedicated worker pool, not “runtime-per-call”.

5. **Rate limiting is easy to bypass by creating many sessions; initialize is not rate-limited**
   - **File:** `src/api/mcp_http_server.rs`
   - **Issue:** Rate limit is **per session** only, and `initialize` has no rate limit. An attacker can create up to `MAX_SESSIONS` (100) and get `100 * 60 = 6000 req/min`, while also forcing expensive CLI/thread activity.
   - **Fix:** Add a **global** and/or **per-source** rate limit (even on localhost), and rate-limit `initialize` and `register_agent` separately.

6. **Auto-discovery uses `localhost`, server binds `127.0.0.1` only (availability break; common on IPv6-first systems)**
   - **Files:**
     - `src/core/auto_discovery.rs` writes `http://localhost:{port}/mcp`
     - `src/api/mcp_http_server.rs` / `src/lib.rs` bind `127.0.0.1:{port}`
   - **Issue:** On many machines, `localhost` resolves to `::1` first. Binding only to IPv4 can make discovery config fail.
   - **Fix:** Bind to both (`[::1]` and `127.0.0.1`) or bind to `localhost` via dual-stack (`[::]:port` with appropriate safeguards), and ensure origin/host validation matches.

---

## MEDIUM

1. **DNS rebinding / CSRF protections rely on `Origin` only; `Host` is not validated; missing Origin is allowed**
   - **File:** `src/api/mcp_http_server.rs` (`validate_origin`)
   - **Issue:** Allowing requests with no `Origin` is convenient for curl/CLI, but it means protection is not “browser-grade robust” and does not validate `Host` / `X-Forwarded-Host`. Some unusual client contexts can omit Origin.
   - **Fix:** Also validate `Host` header (must be localhost/127.0.0.1/::1), and consider requiring a custom header or token for browser contexts.

2. **IPv6 allowlist bug in origin validation**
   - **File:** `src/api/mcp_http_server.rs` (`validate_origin`)
   - **Issue:** Compares `host_str()` to `Some("[::1]")`, but `url::Url::host_str()` returns `"::1"` (without brackets). Legit `Origin: http://[::1]:...` will be rejected.
   - **Fix:** Allow `"::1"`.

3. **Protocol strictness/compatibility gaps (Streamable HTTP + JSON-RPC)**
   - **File:** `src/api/mcp_http_server.rs`
   - **Issues:**
     - Requires `Accept` to contain `application/json`; clients sending `Accept: */*` will be rejected.
     - Does not validate `jsonrpc == "2.0"`.
     - `initialize` ignores requested `protocolVersion` and doesn’t error on mismatch.
   - **Fix:** Follow MCP transport requirements precisely (accept more client variants, validate protocol version, return spec-compliant JSON-RPC errors).

4. **Non-JSON-RPC error responses for session errors**
   - **File:** `src/api/mcp_http_server.rs`
   - **Issue:** For missing/expired session, responses are plain text with HTTP 400/404. Many JSON-RPC/MCP clients expect JSON-RPC error envelopes consistently.
   - **Fix:** Return JSON-RPC error objects with appropriate codes/messages (and consistent HTTP semantics per MCP spec).

5. **`tools/call` returns structured JSON as a string inside `content.text`**
   - **Files:** `src/api/mcp_http_server.rs`, `src/api/mcp_server.rs`
   - **Issue:** `content.to_string()` produces a JSON string; clients must parse text to recover fields (e.g., `token` from `register_agent`), which is fragile and can lead to token mishandling/logging.
   - **Fix:** Return structured results in the MCP tool response format expected by the spec/client (or at least use a JSON content type if MCP supports it).

6. **Unbounded / insufficiently validated parameters**
   - **File:** `src/api/mcp_router.rs` (`handle_get_transactions`)
   - **Issue:** `limit` is user-controlled with no upper bound and can be negative. This can cause heavy DB queries or undefined behavior depending on query implementation.
   - **Fix:** Clamp `limit` (e.g., 1..=100) and validate positivity.

7. **CLI output size not bounded**
   - **File:** `src/cli/executor.rs` (`RealCliExecutor::run`)
   - **Issue:** `wait_with_output()` buffers full stdout/stderr in memory. A misbehaving CLI could output huge data causing memory pressure.
   - **Fix:** Stream output with size limits or enforce maximum captured bytes.

8. **Tool/CLI parameter mismatches & confusing API surface**
   - **Files:** `src/api/mcp_router.rs`, `src/api/mcp_tools.rs`
   - **Issues:**
     - Some tests send `asset`/`memo` but MCP tool schema for `send_payment` does not include them and router ignores them.
     - `send_payment` hardcodes `USDC`; if clients believe asset is selectable, they may assume a different spend semantics.
   - **Fix:** Align tool schema + handler behavior; either accept/validate `asset`/`memo` or remove them everywhere.

9. **Token lifecycle gaps: no rotation/expiry semantics exposed**
   - **Files:** multiple (token hashing/validation is present, but lifecycle ops aren’t)
   - **Issue:** Tokens appear long-lived with no MCP tool to rotate/revoke (revocation exists in app, but not via MCP).
   - **Fix:** Consider adding token rotation/revocation flows (or explicitly document that only the wallet owner can rotate via UI).

---

## LOW

1. **`tools/list` “authenticated” if session has `agent_id` (even if agent is Pending)**
   - **File:** `src/api/mcp_http_server.rs` (`handle_tools_list` + session updated in `handle_register_agent_call`)
   - **Issue:** After `register_agent`, session is treated as authenticated for listing tools even though the token won’t validate until agent becomes `Active`. This is confusing and slightly increases information exposure.
   - **Fix:** Only treat as authenticated if bearer token validates to an **active** agent.

2. **Expired sessions cleaned only on `initialize`**
   - **File:** `src/api/mcp_http_server.rs`
   - **Issue:** Expired sessions can linger (counting against `MAX_SESSIONS`) until someone calls `initialize`.
   - **Fix:** Periodic cleanup task, or cleanup on every request when map size is near capacity.

3. **Stored session protocol version unused**
   - **File:** `src/api/mcp_http_server.rs` (`McpSession.protocol_version`)
   - **Issue:** Recorded but never checked/enforced.
   - **Fix:** Enforce version compatibility.

4. **Potential real-awal CLI drift is acknowledged but not enforced**
   - **File:** `src/cli/commands.rs` (comment re: logout; also `AuthStatus` maps to `status`)
   - **Issue:** If actual awal CLI differs, commands will fail at runtime.
   - **Fix:** Add integration tests against real awal CLI or feature-gate unsupported subcommands.

---

## Summary

CRITICAL: 0  
HIGH: 6  
MEDIUM: 9  

APPROVED: NO
