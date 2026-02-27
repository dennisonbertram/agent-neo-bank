## 3. Test Cases by Component

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
