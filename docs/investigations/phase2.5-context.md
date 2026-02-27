# Phase 2.5 Context Summary

> Generated: 2026-02-27
> Source docs: `docs/architecture/architecture-plan.md` (v2.0), `docs/architecture/testing-specification.md` (v1.0)

---

## 1. Architecture Overview

### High-Level Architecture

Agent Neo Bank is a **Tauri v2 desktop app** with a Rust backend and React frontend (Vite + TypeScript + Tailwind v4 + shadcn/ui). It provides a banking interface for AI agents to send USDC payments under user-controlled spending policies.

**Core design principles:**
- Single source of truth: the Core Service Layer handles all business logic. Three transports (Tauri IPC, REST API, MCP) delegate to it.
- CLI as execution layer: the `awal` (Coinbase Agent Wallet) CLI is the sole interface for wallet operations. Never calls Coinbase APIs directly.
- Policy-first transactions: every outgoing tx passes through Global Policy then per-agent Spending Policy before CLI execution.
- Local-first: all data lives in a local SQLite database. User owns their data.
- Agent-agnostic: any process speaking HTTP, MCP, or Unix sockets can be an agent.
- Async transactions: `/v1/send` always returns 202 Accepted immediately; agents poll for status or provide a webhook callback URL.

### Key Modules (Rust Backend -- `src-tauri/src/`)

| Module | Purpose |
|---|---|
| `core/services.rs` | `CoreServices` struct (Arc, no Mutex). Holds all sub-services. All methods take `&self`. |
| `core/spending_policy.rs` | Per-agent spending policy engine. `BEGIN EXCLUSIVE` SQLite transactions to prevent race conditions. Evaluates per-tx max, daily/weekly/monthly caps, allowlist, auto-approve threshold. |
| `core/global_policy.rs` | Wallet-level controls above all agent policies: global daily/weekly/monthly caps, minimum reserve balance, kill switch. |
| `core/tx_processor.rs` | Transaction lifecycle orchestrator. Policy check -> approval (if needed) -> async CLI execution -> atomic ledger update. Webhook callbacks on completion/failure. |
| `core/agent_registry.rs` | Agent lifecycle: registration (with invitation code), approval, token issuance (encrypted 5-min delivery cache), suspension. |
| `core/auth_service.rs` | OTP auth via CLI. Two-tier token validation: SHA-256 in-memory cache (5-min TTL) for fast path, argon2 verify for slow path. |
| `core/wallet_service.rs` | Balance lookups with 30-second TTL cache. Per-agent balance visibility control. |
| `core/approval_manager.rs` | Approval queue: create, resolve (approve/deny), stale cleanup (every 5 min, 24-hour expiry). |
| `core/event_bus.rs` | Internal pub/sub. Emits Tauri events to frontend (TransactionConfirmed, ApprovalCreated, AgentRegistered, etc.). |
| `core/notification.rs` | OS notification dispatch via `tauri-plugin-notification`. Respects user preferences. |
| `core/invitation.rs` | Invitation code generation/validation. Format: `INV-[a-z0-9]{8}`. Single-use with optional expiry. |
| `cli/executor.rs` | `CliExecutable` trait with `RealCliExecutor` and `MockCliExecutor`. Async via `tokio::process::Command`. |
| `cli/parser.rs` | Parses CLI stdout JSON into `CliOutput` struct. |
| `cli/commands.rs` | Typed `AwalCommand` enum with `to_args()` method. |
| `api/rest_server.rs` | Axum HTTP server on port 7402. Shares `Arc<CoreServices>`. |
| `api/rest_routes.rs` | Route definitions for all REST endpoints. |
| `api/auth_middleware.rs` | Bearer token validation middleware with SHA-256 cache. |
| `api/rate_limiter.rs` | Invitation-code-based rate limiting for registration, token bucket for API. |
| `api/mcp_server.rs` | Per-agent MCP server (stdio/SSE). Token validated on startup, all ops scoped to bound agent. |
| `api/mcp_tools.rs` | MCP tool definitions: send_payment, check_balance, get_spending_limits, request_limit_increase, get_transaction_status, list_my_transactions. |
| `db/schema.rs` | Full SQLite schema: agents, spending_policies, global_policy, transactions, approval_requests, invitation_codes, token_delivery, notification_preferences, spending_ledger, global_spending_ledger. |
| `db/queries.rs` | Typed query functions with `spawn_blocking` for rusqlite. |
| `db/models.rs` | Rust structs for DB rows. |
| `state/app_state.rs` | Shared state struct. |
| `error.rs` | Unified `AppError` type. |
| `config.rs` | App configuration (mock mode, ports, defaults). |

### Data Flow

1. **Agent sends payment** -> REST API (`POST /v1/send`) or MCP (`send_payment` tool)
2. **Auth middleware** validates bearer token (SHA-256 cache -> argon2 fallback)
3. **Transaction record** created in pending state with UTC period keys
4. **Spending Policy Engine** runs inside `BEGIN EXCLUSIVE` SQLite transaction:
   - Check global kill switch
   - Check global minimum reserve balance
   - Check global daily/weekly/monthly caps
   - Check per-agent per-tx max, daily/weekly/monthly caps
   - Check allowlist
   - Determine: AutoApproved / RequiresApproval / Denied
5. **If auto-approved**: background task spawned for CLI execution. Returns 202.
6. **If requires approval**: approval request created, OS notification sent, returns 202 with status "awaiting_approval".
7. **If denied**: returns 403 with reason.
8. **CLI execution** (background): `awal send` command. On success: atomic confirm + ledger update (both agent and global). On failure: mark tx failed, ledger NOT updated.
9. **Webhook callback** (if provided): POST to agent's webhook_url with tx status.
10. **Event bus** emits Tauri event -> frontend updates in real-time via listeners.

### SQLite Tables

- `agents` -- rich metadata: name, description, purpose, agent_type, capabilities, status, token hash, balance_visible, invitation_code
- `spending_policies` -- per-agent: per_tx_max, daily/weekly/monthly caps, auto_approve_max, allowlist
- `global_policy` -- wallet-level: daily/weekly/monthly caps, min_reserve_balance, kill_switch_active/reason
- `transactions` -- full metadata: amount (Decimal string), recipient, chain_tx_hash, status, description, service_name, service_url, reason, webhook_url, period keys
- `approval_requests` -- type (transaction/limit_increase/registration), payload, status, expires_at
- `invitation_codes` -- code, expires_at, max_uses, use_count, label
- `token_delivery` -- encrypted_token, expires_at (5 min), delivered flag
- `spending_ledger` -- (agent_id, period) -> total, tx_count. Period keys: `daily:YYYY-MM-DD`, `weekly:YYYY-WNN`, `monthly:YYYY-MM`
- `global_spending_ledger` -- same period format, aggregate across all agents
- `notification_preferences` -- toggles for each notification type

### React Frontend (`src/`)

- **Pages**: Onboarding, Dashboard, Agents (list), AgentDetail, Transactions, Approvals, Settings, Fund
- **State**: Zustand stores (authStore, agentStore, transactionStore, settingsStore, approvalStore)
- **Real-time**: Tauri event listeners in hooks update Zustand stores
- **UI**: shadcn/ui components (Button, Card, Table, Badge, Dialog, etc.)

### Implementation Phases

| Phase | Goal | Status |
|---|---|---|
| **1a: Plumbing** | Scaffold, DB, CLI wrapper, auth, mock mode, basic shell | Implemented |
| **1b: Agent Operations** | Agent CRUD, spending policy, REST API, rate limiting, tx history | Implemented |
| **2: Multi-Agent & Controls** | Approval flow, MCP server, global policy UI, notifications, event bus | Implemented (Phase 2 waves 1-4 committed) |
| **3: Receive, Earn, Onramp** | Incoming tx detection, Unix socket, Coinbase Onramp, tx export | Not started |
| **4: Polish & Production** | Analytics, dark mode, auto-updater, error recovery, token rotation | Not started |

---

## 2. Integration Test Scenarios (Testing Spec Section 4) -- Detailed Steps

### Scenario 1: Happy Path -- Agent Lifecycle
**File:** `src-tauri/tests/agent_lifecycle.rs`

1. User generates invitation code `"INV-test001"` with label "For test agent" and 24-hour expiry.
2. Agent sends `POST /v1/agents/register` with `{ name: "Test Agent", purpose: "Integration test", agent_type: "test", capabilities: ["send"], invitation_code: "INV-test001" }`.
3. Assert response is 201 with `status: "pending"` and an `agent_id`.
4. Agent polls `GET /v1/agents/register/{agent_id}/status`. Assert `{ status: "pending" }`.
5. User approves the agent via `approve(agent_id)`. Assert token generated and stored in delivery cache.
6. Agent polls status again within 5 minutes. Assert `status: "active"` and `token` starts with `"anb_"`.
7. Agent polls status a third time. Assert `token: null` (already delivered).
8. Agent sends `POST /v1/send` with `Authorization: Bearer <token>`, `{ to: "0xRecipient", amount: 5.00 }`. Agent has `per_tx_max: 25.00`, `auto_approve_max: 10.00`.
9. Assert response is 202 with `status: "executing"`.
10. Wait for background execution. Agent polls `GET /v1/transactions/{tx_id}`. Assert `status: "confirmed"` with `chain_tx_hash`.
11. Agent calls `GET /v1/transactions?limit=10`. Assert transaction appears in list.

### Scenario 2: Spending Limit Enforcement
**File:** `src-tauri/tests/spending_limits.rs`

1. Set up agent with `per_tx_max: 10.00`, `daily_cap: 25.00`, `auto_approve_max: 5.00`.
2. Agent sends 15.00. Assert 403 (exceeds per_tx_max).
3. Agent sends 8.00. Assert 202 with `status: "awaiting_approval"` (above auto_approve, within per_tx).
4. User approves. Assert tx executes. Daily spending = 8.00.
5. Agent sends 9.00. Assert 202. Daily spending would be 17.00 (within 25.00 cap).
6. Agent sends 9.00. Assert 403 (17.00 + 9.00 = 26.00, exceeds daily cap 25.00).
7. Agent sends 8.00. Assert 202 (17.00 + 8.00 = 25.00, exactly at cap -- allowed).

### Scenario 3: Global Policy Enforcement
**File:** `src-tauri/tests/global_policy.rs`

1. Set up two agents (A, B). Global `daily_cap: 50.00`. Each agent: `per_tx_max: 30.00`, `daily_cap: 100.00`.
2. Agent A sends 25.00. Assert 202 (global daily: 25.00).
3. Agent B sends 20.00. Assert 202 (global daily: 45.00).
4. Agent A sends 10.00. Assert 403 (global daily would be 55.00, exceeds 50.00). Individual limits not exhausted, but global cap hit.
5. Agent B sends 3.00. Assert 403 (same reason).

### Scenario 4: Approval Flow
**File:** `src-tauri/tests/approval_flow.rs`

1. Set up agent with `auto_approve_max: 5.00`, `per_tx_max: 50.00`.
2. Agent sends 20.00. Assert 202 with `status: "awaiting_approval"`.
3. Poll tx status. Assert `status: "awaiting_approval"`.
4. User calls `resolve_approval(approval_id, "approved")`.
5. Wait for background execution. Poll tx status. Assert `status: "confirmed"`.
6. Agent sends another 20.00. Assert 202 with `status: "awaiting_approval"`.
7. User calls `resolve_approval(approval_id, "denied")`.
8. Poll tx status. Assert `status: "denied"`.

### Scenario 5: Kill Switch
**File:** `src-tauri/tests/kill_switch.rs`

1. Set up two agents with valid spending limits.
2. Agent A sends 5.00. Assert 202 (succeeds).
3. User activates kill switch: `toggle_kill_switch(true, "Security concern")`.
4. Agent A sends 5.00. Assert 403 with `"kill_switch_active"`.
5. Agent B sends 1.00. Assert 403 with `"kill_switch_active"`.
6. Verify pending approval requests are NOT auto-executed while kill switch active.
7. User deactivates kill switch: `toggle_kill_switch(false, "")`.
8. Agent A sends 5.00. Assert 202 (succeeds again).

### Scenario 6: Token Expiry and Re-Registration
**File:** `src-tauri/tests/token_delivery.rs`

1. Agent registers with valid invitation code. User approves.
2. Simulate 6 minutes passing (token delivery expires at 5 minutes).
3. Agent polls status. Assert `token: null` and message about expiry.
4. Agent must re-register with a new invitation code.

### Scenario 7: CLI Failure Recovery
**File:** `src-tauri/tests/cli_failure_recovery.rs`

1. Set up agent. Configure mock CLI to return `Err(CliError::CommandFailed)` for send.
2. Agent sends 5.00. Assert 202 (accepted, executing in background).
3. Background execution fails. Poll tx status. Assert `status: "failed"` with error message.
4. Verify spending ledger NOT updated (no money sent).
5. Reconfigure mock CLI to return success. Agent retries. Assert new tx executes successfully.

### Scenario 8: Concurrent Transactions
**File:** `src-tauri/tests/concurrent_transactions.rs`

1. Set up two agents. Agent A: `daily_cap: 20.00`. Agent B: `daily_cap: 20.00`. Global `daily_cap: 30.00`.
2. Simultaneously send: Agent A sends 15.00, Agent B sends 15.00 (total 30.00 = exactly at global cap).
3. Assert: `BEGIN EXCLUSIVE` serialization prevents both from passing the global cap check simultaneously. One succeeds, one denied (or both succeed if total exactly at cap).
4. Verify global spending ledger never exceeds the cap.

### Scenario 9: Mock Mode
**File:** `src-tauri/tests/mock_mode.rs`

1. Start app with `ANB_MOCK=true`.
2. Assert health check returns `{ "mock_mode": true }`.
3. Call `GET /v1/balance`. Assert returns fake balance (not error).
4. Register agent (mock mode still requires invitation codes + approval).
5. Send payment. Assert 202. Mock CLI returns fake tx_hash.
6. Poll transaction. Assert `status: "confirmed"` with fake chain_tx_hash.
7. Verify full spending policy engine still runs (mock replaces CLI, not business logic).

---

## 3. CI Pipeline (Testing Spec Section 5) -- Detailed

### Coverage Thresholds

| Layer | Minimum | Tool |
|---|---|---|
| Rust (unit + integration) | 80% | `cargo-tarpaulin` or `cargo-llvm-cov` |
| React (components + hooks + stores) | 70% | Vitest with `@vitest/coverage-v8` |

### CI Jobs (GitHub Actions)

| Job | Steps | Runner |
|---|---|---|
| **lint-rust** | `cargo fmt --check`, `cargo clippy -- -D warnings` | ubuntu-latest |
| **test-rust-unit** | `cargo test --lib --bins` (unit tests only) | ubuntu-latest |
| **test-rust-integration** | `cargo test --test '*'` (integration tests in `tests/`) | ubuntu-latest |
| **coverage-rust** | `cargo tarpaulin --out xml`, fail if < 80% | ubuntu-latest |
| **lint-frontend** | `pnpm lint` | ubuntu-latest |
| **test-frontend** | `pnpm test -- --run` (Vitest) | ubuntu-latest |
| **coverage-frontend** | `pnpm test -- --run --coverage`, fail if < 70% | ubuntu-latest |
| **build** | `cargo tauri build` (full binary) | macos-latest |
| **e2e** | Start app in mock mode, run Playwright tests | macos-latest |

### Branch Protection Rules

- All CI jobs must pass before PR merge
- At least 1 PR review approval required
- PR must be rebased on main (up to date)
- No direct pushes to main

### Integration Tests in CI

- All integration tests use `MockCliExecutor` (no real blockchain ops)
- In-memory SQLite (`:memory:`) for unit tests
- Temporary file-based SQLite for integration tests (cleaned up after each test)
- E2E tests start full app in mock mode (`ANB_MOCK=true`) and use Playwright

---

## 4. Current Project Structure -- What Exists

### Rust Backend (`src-tauri/src/`)

**Implemented modules:**
- `main.rs`, `lib.rs` -- entry points
- `config.rs` -- app configuration
- `error.rs` -- error types
- `test_helpers.rs` -- test fixtures and helpers
- `state/app_state.rs` -- shared state
- `core/` -- services.rs, agent_registry.rs, approval_manager.rs, auth_service.rs, event_bus.rs, global_policy.rs, invitation.rs, notification.rs, spending_policy.rs, tx_processor.rs, wallet_service.rs
- `commands/` -- agents.rs, approvals.rs, auth.rs, budget.rs, invitation_codes.rs, notifications.rs, settings.rs, transactions.rs, wallet.rs
- `api/` -- auth_middleware.rs, mcp_server.rs, mcp_tools.rs, rate_limiter.rs, rest_routes.rs, rest_server.rs
- `cli/` -- commands.rs, executor.rs, parser.rs
- `db/` -- schema.rs, models.rs, queries.rs, migrations/001_initial.sql

### Integration Tests (`src-tauri/tests/`)

**Existing test files:**
- `agent_lifecycle.rs` -- Scenario 1: full agent lifecycle
- `spending_limits.rs` -- Scenario 2: spending limit enforcement
- `global_policy.rs` -- Scenario 3: global policy enforcement
- `approval_flow.rs` -- Scenario 4: approval flow
- `kill_switch.rs` -- Scenario 5: kill switch
- `kill_switch_integration.rs` -- additional kill switch tests
- `limit_increase.rs` -- limit increase request flow
- `mcp_integration.rs` -- MCP server integration
- `mock_mode.rs` -- Scenario 9: mock mode
- `common/mod.rs` -- shared test utilities

**Not yet implemented (from spec):**
- `token_delivery.rs` -- Scenario 6
- `cli_failure_recovery.rs` -- Scenario 7
- `concurrent_transactions.rs` -- Scenario 8
- `rest_api_contracts.rs` -- REST API contract tests
- `mcp_protocol.rs` -- MCP protocol tests
- `api/` subdirectory -- per-endpoint contract tests

### React Frontend (`src/`)

**Existing:**
- `App.tsx`, `main.tsx` -- entry
- `pages/` -- Dashboard, Agents, AgentDetail, Approvals, Onboarding, Transactions, Settings (with GlobalPolicy, InvitationCodes, Notifications sub-pages)
- `components/layout/` -- Header, Shell, Sidebar (with tests)
- `components/dashboard/` -- AgentBudgets, BudgetProgress, GlobalBudget
- `components/onboarding/` -- EmailStep, OtpStep, FundStep, WelcomeStep (with tests)
- `components/shared/` -- CurrencyDisplay, EmptyState, StatusBadge
- `components/ui/` -- badge, button, card, dialog, input, sonner, table
- `hooks/` -- useBalance, useInvoke, useTauriEvent
- `stores/` -- authStore, settingsStore
- `lib/` -- constants, format, tauri, utils
- `test/` -- helpers, render, setup
- `types/index.ts`

**Not yet implemented (from spec):**
- `pages/Fund.tsx` -- deposit/onramp page
- `components/agents/` -- AgentCard, AgentForm, SpendingLimitsEditor, AgentActivityFeed, AllowlistEditor
- `components/transactions/` -- TransactionTable, TransactionRow, TransactionDetail, FilterBar
- `components/approvals/` -- ApprovalCard, ApprovalQueue
- `stores/` -- agentStore, transactionStore, approvalStore
- `hooks/` -- useAgents, useTransactions, useApprovals
- Many component test files listed in the testing spec

---

## 5. Key Technical Decisions

- **No Mutex on CoreServices**: `Arc<CoreServices>` stored directly in Tauri state. Sub-services use interior mutability only where needed (r2d2 pool, RwLock on caches).
- **rusqlite + spawn_blocking**: Synchronous SQLite wrapped in `tokio::task::spawn_blocking()` for async compatibility.
- **BEGIN EXCLUSIVE**: All spending-limit checks and ledger updates use exclusive SQLite transactions to prevent concurrent race conditions.
- **Decimal amounts**: `rust_decimal::Decimal` for all money amounts. Parsed at API boundary via serde.
- **Period keys at creation time**: Transaction period keys (daily/weekly/monthly) stamped at creation, not completion.
- **Two-tier auth**: argon2 for storage, SHA-256 cache (5-min TTL) for per-request validation speed.
- **Token delivery**: Encrypted cache with 5-min expiry, returned exactly once then deleted.
- **Invitation codes**: Replace IP-based rate limiting (meaningless on localhost). User generates codes in UI.
