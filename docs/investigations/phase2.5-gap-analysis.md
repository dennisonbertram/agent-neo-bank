# Phase 2.5 Integration Test Gap Analysis

> **Date:** 2026-02-27
> **Scope:** Comparison of existing integration tests in `src-tauri/tests/` against the 9 scenarios defined in `docs/architecture/testing-specification.md` Section 4.

---

## Test Helpers Summary (`common/mod.rs`)

The following helpers exist and are used across all test files:

| Helper | Purpose |
|---|---|
| `create_test_app()` | Builds full `AppStateAxum` with in-memory DB, `MockCliExecutor`, default test config |
| `create_test_app_with_config(config)` | Same but with custom `AppConfig` |
| `create_test_app_with_db_and_config(db, config)` | Same but with existing DB + config |
| `bearer_request(method, uri, token, body)` | Builds an HTTP request with `Authorization: Bearer` header |
| `body_json(response)` | Parses response body into `serde_json::Value` |
| `register_approve_and_get_token(state, code, name)` | Full register+approve flow, returns `(agent_id, token)` |
| `register_agent_with_policy(state, code, name, per_tx_max, daily_cap, weekly_cap, monthly_cap, auto_approve_max)` | Register+approve+set spending policy, returns `(agent_id, token)` |

**Missing helpers (needed for Scenarios 6, 7, 8):**
- No helper for creating a `MockCliExecutor` that returns errors (needed for Scenario 7)
- No helper for time manipulation / simulating token expiry (needed for Scenario 6)
- No helper for concurrent request submission (needed for Scenario 8)

---

## Scenario-by-Scenario Analysis

### Scenario 1: Happy Path -- Agent Lifecycle

**File:** `src-tauri/tests/agent_lifecycle.rs`
**Spec steps:** 11

#### What Already Exists

| Test Function | Covers Spec Steps |
|---|---|
| `test_lifecycle_register_returns_pending` | Steps 1-4: invitation, register, assert 201+pending, poll status pending |
| `test_lifecycle_approve_and_retrieve_token` | Steps 5-7: approve, poll status -> active + `anb_` token, poll again -> token null |
| `test_lifecycle_send_and_poll_transaction` | Steps 8-11: send 5.00, assert 202+executing, poll tx -> confirmed+chain_tx_hash, list txs |
| `test_lifecycle_full_happy_path` | Steps 1-11 combined in single test |

#### What's Missing

- **Step 1 detail:** Spec says invitation has "24-hour expiry" -- the test creates invitations but does not verify expiry enforcement.
- **Step 6 detail:** Spec says "polls status again within 5 minutes" -- no assertion that the 5-minute window is respected (ties into Scenario 6).
- **Step 8 detail:** Spec says agent has `per_tx_max: 25.00`, `auto_approve_max: 10.00` -- the test uses `per_tx_max: 100`, `auto_approve_max: 50`. The values work but don't match the spec exactly.

#### What Needs to Be Written

- **No new test functions required.** Coverage is comprehensive. Minor improvements:
  - Align spending policy values with spec (`per_tx_max: 25`, `auto_approve_max: 10`) for traceability.
  - Optionally add assertion on invitation expiry (or defer to Scenario 6).

**Verdict: COVERED (minor alignment needed)**

---

### Scenario 2: Spending Limit Enforcement

**File:** `src-tauri/tests/spending_limits.rs`
**Spec steps:** 7

#### What Already Exists

| Test Function | Covers Spec Steps |
|---|---|
| `test_spending_per_tx_max_exceeded_returns_403` | Step 2: amount 15 exceeds per_tx_max 10 -> 403 |
| `test_spending_requires_approval_above_auto_approve` | Step 3: amount 8 above auto_approve 5, within per_tx 10 -> 202 awaiting_approval |
| `test_spending_daily_cap_cumulative_enforcement` | Steps 4-7 (partially): sends 8, 9, 9(denied), 8(at cap) |
| `test_spending_auto_approve_boundary` | Extra: boundary test at exactly auto_approve_max |

#### What's Missing

- **Step 4 (spec):** "User approves. Assert transaction executes. Daily spending is now 8.00." -- The existing test uses `auto_approve_max: 50` to bypass approval entirely. The spec requires the approval flow to be exercised: send 8 -> awaiting_approval -> user approves -> tx executes -> daily = 8.
- **Step 5 (spec):** After approval of step 4, send 9.00, daily would be 17 -> 202. The existing test does this but skips the approval step.
- **Steps 6-7:** Correctly implemented (cumulative cap tracking).

#### What Needs to Be Written

1. **`test_spending_approval_then_cumulative_tracking`** -- New test that follows the spec exactly:
   - Agent with `per_tx_max: 10`, `daily_cap: 25`, `auto_approve_max: 5`
   - Send 8 -> 202 awaiting_approval
   - User approves via `ApprovalManager.resolve()`
   - Wait for background execution
   - Assert daily spending is now 8
   - Send 9 -> 202 (daily 17, within cap)
   - Send 9 -> 403 (daily 26, exceeds cap)
   - Send 8 -> 202 (daily 25, exactly at cap)

**Verdict: PARTIALLY COVERED (approval integration in spending flow is missing)**

---

### Scenario 3: Global Policy Enforcement

**File:** `src-tauri/tests/global_policy.rs`
**Spec steps:** 5

#### What Already Exists

| Test Function | Covers Spec Steps |
|---|---|
| `test_global_daily_cap_enforcement_across_agents` | Steps 1-5: two agents, global cap 50, A sends 25, B sends 20, A sends 10 (denied), B sends 6 (denied) |
| `test_global_daily_cap_allows_at_exact_boundary` | Extra: exact boundary test |

#### What's Missing

- **Step 5 (spec):** Spec says "Agent B sends 3.00 -> 403". The test sends 6 instead of 3. The comment in the code acknowledges this discrepancy (global total is 45, so 45+3=48 < 50 would actually pass). The spec appears to have an error, and the test correctly adjusts. However, a test for "Agent B sends 3 -> 202 (48 < 50)" and then "Agent B sends 3 -> 403 (51 > 50)" would be more thorough.

#### What Needs to Be Written

- **Optional:** `test_global_cap_allows_within_remaining` -- Verify that after A=25 + B=20 = 45, Agent B can still send 5 (exactly at cap) but not 6.

**Verdict: COVERED (spec step 5 has a logical error; test correctly adapts)**

---

### Scenario 4: Approval Flow

**File:** `src-tauri/tests/approval_flow.rs`
**Spec steps:** 8

#### What Already Exists

| Test Function | Covers Spec Steps |
|---|---|
| `test_approval_flow_agent_sends_large_tx_user_approves` | Steps 1-2, 4 (partial): send 50 -> awaiting_approval, verify approval created, user approves, verify resolved |
| `test_approval_flow_agent_sends_large_tx_user_denies` | Steps 6-8 (partial): send 50 -> awaiting_approval, user denies, verify resolved |
| `test_approval_flow_small_tx_auto_approved` | Extra: send below auto_approve -> auto-approved, no approval created |

#### What's Missing

- **Step 3:** "Poll transaction status. Assert `status: 'awaiting_approval'`." -- The test verifies the approval request exists via `ApprovalManager`, but does NOT poll the transaction via `GET /v1/transactions/{tx_id}` to verify the tx status is `awaiting_approval`.
- **Step 5:** "Wait for background execution. Poll transaction status. Assert `status: 'confirmed'`." -- After approval, the test does NOT verify that the transaction transitions to `confirmed`. It only checks the approval is resolved and the transaction record exists.
- **Step 8:** "Poll transaction status. Assert `status: 'denied'`." -- After denial, the test does NOT verify the transaction status becomes `denied`. It only checks the approval is resolved.
- **Second approve/deny cycle:** The spec has steps 6-8 as a second transaction (send another 20, deny it). The existing test uses separate test functions rather than a single sequential flow.

#### What Needs to Be Written

1. **`test_approval_flow_approve_then_tx_confirmed`** -- After user approves:
   - Poll `GET /v1/transactions/{tx_id}` and assert `status: "confirmed"` with `chain_tx_hash`
2. **`test_approval_flow_deny_then_tx_denied`** -- After user denies:
   - Poll `GET /v1/transactions/{tx_id}` and assert `status: "denied"`
3. **`test_approval_flow_full_sequence`** -- Single test covering all 8 spec steps:
   - Send 20 -> awaiting_approval
   - Poll tx -> awaiting_approval
   - Approve -> poll tx -> confirmed
   - Send another 20 -> awaiting_approval
   - Deny -> poll tx -> denied

**Verdict: PARTIALLY COVERED (transaction status polling after resolve is missing; deny->tx status flow is missing)**

---

### Scenario 5: Kill Switch

**File:** `src-tauri/tests/kill_switch.rs` + `src-tauri/tests/kill_switch_integration.rs`
**Spec steps:** 8

#### What Already Exists

**In `kill_switch.rs`:**

| Test Function | Covers Spec Steps |
|---|---|
| `test_kill_switch_blocks_and_resumes` | Steps 1-5, 7-8: two agents, A sends (succeeds), activate, A denied, B denied, deactivate, A succeeds |
| `test_kill_switch_reason_in_response` | Extra: verifies reason string in 403 response |

**In `kill_switch_integration.rs`:**

| Test Function | Covers Spec Steps |
|---|---|
| `test_kill_switch_blocks_all_transactions` | Step 4: activate, small send denied |
| `test_kill_switch_deactivation_resumes_transactions` | Steps 3, 4, 7, 8: activate, denied, deactivate, succeeds |
| `test_kill_switch_pending_approvals_not_auto_resolved` | Step 6: pending approval not auto-denied by kill switch |

#### What's Missing

- **Step 6 (spec):** "Verify any pending approval requests are NOT auto-executed while kill switch is active." -- `test_kill_switch_pending_approvals_not_auto_resolved` covers the "not auto-resolved" part. However, it does NOT test that if a pending approval IS manually approved while the kill switch is active, the resulting transaction execution is blocked.

#### What Needs to Be Written

1. **`test_kill_switch_blocks_execution_of_approved_tx`** -- While kill switch is active:
   - Create a tx requiring approval (before kill switch)
   - Activate kill switch
   - Manually approve the pending approval
   - Verify the transaction does NOT execute (or if it does, it's blocked at the execution layer)

**Verdict: MOSTLY COVERED (edge case of approving during kill switch not tested)**

---

### Scenario 6: Token Expiry and Re-Registration

**Spec file:** `src-tauri/tests/token_delivery.rs`
**Spec steps:** 4

#### What Already Exists

**THIS FILE DOES NOT EXIST.** No tests for token delivery expiry exist anywhere in the test suite.

The `agent_lifecycle.rs` tests cover token delivery (retrieve once -> get token, retrieve again -> null) but do NOT test the 5-minute expiry window.

#### What's Missing -- Everything

- Step 1: Agent registers, user approves (covered by lifecycle tests, but not in a token-expiry context)
- Step 2: Simulate 6 minutes passing (no time manipulation exists)
- Step 3: Poll status -> token null with expiry message
- Step 4: Re-register with new invitation code to get new token

#### What Needs to Be Written

1. **New file: `src-tauri/tests/token_delivery.rs`**
2. **`test_token_delivery_expires_after_5_minutes`**:
   - Register agent, approve
   - Manipulate the token delivery cache timestamp to simulate 6 minutes passing (requires either a test hook in `AgentRegistry` or direct DB/cache manipulation)
   - Poll status -> assert `token: null` and response contains expiry message
3. **`test_token_delivery_succeeds_within_5_minutes`**:
   - Register agent, approve
   - Poll status immediately -> assert token is present
4. **`test_token_reregistration_after_expiry`**:
   - Register agent, approve, let token expire
   - Re-register with new invitation code
   - Approve again
   - Poll status -> get new token
   - Use new token to send -> succeeds
5. **New helper needed:** Time manipulation or direct cache modification for token delivery expiry simulation

**Verdict: NOT COVERED -- New file and 3+ test functions required**

---

### Scenario 7: CLI Failure Recovery

**Spec file:** `src-tauri/tests/cli_failure_recovery.rs`
**Spec steps:** 5

#### What Already Exists

**THIS FILE DOES NOT EXIST.** No tests for CLI failure recovery exist anywhere in the integration test suite.

The `common/mod.rs` uses `MockCliExecutor::with_defaults()` which always succeeds. There is no mechanism to configure it to fail for specific tests.

#### What's Missing -- Everything

- Step 1: Configure mock CLI to return `Err(CliError::CommandFailed)`
- Step 2: Send 5.00 -> 202 (accepted, executing in background)
- Step 3: Background execution fails -> poll tx -> `status: "failed"` with error message
- Step 4: Verify spending ledger was NOT updated
- Step 5: Reconfigure mock CLI to succeed -> retry -> succeeds

#### What Needs to Be Written

1. **New file: `src-tauri/tests/cli_failure_recovery.rs`**
2. **New helper:** `create_test_app_with_failing_cli()` or equivalent that accepts a `MockCliExecutor` configured to fail on send commands
3. **`test_cli_failure_tx_status_is_failed`**:
   - Create app with failing CLI
   - Send 5.00 -> 202
   - Wait for background execution
   - Poll tx -> assert `status: "failed"` with error message
4. **`test_cli_failure_spending_ledger_not_updated`**:
   - Create app with failing CLI
   - Send 5.00 -> 202 (fails in background)
   - Send another 5.00 -> 202 (should succeed policy check since first tx didn't count)
   - Verify daily spending total reflects only successful transactions
5. **`test_cli_failure_retry_succeeds`**:
   - Create app with initially failing CLI
   - Send 5.00 -> fails in background
   - Reconfigure CLI to succeed (requires either swapping the CLI executor or creating a new app instance)
   - Send 5.00 -> 202 -> confirmed
6. **Infrastructure needed:**
   - `MockCliExecutor` must support dynamic response configuration (set to fail, then switch to succeed)
   - Or: `create_test_app_with_db_and_config` helper already exists, so a new app can be built with the same DB but different CLI

**Verdict: NOT COVERED -- New file, 3+ test functions, and new test infrastructure required**

---

### Scenario 8: Concurrent Transactions

**Spec file:** `src-tauri/tests/concurrent_transactions.rs`
**Spec steps:** 4

#### What Already Exists

**THIS FILE DOES NOT EXIST.** No concurrency tests exist anywhere in the integration test suite.

#### What's Missing -- Everything

- Step 1: Two agents, A daily_cap 20, B daily_cap 20, global daily_cap 30
- Step 2: Simultaneously send A=15, B=15 (total 30 = exactly at global cap)
- Step 3: Assert serialization prevents overspending (one succeeds, one denied, OR both succeed if exactly at cap)
- Step 4: Verify global spending ledger never exceeds cap

#### What Needs to Be Written

1. **New file: `src-tauri/tests/concurrent_transactions.rs`**
2. **`test_concurrent_sends_no_overspend`**:
   - Set up two agents with individual caps of 20, global cap of 30
   - Use `tokio::join!` or `tokio::spawn` to send A=15 and B=15 simultaneously
   - Assert: total successful spending <= 30
   - Assert: at least one transaction succeeds
3. **`test_concurrent_sends_exact_at_global_cap`**:
   - Two agents, global cap 30
   - Concurrently send A=15, B=15
   - Verify the global ledger total is correct (either 15 or 30, never 31+)
4. **`test_concurrent_sends_both_within_individual_caps`**:
   - Two agents, individual cap 20 each, global cap 40
   - Concurrently send A=15, B=15 (total 30, within global 40)
   - Both should succeed
5. **Infrastructure needed:**
   - Tests must use `#[tokio::test(flavor = "multi_thread")]` for real concurrency
   - May need to use `Arc<Barrier>` or similar synchronization to ensure truly concurrent submission
   - The `BEGIN EXCLUSIVE` transaction serialization in SQLite should handle correctness, but tests must verify it

**Verdict: NOT COVERED -- New file, 3+ test functions required**

---

### Scenario 9: Mock Mode

**File:** `src-tauri/tests/mock_mode.rs`
**Spec steps:** 7

#### What Already Exists

| Test Function | Covers Spec Steps |
|---|---|
| `test_mock_mode_health_endpoint` | Step 2: health returns `mock_mode: true` |
| `test_mock_mode_balance_returns_fake_data` | Step 3: balance returns fake data |
| `test_mock_mode_full_send_lifecycle` | Steps 4-6: register, send -> 202, poll -> confirmed with fake hash |
| `test_non_mock_mode_health_endpoint` | Extra: non-mock health check |
| `test_mock_mode_multiple_sends_accumulate` | Extra: multiple sends accumulate |

#### What's Missing

- **Step 1:** "Start the application with `ANB_MOCK=true`." -- The test uses `create_test_app()` which has `mock_mode=true` by default. This effectively covers it, but there is no explicit test that verifies `AppConfig::default_test()` has `mock_mode=true`.
- **Step 7:** "Verify the full spending policy engine still runs (mock mode replaces CLI, not business logic)." -- No explicit test verifies that spending limits are enforced in mock mode. The `test_mock_mode_multiple_sends_accumulate` test does accumulate, but doesn't test denial when limits are exceeded.

#### What Needs to Be Written

1. **`test_mock_mode_spending_policy_still_enforced`** -- In mock mode:
   - Create agent with `per_tx_max: 10`
   - Send 15.00 -> assert 403 (policy denied, even in mock mode)
   - Send 5.00 -> assert 202 (succeeds)
   - This proves mock mode only replaces the CLI, not the policy engine

**Verdict: MOSTLY COVERED (spending policy enforcement in mock mode not explicitly tested)**

---

## Summary Table

| Scenario | File Exists | Coverage | New Tests Needed | Priority |
|---|---|---|---|---|
| 1. Agent Lifecycle | Yes | COVERED | 0 (minor alignment) | Low |
| 2. Spending Limits | Yes | PARTIAL | 1 new test | Medium |
| 3. Global Policy | Yes | COVERED | 0-1 optional | Low |
| 4. Approval Flow | Yes | PARTIAL | 2-3 new tests | High |
| 5. Kill Switch | Yes (x2) | MOSTLY | 1 new test | Medium |
| 6. Token Expiry | **NO** | NOT COVERED | **New file + 3 tests** | **High** |
| 7. CLI Failure Recovery | **NO** | NOT COVERED | **New file + 3 tests + infra** | **High** |
| 8. Concurrent Transactions | **NO** | NOT COVERED | **New file + 3 tests** | **High** |
| 9. Mock Mode | Yes | MOSTLY | 1 new test | Low |

---

## Concrete Work Items

### New Files to Create

1. **`src-tauri/tests/token_delivery.rs`** -- Scenario 6 (3 test functions)
2. **`src-tauri/tests/cli_failure_recovery.rs`** -- Scenario 7 (3 test functions)
3. **`src-tauri/tests/concurrent_transactions.rs`** -- Scenario 8 (3 test functions)

### Enhancements to Existing Files

4. **`src-tauri/tests/spending_limits.rs`** -- Add `test_spending_approval_then_cumulative_tracking` (1 test)
5. **`src-tauri/tests/approval_flow.rs`** -- Add `test_approval_flow_approve_then_tx_confirmed`, `test_approval_flow_deny_then_tx_denied`, `test_approval_flow_full_sequence` (2-3 tests)
6. **`src-tauri/tests/kill_switch_integration.rs`** -- Add `test_kill_switch_blocks_execution_of_approved_tx` (1 test)
7. **`src-tauri/tests/mock_mode.rs`** -- Add `test_mock_mode_spending_policy_still_enforced` (1 test)

### Infrastructure / Helpers Needed

8. **`common/mod.rs`** -- Add helper for creating app with failing CLI executor
9. **Token expiry manipulation** -- Need a mechanism to simulate time passage for token delivery cache (either a test hook in `AgentRegistry` or direct timestamp manipulation)
10. **`MockCliExecutor` enhancements** -- Support for dynamic response switching (fail then succeed) for Scenario 7

### Total Estimated New Test Functions: 14-16

| Category | Count |
|---|---|
| New test files | 3 |
| New test functions (in new files) | 9 |
| New test functions (in existing files) | 5-7 |
| New/modified helpers | 2-3 |
| **Total new test functions** | **14-16** |
