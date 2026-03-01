## CRITICAL

### 1) `GET /mcp` SSE stream is an unthrottled infinite tight loop → single-request DoS
**File:** `src/api/mcp_http_server.rs` (`handle_get`)  
**What happens:**  
```rust
let sse_stream = stream::repeat_with(|| Ok(Event::default().comment("keep-alive")));
```
This produces events as fast as the client can read (no timer), so one client can peg CPU, saturate loopback bandwidth, and keep the connection busy indefinitely. `KeepAlive::interval()` does **not** fix this because the stream is never idle.

**Why it’s critical:** This is a trivial local DoS against the wallet process (and potentially the UI), and it violates the intent of SSE keep-alives.

**Fix:** Build the SSE stream from a timer, e.g. `tokio_stream::wrappers::IntervalStream`, and emit at a fixed cadence; also enforce per-session/per-IP limits on concurrent SSE connections.

---

### 2) Spending “reservation” commits are not atomic with transaction + approval persistence → budget leakage / inconsistent financial state
**Files:** `src/api/mcp_router.rs`, `src/db/queries.rs`  
**Where:** `handle_send_payment`, `handle_trade_tokens`, `handle_pay_x402` call `queries::check_policy_and_reserve_atomic(...)` (which **commits** ledger reservations) and only **after** that do they:
- execute CLI (may succeed and actually spend funds), then
- `queries::insert_transaction(...)`, and sometimes
- `queries::insert_approval_request(...)`

These are separate DB operations on separate pooled connections/transactions.

**Failure modes (all bad):**
- Reservation committed, then `insert_transaction` fails → caps consumed forever with no tx record.
- Reservation committed, then CLI succeeds (funds moved), then DB write fails → real spend happened, but no durable audit/tx record (and caps may be wrong).
- `RequiresApproval`: reservation committed, `insert_transaction` succeeds, `insert_approval_request` fails → funds are “reserved” but there may be no approval workflow record to ever resolve it.

**Fix:** Make the policy decision + reservation + tx insert (+ approval insert when needed) one DB transaction on the **same connection**. In practice:
- have a single “create_tx_with_policy_reservation_atomic(...)” query that does: BEGIN → check → reserve → insert tx → insert approval (optional) → COMMIT.
- only execute the CLI after the DB transaction commits (and update tx status in a follow-up transaction).

---

## HIGH

### 3) Global “min reserve balance” / overdraft prevention is effectively broken (balance always passed as `"0"`)
**File:** `src/api/mcp_router.rs`  
**Where:** `check_policy_and_reserve_atomic(..., current_balance="0", ...)` in:
- `handle_send_payment`
- `handle_trade_tokens`
- `handle_pay_x402`

**Impact:**
- If `global_policy.min_reserve_balance > 0`, **all** spends will likely be denied (because `0 - amount < min_reserve`).
- If it’s `0`, then you’re not enforcing any “don’t drain the wallet” constraint at all (policy checks only look at ledgers, not actual funds).

**Fix:** Fetch real balance (from CLI or cached) and pass it into the policy check. If balance is unavailable, explicitly choose a safe behavior (typically “deny” with a clear error, or require approval).

---

### 4) Reservation lifecycle for approval/async outcomes is incomplete → permanent budget lock or inconsistent caps
**Files:** `src/api/mcp_router.rs`, `src/db/queries.rs`  
**Issue:** `check_policy_and_reserve_atomic` reserves for both `AutoApproved` **and** `RequiresApproval`. The code shown only rolls back on immediate CLI failure. There is no demonstrated path that rolls back reservations when:
- an approval is denied,
- an approval expires,
- a “pending” transaction later fails on-chain.

**Impact:** Budgets can get stuck consumed permanently, blocking legitimate spending, or drifting away from reality.

**Fix:** Ensure every non-confirmed terminal state triggers rollback (denied/expired/failed), and make it idempotent *per tx_id* (see next item).

---

### 5) Rollback is not tied to a specific reservation/tx → can undercount spending and enable overspending if misused
**File:** `src/db/queries.rs` (`rollback_reservation`)  
Rollback decrements ledger totals/counts by amount/period, clamps to zero, and is not linked to a transaction id or reservation id. If any internal caller triggers rollback twice (or with a larger amount), it can artificially reduce totals and allow further spending.

Even if not externally exposed today, this is a dangerous primitive for a financial system.

**Fix:** Track reservations by `tx_id` (or a dedicated reservation table), and make rollback/commit operations enforce “exactly-once” semantics.

---

### 6) CLI execution model can create excessive threads and amplify load
**File:** `src/api/mcp_router.rs` (`run_cli`) + `src/api/mcp_http_server.rs` (`handle_tools_call`)  
**Pattern:**
- HTTP: `spawn_blocking` per tool call
- Inside router: `run_cli` spawns an additional OS thread (`std::thread::scope(... spawn ... join ...)`) to call `handle.block_on(...)`

**Impact:** Under load, this can create many OS threads, increasing latency and risk of resource exhaustion (especially combined with the SSE issue).

**Fix:** Prefer a single async path for CLI calls:
- don’t create an extra OS thread inside `spawn_blocking`;
- or make router handlers async and call CLI directly without blocking gymnastics.

---

### 7) Origin / CSRF posture: allowing missing `Origin` enables some browser-based loopback attacks (esp. registration/DoS)
**File:** `src/api/mcp_http_server.rs` (`validate_origin`)  
Requests with **no** `Origin` are allowed. That’s convenient for curl/CLI, but it weakens protection against browser cross-site primitives where `Origin` can be absent or unusual (varies by mechanism/browser).

**Impact:** An attacker webpage may be able to hit `127.0.0.1` endpoints and:
- create sessions,
- attempt invitation-code guessing,
- consume rate limits/session capacity.

Tool calls still require Bearer tokens (good), but `register_agent` is unauthenticated (by design) and could be abused if invitation codes are low entropy.

**Fix options:**
- Require a local-only shared secret header for all requests (best for loopback services).
- Or require `Origin` for browser-y requests and separately allow a dedicated CLI mode (unix socket, or explicit flag).
- Ensure invitation codes are high entropy and rate limit registration more aggressively.

---

## MEDIUM

### 8) MCP protocol negotiation: `initialize.params.protocolVersion` is ignored
**File:** `src/api/mcp_http_server.rs` (`handle_initialize`)  
The server does not validate the client-requested protocol version/capabilities. For MCP Streamable HTTP, mismatched versions should be rejected or negotiated per spec.

**Fix:** Parse `params.protocolVersion`; if unsupported, return a JSON-RPC error indicating incompatible version.

---

### 9) Streamable HTTP compliance gaps / rough edges
**File:** `src/api/mcp_http_server.rs`  
Examples:
- `DELETE /mcp` returns `200 OK`; some specs expect `204 No Content`.
- `POST /mcp` requires `Accept` contains `application/json` but doesn’t enforce/validate other MCP-recommended values (e.g. `text/event-stream`) and doesn’t validate `Content-Type: application/json`.
- Error responses for missing headers use plain text + non-JSON-RPC HTTP statuses; may be acceptable at transport level but can diverge from client expectations.

**Fix:** Align strictly with the MCP 2025-11-25 Streamable HTTP requirements (header semantics, status codes, and error formats).

---

### 10) Tool result encoding uses `content: [{type:"text", text: content.to_string()}]` even for JSON objects
**Files:** `src/api/mcp_http_server.rs`, `src/api/mcp_server.rs`  
This forces clients to parse JSON embedded in a string and increases the chance of mishandling (especially for `register_agent` token delivery).

**Fix:** Return structured JSON content type (if MCP supports it) or at least include both a text summary and a JSON payload field.

---

### 11) Session cleanup only runs on `initialize`
**File:** `src/api/mcp_http_server.rs` (`cleanup_expired_sessions`)  
Expired sessions are cleaned only during `initialize`. If no one calls `initialize`, expired sessions may remain in memory until process restart (bounded by `MAX_SESSIONS`, but still).

**Fix:** Add a periodic cleanup task or prune opportunistically on other requests too.

---

### 12) Minor correctness: IPv6 localhost origin check likely wrong
**File:** `src/api/mcp_http_server.rs` (`validate_origin`)  
It matches `Some("[::1]")`, but `url::Url::host_str()` for IPv6 is typically `"::1"` (no brackets). This rejects legitimate `http://[::1]:...` origins.

---

### 13) DB errors during token validation are treated as “invalid token”
**File:** `src/api/mcp_http_server.rs` (`validate_bearer_token`) → `queries::get_agent_by_token_hash` returns `Option` and swallows connection errors.  
This is “fail closed” (good for security) but bad operationally: DB outages become auth failures with misleading “Invalid token”.

**Fix:** propagate DB errors distinctly (e.g., `-32603 Internal error`).

---

### 14) `register_agent` invitation consumption + agent insert + used_by set is not transactional
**Files:** `src/api/mcp_router.rs` (`handle_register_agent`), `src/db/queries.rs`  
If the process crashes between consume → insert_agent → set_used_by, you can end up with consumed codes or inconsistent linkage.

**Fix:** Wrap all registration DB operations in a single transaction.

---

## LOW

### 15) Some MCP tool arguments are ignored or hardcoded
**File:** `src/api/mcp_router.rs`  
- `send_payment` ignores `asset` and `memo` (asset hardcoded to `USDC`, memo empty).  
Not necessarily wrong, but surprising vs tests/examples and easy to misinterpret.

---

### 16) `AwalCommand::AuthLogout` is noted as possibly non-existent
**File:** `src/cli/commands.rs`  
Not directly MCP tool-related, but can break UI flows if exposed.

---

## Summary
CRITICAL: 2  
HIGH: 5  
MEDIUM: 7  
LOW: 2  

APPROVED: NO
