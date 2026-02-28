# Phase 2.5 Code Review — Tally Agentic Wallet

**Reviewer**: Claude Opus 4.6 (external review)
**Date**: 2026-02-27
**Scope**: TOCTOU fix, integration tests, Playwright E2E, MCP E2E, CI pipeline, security, architecture

---

## Executive Summary

Phase 2.5 delivers a solid TOCTOU fix using SQLite `BEGIN EXCLUSIVE` transactions, comprehensive Rust integration tests including real concurrency testing with file-based SQLite, and a well-structured Playwright E2E approach for the Tauri frontend. The MCP server has thorough E2E coverage. However, several issues ranging from **CRITICAL** to **LOW** severity were identified.

**Verdict**: Strong foundation with targeted fixes needed before production.

| Severity | Count |
|----------|-------|
| CRITICAL | 2 |
| HIGH     | 4 |
| MEDIUM   | 6 |
| LOW      | 5 |

---

## 1. TOCTOU Fix: `check_policy_and_reserve_atomic` / `rollback_reservation`

### What Was Done

The original `SpendingPolicyEngine::evaluate()` was a read-only check with no reservation. Between the check and the ledger update (in `execute_send`), concurrent requests could all pass the policy check and overspend. Phase 2.5 introduced:

1. **`check_policy_and_reserve_atomic()`** — Single `BEGIN EXCLUSIVE` transaction that reads policy, reads all ledger totals, checks all caps (per-agent + global), and writes the reservation if approved.
2. **`rollback_reservation()`** — Decrements ledger entries when CLI execution fails or approval is denied, also using `BEGIN EXCLUSIVE`.
3. **Background execution** that either confirms (keeping the reservation) or rolls back on failure.

### Assessment: **GOOD with caveats**

**Strengths:**
- `BEGIN EXCLUSIVE` correctly serializes all writers at the SQLite level
- Policy check and ledger reservation are truly atomic — no gap for TOCTOU
- Rollback path is implemented for both CLI failure and approval denial
- Both agent-level and global-level ledgers are reserved/rolled back together
- Concurrency tests prove the fix with file-based SQLite (pool_size=4)

**Issues Found:**

#### CRITICAL-1: Floating-point arithmetic in ledger UPSERT

**File**: `/Users/dennisonbertram/Develop/apps/agent-neo-bank/src-tauri/src/db/queries.rs` (lines 1688-1697)

```sql
total = CAST((CAST(spending_ledger.total AS REAL) + CAST(?3 AS REAL)) AS TEXT)
```

The ledger uses `CAST(... AS REAL)` which converts to IEEE 754 `f64`. This introduces floating-point rounding errors. For a financial application, amounts like `0.1 + 0.2 != 0.3` in floating-point. While `rust_decimal::Decimal` is used in Rust code, the SQL layer throws away that precision.

**Impact**: Over many transactions, cumulative rounding errors could allow slight overspend or underspend. A malicious agent could craft amounts that exploit rounding to slowly exceed caps.

**Fix**: Store amounts as integer cents (or use SQLite's text arithmetic via a custom function), or compute the new total in Rust using `Decimal` and write the final string back:

```sql
-- Instead of arithmetic in SQL:
UPDATE spending_ledger SET total = ?3, tx_count = tx_count + 1, updated_at = ?4
WHERE agent_id = ?1 AND period = ?2
```

Where `?3` is the new total computed in Rust with `Decimal`.

#### CRITICAL-2: Rollback uses same floating-point arithmetic

**File**: `/Users/dennisonbertram/Develop/apps/agent-neo-bank/src-tauri/src/db/queries.rs` (lines 1755-1763)

```sql
total = CAST((CAST(spending_ledger.total AS REAL) - CAST(?3 AS REAL)) AS TEXT)
```

Same floating-point issue as above, but worse: after rollback, the ledger total could become slightly negative (e.g., `-0.0000000000001`) or not return to the exact pre-reservation value, leading to phantom spending capacity loss.

#### HIGH-1: No `updated_at` update in rollback

**File**: `/Users/dennisonbertram/Develop/apps/agent-neo-bank/src-tauri/src/db/queries.rs` (lines 1755-1763)

The rollback UPDATE does not set `updated_at`. The `_now_ts` parameter is unused (prefixed with underscore). This means the ledger's `updated_at` timestamp still reflects the original reservation time, which could confuse audit trails and debugging.

#### HIGH-2: `tx_count` can go negative after rollback

After rollback, `tx_count = spending_ledger.tx_count - 1` could theoretically go to 0 or negative if there's a bug in the reservation path. There's no `CHECK(tx_count >= 0)` constraint or guard.

#### MEDIUM-1: `RequiresApproval` reserves immediately but has no timeout cleanup

When a transaction requires approval, the amount is reserved in the ledger immediately. If the approval is never resolved (neither approved nor denied/expired), the reservation is never rolled back. The approval has a 24h expiry (`expires_at`), but there's no background job or cron that cleans up expired reservations.

**Impact**: Expired approval requests permanently reduce available spending capacity.

**Fix**: Add a periodic task that finds expired `AwaitingApproval` transactions and calls `rollback_reservation`.

#### MEDIUM-2: No global spending ledger cleanup on rollback for `RequiresApproval` denials

When an approval is denied by the human operator (via `resolve_approval`), the code should call `rollback_reservation` for both agent and global ledgers. Need to verify the approval resolution path handles this.

---

## 2. Integration Test Quality and Coverage

### Assessment: **STRONG**

**Test Files Reviewed:**
- `tests/concurrent_transactions.rs` — 4 tests covering concurrency
- `tests/approval_flow.rs`
- `tests/kill_switch.rs` / `tests/kill_switch_integration.rs`
- `tests/mcp_integration.rs` / `tests/mcp_e2e.rs`
- `tests/spending_limits.rs`
- `tests/limit_increase.rs`
- `tests/agent_lifecycle.rs`
- `tests/token_delivery.rs`
- `tests/mock_mode.rs`
- `tests/cli_failure_recovery.rs`

**Strengths:**
- `concurrent_transactions.rs` uses **file-based SQLite** with `pool_size=4` — this is excellent and avoids the common trap of in-memory SQLite serializing everything through a single connection
- `test_concurrent_sends_strict_cap_enforcement` asserts **exactly** 4 of 5 succeed, proving the atomic reservation works
- Uses `tokio::test(flavor = "multi_thread", worker_threads = 4)` for real parallel execution
- Global cap enforcement is tested alongside per-agent caps
- CLI failure rollback is tested end-to-end
- The `common/mod.rs` test helpers are well-structured with `register_agent_with_policy` doing full lifecycle setup

**Issues Found:**

#### HIGH-3: Concurrency test relies on `tokio::time::sleep` for background completion

**File**: `/Users/dennisonbertram/Develop/apps/agent-neo-bank/src-tauri/tests/concurrent_transactions.rs` (lines 212-213, 310-311, 395, 510)

```rust
tokio::time::sleep(Duration::from_secs(2)).await;
```

The tests sleep 2-3 seconds waiting for background execution to complete. This is inherently flaky:
- On slow CI machines, 2-3 seconds may not be enough
- On fast machines, it wastes time unnecessarily

**Fix**: Use the `broadcast::Receiver<TxEvent>` to wait for all expected events, or poll the DB for the expected state with a timeout.

#### MEDIUM-3: No test for double-rollback safety

There's no test verifying that calling `rollback_reservation` twice for the same transaction doesn't cause negative ledger values. In a real system, network retries or error-handling bugs could trigger double rollback.

#### LOW-1: Test assertion uses `f64` comparison for financial values

**File**: `/Users/dennisonbertram/Develop/apps/agent-neo-bank/src-tauri/tests/concurrent_transactions.rs` (lines 222-226)

```rust
let total: f64 = global_ledger.total.parse().unwrap();
assert!((total - 15.0).abs() < 0.01, ...);
```

Tests parse the ledger total as `f64` for assertions, which masks the CRITICAL-1 floating-point issue in the SQL. Tests should parse as `Decimal` and assert exact equality.

---

## 3. Playwright E2E Test Approach (Mocking Tauri Invoke)

### Assessment: **GOOD approach, limited depth**

**Architecture:**
- `tests/e2e/fixtures.ts` injects `window.__TAURI_INTERNALS__.invoke` as a mock
- Each test provides a map of command -> response
- Tests run against the Vite dev server (port 1420) without a Tauri binary
- 6 spec files covering: onboarding, agents, approvals, transactions, kill switch, global policy

**Strengths:**
- Clean mock injection via `addInitScript` — runs before any page JavaScript
- Mock supports all Tauri v2 internals (`convertFileSrc`, `transformCallback`, `metadata`)
- Shared test data factories (`testAgents`, `testApprovals`, etc.) in fixtures
- Tests verify both rendering and user interactions (clicks, navigation)

**Issues Found:**

#### HIGH-4: Mock does not capture or verify invoke arguments

**File**: `/Users/dennisonbertram/Develop/apps/agent-neo-bank/tests/e2e/fixtures.ts` (line 18)

```typescript
invoke: (cmd: string, _args?: unknown) => {
    if (cmd in mockData) {
        return Promise.resolve(mockData[cmd]);
    }
    return Promise.resolve(null);
},
```

The mock **ignores the arguments** passed to `invoke()`. This means:
- Tests cannot verify that the frontend sends correct arguments to the backend
- The approval `resolve_approval` test (approval-queue.spec.ts:66) cannot verify that the correct approval ID and decision were sent
- The `toggle_kill_switch` test cannot verify the correct active/reason arguments

**Fix**: Add argument capture to the mock:

```typescript
const calls: Array<{cmd: string, args: unknown}> = [];
invoke: (cmd: string, args?: unknown) => {
    calls.push({ cmd, args });
    // ...
};
```

Expose `calls` on `window` for test assertions.

#### MEDIUM-4: No error state testing in E2E

None of the 6 E2E spec files test error scenarios:
- What happens when an invoke fails (returns `Promise.reject`)?
- Network errors during onboarding OTP submission?
- Kill switch activation failure?

The mock always resolves successfully or returns `null`.

#### MEDIUM-5: No navigation/routing guard tests

Tests navigate directly to routes but don't verify that unauthenticated users are redirected from protected routes, or that the app handles missing mock data gracefully.

#### LOW-2: `webServer.reuseExistingServer: true` can cause stale server issues

**File**: `/Users/dennisonbertram/Develop/apps/agent-neo-bank/playwright.config.ts` (line 20)

If a previous dev server is still running with old code, the E2E tests will connect to it instead of starting a fresh one.

---

## 4. MCP E2E Test Thoroughness

### Assessment: **EXCELLENT**

**File**: `/Users/dennisonbertram/Develop/apps/agent-neo-bank/src-tauri/tests/mcp_e2e.rs`

**Tests (16 total):**
1. `test_mcp_send_payment_e2e` — Full send + DB verification
2. `test_mcp_check_balance_e2e` — Balance response shape
3. `test_mcp_get_spending_limits_e2e` — All policy fields verified
4. `test_mcp_invalid_token_rejected` — Empty DB token rejection
5. `test_mcp_invalid_token_with_agents_present` — SHA-256 mismatch
6. `test_mcp_valid_token_accepted` — SHA-256 match + agent binding
7. `test_mcp_list_tools` — 6 tools, schema validation
8. `test_mcp_initialize_e2e` — Protocol version, capabilities
9. `test_mcp_spending_policy_per_agent_isolation` — Agent A sees A's limits, not B's
10. `test_mcp_transaction_isolation_between_agents` — Agent A's tx invisible to B
11. `test_mcp_suspended_agent_cannot_create_server`
12. `test_mcp_pending_agent_cannot_create_server`
13. `test_mcp_full_lifecycle` — Token validate -> initialize -> list -> balance -> send -> get_transactions -> limits
14. `test_mcp_unknown_method_returns_error` — JSON-RPC -32601
15. `test_mcp_unknown_tool_returns_error`
16. `test_mcp_send_payment_missing_fields`
17. `test_mcp_request_limit_increase_e2e`
18. `test_mcp_json_rpc_response_format` — Serialization shape

**Strengths:**
- Full lifecycle test exercises the complete MCP flow
- Agent isolation is tested for both policy reads and transaction visibility
- Token validation tests both empty-DB and wrong-token-with-agents scenarios
- JSON-RPC response format is verified (skip_serializing_if for null fields)
- Missing field validation for required parameters

**Issues Found:**

#### MEDIUM-6: MCP `send_payment` bypasses spending policy

**File**: `/Users/dennisonbertram/Develop/apps/agent-neo-bank/src-tauri/src/api/mcp_server.rs` (lines 193-255)

The MCP `handle_send_payment` creates a transaction directly via `queries::insert_transaction()` with status `Pending` — it does **NOT** call `check_policy_and_reserve_atomic()` or go through `TransactionProcessor::process_send()`. This means:

- MCP payments are **not subject to spending policy checks**
- No per-tx-max, daily cap, weekly cap, monthly cap enforcement
- No global policy enforcement (kill switch, reserve balance)
- No spending ledger reservation
- A malicious agent could bypass all spending controls via the MCP interface

**Impact**: **This is a security vulnerability.** The REST API enforces policies through `TransactionProcessor`, but the MCP path is completely unprotected.

**Fix**: Route MCP payments through `TransactionProcessor::process_send()` or at minimum call `check_policy_and_reserve_atomic()`.

#### LOW-3: MCP period key format differs from `spending_policy.rs`

**File**: `/Users/dennisonbertram/Develop/apps/agent-neo-bank/src-tauri/src/api/mcp_server.rs` (lines 239-241)

```rust
period_daily: now.format("%Y-%m-%d").to_string(),
period_weekly: format!("{}-W{:02}", now.format("%Y"), now.iso_week().week()),
period_monthly: now.format("%Y-%m").to_string(),
```

But `spending_policy.rs` uses:
```rust
format!("daily:{}", dt.format("%Y-%m-%d"))
format!("weekly:{}-W{:02}", week.year(), week.week())
format!("monthly:{}", dt.format("%Y-%m"))
```

The MCP path omits the `daily:`, `weekly:`, `monthly:` prefixes. If MCP transactions were to interact with the spending ledger (currently they don't due to MEDIUM-6), they would be written to different period keys and not be counted toward caps.

---

## 5. CI Pipeline Correctness

### Assessment: **MOSTLY CORRECT with gaps**

**File**: `/Users/dennisonbertram/Develop/apps/agent-neo-bank/.github/workflows/ci.yml`

**Jobs (9 total):**
1. `lint-rust` — fmt + clippy
2. `test-rust-unit` — `cargo test --lib --bins`
3. `test-rust-integration` — `cargo test --test '*'`
4. `coverage-rust` — tarpaulin with 80% threshold
5. `lint-frontend` — `npm run lint`
6. `test-frontend` — `npm test`
7. `coverage-frontend` — vitest coverage with 70% threshold
8. `build` — macOS Tauri build
9. `e2e` — Playwright on macOS, depends on `build`

**Strengths:**
- Proper separation of unit and integration tests
- Coverage thresholds enforced (80% Rust, 70% frontend)
- Cargo caching with proper keys
- macOS build for Tauri compatibility
- Playwright E2E with chromium

**Issues Found:**

#### HIGH-5 (Downgraded from original assessment): `lint-frontend` will fail — no `lint` script

**File**: `/Users/dennisonbertram/Develop/apps/agent-neo-bank/.github/workflows/ci.yml` (line 174)

The CI file itself notes: `# NOTE: No "lint" script exists in package.json yet.` This means the `lint-frontend` job will fail on every PR, blocking merges.

#### LOW-4: E2E job depends on `build` but doesn't use the build artifact

The `e2e` job depends on `build` (`needs: build`) and downloads no artifacts. The E2E tests use `npm run dev` (Vite dev server) with mocked Tauri, so the build dependency is unnecessary. This wastes CI time.

#### LOW-5: Rust lint and test jobs run on `ubuntu-latest` but build is `macos-latest`

While this is intentional (Linux is faster/cheaper for lint/test, macOS needed for Tauri build), clippy on Linux may miss macOS-specific compilation issues. Consider adding a macOS clippy pass or at minimum ensuring the build job catches them.

---

## 6. Remaining Security Concerns and Race Conditions

### CRITICAL-2 (See Section 1): MCP policy bypass

The MCP `send_payment` tool creates transactions without any policy enforcement. This is the most significant security issue in the codebase.

### Race condition: Balance check staleness

**File**: `/Users/dennisonbertram/Develop/apps/agent-neo-bank/src-tauri/src/core/tx_processor.rs` (line 103)

```rust
let balance = *self.current_balance.read().await;
```

The balance is read from an `RwLock<Decimal>` cached value, then passed to the atomic policy check. But the balance is never updated after transactions succeed. In the test setup, it's initialized to `dec!(10000)` and never changed.

**Impact**: The global policy's `min_reserve_balance` check always compares against the initial cached balance, not the true current balance. After many successful transactions, the actual balance could be far lower than the cached value, bypassing the reserve balance protection.

### No rate limiting on MCP path

The REST API has `RateLimiter` middleware, but the MCP stdio path has no rate limiting. An agent could flood the MCP server with rapid requests.

### Token validation iterates all active agents

**File**: `/Users/dennisonbertram/Develop/apps/agent-neo-bank/src-tauri/src/api/mcp_server.rs` (line 89)

```rust
let agents = queries::list_agents_by_status(&db, &AgentStatus::Active)?;
for agent in &agents {
```

This loads **all** active agents and iterates linearly to find a matching token hash. With many agents, this is O(n) per authentication and could be a DoS vector. It also loads all agent data (including capabilities, metadata, etc.) when only `api_token_hash` is needed.

**Fix**: Add a DB index on `api_token_hash` and query directly:
```sql
SELECT id FROM agents WHERE api_token_hash = ?1 AND status = 'active'
```

---

## 7. Code Quality, Error Handling, and Edge Cases

### Positive Observations

- **Decimal handling**: `rust_decimal::Decimal` is used throughout Rust code for financial amounts. Excellent choice.
- **Error types**: `AppError` enum is comprehensive with proper variant mapping to HTTP status codes and JSON-RPC error codes.
- **Test helpers**: `test_helpers.rs` provides consistent factory functions (`create_test_agent`, `create_test_spending_policy`, `setup_test_db`).
- **Broadcast events**: Transaction lifecycle events are emitted via `tokio::broadcast`, enabling reactive UI updates.
- **Serialization**: JSON-RPC responses use `#[serde(skip_serializing_if = "Option::is_none")]` to omit null fields.

### Issues

#### MEDIUM-7: Fire-and-forget webhook with no retry

**File**: `/Users/dennisonbertram/Develop/apps/agent-neo-bank/src-tauri/src/core/tx_processor.rs` (lines 328-341)

```rust
let _ = reqwest::Client::new()
    .post(&url)
    .json(&serde_json::json!({...}))
    .send()
    .await;
```

Webhook delivery is fire-and-forget with no retry, no timeout, no logging of failure. The `let _` silently discards any errors (DNS failure, connection refused, HTTP 500).

#### Error discarding pattern

Multiple locations use `let _ =` to discard errors silently:

- `tx_processor.rs:130` — event send on denial
- `tx_processor.rs:283-290` — rollback failure after status update failure
- `tx_processor.rs:315-323` — rollback failure after CLI failure

While some of these are intentional (best-effort), the rollback failures in particular should be logged, as a failed rollback means the spending ledger is permanently overstated.

#### Global policy `weekly_period_key()` format differs

**File**: `/Users/dennisonbertram/Develop/apps/agent-neo-bank/src-tauri/src/core/global_policy.rs` (line 36)

```rust
format!("weekly:{}", now.format("%G-W%V"))
```

But `spending_policy.rs` (line 32):
```rust
format!("weekly:{}-W{:02}", week.year(), week.week())
```

`%G` (ISO week-based year) vs `week.year()` (also ISO) should be equivalent, but `%V` vs `week.week()` with `:02` formatting could differ. This inconsistency could cause period key mismatches between agent and global ledgers.

---

## Summary of Findings

### CRITICAL (Must Fix Before Production)

| ID | Issue | Location |
|----|-------|----------|
| CRITICAL-1 | Floating-point arithmetic in SQL ledger UPSERT | `queries.rs:1688-1697` |
| CRITICAL-2 | MCP `send_payment` bypasses spending policy entirely | `mcp_server.rs:193-255` |

### HIGH (Should Fix Before Production)

| ID | Issue | Location |
|----|-------|----------|
| HIGH-1 | Rollback does not update `updated_at` | `queries.rs:1755-1763` |
| HIGH-2 | `tx_count` can go negative after rollback | `queries.rs:1758` |
| HIGH-3 | Concurrency tests rely on sleep instead of events | `concurrent_transactions.rs` |
| HIGH-4 | E2E mock ignores invoke arguments | `fixtures.ts:18` |
| HIGH-5 | CI `lint-frontend` will fail — no lint script | `ci.yml:174` |

### MEDIUM

| ID | Issue | Location |
|----|-------|----------|
| MEDIUM-1 | No cleanup for expired `RequiresApproval` reservations | `tx_processor.rs` |
| MEDIUM-2 | Approval denial path may not rollback ledger | `rest_routes.rs` |
| MEDIUM-3 | No double-rollback safety test | `tests/` |
| MEDIUM-4 | No error state testing in E2E | `tests/e2e/` |
| MEDIUM-5 | No auth/routing guard tests | `tests/e2e/` |
| MEDIUM-6 | MCP period key format mismatch | `mcp_server.rs:239-241` |
| MEDIUM-7 | Fire-and-forget webhook with no retry/logging | `tx_processor.rs:328-341` |

### LOW

| ID | Issue | Location |
|----|-------|----------|
| LOW-1 | Test assertions use f64 for financial values | `concurrent_transactions.rs` |
| LOW-2 | `reuseExistingServer: true` can use stale server | `playwright.config.ts` |
| LOW-3 | MCP period key prefix missing | `mcp_server.rs:239-241` |
| LOW-4 | E2E depends on build job but doesn't use artifact | `ci.yml:264` |
| LOW-5 | No macOS clippy in CI | `ci.yml` |

---

## Recommended Fix Priority

1. **CRITICAL-2**: Route MCP payments through `TransactionProcessor` — highest security risk
2. **CRITICAL-1**: Replace SQL floating-point arithmetic with Rust `Decimal` computation
3. **MEDIUM-1**: Add background cleanup for expired approval reservations
4. **HIGH-5**: Add `lint` script to `package.json`
5. **HIGH-4**: Add argument capture to E2E mock
6. **MEDIUM-6/LOW-3**: Standardize period key format across MCP and spending policy
7. **HIGH-1/HIGH-2**: Fix rollback to update `updated_at` and add `tx_count >= 0` guard
