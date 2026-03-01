# MCP Server Automated Test Plan

**Based on**: Manual E2E test results from 2026-02-28
**Goal**: Automate the 27 test cases validated manually, plus regression tests for the 5 bugs found.

## Test Architecture

### Framework Choice
- **Rust integration tests** in `src-tauri/tests/` — tests run against the actual MCP HTTP server
- **reqwest** for HTTP client (already a dependency)
- **tokio::test** for async test runtime
- **Test DB**: Use a separate SQLite file per test run to avoid conflicts

### Test Fixture Setup
Each test suite needs:
1. Start the MCP HTTP server on a random available port
2. Create a test SQLite database with seed data
3. Mock the awal CLI responses (to avoid real wallet operations)
4. Clean up after each test

### CLI Mocking Strategy
The biggest lesson from manual testing: **all wallet operations go through the awal CLI**. For automated tests:
- Create a mock `awal` binary (shell script or compiled) that returns predictable JSON
- Set `PATH` to prioritize the mock binary
- Test both success and failure CLI responses
- Separate integration tests (with real CLI, gated behind `--features=integration`) from unit tests (mocked CLI)

## Test Suites

### Suite 1: Discovery & Auto-Configuration

```
test_autodiscovery_mcp_json_exists
test_autodiscovery_mcp_json_has_correct_url
test_autodiscovery_claude_md_has_wallet_instructions
test_autodiscovery_claude_md_mentions_register_agent
```

### Suite 2: Session Lifecycle

```
test_initialize_returns_session_id
test_initialize_returns_correct_protocol_version
test_initialize_returns_server_info
test_session_persists_across_requests
test_deleted_session_returns_404
test_invalid_session_id_returns_404
test_sse_stream_sends_keepalive
test_sse_stream_requires_session_id
test_concurrent_sessions_are_independent
```

### Suite 3: Authentication & Authorization

```
test_unauthenticated_sees_only_register_agent
test_authenticated_sees_all_tools
test_invalid_token_falls_back_to_unauthenticated
test_expired_token_falls_back_to_unauthenticated
test_no_origin_header_returns_403
test_wrong_origin_returns_403
test_localhost_origin_accepted
test_127_0_0_1_origin_accepted
```

### Suite 4: Agent Registration

```
test_register_with_valid_invitation_code
test_register_returns_token_and_agent_id
test_register_status_is_pending
test_register_with_invalid_code_fails
test_register_with_expired_code_fails
test_register_with_used_code_fails
test_register_with_missing_fields_fails
test_registered_agent_appears_in_db
test_invitation_code_use_count_incremented
```

### Suite 5: Read-Only Tools (Mocked CLI)

```
test_get_address_returns_wallet_address
test_get_address_without_wallet_auth_returns_friendly_error  # BUG-1 regression
test_check_balance_returns_all_assets
test_check_balance_formats_decimals_correctly
test_get_spending_limits_returns_policy
test_get_spending_limits_without_policy_returns_error
test_get_transactions_returns_empty_list_initially
test_get_transactions_respects_limit_param
test_get_transactions_respects_status_filter
test_get_agent_info_returns_correct_profile
```

### Suite 6: Spending Policy Enforcement

```
test_send_within_per_tx_max_succeeds
test_send_exceeding_per_tx_max_denied
test_send_exceeding_daily_cap_denied
test_send_within_auto_approve_max_auto_approves
test_send_above_auto_approve_requires_approval
test_negative_amount_rejected_at_validation  # BUG-2 regression
test_zero_amount_rejected_at_validation
test_invalid_address_rejected_at_validation  # BUG-3 regression
test_missing_amount_returns_32602
test_missing_to_returns_32602
test_policy_accumulates_daily_spending
test_policy_resets_daily_at_midnight
```

### Suite 7: Financial Operations (Mocked CLI)

```
test_send_payment_creates_transaction_record
test_send_payment_reserves_budget_before_execution
test_send_payment_confirms_after_cli_success  # BUG-4 regression
test_send_payment_rolls_back_on_cli_failure
test_send_payment_returns_tx_hash_on_success  # BUG-4 regression
test_trade_tokens_creates_transaction_record
test_trade_tokens_subject_to_spending_policy
test_trade_tokens_records_from_and_to_assets
```

### Suite 8: x402 Services

```
test_list_x402_services_returns_bazaar
test_search_x402_services_filters_by_query  # BUG-5 regression
test_search_x402_services_empty_query_returns_all
test_get_x402_details_returns_payment_info
test_get_x402_details_shows_amount_and_network
test_pay_x402_subject_to_spending_policy
test_pay_x402_creates_transaction_record
test_pay_x402_passes_custom_headers
test_pay_x402_passes_request_body
```

### Suite 9: Limit Increase Requests

```
test_request_limit_increase_creates_approval_request
test_request_limit_increase_returns_request_id
test_request_limit_increase_status_is_pending
test_approval_request_appears_in_db
test_approved_request_updates_spending_policy
test_rejected_request_does_not_change_policy
```

### Suite 10: Error Handling

```
test_unknown_tool_returns_method_not_found
test_malformed_json_returns_parse_error
test_missing_jsonrpc_version_returns_error
test_wrong_method_returns_method_not_found
test_internal_cli_errors_sanitized  # BUG-1 regression
test_rate_limiting_returns_429
```

## Bug-Specific Regression Tests

### BUG-1: CLI internals in error messages
```rust
#[test]
async fn test_cli_error_does_not_leak_npx_commands() {
    // Mock CLI to return auth error
    // Verify error message does NOT contain "npx", "awal", "auth login"
    // Verify error message IS agent-friendly
}
```

### BUG-2: Negative amount validation
```rust
#[test]
async fn test_negative_amount_rejected_before_cli_call() {
    // Send amount: "-1.00"
    // Verify: error code -32602 (Invalid input)
    // Verify: no transaction record created
    // Verify: no spending reservation consumed
}
```

### BUG-3: Address validation
```rust
#[test]
async fn test_invalid_eth_address_rejected_before_cli_call() {
    // Send to: "not-an-address"
    // Verify: error code -32602 (Invalid input)
    // Verify: error message mentions address format
    // Also test: too short, too long, missing 0x prefix, non-hex chars
}
```

### BUG-4: Transaction status confirmation
```rust
#[test]
async fn test_successful_send_transitions_to_completed() {
    // Mock CLI to return success with tx hash
    // Verify: transaction status is "completed" (not "pending")
    // Verify: chain_tx_hash is populated
}

#[test]
async fn test_successful_trade_transitions_to_completed() {
    // Same for trade_tokens
}
```

### BUG-5: x402 search filtering
```rust
#[test]
async fn test_search_x402_filters_results() {
    // Search for "podcast"
    // Verify: results only contain services matching "podcast"
    // Verify: results != full listing
}
```

### BUG-6: x402 response body not returned
```rust
#[test]
async fn test_pay_x402_returns_service_response_body() {
    // Mock CLI to return successful x402 payment with response body
    // Verify: MCP tool response includes the service's HTTP response body
    // Verify: agent can access the content it paid for
}

#[test]
async fn test_pay_x402_returns_response_headers() {
    // Verify: response includes relevant headers (content-type, etc.)
}
```

## Implementation Priority

1. **P0 — Bug regressions**: BUG-1 through BUG-5 (prevents shipping these bugs again)
2. **P1 — Auth & security**: Suite 3 (CSRF, token validation)
3. **P1 — Policy enforcement**: Suite 6 (spending limits are the core safety feature)
4. **P2 — Session lifecycle**: Suite 2 (protocol correctness)
5. **P2 — Registration flow**: Suite 4 (agent onboarding)
6. **P3 — Financial ops**: Suites 7, 8 (mocked CLI tests)
7. **P3 — Discovery**: Suite 1 (static file checks)

## CI Integration

```yaml
# .github/workflows/mcp-tests.yml
test-mcp-unit:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable
    - name: Run MCP unit tests (mocked CLI)
      run: cargo test --package tally-agentic-wallet --test mcp_tests

test-mcp-integration:
  runs-on: ubuntu-latest
  if: github.event_name == 'push' && github.ref == 'refs/heads/main'
  steps:
    - uses: actions/checkout@v4
    - name: Install Node + awal
      run: npm ci
    - name: Run MCP integration tests (real CLI, testnet)
      run: cargo test --package tally-agentic-wallet --test mcp_integration --features=integration
      env:
        AWAL_TESTNET: "true"
```

## Estimated Effort

| Suite | Tests | Est. Hours |
|-------|-------|------------|
| Bug regressions (P0) | 7 | 3h |
| Auth & security (P1) | 8 | 3h |
| Policy enforcement (P1) | 12 | 4h |
| Session lifecycle (P2) | 9 | 3h |
| Registration flow (P2) | 9 | 3h |
| Financial ops (P3) | 8 | 3h |
| x402 services (P3) | 9 | 3h |
| Discovery (P3) | 4 | 1h |
| Error handling | 6 | 2h |
| Limit increase | 6 | 2h |
| **Total** | **78** | **~27h** |
