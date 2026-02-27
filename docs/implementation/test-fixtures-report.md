# Test Fixtures and Helpers - Implementation Report

> **Date:** 2026-02-27
> **Task:** Phase 1a Task #2 - Create test fixtures and helpers

## Files Created

### 1. Rust Test Helpers: `src-tauri/src/test_helpers.rs`

Module gated with `#[cfg(test)]` containing:

**Stub types** (to be replaced by `db::models` imports later):
- `CliOutput` - CLI command result struct
- `AgentStatus` enum (Pending, Active, Suspended, Revoked) with Display impl
- `TxStatus` enum (Pending, Approved, Executing, Confirmed, Failed, Denied) with Display impl
- `TxType` enum (Send, Receive, Earn) with Display impl
- `Agent` struct - all 15 fields matching DB schema
- `Transaction` struct - all 22 fields matching DB schema
- `SpendingPolicy` struct - all 8 fields matching DB schema
- `InvitationCode` struct - all 8 fields matching DB schema

**CLI output fixtures:**
- `mock_balance_output()` - returns `{ balance: "1247.83", asset: "USDC" }`
- `mock_send_output(tx_hash)` - returns `{ tx_hash: "<hash>" }`
- `mock_auth_status_authenticated()` - returns `{ authenticated: true, email: "test@example.com" }`
- `mock_auth_status_unauthenticated()` - returns `{ authenticated: false }`
- `mock_cli_error_output(error_msg)` - returns failed output with stderr

**Factory functions:**
- `create_test_agent(name, status)` - UUID-based agent with defaults
- `create_test_agent_with_token(name)` - active agent + raw token string
- `create_test_tx(agent_id, amount, status)` - transaction with auto-computed period keys
- `create_test_spending_policy(agent_id, per_tx_max, daily_cap, weekly_cap, monthly_cap, auto_approve_max)`
- `create_test_invitation(code, label)` - invitation with 24h expiry

**Unit tests for factories:**
- `test_create_test_agent_has_valid_uuid`
- `test_create_test_agent_with_token_returns_token`
- `test_mock_balance_output_is_valid_json`
- `test_create_test_tx_has_correct_periods`

### 2. React Test Setup: `src/test/setup.ts`

Mocks for Tauri API bindings:
- `@tauri-apps/api/core` - mocked `invoke`
- `@tauri-apps/api/event` - mocked `listen` and `emit`

### 3. React Test Helpers: `src/test/helpers.ts`

- `mockInvoke(responses)` - configure mocked invoke to return responses by command name
- `createMockAgent(overrides?)` - Agent factory with sensible defaults
- `createMockTransaction(overrides?)` - Transaction factory with auto-computed period keys
- `createMockSpendingPolicy(overrides?)` - SpendingPolicy factory with defaults

### 4. TypeScript Types: `src/types/index.ts`

All shared types matching the Rust DB models:
- `Agent`, `AgentStatus`
- `Transaction`, `TxType`, `TxStatus`
- `SpendingPolicy`
- `GlobalPolicy`
- `InvitationCode`
- `ApprovalRequest`, `ApprovalRequestType`, `ApprovalRequestStatus`
- `NotificationPreferences`

## Notes

- Rust types are stubs defined inline within the `#[cfg(test)]` module. They will be replaced by real imports from `db::models` once the database layer is implemented.
- The `create_test_tx` function takes `amount` as `&str` (not `Decimal`) since the DB stores amounts as text strings. This avoids a dependency on `rust_decimal` in the test helpers.
- All timestamp fields use Unix epoch seconds (i64 in Rust, number in TypeScript).
- Period keys follow the format: `daily:YYYY-MM-DD`, `weekly:YYYY-WNN`, `monthly:YYYY-MM`.
