# Phase 3: Receive, Earn, Onramp, Chain Monitoring

```
Implement Phase 3 (Receive, Earn, Onramp, Chain Monitoring) of Agent Neo Bank using the swarm skill.

## Context
- Architecture plan: docs/architecture/architecture-plan.md (Section 12, Phase 3)
- Testing spec: docs/architecture/testing-specification.md
- Phases 1a, 1b, 2 complete. Phase 2.5 (E2E testing & hardening) complete. TOCTOU bug fixed.
- Full agent lifecycle works: registration, approval, spending policy, transactions, MCP, approvals queue, notifications, dashboard. All 9 integration scenarios passing. CI pipeline enforcing coverage thresholds.

## Important: Write Phase 3 Test Cases First
The testing spec does NOT yet have detailed test cases for Phase 3 components. The FIRST task in Wave 1 is writing the test specification (new sections 3.13-3.18 in testing-specification.md) BEFORE any implementation begins. This is non-negotiable TDD.

## Phase 3 Tasks

### Wave 1 (parallelize — write tests + independent backend work)

1. **Write Phase 3 test cases** (`docs/architecture/testing-specification.md`)
   - Add Section 3.13: Transaction Monitor Service tests (WebSocket connection, incoming tx detection, reconnection, stale data)
   - Add Section 3.14: Receive transaction tracking tests (attribution to agent, global balance credit, duplicate detection)
   - Add Section 3.15: x402 earn tracking tests (agent reports earnings, ledger update, category tagging)
   - Add Section 3.16: Unix socket server tests (same as REST API contract tests but over socket)
   - Add Section 3.17: Transaction export tests (CSV format, filters, date range, empty export)
   - Add Section 3.18: Network toggle tests (sepolia ↔ mainnet switch, confirmation required, config persistence)
   - Add integration scenarios 10-12 to Section 4

2. **Transaction Monitor Service — cloud backend** (separate repo or `services/tx-monitor/`)
   - Lightweight Node.js/TypeScript service (or Rust)
   - Polls Alchemy for incoming transactions to registered wallet addresses
   - WebSocket server pushes notifications to connected desktop apps
   - Endpoints: POST /register-address, DELETE /unregister-address, WebSocket /ws/events
   - NO Alchemy API key in the desktop app — the monitor service holds it
   - For dev: can run locally. For prod: deployed to Railway or similar
   - Tests: polling logic, WebSocket push, address registration, reconnection

3. **Unix domain socket server** (`src-tauri/src/api/unix_socket.rs`)
   - Same protocol as REST API, over Unix socket at /tmp/agent-neo-bank.sock
   - Socket permissions 0600 (owner only)
   - Same auth middleware (bearer token)
   - Tests: Section 3.16 — mirror all REST API contract tests over socket transport

### Wave 2 (depends on Wave 1 test cases being written)

4. **Receive transaction tracking** (`src-tauri/src/core/tx_processor.rs`)
   - WebSocket client connects to Transaction Monitor Service
   - On incoming tx event: create transaction record with type "receive", attribute to wallet
   - Credit to global balance (update cached balance)
   - Emit event for UI update + OS notification
   - Duplicate detection (same chain tx hash = skip)
   - Tests: Section 3.14

5. **x402 earn tracking** (`src-tauri/src/core/tx_processor.rs`, new API endpoint)
   - New endpoint: POST /v1/earn — agent reports earnings from x402 services
   - Request: { amount, service_name, service_url, description, chain_tx_hash }
   - Validates chain_tx_hash against monitor service (or accepts on trust for v1)
   - Creates transaction record with type "earn", attributed to reporting agent
   - Tests: Section 3.15

6. **Network toggle** (`src-tauri/src/core/config.rs`, `src/pages/Settings/Network.tsx`)
   - Setting to switch between Base Sepolia and Base mainnet
   - Confirmation dialog with warning ("You are switching to mainnet — real funds")
   - Persisted to app_config table
   - CLI commands use the configured network
   - Tests: Section 3.18

### Wave 3 (depends on Wave 2)

7. **Coinbase Onramp integration** (`src/pages/Fund/Onramp.tsx`)
   - Embed Coinbase Onramp widget (iframe/webview)
   - Pre-fill wallet address from CLI
   - User buys USDC with card/bank → funds arrive in wallet
   - On completion: trigger balance refresh
   - Tests: widget renders, address pre-filled, completion callback

8. **Manual deposit flow** (`src/pages/Fund/ManualDeposit.tsx`)
   - Display wallet address (from `awal wallet address`)
   - Copy-to-clipboard button
   - QR code generation (address encoded)
   - "I've sent funds" button triggers balance refresh
   - Tests: address displays, copy works, QR renders

9. **Spending breakdown charts** (`src/components/dashboard/SpendingBreakdown.tsx`)
   - By agent: bar chart showing each agent's spending
   - By category: pie chart (APIs, hosting, tools, etc.)
   - By service: top services by spend
   - By time: daily trend line chart
   - Time range selector (7d, 30d, 90d, all)
   - Data from transaction history with aggregation queries
   - Tests: renders with data, handles empty state, time range filter works

10. **Transaction export** (`src-tauri/src/commands/transactions.rs`, `src/pages/Transactions.tsx`)
    - Export button in transaction history UI
    - CSV format: date, agent, type, amount, recipient, status, description, category, chain_tx_hash
    - Filters applied before export (date range, agent, status, type)
    - File save dialog via Tauri
    - Tests: Section 3.17

11. **Transaction search** (`src/pages/Transactions.tsx`)
    - Full-text search input in transaction history
    - Searches across: recipient address, description, service_name, agent name, chain_tx_hash
    - Debounced input, results update in real-time
    - Backend: SQL LIKE query or FTS5 if needed
    - Tests: search returns matches, no results state, debounce works

### Wave 4 (depends on Wave 3)

12. **Integration tests for Phase 3**
    - Scenario 10: Receive flow — monitor detects incoming tx → desktop app notified via WebSocket → transaction record created → balance updated → appears in history
    - Scenario 11: Earn flow — agent calls POST /v1/earn → transaction created → attributed to agent → appears in dashboard breakdown
    - Scenario 12: Full funding flow — user deposits via manual flow → monitor detects → balance updates → agent can now spend the new funds
    - Unix socket: run happy path scenario over socket transport
    - Network toggle: switch networks, verify CLI commands use correct network

13. **Update CI pipeline**
    - Add Transaction Monitor Service tests to CI
    - Add Unix socket integration tests
    - Add Playwright E2E for: Fund page (Onramp widget, manual deposit), spending breakdown charts, transaction export, search

## Rules
- STRICT TDD: Wave 1 Task #1 (write test cases) must complete before ANY implementation starts.
- All other TDD rules apply: tests first for every module.
- Use worktree isolation for all code-writing agents.
- Commit after each passing wave.
- Transaction Monitor Service is a separate deployable — it gets its own test suite.
- Desktop app connects to monitor via WebSocket — test with a mock WebSocket server in integration tests.
- No Alchemy API key in the desktop app. Ever.

## Phase 3 Deliverable
Full transaction lifecycle: send + receive + earn. Users can fund via Coinbase Onramp or manual deposit. Incoming transactions detected by cloud monitor service and pushed to app. Rich spending analytics (by agent, category, service, time). Transaction export to CSV. Full-text search. Unix socket transport available. Network toggle (testnet/mainnet). All integration scenarios passing.
```
