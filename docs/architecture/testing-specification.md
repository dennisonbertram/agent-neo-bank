# Tally Agentic Wallet -- TDD Testing Specification

> **Version:** 1.0
> **Date:** 2026-02-27
> **Status:** Draft
> **Companion to:** `docs/architecture/architecture-plan.md` v2.0

---

## Table of Contents

1. [TDD Methodology](#1-tdd-methodology)
2. [Test File Conventions](#2-test-file-conventions)
3. [Test Cases by Component](#3-test-cases-by-component)
4. [Integration Test Scenarios](#4-integration-test-scenarios)
5. [CI Pipeline Requirements](#5-ci-pipeline-requirements)
6. [Test Fixtures and Helpers](#6-test-fixtures-and-helpers)

---

## 1. TDD Methodology

### Core Principle

**Tests are written FIRST, before implementation code.** Every module gets a failing test suite before a single line of production code is written.

### Red-Green-Refactor Cycle

1. **RED:** Write a failing test that describes the expected behavior of a function or module. The test must fail because the implementation does not yet exist.
2. **GREEN:** Write the minimum amount of implementation code required to make the test pass. Do not optimize or generalize prematurely.
3. **REFACTOR:** Clean up the implementation and the test. Remove duplication, improve naming, extract helpers. All tests must still pass after refactoring.
4. **REPEAT:** Pick the next behavior and start a new Red-Green-Refactor cycle.

### Workflow Rules

- **No implementation without a failing test.** If you cannot write a test for a behavior, the behavior is not well-defined enough to implement.
- **No PR merges without passing tests for all new/changed code.** Branch protection enforces this (see Section 5).
- **Tests must be deterministic.** No reliance on wall-clock time, network, or filesystem state. Use injected clocks, mock executors, and in-memory databases.
- **Tests must be fast.** Unit tests should complete in under 1 second each. Integration tests may take up to 5 seconds per scenario.
- **Test names describe behavior, not implementation.** Use the pattern `test_<module>_<behavior>_<expected_outcome>` for Rust and `<Component> > <behavior> > <expected outcome>` for React.

### TDD Per Implementation Phase

Each implementation phase in the architecture plan begins with writing tests:

- **Phase 1a (Plumbing):** Write test fixtures and helpers FIRST. Then write failing tests for CLI wrapper, auth service, and database layer before implementing any of them.
- **Phase 1b (Agent Operations):** Write failing tests for spending policy, global policy, transaction processor, agent registry, invitation system, and REST API endpoints before implementation.
- **Phase 2 (Multi-Agent & Controls):** Write failing tests for approval manager, MCP server, rate limiter, and event bus before implementation.
- **Phase 3 (Receive, Earn, Onramp):** Write failing tests for incoming transaction detection, Unix socket server, and transaction export before implementation.
- **Phase 4 (Polish):** Write failing tests for token rotation, error recovery, and auto-updater before implementation.

---

## 2. Test File Conventions

### Rust Unit Tests

Every Rust module file contains an inline `#[cfg(test)] mod tests` block at the bottom. This keeps unit tests colocated with the code they test.

| Source Module | Test Location (inline) |
|---|---|
| `src-tauri/src/cli/executor.rs` | `#[cfg(test)] mod tests` at bottom of `executor.rs` |
| `src-tauri/src/cli/parser.rs` | `#[cfg(test)] mod tests` at bottom of `parser.rs` |
| `src-tauri/src/cli/commands.rs` | `#[cfg(test)] mod tests` at bottom of `commands.rs` |
| `src-tauri/src/core/auth_service.rs` | `#[cfg(test)] mod tests` at bottom of `auth_service.rs` |
| `src-tauri/src/core/spending_policy.rs` | `#[cfg(test)] mod tests` at bottom of `spending_policy.rs` |
| `src-tauri/src/core/global_policy.rs` | `#[cfg(test)] mod tests` at bottom of `global_policy.rs` |
| `src-tauri/src/core/tx_processor.rs` | `#[cfg(test)] mod tests` at bottom of `tx_processor.rs` |
| `src-tauri/src/core/agent_registry.rs` | `#[cfg(test)] mod tests` at bottom of `agent_registry.rs` |
| `src-tauri/src/core/invitation.rs` | `#[cfg(test)] mod tests` at bottom of `invitation.rs` |
| `src-tauri/src/core/wallet_service.rs` | `#[cfg(test)] mod tests` at bottom of `wallet_service.rs` |
| `src-tauri/src/core/approval_manager.rs` | `#[cfg(test)] mod tests` at bottom of `approval_manager.rs` |
| `src-tauri/src/core/event_bus.rs` | `#[cfg(test)] mod tests` at bottom of `event_bus.rs` |
| `src-tauri/src/core/notification.rs` | `#[cfg(test)] mod tests` at bottom of `notification.rs` |
| `src-tauri/src/db/queries.rs` | `#[cfg(test)] mod tests` at bottom of `queries.rs` |
| `src-tauri/src/db/schema.rs` | `#[cfg(test)] mod tests` at bottom of `schema.rs` |
| `src-tauri/src/db/models.rs` | `#[cfg(test)] mod tests` at bottom of `models.rs` |
| `src-tauri/src/api/rest_handlers.rs` | `#[cfg(test)] mod tests` at bottom of `rest_handlers.rs` |
| `src-tauri/src/api/auth_middleware.rs` | `#[cfg(test)] mod tests` at bottom of `auth_middleware.rs` |
| `src-tauri/src/api/rate_limiter.rs` | `#[cfg(test)] mod tests` at bottom of `rate_limiter.rs` |
| `src-tauri/src/api/mcp_tools.rs` | `#[cfg(test)] mod tests` at bottom of `mcp_tools.rs` |
| `src-tauri/src/error.rs` | `#[cfg(test)] mod tests` at bottom of `error.rs` |
| `src-tauri/src/config.rs` | `#[cfg(test)] mod tests` at bottom of `config.rs` |

### Rust Integration Tests

Integration tests live in `src-tauri/tests/` as separate files. Each file tests a complete end-to-end scenario using the mock CLI executor and an in-memory SQLite database.

| Integration Test File | Scenario |
|---|---|
| `src-tauri/tests/agent_lifecycle.rs` | Full agent lifecycle: register with invitation code, approve, get token, send payment, view history |
| `src-tauri/tests/spending_limits.rs` | Spending policy enforcement across per-tx, daily, weekly, monthly caps |
| `src-tauri/tests/global_policy.rs` | Global caps, minimum reserve, kill switch enforcement |
| `src-tauri/tests/approval_flow.rs` | Transaction requires approval, user approves/denies, expiry cleanup |
| `src-tauri/tests/kill_switch.rs` | Kill switch activation, pending transaction handling, deactivation |
| `src-tauri/tests/token_delivery.rs` | Token delivery cache: creation, single retrieval, expiry |
| `src-tauri/tests/cli_failure_recovery.rs` | CLI errors mid-transaction, rollback verification, retry |
| `src-tauri/tests/concurrent_transactions.rs` | Multiple agents sending simultaneously, race condition prevention |
| `src-tauri/tests/mock_mode.rs` | Full app behavior with mock CLI executor |
| `src-tauri/tests/rest_api_contracts.rs` | REST API endpoint contract validation (request/response shapes) |
| `src-tauri/tests/mcp_protocol.rs` | MCP tool call dispatch and response validation |

### React Component Tests

Colocated `*.test.tsx` files next to each component.

| Component | Test File |
|---|---|
| `src/pages/Dashboard.tsx` | `src/pages/Dashboard.test.tsx` |
| `src/pages/AgentList.tsx` | `src/pages/AgentList.test.tsx` |
| `src/pages/AgentDetail.tsx` | `src/pages/AgentDetail.test.tsx` |
| `src/pages/Transactions.tsx` | `src/pages/Transactions.test.tsx` |
| `src/pages/Approvals.tsx` | `src/pages/Approvals.test.tsx` |
| `src/pages/Onboarding.tsx` | `src/pages/Onboarding.test.tsx` |
| `src/pages/Settings.tsx` | `src/pages/Settings.test.tsx` |
| `src/pages/Fund.tsx` | `src/pages/Fund.test.tsx` |
| `src/components/dashboard/BalanceCard.tsx` | `src/components/dashboard/BalanceCard.test.tsx` |
| `src/components/dashboard/SpendingChart.tsx` | `src/components/dashboard/SpendingChart.test.tsx` |
| `src/components/dashboard/RecentTransactions.tsx` | `src/components/dashboard/RecentTransactions.test.tsx` |
| `src/components/dashboard/AgentStatusGrid.tsx` | `src/components/dashboard/AgentStatusGrid.test.tsx` |
| `src/components/dashboard/BudgetUtilization.tsx` | `src/components/dashboard/BudgetUtilization.test.tsx` |
| `src/components/agents/AgentCard.tsx` | `src/components/agents/AgentCard.test.tsx` |
| `src/components/agents/AgentForm.tsx` | `src/components/agents/AgentForm.test.tsx` |
| `src/components/agents/SpendingLimitsEditor.tsx` | `src/components/agents/SpendingLimitsEditor.test.tsx` |
| `src/components/agents/AllowlistEditor.tsx` | `src/components/agents/AllowlistEditor.test.tsx` |
| `src/components/agents/AgentActivityFeed.tsx` | `src/components/agents/AgentActivityFeed.test.tsx` |
| `src/components/transactions/TransactionTable.tsx` | `src/components/transactions/TransactionTable.test.tsx` |
| `src/components/transactions/TransactionRow.tsx` | `src/components/transactions/TransactionRow.test.tsx` |
| `src/components/transactions/TransactionDetail.tsx` | `src/components/transactions/TransactionDetail.test.tsx` |
| `src/components/transactions/FilterBar.tsx` | `src/components/transactions/FilterBar.test.tsx` |
| `src/components/approvals/ApprovalCard.tsx` | `src/components/approvals/ApprovalCard.test.tsx` |
| `src/components/approvals/ApprovalQueue.tsx` | `src/components/approvals/ApprovalQueue.test.tsx` |
| `src/components/onboarding/EmailStep.tsx` | `src/components/onboarding/EmailStep.test.tsx` |
| `src/components/onboarding/OtpStep.tsx` | `src/components/onboarding/OtpStep.test.tsx` |
| `src/components/onboarding/FundStep.tsx` | `src/components/onboarding/FundStep.test.tsx` |
| `src/components/onboarding/WelcomeStep.tsx` | `src/components/onboarding/WelcomeStep.test.tsx` |
| `src/components/shared/CurrencyDisplay.tsx` | `src/components/shared/CurrencyDisplay.test.tsx` |
| `src/components/shared/StatusBadge.tsx` | `src/components/shared/StatusBadge.test.tsx` |
| `src/components/shared/EmptyState.tsx` | `src/components/shared/EmptyState.test.tsx` |
| `src/components/shared/ConfirmDialog.tsx` | `src/components/shared/ConfirmDialog.test.tsx` |
| `src/hooks/useBalance.ts` | `src/hooks/useBalance.test.ts` |
| `src/hooks/useAgents.ts` | `src/hooks/useAgents.test.ts` |
| `src/hooks/useTransactions.ts` | `src/hooks/useTransactions.test.ts` |
| `src/hooks/useApprovals.ts` | `src/hooks/useApprovals.test.ts` |
| `src/hooks/useTauriEvent.ts` | `src/hooks/useTauriEvent.test.ts` |
| `src/hooks/useInvoke.ts` | `src/hooks/useInvoke.test.ts` |
| `src/stores/authStore.ts` | `src/stores/authStore.test.ts` |
| `src/stores/agentStore.ts` | `src/stores/agentStore.test.ts` |
| `src/stores/transactionStore.ts` | `src/stores/transactionStore.test.ts` |
| `src/stores/settingsStore.ts` | `src/stores/settingsStore.test.ts` |
| `src/stores/approvalStore.ts` | `src/stores/approvalStore.test.ts` |
| `src/lib/format.ts` | `src/lib/format.test.ts` |

### API Contract Tests

Dedicated directory for endpoint contract tests using HTTP-level testing.

| Test File | Scope |
|---|---|
| `src-tauri/tests/api/register_agent.rs` | `POST /v1/agents/register` -- all request/response shapes |
| `src-tauri/tests/api/register_status.rs` | `GET /v1/agents/register/{id}/status` -- polling, token delivery |
| `src-tauri/tests/api/send_payment.rs` | `POST /v1/send` -- auto-approve, require-approval, deny, kill-switch |
| `src-tauri/tests/api/get_balance.rs` | `GET /v1/balance` -- visible, hidden, cached |
| `src-tauri/tests/api/spending_limits.rs` | `GET /v1/spending/limits` -- current limits and usage |
| `src-tauri/tests/api/request_increase.rs` | `POST /v1/spending/request-increase` -- approval creation |
| `src-tauri/tests/api/get_transaction.rs` | `GET /v1/transactions/{tx_id}` -- all status states |
| `src-tauri/tests/api/list_transactions.rs` | `GET /v1/transactions` -- pagination, filters |
| `src-tauri/tests/api/list_agents.rs` | `GET /v1/agents` -- pagination |
| `src-tauri/tests/api/list_approvals.rs` | `GET /v1/approvals` -- pagination, status filter |
| `src-tauri/tests/api/health_check.rs` | `GET /v1/health` -- no auth, mock mode flag |

---

## 3. Test Cases by Component

### 3.1 CLI Wrapper (`cli/executor.rs`)

#### `test_cli_parse_balance_output_success`
- **Given:** The CLI returns stdout `{"balance": "1247.83", "asset": "USDC"}` with exit code 0
- **When:** `CliOutput::parse()` is called on the raw output
- **Then:** Returns `CliOutput` with `success: true`, `data.balance == "1247.83"`, `data.asset == "USDC"`

#### `test_cli_parse_send_output_with_tx_hash`
- **Given:** The CLI returns stdout `{"tx_hash": "0xabc123..."}` with exit code 0
- **When:** `CliOutput::parse()` is called
- **Then:** Returns `CliOutput` with `success: true`, `data.tx_hash == "0xabc123..."`

#### `test_cli_parse_auth_status_authenticated`
- **Given:** The CLI returns stdout `{"authenticated": true, "email": "user@example.com"}` with exit code 0
- **When:** `CliOutput::parse()` is called
- **Then:** Returns `CliOutput` with `success: true`, `data.authenticated == true`

#### `test_cli_nonzero_exit_code_returns_error`
- **Given:** The CLI returns stderr `"Error: insufficient funds"` with exit code 1
- **When:** `RealCliExecutor::run()` is called
- **Then:** Returns `Err(CliError::CommandFailed)` with the stderr message preserved

#### `test_cli_timeout_returns_error`
- **Given:** The CLI process does not complete within the configured timeout (e.g., 30 seconds)
- **When:** `RealCliExecutor::run()` is called
- **Then:** Returns `Err(CliError::Timeout)` and the child process is killed

#### `test_cli_session_expired_detected`
- **Given:** The CLI returns stdout `{"authenticated": false}` for `awal auth status`
- **When:** `CliOutput::parse()` is called
- **Then:** Returns `CliOutput` with `success: true` but `data.authenticated == false`, allowing the caller to detect expired session

#### `test_cli_binary_not_found`
- **Given:** `RealCliExecutor` is constructed with a binary path that does not exist (`/nonexistent/awal`)
- **When:** `run()` is called with any command
- **Then:** Returns `Err(CliError::NotFound)` with a descriptive message

#### `test_mock_executor_returns_canned_balance`
- **Given:** `MockCliExecutor` is configured with a canned balance response
- **When:** `run(AwalCommand::GetBalance)` is called
- **Then:** Returns the canned response with `success: true` and the preconfigured balance

#### `test_mock_executor_returns_canned_send`
- **Given:** `MockCliExecutor` is configured with a canned send response
- **When:** `run(AwalCommand::Send { ... })` is called
- **Then:** Returns a fake tx_hash in the response

#### `test_mock_executor_returns_default_for_unknown_command`
- **Given:** `MockCliExecutor` has no canned response for `AwalCommand::AuthLogout`
- **When:** `run(AwalCommand::AuthLogout)` is called
- **Then:** Returns a default `CliOutput` with `success: true` and empty data

#### `test_cli_command_to_args_send`
- **Given:** `AwalCommand::Send { to: "0x123", amount: Decimal::new(500, 2), asset: "USDC" }`
- **When:** `to_args()` is called
- **Then:** Returns `["send", "--to", "0x123", "--amount", "5.00", "--asset", "USDC"]`

#### `test_cli_command_to_args_auth_login`
- **Given:** `AwalCommand::AuthLogin { email: "user@example.com" }`
- **When:** `to_args()` is called
- **Then:** Returns `["auth", "login", "--email", "user@example.com"]`

### 3.2 Auth Service (`core/auth_service.rs`)

#### `test_auth_otp_login_calls_cli`
- **Given:** A mock CLI executor that expects `AwalCommand::AuthLogin`
- **When:** `auth_service.login("user@example.com")` is called
- **Then:** The mock CLI is called with the correct email. Returns success.

#### `test_auth_otp_verify_success`
- **Given:** A mock CLI executor that returns success for `AwalCommand::AuthVerify`
- **When:** `auth_service.verify("user@example.com", "123456")` is called
- **Then:** Returns `Ok(AuthResult::Verified)` and the user is marked authenticated

#### `test_auth_otp_verify_invalid_code`
- **Given:** A mock CLI executor that returns failure for `AwalCommand::AuthVerify`
- **When:** `auth_service.verify("user@example.com", "000000")` is called
- **Then:** Returns `Err(AppError::InvalidOtp)`

#### `test_auth_token_validation_sha256_cache_hit`
- **Given:** A valid token that was previously validated and cached (within 5-minute TTL)
- **When:** `auth_service.validate_token("anb_validtoken123")` is called
- **Then:** Returns `Ok(agent_id)` without calling argon2 verify. Cache lookup is O(1).

#### `test_auth_token_validation_sha256_cache_miss_argon2_fallback`
- **Given:** A valid token that is NOT in the SHA-256 cache, and the database contains the argon2 hash of this token
- **When:** `auth_service.validate_token("anb_validtoken123")` is called
- **Then:** Returns `Ok(agent_id)` after running argon2 verify. The SHA-256 hash is now added to the cache.

#### `test_auth_token_validation_cache_expired_triggers_argon2`
- **Given:** A token that was cached but the cache entry is older than 5 minutes (TTL expired)
- **When:** `auth_service.validate_token("anb_validtoken123")` is called
- **Then:** The expired cache entry is ignored. Argon2 verify runs against the database. Cache is refreshed.

#### `test_auth_token_validation_invalid_token`
- **Given:** A token `"anb_invalidtoken"` that does not match any active agent's argon2 hash
- **When:** `auth_service.validate_token("anb_invalidtoken")` is called
- **Then:** Returns `Err(AppError::InvalidToken)`

#### `test_auth_token_validation_suspended_agent_rejected`
- **Given:** An agent whose status is `suspended` (token hash exists but agent is not active)
- **When:** `auth_service.validate_token("anb_suspended_agent_token")` is called
- **Then:** Returns `Err(AppError::InvalidToken)` because only active agents are checked

#### `test_auth_cache_populated_after_first_validation`
- **Given:** An empty SHA-256 cache and a valid token
- **When:** `validate_token()` is called once
- **Then:** A subsequent call with the same token hits the cache (verified by asserting argon2 is not called again)

#### `test_auth_logout_clears_session`
- **Given:** An authenticated session
- **When:** `auth_service.logout()` is called
- **Then:** The CLI `auth logout` command is invoked and the auth state is cleared

### 3.3 Spending Policy Engine (`core/spending_policy.rs`)

#### `test_spending_policy_auto_approves_below_threshold`
- **Given:** Agent has `auto_approve_max: 10.00`, `per_tx_max: 25.00`, `daily_cap: 100.00`. No spending today.
- **When:** `evaluate(agent_id, Decimal::new(500, 2), "0xrecipient")` (amount = 5.00)
- **Then:** Returns `PolicyDecision::AutoApproved`

#### `test_spending_policy_requires_approval_above_threshold`
- **Given:** Agent has `auto_approve_max: 10.00`, `per_tx_max: 25.00`, `daily_cap: 100.00`. No spending today.
- **When:** `evaluate(agent_id, Decimal::new(1500, 2), "0xrecipient")` (amount = 15.00)
- **Then:** Returns `PolicyDecision::RequiresApproval` with reason mentioning auto-approve threshold

#### `test_spending_policy_denies_when_per_tx_limit_exceeded`
- **Given:** Agent has `per_tx_max: 25.00`
- **When:** `evaluate(agent_id, Decimal::new(2600, 2), "0xrecipient")` (amount = 26.00)
- **Then:** Returns `PolicyDecision::Denied` with reason `"Amount 26.00 exceeds per-tx limit of 25.00"`

#### `test_spending_policy_denies_when_daily_cap_exceeded`
- **Given:** Agent has `daily_cap: 100.00`. Today's spending so far: 95.00.
- **When:** `evaluate(agent_id, Decimal::new(600, 2), "0xrecipient")` (amount = 6.00, total would be 101.00)
- **Then:** Returns `PolicyDecision::Denied` with reason mentioning daily cap

#### `test_spending_policy_denies_when_weekly_cap_exceeded`
- **Given:** Agent has `weekly_cap: 500.00`. This week's spending so far: 498.00.
- **When:** `evaluate(agent_id, Decimal::new(300, 2), "0xrecipient")` (amount = 3.00, total would be 501.00)
- **Then:** Returns `PolicyDecision::Denied` with reason mentioning weekly cap

#### `test_spending_policy_denies_when_monthly_cap_exceeded`
- **Given:** Agent has `monthly_cap: 1500.00`. This month's spending so far: 1490.00.
- **When:** `evaluate(agent_id, Decimal::new(1100, 2), "0xrecipient")` (amount = 11.00, total would be 1501.00)
- **Then:** Returns `PolicyDecision::Denied` with reason mentioning monthly cap

#### `test_spending_policy_exact_amount_equals_limit_allowed`
- **Given:** Agent has `per_tx_max: 25.00`, `daily_cap: 100.00`. Today's spending: 75.00.
- **When:** `evaluate(agent_id, Decimal::new(2500, 2), "0xrecipient")` (amount = 25.00, daily total = exactly 100.00)
- **Then:** Returns `PolicyDecision::AutoApproved` (or `RequiresApproval` depending on auto_approve_max). The amount equal to the limit is NOT denied.

#### `test_spending_policy_just_over_limit_denied`
- **Given:** Agent has `daily_cap: 100.00`. Today's spending: 75.00.
- **When:** `evaluate(agent_id, Decimal::new(2501, 2), "0xrecipient")` (amount = 25.01, daily total = 100.01)
- **Then:** Returns `PolicyDecision::Denied`

#### `test_spending_policy_allowlist_enforced_recipient_allowed`
- **Given:** Agent has `allowlist: ["0xAllowed1", "0xAllowed2"]`
- **When:** `evaluate(agent_id, Decimal::new(500, 2), "0xAllowed1")` (amount = 5.00)
- **Then:** Does NOT deny based on allowlist. Decision is based on spending limits only.

#### `test_spending_policy_allowlist_enforced_recipient_blocked`
- **Given:** Agent has `allowlist: ["0xAllowed1"]`
- **When:** `evaluate(agent_id, Decimal::new(500, 2), "0xNotAllowed")` (amount = 5.00)
- **Then:** Returns `PolicyDecision::Denied` with reason `"Recipient not in allowlist"`

#### `test_spending_policy_empty_allowlist_allows_any_recipient`
- **Given:** Agent has `allowlist: []` (empty)
- **When:** `evaluate(agent_id, Decimal::new(500, 2), "0xAnyAddress")` (amount = 5.00)
- **Then:** Does NOT deny based on allowlist. Empty list means no restriction.

#### `test_spending_policy_multiple_rules_combined`
- **Given:** Agent has `per_tx_max: 50.00`, `daily_cap: 100.00`, `weekly_cap: 500.00`, `auto_approve_max: 10.00`, `allowlist: ["0xValid"]`. Today's spending: 40.00.
- **When:** `evaluate(agent_id, Decimal::new(4500, 2), "0xValid")` (amount = 45.00, daily total = 85.00)
- **Then:** Returns `PolicyDecision::RequiresApproval` (within all caps, but above auto-approve)

#### `test_spending_policy_zero_caps_mean_denied`
- **Given:** Agent has all caps set to `0.00` (default for new agents)
- **When:** `evaluate(agent_id, Decimal::new(100, 2), "0xrecipient")` (amount = 1.00)
- **Then:** Returns `PolicyDecision::Denied` because per_tx_max is 0

### 3.4 Global Policy Engine (`core/global_policy.rs`)

#### `test_global_policy_allows_within_daily_cap`
- **Given:** Global `daily_cap: 500.00`. Today's global spending: 200.00.
- **When:** Global policy check for amount 50.00
- **Then:** Passes (not denied by global policy)

#### `test_global_policy_denies_when_daily_cap_exceeded`
- **Given:** Global `daily_cap: 500.00`. Today's global spending: 480.00.
- **When:** Global policy check for amount 25.00 (total would be 505.00)
- **Then:** Returns `PolicyDecision::Denied` with reason mentioning global daily cap

#### `test_global_policy_denies_when_weekly_cap_exceeded`
- **Given:** Global `weekly_cap: 2000.00`. This week's global spending: 1990.00.
- **When:** Global policy check for amount 15.00
- **Then:** Returns `PolicyDecision::Denied` with reason mentioning global weekly cap

#### `test_global_policy_denies_when_monthly_cap_exceeded`
- **Given:** Global `monthly_cap: 5000.00`. This month's global spending: 4995.00.
- **When:** Global policy check for amount 10.00
- **Then:** Returns `PolicyDecision::Denied` with reason mentioning global monthly cap

#### `test_global_policy_minimum_reserve_prevents_overdraw`
- **Given:** Global `min_reserve_balance: 100.00`. Current wallet balance: 150.00.
- **When:** Global policy check for amount 60.00 (remaining balance would be 90.00 < 100.00 reserve)
- **Then:** Returns `PolicyDecision::Denied` with reason `"Would drop balance below minimum reserve of 100.00"`

#### `test_global_policy_minimum_reserve_allows_safe_tx`
- **Given:** Global `min_reserve_balance: 100.00`. Current wallet balance: 500.00.
- **When:** Global policy check for amount 50.00 (remaining balance = 450.00)
- **Then:** Passes (not denied by reserve check)

#### `test_global_policy_kill_switch_denies_all`
- **Given:** Global `kill_switch_active: true`, `kill_switch_reason: "Suspicious activity"`
- **When:** Global policy check for any amount
- **Then:** Returns `PolicyDecision::Denied` with reason `"Emergency kill switch active: Suspicious activity"`

#### `test_global_policy_kill_switch_denies_even_with_remaining_limits`
- **Given:** Kill switch active. Agent has remaining budget (daily_cap not hit, per_tx under limit).
- **When:** Global policy check for a small amount (1.00)
- **Then:** Returns `PolicyDecision::Denied`. Kill switch overrides all other policy.

#### `test_global_policy_zero_cap_means_unlimited`
- **Given:** Global `daily_cap: 0.00` (zero means unlimited per architecture spec)
- **When:** Global policy check for amount 999999.00
- **Then:** Passes (zero cap is not enforced)

#### `test_global_policy_reserve_edge_exact_balance`
- **Given:** Global `min_reserve_balance: 100.00`. Current wallet balance: 200.00.
- **When:** Global policy check for amount 100.00 (remaining = exactly 100.00 = reserve)
- **Then:** Passes (equal to reserve is acceptable, not below)

### 3.5 Transaction Processor (`core/tx_processor.rs`)

#### `test_tx_processor_successful_send_returns_202`
- **Given:** Agent with valid spending limits. Mock CLI returns success with tx_hash.
- **When:** `process_send(agent_id, SendRequest { to: "0x123", amount: 5.00, ... })` is called
- **Then:** Returns `TransactionResult::Accepted` with `status: "executing"` and a `tx_id`

#### `test_tx_processor_async_202_response_immediate`
- **Given:** A valid send request that passes policy
- **When:** `process_send()` is called
- **Then:** Returns `202 Accepted` immediately. The CLI execution happens in a background task. The response does not block on CLI completion.

#### `test_tx_processor_ledger_update_atomicity`
- **Given:** A transaction that is confirmed by the CLI
- **When:** `execute_send()` completes successfully
- **Then:** Both the transaction status update (to "confirmed") and the spending ledger update (agent + global) happen inside a single `BEGIN EXCLUSIVE` transaction. If either fails, both roll back.

#### `test_tx_processor_webhook_callback_on_success`
- **Given:** A send request with `webhook_url: "http://localhost:8080/webhook"`. Mock CLI returns success.
- **When:** The background execution completes
- **Then:** A POST request is made to the webhook URL with `{ "tx_id": "...", "status": "confirmed" }`

#### `test_tx_processor_webhook_callback_on_failure`
- **Given:** A send request with `webhook_url: "http://localhost:8080/webhook"`. Mock CLI returns error.
- **When:** The background execution fails
- **Then:** A POST request is made to the webhook URL with `{ "tx_id": "...", "status": "failed" }`

#### `test_tx_processor_cli_failure_mid_transaction_marks_failed`
- **Given:** A send request that passes policy. Mock CLI returns `Err(CliError::CommandFailed)`.
- **When:** The background execution runs
- **Then:** The transaction status is updated to `"failed"` with the error message. The spending ledger is NOT updated (no money was actually sent).

#### `test_tx_processor_rollback_on_ledger_update_error`
- **Given:** CLI send succeeds but the subsequent ledger update fails (simulated DB error)
- **When:** `execute_send()` runs
- **Then:** The transaction is marked as `"failed"`. The ledger is NOT updated. The atomic transaction rolled back.

#### `test_tx_processor_requires_approval_above_auto_approve`
- **Given:** Agent has `auto_approve_max: 10.00`. Send request for 15.00.
- **When:** `process_send()` is called
- **Then:** Returns `TransactionResult::Accepted` with `status: "awaiting_approval"`. An approval request is created. OS notification is sent.

#### `test_tx_processor_denied_exceeds_per_tx_max`
- **Given:** Agent has `per_tx_max: 25.00`. Send request for 30.00.
- **When:** `process_send()` is called
- **Then:** Returns `TransactionResult::Denied` with reason. Transaction status set to "denied".

#### `test_tx_processor_period_keys_set_at_creation_time`
- **Given:** A send request
- **When:** `process_send()` creates the transaction record
- **Then:** `period_daily`, `period_weekly`, `period_monthly` are set based on `Utc::now()` at creation time, not at completion time

#### `test_tx_processor_event_emitted_on_confirmation`
- **Given:** A transaction that completes successfully
- **When:** `execute_send()` finishes
- **Then:** `Event::TransactionConfirmed(tx_id)` is emitted on the event bus

#### `test_tx_processor_event_emitted_on_denial`
- **Given:** A transaction that is denied by policy
- **When:** `process_send()` returns denial
- **Then:** `Event::TransactionDenied(tx_id)` is emitted on the event bus

### 3.6 Agent Registry (`core/agent_registry.rs`)

#### `test_agent_register_with_valid_invitation_code`
- **Given:** A valid, unused invitation code `"INV-abc123"` exists in the database
- **When:** `register(AgentRegistrationRequest { name: "Claude", invitation_code: "INV-abc123", ... })` is called
- **Then:** Returns `Ok(AgentRegistrationResult { agent_id, status: "pending" })`. Agent is inserted with status "pending". Invitation code is marked as used.

#### `test_agent_register_with_invalid_code_rejected`
- **Given:** No invitation code `"INV-bogus"` exists in the database
- **When:** `register(AgentRegistrationRequest { invitation_code: "INV-bogus", ... })` is called
- **Then:** Returns `Err(AppError::InvalidInvitationCode)`

#### `test_agent_register_with_expired_code_rejected`
- **Given:** An invitation code exists but `expires_at` is in the past
- **When:** `register()` is called with the expired code
- **Then:** Returns `Err(AppError::InvitationCodeExpired)`

#### `test_agent_register_with_already_used_code_rejected`
- **Given:** An invitation code with `max_uses: 1` and `use_count: 1`
- **When:** `register()` is called with this code
- **Then:** Returns `Err(AppError::InvitationCodeExpired)` (or a more specific "already used" error)

#### `test_agent_register_creates_pending_approval_request`
- **Given:** Valid registration request
- **When:** `register()` is called
- **Then:** An approval request is created with `request_type: "registration"`, `status: "pending"`, and `expires_at` set to 24 hours from now

#### `test_agent_register_creates_zero_spending_policy`
- **Given:** Valid registration request
- **When:** `register()` is called
- **Then:** A spending policy is created for the agent with all limits set to 0 (nothing allowed until user configures)

#### `test_agent_register_notification_sent`
- **Given:** Valid registration request
- **When:** `register()` is called
- **Then:** An OS notification is sent with the agent's name

#### `test_agent_register_rich_metadata_stored`
- **Given:** Registration request with `purpose: "Coding assistant"`, `agent_type: "coding_assistant"`, `capabilities: ["send", "receive"]`
- **When:** `register()` is called and the agent is retrieved from the database
- **Then:** All metadata fields are stored correctly

#### `test_agent_approve_generates_token_and_delivers`
- **Given:** A pending agent
- **When:** `approve(agent_id)` is called
- **Then:** A token with `"anb_"` prefix is generated. The argon2 hash is stored in the agents table. The encrypted token is stored in `token_delivery` with 5-minute expiry.

#### `test_agent_token_delivery_returns_once_then_deletes`
- **Given:** An approved agent with a token in the delivery cache
- **When:** `retrieve_token(agent_id)` is called the first time
- **Then:** Returns `Ok(Some("anb_..."))`. Second call returns `Ok(None)`.

#### `test_agent_token_delivery_expired_returns_none`
- **Given:** An approved agent whose token delivery cache `expires_at` is in the past (simulated by setting a past timestamp)
- **When:** `retrieve_token(agent_id)` is called
- **Then:** Returns `Ok(None)`

#### `test_agent_duplicate_registration_with_same_code_rejected`
- **Given:** A single-use invitation code that has already been used by another agent
- **When:** A second registration attempt uses the same code
- **Then:** Returns `Err(AppError::InvitationCodeExpired)` (code already at max uses)

### 3.7 Invitation Code System (`core/invitation.rs`)

#### `test_invitation_code_generation_format`
- **Given:** User requests a new invitation code with label "For Claude Code"
- **When:** `invitation_manager.generate("For Claude Code", Some(24))` is called
- **Then:** Returns a code matching format `INV-[a-z0-9]{8}`. The code is stored in the database with `max_uses: 1`, `expires_at` 24 hours from now, and the provided label.

#### `test_invitation_code_validation_valid`
- **Given:** A valid, unused invitation code exists
- **When:** `invitation_manager.validate("INV-abc12345")` is called
- **Then:** Returns `Ok(InvitationCode { ... })` with the code's details

#### `test_invitation_code_validation_nonexistent`
- **Given:** No code `"INV-doesntexist"` in the database
- **When:** `invitation_manager.validate("INV-doesntexist")` is called
- **Then:** Returns `Err(AppError::InvalidInvitationCode)`

#### `test_invitation_code_expiry_enforced`
- **Given:** A code with `expires_at` set to a timestamp in the past
- **When:** `invitation_manager.validate()` is called
- **Then:** Returns `Err(AppError::InvitationCodeExpired)`

#### `test_invitation_code_single_use_enforced`
- **Given:** A code with `max_uses: 1` and `use_count: 1` (already used once)
- **When:** `invitation_manager.validate()` is called
- **Then:** Returns `Err(AppError::InvitationCodeExpired)`

#### `test_invitation_code_max_active_codes_limit`
- **Given:** The system has a configurable max active codes limit (e.g., 50). 50 active codes already exist.
- **When:** `invitation_manager.generate()` is called
- **Then:** Returns `Err(AppError::MaxActiveCodesReached)`

#### `test_invitation_code_no_expiry_if_hours_not_set`
- **Given:** User requests a code with `expires_in_hours: None`
- **When:** `invitation_manager.generate("Permanent", None)` is called
- **Then:** The code is stored with `expires_at: None` (never expires)

### 3.8 Balance Cache (`core/wallet_service.rs`)

#### `test_balance_cache_hit_within_ttl`
- **Given:** A cached balance of 1247.83 USDC fetched 10 seconds ago. TTL is 30 seconds.
- **When:** `balance_cache.get_or_fetch(cli)` is called
- **Then:** Returns the cached balance. The CLI is NOT called (verified by mock assertion).

#### `test_balance_cache_miss_triggers_cli_call`
- **Given:** An empty cache (no previous fetch)
- **When:** `balance_cache.get_or_fetch(cli)` is called
- **Then:** The CLI `GetBalance` command is called. The result is cached and returned.

#### `test_balance_cache_ttl_expiry_refetches`
- **Given:** A cached balance fetched 31 seconds ago. TTL is 30 seconds.
- **When:** `balance_cache.get_or_fetch(cli)` is called
- **Then:** The CLI is called again (cache is stale). The new result replaces the old cache entry.

#### `test_balance_cache_concurrent_access_single_fetch`
- **Given:** An empty cache. Multiple concurrent calls to `get_or_fetch()` arrive simultaneously.
- **When:** All calls execute
- **Then:** The CLI is called only once (the write lock prevents duplicate fetches). All callers receive the same result.

#### `test_balance_visibility_per_agent_hidden`
- **Given:** An agent with `balance_visible: false`
- **When:** The balance endpoint is called with this agent's token
- **Then:** Returns `{ balance: null, balance_visible: false }` -- the balance is not exposed

#### `test_balance_visibility_per_agent_visible`
- **Given:** An agent with `balance_visible: true`
- **When:** The balance endpoint is called with this agent's token
- **Then:** Returns the full balance information

### 3.9 REST API Endpoints (`api/routes.rs`)

#### `test_api_health_check_no_auth`
- **Given:** No authentication header
- **When:** `GET /v1/health`
- **Then:** Returns 200 with `{ "status": "ok", "version": "...", "network": "...", "mock_mode": false }`

#### `test_api_send_valid_request_returns_202`
- **Given:** Valid bearer token. Agent with spending limits that allow the amount.
- **When:** `POST /v1/send` with `{ "to": "0x123", "amount": 5.00 }`
- **Then:** Returns 202 with `{ "tx_id": "...", "status": "executing" }`

#### `test_api_send_missing_auth_returns_401`
- **Given:** No `Authorization` header
- **When:** `POST /v1/send`
- **Then:** Returns 401 Unauthorized

#### `test_api_send_expired_token_returns_401`
- **Given:** A bearer token for a revoked agent
- **When:** `POST /v1/send`
- **Then:** Returns 401 Unauthorized

#### `test_api_send_invalid_amount_returns_400`
- **Given:** Valid token. Malformed request body `{ "to": "0x123", "amount": "not-a-number" }`
- **When:** `POST /v1/send`
- **Then:** Returns 400 Bad Request with descriptive error

#### `test_api_send_missing_to_field_returns_400`
- **Given:** Valid token. Request body `{ "amount": 5.00 }` (missing "to")
- **When:** `POST /v1/send`
- **Then:** Returns 400 Bad Request

#### `test_api_send_policy_denied_returns_403`
- **Given:** Valid token. Agent's per_tx_max is 10.00. Amount is 50.00.
- **When:** `POST /v1/send`
- **Then:** Returns 403 with `{ "error": "policy_denied", "message": "Amount exceeds per-tx limit..." }`

#### `test_api_send_kill_switch_returns_403`
- **Given:** Valid token. Kill switch is active.
- **When:** `POST /v1/send`
- **Then:** Returns 403 with `{ "error": "kill_switch_active" }`

#### `test_api_register_with_valid_code_returns_201`
- **Given:** Valid invitation code in request body
- **When:** `POST /v1/agents/register` with `{ "name": "Agent", "invitation_code": "INV-valid" }`
- **Then:** Returns 201 with `{ "agent_id": "...", "status": "pending" }`

#### `test_api_register_invalid_code_returns_400`
- **Given:** Invalid invitation code
- **When:** `POST /v1/agents/register`
- **Then:** Returns 400 with descriptive error

#### `test_api_list_transactions_pagination`
- **Given:** 50 transactions in the database. Valid token.
- **When:** `GET /v1/transactions?limit=10&offset=20`
- **Then:** Returns 200 with 10 transactions, `total: 50`, `limit: 10`, `offset: 20`

#### `test_api_list_transactions_filter_by_status`
- **Given:** Transactions with various statuses. Valid token.
- **When:** `GET /v1/transactions?status=confirmed`
- **Then:** Returns only transactions with status "confirmed"

#### `test_api_rate_limiter_blocks_excess_requests`
- **Given:** Rate limit of 60 requests per minute
- **When:** 61 requests are made within one minute
- **Then:** The 61st request returns 429 Too Many Requests

### 3.10 MCP Server

#### `test_mcp_send_payment_tool_dispatch`
- **Given:** A valid MCP session bound to agent_id. Agent has spending limits.
- **When:** Tool call `send_payment` with `{ "to": "0x123", "amount": "5.00" }`
- **Then:** The core `process_send()` is called with the correct agent_id and parameters. Returns MCP response with tx_id and status.

#### `test_mcp_check_balance_response_shape`
- **Given:** A valid MCP session. Balance is 1247.83.
- **When:** Tool call `check_balance` with `{}`
- **Then:** Returns MCP response with `{ "balance": "1247.83", "asset": "USDC", "network": "...", "address": "..." }`

#### `test_mcp_per_agent_auth_validation`
- **Given:** An MCP server started with token `"anb_agent1_token"`
- **When:** Any tool call is made
- **Then:** All operations are scoped to the agent bound to that token. The agent_id is automatically injected into all core service calls.

#### `test_mcp_unknown_tool_returns_error`
- **Given:** A valid MCP session
- **When:** Tool call `nonexistent_tool` with `{}`
- **Then:** Returns MCP error response with `"Unknown tool: nonexistent_tool"`

#### `test_mcp_invalid_token_fails_on_startup`
- **Given:** An invalid token `"anb_invalid"` passed to `McpServer::start_stdio()`
- **When:** The server attempts to start
- **Then:** Returns `Err(AppError::InvalidToken)` and the server does not start

#### `test_mcp_get_spending_limits_response_shape`
- **Given:** A valid MCP session. Agent has configured spending limits.
- **When:** Tool call `get_spending_limits` with `{}`
- **Then:** Returns limits with current usage included (today, this_week, this_month totals)

### 3.11 Approval Manager

#### `test_approval_create_request`
- **Given:** An agent and a transaction that requires approval
- **When:** `approval_manager.create_request(agent_id, ApprovalType::Transaction, tx, reason)` is called
- **Then:** An approval request is inserted with `status: "pending"`, `expires_at` set to 24 hours from now, and associated tx_id

#### `test_approval_user_approves`
- **Given:** A pending approval request with an associated transaction
- **When:** `approval_manager.resolve(approval_id, "approved")` is called
- **Then:** Approval status is updated to "approved". The associated transaction is executed (background task started).

#### `test_approval_user_denies`
- **Given:** A pending approval request with an associated transaction
- **When:** `approval_manager.resolve(approval_id, "denied")` is called
- **Then:** Approval status is updated to "denied". The associated transaction status is updated to "denied".

#### `test_approval_expiry_cleanup`
- **Given:** An approval request with `expires_at` in the past and `status: "pending"`
- **When:** `expire_stale_approvals()` runs
- **Then:** The approval is updated to `status: "expired"`. The associated transaction is marked as `"failed"` with message "Approval request expired".

#### `test_approval_notification_sent_on_creation`
- **Given:** A new approval request is created
- **When:** `create_request()` completes
- **Then:** An OS notification with `NotificationEvent::ApprovalRequired` is dispatched

#### `test_approval_already_resolved_cannot_be_changed`
- **Given:** An approval that has already been approved
- **When:** `resolve(approval_id, "denied")` is called again
- **Then:** Returns `Err(AppError::ApprovalAlreadyResolved)` or silently ignores

### 3.12 Rate Limiter

#### `test_rate_limiter_allows_within_limit`
- **Given:** Rate limit of 60 requests per minute for a given agent token
- **When:** 30 requests are made within the window
- **Then:** All 30 requests pass the rate limiter

#### `test_rate_limiter_blocks_at_limit`
- **Given:** Rate limit of 60 requests per minute
- **When:** Exactly 60 requests are made (at limit)
- **Then:** All 60 pass. The 61st is blocked.

#### `test_rate_limiter_blocks_over_limit`
- **Given:** Rate limit of 60 requests per minute
- **When:** 80 requests are made within one minute
- **Then:** The first 60 pass. Requests 61-80 return 429 Too Many Requests.

#### `test_rate_limiter_window_reset`
- **Given:** Rate limit of 60 requests per minute. 60 requests made in the first window.
- **When:** The window resets (simulated clock advance by 60 seconds) and a new request arrives
- **Then:** The new request passes. The counter is reset.

#### `test_rate_limiter_per_agent_isolation`
- **Given:** Two agents with separate tokens. Rate limit is 60/min per agent.
- **When:** Agent A makes 50 requests and Agent B makes 50 requests
- **Then:** All 100 requests pass. Each agent's counter is independent.

---

## 4. Integration Test Scenarios

### Scenario 1: Happy Path -- Agent Lifecycle

**File:** `src-tauri/tests/agent_lifecycle.rs`

1. User generates an invitation code `"INV-test001"` with label "For test agent" and 24-hour expiry.
2. Agent sends `POST /v1/agents/register` with `{ name: "Test Agent", purpose: "Integration test", agent_type: "test", capabilities: ["send"], invitation_code: "INV-test001" }`.
3. Assert response is 201 with `status: "pending"` and an `agent_id`.
4. Agent polls `GET /v1/agents/register/{agent_id}/status`. Assert response is `{ status: "pending" }`.
5. User approves the agent via `approve(agent_id)`. Assert a token is generated and stored in delivery cache.
6. Agent polls status again within 5 minutes. Assert response includes `status: "active"` and a `token` starting with `"anb_"`.
7. Agent polls status a third time. Assert `token: null` (already delivered).
8. Agent sends `POST /v1/send` with `Authorization: Bearer <token>`, `{ to: "0xRecipient", amount: 5.00 }`. Agent has `per_tx_max: 25.00`, `auto_approve_max: 10.00`.
9. Assert response is 202 with `status: "executing"`.
10. Wait for background execution. Agent polls `GET /v1/transactions/{tx_id}`. Assert status is `"confirmed"` with a `chain_tx_hash`.
11. Agent calls `GET /v1/transactions?limit=10`. Assert the transaction appears in the list.

### Scenario 2: Spending Limit Enforcement

**File:** `src-tauri/tests/spending_limits.rs`

1. Set up an agent with `per_tx_max: 10.00`, `daily_cap: 25.00`, `auto_approve_max: 5.00`.
2. Agent sends `POST /v1/send` with `amount: 15.00`. Assert response is 403 (exceeds per_tx_max).
3. Agent sends `POST /v1/send` with `amount: 8.00`. Assert response is 202 with `status: "awaiting_approval"` (above auto_approve, within per_tx).
4. User approves. Assert transaction executes. Daily spending is now 8.00.
5. Agent sends `POST /v1/send` with `amount: 9.00`. Assert 202. Daily spending would be 17.00 (within 25.00 cap).
6. Agent sends `POST /v1/send` with `amount: 9.00`. Assert 403 (17.00 + 9.00 = 26.00, exceeds daily cap of 25.00).
7. Agent sends `POST /v1/send` with `amount: 8.00`. Assert 202 (17.00 + 8.00 = 25.00, exactly at cap -- allowed).

### Scenario 3: Global Policy Enforcement

**File:** `src-tauri/tests/global_policy.rs`

1. Set up two agents (Agent A, Agent B). Global `daily_cap: 50.00`. Each agent has `per_tx_max: 30.00`, `daily_cap: 100.00`.
2. Agent A sends 25.00. Assert 202 (global daily: 25.00).
3. Agent B sends 20.00. Assert 202 (global daily: 45.00).
4. Agent A sends 10.00. Assert 403 (global daily would be 55.00, exceeds 50.00 cap). Agent A's individual limits are not exhausted, but the global cap is.
5. Agent B sends 3.00. Assert 403 (same reason -- global cap hit).

### Scenario 4: Approval Flow

**File:** `src-tauri/tests/approval_flow.rs`

1. Set up agent with `auto_approve_max: 5.00`, `per_tx_max: 50.00`.
2. Agent sends `POST /v1/send` with `amount: 20.00`. Assert 202 with `status: "awaiting_approval"`.
3. Poll transaction status. Assert `status: "awaiting_approval"`.
4. User calls `resolve_approval(approval_id, "approved")`.
5. Wait for background execution. Poll transaction status. Assert `status: "confirmed"`.
6. Agent sends another 20.00. Assert 202 with `status: "awaiting_approval"`.
7. User calls `resolve_approval(approval_id, "denied")`.
8. Poll transaction status. Assert `status: "denied"`.

### Scenario 5: Kill Switch

**File:** `src-tauri/tests/kill_switch.rs`

1. Set up two agents with valid spending limits.
2. Agent A sends 5.00. Assert 202 (succeeds).
3. User activates kill switch: `toggle_kill_switch(true, "Security concern")`.
4. Agent A sends 5.00. Assert 403 with `"kill_switch_active"`.
5. Agent B sends 1.00. Assert 403 with `"kill_switch_active"`.
6. Verify any pending approval requests are NOT auto-executed while kill switch is active.
7. User deactivates kill switch: `toggle_kill_switch(false, "")`.
8. Agent A sends 5.00. Assert 202 (succeeds again).

### Scenario 6: Token Expiry and Re-Registration

**File:** `src-tauri/tests/token_delivery.rs`

1. Agent registers with valid invitation code. User approves.
2. Simulate 6 minutes passing (token delivery expires at 5 minutes).
3. Agent polls `GET /v1/agents/register/{agent_id}/status`. Assert `token: null` and message about expiry.
4. Agent must re-register with a new invitation code to get a new token.

### Scenario 7: CLI Failure Recovery

**File:** `src-tauri/tests/cli_failure_recovery.rs`

1. Set up agent with valid limits. Configure mock CLI to return `Err(CliError::CommandFailed)` for the send command.
2. Agent sends `POST /v1/send` with `amount: 5.00`. Assert 202 (accepted, executing in background).
3. Background execution fails. Poll transaction status. Assert `status: "failed"` with error message.
4. Verify the spending ledger was NOT updated (no money was sent).
5. Reconfigure mock CLI to return success. Agent retries the same send. Assert new transaction executes successfully.

### Scenario 8: Concurrent Transactions

**File:** `src-tauri/tests/concurrent_transactions.rs`

1. Set up two agents. Agent A has `daily_cap: 20.00`. Agent B has `daily_cap: 20.00`. Global `daily_cap: 30.00`.
2. Simultaneously send: Agent A sends 15.00, Agent B sends 15.00 (total would be 30.00 = exactly at global cap, but each within their individual cap).
3. Assert: One transaction succeeds and one is denied (because the `BEGIN EXCLUSIVE` serialization prevents both from passing the global cap check simultaneously). OR both succeed if total is exactly at cap. The key assertion is: no race condition leads to overspending.
4. Verify the global spending ledger shows exactly the sum of successful transactions, never exceeding the cap.

### Scenario 9: Mock Mode

**File:** `src-tauri/tests/mock_mode.rs`

1. Start the application with `ANB_MOCK=true`.
2. Assert health check returns `{ "mock_mode": true }`.
3. Call `GET /v1/balance`. Assert returns a fake balance (not an error).
4. Register an agent (mock mode still requires invitation codes and approval for realistic testing).
5. Send a payment. Assert 202. Mock CLI returns a fake tx_hash.
6. Poll transaction. Assert `status: "confirmed"` with a fake chain_tx_hash.
7. Verify the full spending policy engine still runs (mock mode replaces CLI, not business logic).

---

## 5. CI Pipeline Requirements

### Coverage Thresholds

| Layer | Minimum Coverage | Tool |
|---|---|---|
| Rust (unit + integration) | 80% | `cargo-tarpaulin` or `cargo-llvm-cov` |
| React (components + hooks + stores) | 70% | Vitest with `@vitest/coverage-v8` |

### CI Jobs (GitHub Actions)

The CI pipeline runs the following jobs. All jobs must pass before a PR can be merged.

| Job | Steps | Runs On |
|---|---|---|
| **lint-rust** | `cargo fmt --check`, `cargo clippy -- -D warnings` | ubuntu-latest |
| **test-rust-unit** | `cargo test --lib --bins` (unit tests only) | ubuntu-latest |
| **test-rust-integration** | `cargo test --test '*'` (integration tests in `tests/`) | ubuntu-latest |
| **coverage-rust** | `cargo tarpaulin --out xml`, fail if < 80% | ubuntu-latest |
| **lint-frontend** | `pnpm lint` | ubuntu-latest |
| **test-frontend** | `pnpm test -- --run` (Vitest) | ubuntu-latest |
| **coverage-frontend** | `pnpm test -- --run --coverage`, fail if < 70% | ubuntu-latest |
| **build** | `cargo tauri build` (full binary build) | macos-latest |
| **e2e** | Start app in mock mode, run Playwright tests | macos-latest |

### Branch Protection Rules

- **Required status checks:** All CI jobs above must pass.
- **Require pull request reviews:** At least 1 approval.
- **Require branches to be up to date:** PR must be rebased on main.
- **No direct pushes to main:** All changes via PR.

### Integration Tests in CI

- All integration tests run against the `MockCliExecutor`. No real blockchain operations in CI.
- In-memory SQLite databases are used (`:memory:`) for unit tests.
- Temporary file-based SQLite databases are used for integration tests (cleaned up after each test).
- E2E tests start the full app in mock mode (`ANB_MOCK=true`) and use Playwright to test UI flows.

---

## 6. Test Fixtures and Helpers

### 6.1 Rust Test Fixtures

All shared test fixtures live in `src-tauri/src/test_helpers.rs` (compiled only under `#[cfg(test)]`).

#### Mock CLI Output Strings

```rust
#[cfg(test)]
pub mod fixtures {
    use crate::cli::executor::CliOutput;
    use serde_json::json;

    pub fn mock_balance_output() -> CliOutput {
        CliOutput {
            success: true,
            data: json!({ "balance": "1247.83", "asset": "USDC" }),
            raw: r#"{"balance": "1247.83", "asset": "USDC"}"#.to_string(),
            stderr: String::new(),
        }
    }

    pub fn mock_send_output(tx_hash: &str) -> CliOutput {
        CliOutput {
            success: true,
            data: json!({ "tx_hash": tx_hash }),
            raw: format!(r#"{{"tx_hash": "{}"}}"#, tx_hash),
            stderr: String::new(),
        }
    }

    pub fn mock_auth_status_authenticated() -> CliOutput {
        CliOutput {
            success: true,
            data: json!({ "authenticated": true, "email": "test@example.com" }),
            raw: r#"{"authenticated": true, "email": "test@example.com"}"#.to_string(),
            stderr: String::new(),
        }
    }

    pub fn mock_auth_status_unauthenticated() -> CliOutput {
        CliOutput {
            success: true,
            data: json!({ "authenticated": false }),
            raw: r#"{"authenticated": false}"#.to_string(),
            stderr: String::new(),
        }
    }

    pub fn mock_cli_error_output(error_msg: &str) -> CliOutput {
        CliOutput {
            success: false,
            data: json!({}),
            raw: String::new(),
            stderr: error_msg.to_string(),
        }
    }
}
```

#### Test Agent Data

```rust
#[cfg(test)]
pub mod test_agents {
    use crate::db::models::{Agent, AgentStatus};
    use chrono::Utc;

    pub fn create_test_agent(name: &str, status: AgentStatus) -> Agent {
        Agent {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            description: format!("Test agent: {}", name),
            purpose: "Integration testing".to_string(),
            agent_type: "test".to_string(),
            capabilities: vec!["send".to_string()],
            status,
            api_token_hash: None,
            token_prefix: None,
            balance_visible: true,
            invitation_code: "INV-test".to_string(),
            created_at: Utc::now().timestamp(),
            updated_at: Utc::now().timestamp(),
            last_active_at: None,
            metadata: "{}".to_string(),
        }
    }

    pub fn create_test_agent_with_token(name: &str) -> (Agent, String) {
        let raw_token = format!("anb_test_{}", uuid::Uuid::new_v4().to_string().replace("-", "")[..16].to_string());
        let token_hash = format!("argon2_hash_of_{}", raw_token); // Placeholder in tests
        let mut agent = create_test_agent(name, AgentStatus::Active);
        agent.api_token_hash = Some(token_hash);
        agent.token_prefix = Some(raw_token[..12].to_string());
        (agent, raw_token)
    }
}
```

#### Test Transaction Data

```rust
#[cfg(test)]
pub mod test_transactions {
    use crate::db::models::{Transaction, TxType, TxStatus};
    use chrono::Utc;
    use rust_decimal::Decimal;

    pub fn create_test_tx(agent_id: &str, amount: Decimal, status: TxStatus) -> Transaction {
        let now = Utc::now();
        Transaction {
            id: uuid::Uuid::new_v4().to_string(),
            agent_id: Some(agent_id.to_string()),
            tx_type: TxType::Send,
            amount: amount.to_string(),
            asset: "USDC".to_string(),
            recipient: Some("0xTestRecipient".to_string()),
            sender: None,
            chain_tx_hash: None,
            status,
            category: "test".to_string(),
            memo: "Test transaction".to_string(),
            description: "Test transaction for integration test".to_string(),
            service_name: "Test Service".to_string(),
            service_url: "https://test.example.com".to_string(),
            reason: "Testing".to_string(),
            webhook_url: None,
            error_message: None,
            period_daily: format!("daily:{}", now.format("%Y-%m-%d")),
            period_weekly: format!("weekly:{}", now.format("%G-W%V")),
            period_monthly: format!("monthly:{}", now.format("%Y-%m")),
            created_at: now.timestamp(),
            updated_at: now.timestamp(),
        }
    }
}
```

### 6.2 Rust Test Helper Functions

```rust
#[cfg(test)]
pub mod helpers {
    use crate::db::Database;
    use crate::cli::executor::MockCliExecutor;
    use crate::core::services::CoreServices;
    use std::sync::Arc;

    /// Create an in-memory SQLite database with the full schema applied.
    /// Used for unit tests where speed is critical.
    pub fn setup_test_db() -> Arc<Database> {
        let db = Database::new_in_memory().expect("Failed to create in-memory DB");
        db.run_migrations().expect("Failed to run migrations");
        Arc::new(db)
    }

    /// Create a file-backed SQLite database in a temp directory.
    /// Used for integration tests that need persistence across function calls.
    pub fn setup_test_db_file() -> (Arc<Database>, tempfile::TempDir) {
        let tmp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let db_path = tmp_dir.path().join("test.db");
        let db = Database::new(db_path).expect("Failed to create file DB");
        db.run_migrations().expect("Failed to run migrations");
        (Arc::new(db), tmp_dir) // tmp_dir must be kept alive for the file to exist
    }

    /// Create a MockCliExecutor with default realistic responses
    /// for balance, send, auth status, etc.
    pub fn mock_cli_executor() -> Arc<MockCliExecutor> {
        let mut mock = MockCliExecutor::new();
        mock.set_response("get_balance", super::fixtures::mock_balance_output());
        mock.set_response("send", super::fixtures::mock_send_output("0xfake_tx_hash_123"));
        mock.set_response("auth_status", super::fixtures::mock_auth_status_authenticated());
        Arc::new(mock)
    }

    /// Create a full CoreServices instance with mock CLI and in-memory DB.
    /// Suitable for integration-style tests within a single test function.
    pub async fn setup_test_core_services() -> Arc<CoreServices> {
        let db = setup_test_db();
        let cli = mock_cli_executor();
        let config = crate::config::AppConfig::default_test();
        Arc::new(
            CoreServices::new(db, cli, config)
                .await
                .expect("Failed to create CoreServices")
        )
    }

    /// Create a test spending policy for an agent.
    pub fn create_test_spending_policy(
        agent_id: &str,
        per_tx_max: &str,
        daily_cap: &str,
        weekly_cap: &str,
        monthly_cap: &str,
        auto_approve_max: &str,
    ) -> crate::db::models::SpendingPolicy {
        crate::db::models::SpendingPolicy {
            agent_id: agent_id.to_string(),
            per_tx_max: per_tx_max.to_string(),
            daily_cap: daily_cap.to_string(),
            weekly_cap: weekly_cap.to_string(),
            monthly_cap: monthly_cap.to_string(),
            auto_approve_max: auto_approve_max.to_string(),
            allowlist: vec![],
            updated_at: chrono::Utc::now().timestamp(),
        }
    }

    /// Create a test invitation code.
    pub fn create_test_invitation(code: &str, label: &str) -> crate::db::models::InvitationCode {
        let now = chrono::Utc::now().timestamp();
        crate::db::models::InvitationCode {
            code: code.to_string(),
            created_at: now,
            expires_at: Some(now + 86400), // 24 hours
            used_by: None,
            used_at: None,
            max_uses: 1,
            use_count: 0,
            label: label.to_string(),
        }
    }
}
```

### 6.3 Database Setup and Teardown

**Unit tests (in-memory SQLite):**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::helpers::setup_test_db;

    #[tokio::test]
    async fn test_example() {
        let db = setup_test_db(); // Fresh in-memory DB, auto-dropped at end of test
        // ... test code ...
    } // DB is dropped here, no cleanup needed
}
```

**Integration tests (file-backed SQLite):**
```rust
// tests/agent_lifecycle.rs
use tally_agentic_wallet::test_helpers::helpers::setup_test_db_file;

#[tokio::test]
async fn test_full_agent_lifecycle() {
    let (db, _tmp_dir) = setup_test_db_file(); // File DB in temp directory
    // ... test code ...
} // _tmp_dir dropped here, temp directory and DB file are cleaned up
```

### 6.4 React Test Helpers

Located in `src/test/helpers.ts` and `src/test/setup.ts`.

```typescript
// src/test/setup.ts
import { vi } from 'vitest';

// Mock Tauri invoke globally
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

// Mock Tauri event listeners
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
  emit: vi.fn(),
}));
```

```typescript
// src/test/helpers.ts
import { invoke } from '@tauri-apps/api/core';
import type { Agent, Transaction, SpendingPolicy } from '../types';

export function mockInvoke(responses: Record<string, unknown>) {
  const mockFn = invoke as ReturnType<typeof vi.fn>;
  mockFn.mockImplementation((cmd: string, args?: Record<string, unknown>) => {
    if (cmd in responses) {
      const response = responses[cmd];
      return Promise.resolve(typeof response === 'function' ? response(args) : response);
    }
    return Promise.reject(new Error(`Unexpected invoke: ${cmd}`));
  });
}

export function createMockAgent(overrides: Partial<Agent> = {}): Agent {
  return {
    id: 'agent-001',
    name: 'Test Agent',
    description: 'A test agent',
    purpose: 'Testing',
    agent_type: 'test',
    capabilities: ['send'],
    status: 'active',
    token_prefix: 'anb_test1234',
    balance_visible: true,
    created_at: Date.now() / 1000,
    updated_at: Date.now() / 1000,
    last_active_at: Date.now() / 1000,
    ...overrides,
  };
}

export function createMockTransaction(overrides: Partial<Transaction> = {}): Transaction {
  return {
    id: 'tx-001',
    agent_id: 'agent-001',
    tx_type: 'send',
    amount: '5.00',
    asset: 'USDC',
    recipient: '0xTestRecipient',
    status: 'confirmed',
    category: 'test',
    memo: 'Test transaction',
    created_at: Date.now() / 1000,
    updated_at: Date.now() / 1000,
    ...overrides,
  };
}

export function createMockSpendingPolicy(overrides: Partial<SpendingPolicy> = {}): SpendingPolicy {
  return {
    per_tx_max: '25.00',
    daily_cap: '100.00',
    weekly_cap: '500.00',
    monthly_cap: '1500.00',
    auto_approve_max: '10.00',
    usage: { today: '0.00', this_week: '0.00', this_month: '0.00' },
    allowlist: [],
    ...overrides,
  };
}
```

---

## Appendix: Test Naming Convention Summary

### Rust

Pattern: `test_<module>_<behavior>_<expected_outcome>`

Examples:
- `test_spending_policy_denies_when_daily_cap_exceeded`
- `test_cli_nonzero_exit_code_returns_error`
- `test_auth_token_validation_sha256_cache_hit`

### React

Pattern: `describe('<Component>') > it('<behavior> <expected outcome>')`

Examples:
- `describe('BalanceCard') > it('displays formatted balance when loaded')`
- `describe('AgentCard') > it('shows pending badge for unapproved agents')`
- `describe('SpendingLimitsEditor') > it('calls update callback with new limits on save')`

### Integration Tests

Pattern: `test_<scenario>_<step_description>`

Examples:
- `test_agent_lifecycle_register_approve_send_confirm`
- `test_spending_limits_deny_when_daily_cap_reached`
- `test_kill_switch_blocks_all_agents`
