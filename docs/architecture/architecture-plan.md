# Agent Neo Bank -- Architecture Plan

> **Version:** 2.0
> **Date:** 2026-02-27
> **Status:** Draft (Updated with Codex review fixes + user feedback)

---

## Table of Contents

1. [System Overview](#1-system-overview)
2. [Directory Structure](#2-directory-structure)
3. [Rust Backend Architecture](#3-rust-backend-architecture)
4. [React Frontend Architecture](#4-react-frontend-architecture)
5. [Agent Communication Protocol](#5-agent-communication-protocol)
6. [Data Flow Diagrams](#6-data-flow-diagrams)
7. [Security Model](#7-security-model)
8. [Global Policy & Wallet Controls](#8-global-policy--wallet-controls) **(NEW)**
9. [Transaction Monitor Service](#9-transaction-monitor-service) **(NEW)**
10. [Agent Registration Skill](#10-agent-registration-skill) **(NEW)**
11. [Build & Development](#11-build--development)
12. [Implementation Phases](#12-implementation-phases)

---

## 1. System Overview

### 1.1 High-Level Architecture

```
+------------------------------------------------------------------+
|                        TAURI v2 SHELL                            |
|                                                                  |
|  +---------------------------+  +-----------------------------+  |
|  |    REACT FRONTEND         |  |     RUST BACKEND            |  |
|  |    (WebView)              |  |                             |  |
|  |                           |  |  +---------------------+   |  |
|  |  +---------------------+  |  |  | Tauri IPC Commands  |   |  |
|  |  | Dashboard           |<-+--+->| (frontend <-> rust) |   |  |
|  |  | Agent Management    |  |  |  +---------------------+   |  |
|  |  | Transaction History |  |  |            |               |  |
|  |  | Settings            |  |  |  +---------v-----------+   |  |
|  |  | Onboarding          |  |  |  |   Core Service      |   |  |
|  |  | Invitation Codes    |  |  |  |   Layer             |   |  |
|  |  +---------------------+  |  |  |                     |   |  |
|  |                           |  |  |  - SpendingPolicy   |   |  |
|  |  Vite + React + TS        |  |  |  - GlobalPolicy     |   |  |
|  |  Tailwind v4 + shadcn/ui  |  |  |  - TxProcessor      |   |  |
|  +---------------------------+  |  |  - AgentRegistry     |   |  |
|                                 |  |  - AuthService       |   |  |
|                                 |  |  - BalanceCache      |   |  |
|                                 |  |  - NotificationMgr   |   |  |
|                                 |  |  +-------------------+   |  |
|                                 |  |       |      |     |     |  |
|                                 |  |  +----v--+ +-v---+ |     |  |
|                                 |  |  |SQLite | |CLI  | |     |  |
|                                 |  |  |       | |Wrap | |     |  |
|                                 |  |  +-------+ +-----+ |     |  |
|                                 |  +---------------------+   |  |
|                                 +-----------------------------+  |
+------------------------------------------------------------------+
         |                    |                    |          |
         v                    v                    v          v
  +-------------+   +------------------+   +-----------+  +-----------+
  | REST API    |   | MCP Server       |   | Unix Sock |  | WebSocket |
  | :7402       |   | (stdio/sse)      |   | /tmp/anb  |  | (monitor) |
  +------+------+   +--------+---------+   +-----+-----+  +-----+-----+
         |                    |                   |              |
         +--------------------+-------------------+              |
                              |                                  |
                    +---------v----------+            +----------v-----------+
                    |   AI AGENTS        |            | Transaction Monitor  |
                    |  (Claude Code,     |            | Service (cloud)      |
                    |   custom agents,   |            | - Alchemy polling    |
                    |   scripts)         |            | - Incoming tx detect |
                    +--------------------+            | - Push notifications |
                                                      +----------------------+
```

### 1.2 Component Relationships

```
                    +------------------+
                    |  Tauri App       |
                    |  (orchestrator)  |
                    +--------+---------+
                             |
              +--------------+---+--------------+
              |              |   |              |
     +--------v---+  +------v-+ | +----v--------+
     | WebView    |  | Axum   | | | MCP Server  |
     | (React UI) |  | Server | | | (per-agent) |
     +--------+---+  +------+-+ | +----+--------+
              |              |   |      |
              +--------------+---+------+
                             |
                    +--------v---------+
                    | CoreServices     |  (UPDATED: Arc<CoreServices>, &self methods)
                    | (Arc, no Mutex)  |
                    +--------+---------+
                             |
         +----------+--------+--------+----------+
         |          |        |        |          |
  +------v---+ +---v----+ +-v------+ +-v------+ +v-----------+
  | SQLite   | | CLI    | | Notif  | | Global | | Balance    |
  | (pool +  | | Wrap   | | Mgr    | | Policy | | Cache      |
  | blocking)| | (awal) | |        | | Engine | | (30s TTL)  |
  +----------+ +--------+ +--------+ +--------+ +------------+
```

### 1.3 Key Design Principles

1. **Single source of truth**: The Core Service Layer handles all business logic. All three transports (IPC, REST, MCP) delegate to it.
2. **CLI as execution layer**: We never talk to Coinbase APIs directly. The `awal` CLI is the sole interface for wallet operations.
3. **Policy-first transactions**: Every outgoing transaction passes through both the Global Policy and per-agent Spending Policy engines before CLI execution. **(UPDATED)**
4. **Local-first**: All core data lives in a local SQLite database. The user owns their data. A lightweight cloud service handles only chain monitoring for incoming transaction notifications. **(UPDATED)**
5. **Agent-agnostic**: The API layer does not assume Claude Code. Any process that can speak HTTP, MCP, or Unix sockets can be an agent.
6. **Async transactions**: `/v1/send` always returns `202 Accepted` immediately. Agents poll for final status or provide a webhook callback URL. **(NEW)**
7. **Rich agent metadata**: Agents provide identity, purpose, and per-transaction context. This metadata powers the dashboard so users understand what agents are doing and why. **(NEW)**

---

## 2. Directory Structure

```
agent-neo-bank/
|
+-- src-tauri/                         # Rust backend (Tauri v2)
|   +-- Cargo.toml
|   +-- tauri.conf.json
|   +-- capabilities/                  # Tauri v2 permission capabilities
|   |   +-- default.json
|   |   +-- agent-api.json
|   +-- src/
|   |   +-- main.rs                    # Tauri entry point
|   |   +-- lib.rs                     # Library root, module declarations
|   |   |
|   |   +-- commands/                  # Tauri IPC command handlers
|   |   |   +-- mod.rs
|   |   |   +-- auth.rs               # Login, verify, logout
|   |   |   +-- agents.rs             # Agent CRUD, approval
|   |   |   +-- transactions.rs       # Transaction queries, manual send
|   |   |   +-- spending.rs           # Policy read/write
|   |   |   +-- global_policy.rs      # Global wallet controls (NEW)
|   |   |   +-- settings.rs           # App config, notifications
|   |   |   +-- wallet.rs             # Balance, address, network
|   |   |   +-- onramp.rs             # Coinbase Onramp widget URL
|   |   |   +-- invitations.rs        # Invitation code generation (NEW)
|   |   |
|   |   +-- core/                     # Core service layer (transport-agnostic)
|   |   |   +-- mod.rs
|   |   |   +-- services.rs           # CoreServices struct definition (NEW)
|   |   |   +-- agent_registry.rs     # Agent lifecycle management
|   |   |   +-- spending_policy.rs    # Budget validation engine
|   |   |   +-- global_policy.rs      # Global wallet-level controls (NEW)
|   |   |   +-- tx_processor.rs       # Transaction execution pipeline (async 202 model)
|   |   |   +-- auth_service.rs       # OTP auth wrapping CLI + token cache (UPDATED)
|   |   |   +-- wallet_service.rs     # Balance, address lookups + balance cache (UPDATED)
|   |   |   +-- approval_manager.rs   # Approval queue + resolution + stale cleanup
|   |   |   +-- notification.rs       # OS notification dispatch
|   |   |   +-- event_bus.rs          # Internal event system
|   |   |   +-- invitation.rs         # Invitation code manager (NEW)
|   |   |
|   |   +-- db/                       # Database layer
|   |   |   +-- mod.rs
|   |   |   +-- schema.rs             # Table creation, migrations
|   |   |   +-- models.rs             # Rust structs for DB rows
|   |   |   +-- queries.rs            # Typed query functions
|   |   |   +-- migrations/           # SQL migration files
|   |   |       +-- 001_initial.sql
|   |   |
|   |   +-- cli/                      # Coinbase Agent Wallet CLI wrapper
|   |   |   +-- mod.rs
|   |   |   +-- executor.rs           # Spawn process, capture output
|   |   |   +-- parser.rs             # Parse CLI stdout/stderr
|   |   |   +-- commands.rs           # Typed command builders
|   |   |
|   |   +-- api/                      # Agent-facing API servers
|   |   |   +-- mod.rs
|   |   |   +-- rest_server.rs        # Axum HTTP server on :7402
|   |   |   +-- rest_routes.rs        # Route definitions
|   |   |   +-- rest_handlers.rs      # Request handlers
|   |   |   +-- mcp_server.rs         # MCP protocol server (per-agent spawn) (UPDATED)
|   |   |   +-- mcp_tools.rs          # MCP tool definitions
|   |   |   +-- unix_socket.rs        # Unix domain socket server
|   |   |   +-- auth_middleware.rs     # Token validation + SHA-256 cache (UPDATED)
|   |   |   +-- rate_limiter.rs       # Invitation-code-based rate limiting (NEW)
|   |   |   +-- types.rs              # Shared API request/response types (Decimal amounts)
|   |   |
|   |   +-- state/                    # Application state
|   |   |   +-- mod.rs
|   |   |   +-- app_state.rs          # Shared state struct (Arc<...>)
|   |   |
|   |   +-- error.rs                  # Unified error types
|   |   +-- config.rs                 # App configuration
|   |
|   +-- icons/                        # App icons
|   +-- data/                         # Default data / seed files
|
+-- src/                              # React frontend
|   +-- main.tsx                      # React entry point
|   +-- App.tsx                       # Root component, router
|   +-- vite-env.d.ts
|   |
|   +-- pages/                        # Top-level route pages
|   |   +-- Onboarding.tsx            # Auth + first-run setup
|   |   +-- Dashboard.tsx             # Main bank dashboard
|   |   +-- AgentList.tsx             # All agents overview
|   |   +-- AgentDetail.tsx           # Single agent deep dive
|   |   +-- Transactions.tsx          # Full transaction history
|   |   +-- Settings.tsx              # App settings + notifications
|   |   +-- Approvals.tsx             # Pending approval queue
|   |   +-- Fund.tsx                  # Deposit / onramp page
|   |
|   +-- components/                   # Reusable UI components
|   |   +-- layout/
|   |   |   +-- Sidebar.tsx           # Nav sidebar
|   |   |   +-- Header.tsx            # Top bar with balance
|   |   |   +-- Shell.tsx             # App shell wrapper
|   |   |
|   |   +-- dashboard/
|   |   |   +-- BalanceCard.tsx
|   |   |   +-- SpendingChart.tsx
|   |   |   +-- RecentTransactions.tsx
|   |   |   +-- AgentStatusGrid.tsx
|   |   |   +-- BudgetUtilization.tsx
|   |   |
|   |   +-- agents/
|   |   |   +-- AgentCard.tsx
|   |   |   +-- AgentForm.tsx
|   |   |   +-- SpendingLimitsEditor.tsx
|   |   |   +-- AgentActivityFeed.tsx
|   |   |   +-- AllowlistEditor.tsx
|   |   |
|   |   +-- transactions/
|   |   |   +-- TransactionTable.tsx
|   |   |   +-- TransactionRow.tsx
|   |   |   +-- TransactionDetail.tsx
|   |   |   +-- FilterBar.tsx
|   |   |
|   |   +-- approvals/
|   |   |   +-- ApprovalCard.tsx
|   |   |   +-- ApprovalQueue.tsx
|   |   |
|   |   +-- onboarding/
|   |   |   +-- EmailStep.tsx
|   |   |   +-- OtpStep.tsx
|   |   |   +-- FundStep.tsx
|   |   |   +-- WelcomeStep.tsx
|   |   |
|   |   +-- shared/
|   |       +-- CurrencyDisplay.tsx   # Format USD/USDC amounts
|   |       +-- StatusBadge.tsx
|   |       +-- EmptyState.tsx
|   |       +-- ConfirmDialog.tsx
|   |
|   +-- hooks/                        # Custom React hooks
|   |   +-- useBalance.ts
|   |   +-- useAgents.ts
|   |   +-- useTransactions.ts
|   |   +-- useApprovals.ts
|   |   +-- useTauriEvent.ts          # Listen to Tauri events
|   |   +-- useInvoke.ts              # Typed Tauri invoke wrapper
|   |
|   +-- lib/                          # Utilities
|   |   +-- tauri.ts                  # Tauri invoke/event helpers
|   |   +-- format.ts                 # Currency, date formatting
|   |   +-- constants.ts
|   |
|   +-- stores/                       # State management (zustand)
|   |   +-- authStore.ts
|   |   +-- agentStore.ts
|   |   +-- transactionStore.ts
|   |   +-- settingsStore.ts
|   |   +-- approvalStore.ts
|   |
|   +-- types/                        # Shared TypeScript types
|       +-- agent.ts
|       +-- transaction.ts
|       +-- spending.ts
|       +-- api.ts
|
+-- public/                           # Static assets
|   +-- logo.svg
|
+-- docs/                             # Documentation
|   +-- architecture/
|   |   +-- architecture-plan.md      # This file
|   +-- reference/
|   +-- investigations/
|   +-- implementation/
|
+-- skills/                           # Agent skills (NEW)
|   +-- agent-neo-bank.md            # Registration & usage skill for AI agents
|
+-- package.json
+-- tsconfig.json
+-- vite.config.ts
+-- tailwind.config.ts
+-- components.json                   # shadcn/ui config
+-- index.html
+-- .env                              # Local env vars (gitignored)
+-- .gitignore
+-- README.md
```

---

## 3. Rust Backend Architecture

### 3.1 Tauri IPC Commands

Tauri v2 uses the `#[tauri::command]` macro to expose Rust functions to the frontend via `invoke()`. All commands are thin wrappers that delegate to the Core Service Layer.

| Command | Module | Description |
|---|---|---|
| `auth_login` | `commands::auth` | Initiate email OTP via `awal auth login` |
| `auth_verify` | `commands::auth` | Verify OTP via `awal auth verify` |
| `auth_logout` | `commands::auth` | Log out, clear session |
| `auth_status` | `commands::auth` | Check if authenticated |
| `get_balance` | `commands::wallet` | Fetch wallet balance via CLI |
| `get_address` | `commands::wallet` | Get wallet deposit address |
| `get_network` | `commands::wallet` | Current network (sepolia/mainnet) |
| `set_network` | `commands::wallet` | Toggle network |
| `list_agents` | `commands::agents` | List all agents with status |
| `get_agent` | `commands::agents` | Single agent detail |
| `create_agent` | `commands::agents` | Pre-create agent profile |
| `approve_agent` | `commands::agents` | Approve pending agent, issue token |
| `suspend_agent` | `commands::agents` | Suspend an agent |
| `delete_agent` | `commands::agents` | Remove agent |
| `get_spending_policy` | `commands::spending` | Read agent's policy |
| `set_spending_policy` | `commands::spending` | Update agent's limits |
| `list_transactions` | `commands::transactions` | Query tx history with filters |
| `get_transaction` | `commands::transactions` | Single tx detail |
| `manual_send` | `commands::transactions` | User-initiated send from UI |
| `list_approvals` | `commands::transactions` | Pending approval requests |
| `resolve_approval` | `commands::transactions` | Approve or deny a request |
| `get_settings` | `commands::settings` | Read app settings |
| `update_settings` | `commands::settings` | Write app settings |
| `get_onramp_url` | `commands::onramp` | Generate Coinbase Onramp URL |
| `export_transactions` | `commands::transactions` | Export to CSV |

**Example command implementation (UPDATED -- no Mutex, uses `Arc<CoreServices>` directly):**

```rust
#[tauri::command]
async fn list_agents(
    state: tauri::State<'_, Arc<CoreServices>>,
    limit: Option<u32>,     // (UPDATED) pagination support
    offset: Option<u32>,    // (UPDATED) pagination support
) -> Result<PaginatedResult<AgentSummary>, AppError> {
    state.agent_registry.list_all(limit.unwrap_or(50), offset.unwrap_or(0)).await
}
```

> **Design note (UPDATED):** `CoreServices` is stored as `Arc<CoreServices>` in Tauri state -- no wrapping `Mutex`. All core service methods take `&self`, not `&mut self`. Sub-services use interior mutability only where needed (e.g., the SQLite connection pool handles its own concurrency via `r2d2`). See Section 3.12 for the full `CoreServices` struct definition.

### 3.2 Local API Server (Axum)

An Axum HTTP server starts alongside the Tauri app on port `7402`. This is the primary transport for AI agents.

**Server Lifecycle:**
1. Tauri `setup()` hook spawns a tokio task running the Axum server.
2. The Axum server shares `Arc<CoreServices>` with the Tauri commands -- no Mutex wrapping. **(UPDATED)**
3. On app exit, Tauri's `on_exit` hook sends a shutdown signal to the Axum server via a `tokio::sync::watch` channel.

```rust
// In main.rs setup hook (UPDATED: no Mutex, CLI health check, mock mode)
let cli: Arc<dyn CliExecutable> = if config.mock_mode {
    Arc::new(MockCliExecutor::new())         // ANB_MOCK=true or --mock flag
} else {
    // Health check: verify CLI is available and session is valid
    let real_cli = RealCliExecutor::new(&config.awal_binary_path)?;
    match real_cli.run(AwalCommand::AuthStatus).await {
        Ok(output) if output.success => Arc::new(real_cli),
        Ok(_) => return Err(AppError::CliSessionExpired),   // redirect to re-auth
        Err(_) => return Err(AppError::CliNotFound),         // show onboarding step
    }
};

let core = Arc::new(CoreServices::new(db, cli, config).await?);
let core_for_axum = core.clone();

tauri::async_runtime::spawn(async move {
    api::rest_server::start(core_for_axum, shutdown_rx).await;
});

// Also spawn stale approval cleanup task (every 5 minutes)
let core_for_cleanup = core.clone();
tauri::async_runtime::spawn(async move {
    core_for_cleanup.approval_manager.run_cleanup_loop().await;
});
```

**Port Selection:**
- Default: `7402` (mnemonic: "x402" payment protocol reference)
- Configurable in settings
- On startup, checks if port is occupied; fails loudly if so

### 3.3 CLI Wrapper Module

The CLI wrapper provides a typed, async interface to the Coinbase Agent Wallet CLI (`awal`).

> **Important (UPDATED):** `rusqlite` is synchronous. All database calls MUST be wrapped in `tokio::task::spawn_blocking()` to avoid blocking the async runtime. This is the standard pattern throughout the codebase. (Alternative: `sqlx` with async SQLite support, but `spawn_blocking` with `rusqlite` is simpler and well-understood.)

```rust
// cli/commands.rs

pub enum AwalCommand {
    AuthLogin { email: String },
    AuthVerify { email: String, otp: String },
    AuthStatus,                                     // (NEW) used for health check
    GetBalance,
    GetAddress,
    Send { to: String, amount: Decimal, asset: String },  // (UPDATED) Decimal not String
    // ... etc
}

impl AwalCommand {
    pub fn to_args(&self) -> Vec<String> { ... }
}
```

```rust
// cli/executor.rs (UPDATED: trait-based for mock support, spawn_blocking for DB)

#[async_trait]
pub trait CliExecutable: Send + Sync {
    async fn run(&self, cmd: AwalCommand) -> Result<CliOutput, CliError>;
}

pub struct RealCliExecutor {
    binary_path: PathBuf,
    network: Network,
}

impl CliExecutable for RealCliExecutor {
    async fn run(&self, cmd: AwalCommand) -> Result<CliOutput, CliError> {
        let args = cmd.to_args();
        let output = tokio::process::Command::new(&self.binary_path)
            .args(&args)
            .env("AWAL_NETWORK", self.network.as_str())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?
            .wait_with_output()
            .await?;

        CliOutput::parse(output)
    }
}

/// (NEW) Mock executor for testing and --mock mode.
/// Returns realistic fake data without spawning awal.
/// Activated via ANB_MOCK=true env var or --mock CLI flag.
pub struct MockCliExecutor {
    responses: HashMap<String, CliOutput>,
}

impl CliExecutable for MockCliExecutor {
    async fn run(&self, cmd: AwalCommand) -> Result<CliOutput, CliError> {
        // Return canned responses: fake balance, fake tx hashes, etc.
        Ok(self.responses.get(&cmd.key()).cloned().unwrap_or_default())
    }
}
```

```rust
// cli/parser.rs -- parse stdout from awal into structured data

pub struct CliOutput {
    pub success: bool,
    pub data: serde_json::Value,  // Parsed JSON if available
    pub raw: String,               // Raw stdout
    pub stderr: String,
}
```

**spawn_blocking pattern for all DB calls (NEW):**

```rust
// Example: wrapping a rusqlite call in spawn_blocking
pub async fn get_agent(&self, agent_id: &str) -> Result<Agent, AppError> {
    let pool = self.pool.clone();
    let id = agent_id.to_string();
    tokio::task::spawn_blocking(move || {
        let conn = pool.get()?;
        let agent = conn.query_row(
            "SELECT * FROM agents WHERE id = ?1",
            params![id],
            |row| Agent::from_row(row),
        )?;
        Ok(agent)
    })
    .await?
}
```

**Whitelisted commands** (only these can be executed):
- `auth login`, `auth verify`, `auth status`, `auth logout`
- `wallet balance`, `wallet address`
- `send`
- `config get`, `config set`

**Balance Caching (NEW):**

CLI responses (especially balance) are cached with a 30-second TTL. One CLI call per TTL period regardless of how many agents or UI polls request the balance. Implemented via a `tokio::sync::RwLock<Option<CachedBalance>>` in the wallet service.

```rust
pub struct BalanceCache {
    cached: RwLock<Option<CachedBalance>>,
    ttl: Duration,  // 30 seconds
}

struct CachedBalance {
    balance: Decimal,
    asset: String,
    fetched_at: Instant,
}

impl BalanceCache {
    pub async fn get_or_fetch(&self, cli: &dyn CliExecutable) -> Result<BalanceInfo, AppError> {
        // Read lock: check cache
        if let Some(cached) = self.cached.read().await.as_ref() {
            if cached.fetched_at.elapsed() < self.ttl {
                return Ok(cached.into());
            }
        }
        // Write lock: fetch and update
        let mut cache = self.cached.write().await;
        let output = cli.run(AwalCommand::GetBalance).await?;
        let balance = parse_balance(&output)?;
        *cache = Some(CachedBalance { balance, asset: "USDC".into(), fetched_at: Instant::now() });
        Ok(cache.as_ref().unwrap().into())
    }
}
```

### 3.4 SQLite Schema

All tables use `TEXT` for UUIDs and `INTEGER` for timestamps (Unix epoch seconds). All timestamps are explicitly UTC. **(UPDATED)**

> **Concurrency note (NEW):** All spending-limit checks and ledger updates MUST be wrapped in `BEGIN EXCLUSIVE` transactions to prevent race conditions when concurrent agent requests arrive. The read-check-execute-update cycle for spending is serialized per agent via SQLite's exclusive locking.

```sql
-- 001_initial.sql

-- Application configuration (key-value)
CREATE TABLE IF NOT EXISTS app_config (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- Agent registry (UPDATED: rich metadata fields)
CREATE TABLE IF NOT EXISTS agents (
    id                TEXT PRIMARY KEY,           -- UUID v4
    name              TEXT NOT NULL,
    description       TEXT DEFAULT '',
    purpose           TEXT DEFAULT '',            -- (NEW) what the agent is built for
    agent_type        TEXT DEFAULT '',            -- (NEW) e.g., "coding_assistant", "research"
    capabilities      TEXT DEFAULT '[]',          -- (NEW) JSON array: ["send", "receive"]
    status            TEXT NOT NULL DEFAULT 'pending',  -- pending | active | suspended | revoked
    api_token_hash    TEXT,                       -- argon2 hash of the agent's bearer token
    token_prefix      TEXT,                       -- first 8 chars for display (e.g., "anb_a3f8...")
    balance_visible   INTEGER NOT NULL DEFAULT 1, -- (NEW) whether agent can see wallet balance
    invitation_code   TEXT,                       -- (NEW) the invitation code used to register
    created_at        INTEGER NOT NULL,
    updated_at        INTEGER NOT NULL,
    last_active_at    INTEGER,
    metadata          TEXT DEFAULT '{}'           -- JSON blob for extensible data
);

CREATE INDEX idx_agents_status ON agents(status);

-- Spending policies (one per agent)
CREATE TABLE IF NOT EXISTS spending_policies (
    agent_id         TEXT PRIMARY KEY REFERENCES agents(id) ON DELETE CASCADE,
    per_tx_max       TEXT NOT NULL DEFAULT '0',     -- Decimal string in USDC
    daily_cap        TEXT NOT NULL DEFAULT '0',
    weekly_cap       TEXT NOT NULL DEFAULT '0',
    monthly_cap      TEXT NOT NULL DEFAULT '0',
    auto_approve_max TEXT NOT NULL DEFAULT '0',     -- Below this = auto-approve
    allowlist        TEXT DEFAULT '[]',             -- JSON array of allowed addresses/domains
    updated_at       INTEGER NOT NULL
);

-- Global policy (NEW) -- wallet-level controls above all agent policies
CREATE TABLE IF NOT EXISTS global_policy (
    id                   TEXT PRIMARY KEY DEFAULT 'default',
    daily_cap            TEXT NOT NULL DEFAULT '0',       -- Global daily spending cap across all agents
    weekly_cap           TEXT NOT NULL DEFAULT '0',       -- Global weekly cap
    monthly_cap          TEXT NOT NULL DEFAULT '0',       -- Global monthly cap
    min_reserve_balance  TEXT NOT NULL DEFAULT '0',       -- Refuse txs that would drop below this
    kill_switch_active   INTEGER NOT NULL DEFAULT 0,      -- 1 = all agent operations suspended
    kill_switch_reason   TEXT DEFAULT '',
    updated_at           INTEGER NOT NULL
);

-- Global spending ledger (NEW) -- aggregate across all agents
CREATE TABLE IF NOT EXISTS global_spending_ledger (
    period     TEXT PRIMARY KEY,               -- 'daily:2026-02-27' | 'weekly:2026-W09' | 'monthly:2026-02'
    total      TEXT NOT NULL DEFAULT '0',
    tx_count   INTEGER NOT NULL DEFAULT 0,
    updated_at INTEGER NOT NULL
);

-- Transactions (UPDATED: rich metadata, webhook support)
CREATE TABLE IF NOT EXISTS transactions (
    id              TEXT PRIMARY KEY,             -- UUID v4
    agent_id        TEXT REFERENCES agents(id),   -- NULL for user-initiated txs
    tx_type         TEXT NOT NULL,                -- send | receive | earn
    amount          TEXT NOT NULL,                -- Decimal string in USDC
    asset           TEXT NOT NULL DEFAULT 'USDC',
    recipient       TEXT,                         -- Address or service identifier
    sender          TEXT,                         -- For receive txs
    chain_tx_hash   TEXT,                         -- On-chain tx hash when available
    status          TEXT NOT NULL DEFAULT 'pending', -- pending | approved | executing | confirmed | failed | denied
    category        TEXT DEFAULT 'uncategorized',
    memo            TEXT DEFAULT '',
    description     TEXT DEFAULT '',              -- (NEW) detailed description from agent
    service_name    TEXT DEFAULT '',              -- (NEW) what service this payment is for
    service_url     TEXT DEFAULT '',              -- (NEW) URL of the service
    reason          TEXT DEFAULT '',              -- (NEW) why the agent needs this payment
    webhook_url     TEXT,                         -- (NEW) optional callback URL for status updates
    error_message   TEXT,
    period_daily    TEXT,                         -- (NEW) UTC period key at creation time
    period_weekly   TEXT,                         -- (NEW) UTC period key at creation time
    period_monthly  TEXT,                         -- (NEW) UTC period key at creation time
    created_at      INTEGER NOT NULL,
    updated_at      INTEGER NOT NULL
);

CREATE INDEX idx_tx_agent ON transactions(agent_id);
CREATE INDEX idx_tx_status ON transactions(status);
CREATE INDEX idx_tx_created ON transactions(created_at);
CREATE INDEX idx_tx_type ON transactions(tx_type);

-- Approval requests (UPDATED: expires_at for stale cleanup)
CREATE TABLE IF NOT EXISTS approval_requests (
    id           TEXT PRIMARY KEY,              -- UUID v4
    agent_id     TEXT NOT NULL REFERENCES agents(id),
    request_type TEXT NOT NULL,                 -- transaction | limit_increase | registration
    payload      TEXT NOT NULL,                 -- JSON: the full request details
    status       TEXT NOT NULL DEFAULT 'pending', -- pending | approved | denied | expired
    tx_id        TEXT REFERENCES transactions(id), -- Links to tx if type=transaction
    expires_at   INTEGER NOT NULL,              -- (NEW) auto-expire after this timestamp
    created_at   INTEGER NOT NULL,
    resolved_at  INTEGER,
    resolved_by  TEXT                           -- 'user' or 'auto'
);

CREATE INDEX idx_approval_status ON approval_requests(status);
CREATE INDEX idx_approval_agent ON approval_requests(agent_id);
CREATE INDEX idx_approval_expires ON approval_requests(expires_at);  -- (NEW)

-- Invitation codes (NEW) -- user-generated codes for agent registration
CREATE TABLE IF NOT EXISTS invitation_codes (
    code         TEXT PRIMARY KEY,              -- Short alphanumeric code
    created_at   INTEGER NOT NULL,
    expires_at   INTEGER,                       -- Optional expiry
    used_by      TEXT REFERENCES agents(id),    -- NULL until used
    used_at      INTEGER,
    max_uses     INTEGER NOT NULL DEFAULT 1,    -- Usually 1 (one-time use)
    use_count    INTEGER NOT NULL DEFAULT 0,
    label        TEXT DEFAULT ''                -- User-facing label ("for Claude Code")
);

-- Token delivery cache (NEW) -- short-lived encrypted token storage
CREATE TABLE IF NOT EXISTS token_delivery (
    agent_id     TEXT PRIMARY KEY REFERENCES agents(id),
    encrypted_token TEXT NOT NULL,              -- AES-encrypted bearer token
    created_at   INTEGER NOT NULL,
    expires_at   INTEGER NOT NULL,              -- created_at + 300 (5 minutes)
    delivered    INTEGER NOT NULL DEFAULT 0     -- 1 after first retrieval (then deleted)
);

-- Notification preferences
CREATE TABLE IF NOT EXISTS notification_preferences (
    id         TEXT PRIMARY KEY DEFAULT 'default',
    enabled    INTEGER NOT NULL DEFAULT 1,
    on_all_tx  INTEGER NOT NULL DEFAULT 0,
    on_large_tx INTEGER NOT NULL DEFAULT 1,
    large_tx_threshold TEXT NOT NULL DEFAULT '10.00',
    on_errors  INTEGER NOT NULL DEFAULT 1,
    on_limit_requests INTEGER NOT NULL DEFAULT 1,
    on_agent_registration INTEGER NOT NULL DEFAULT 1
);

-- Spending ledger (rolling aggregates for fast policy checks)
-- (UPDATED) All timestamps are UTC. Period is determined at transaction creation time, not completion.
-- First transaction of a new period performs an INSERT; subsequent transactions UPDATE the total.
-- All reads and writes to this table for a given agent MUST use BEGIN EXCLUSIVE transactions.
CREATE TABLE IF NOT EXISTS spending_ledger (
    agent_id   TEXT NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    period     TEXT NOT NULL,                  -- 'daily:2026-02-27' | 'weekly:2026-W09' | 'monthly:2026-02'
    total      TEXT NOT NULL DEFAULT '0',      -- Running total for this period
    tx_count   INTEGER NOT NULL DEFAULT 0,
    updated_at INTEGER NOT NULL,
    PRIMARY KEY (agent_id, period)
);
```

> **Spending ledger timezone (NEW):** All period keys use UTC. The period is stamped on the transaction at creation time (the `period_daily`, `period_weekly`, `period_monthly` columns on `transactions`). The ledger upsert pattern: `INSERT ... ON CONFLICT(agent_id, period) DO UPDATE SET total = total + ?1, tx_count = tx_count + 1`.

### 3.5 Spending Policy Engine

The spending policy engine is the core gatekeeper. Every outbound transaction must pass through both the **Global Policy** and the **per-agent Spending Policy**. **(UPDATED)**

> **Race condition fix (UPDATED):** The entire read-check-execute-update cycle is wrapped in a `BEGIN EXCLUSIVE` SQLite transaction. This serializes concurrent transactions per agent, preventing two simultaneous requests from both passing a limit that only one should pass.

```rust
// core/spending_policy.rs (UPDATED)

pub struct SpendingPolicyEngine {
    db: Arc<Database>,
}

pub enum PolicyDecision {
    AutoApproved,                    // Within auto-approve threshold
    RequiresApproval { reason: String }, // Exceeds threshold, queue for user
    Denied { reason: String },       // Hard limit exceeded, reject immediately
}

impl SpendingPolicyEngine {
    /// Evaluate a transaction against BOTH global and agent spending policies.
    /// This entire method runs inside a BEGIN EXCLUSIVE transaction (via spawn_blocking).
    pub async fn evaluate(
        &self,
        agent_id: &str,
        amount: Decimal,
        recipient: &str,
    ) -> Result<PolicyDecision, AppError> {
        let db = self.db.clone();
        let agent_id = agent_id.to_string();
        let amount = amount;
        let recipient = recipient.to_string();

        tokio::task::spawn_blocking(move || {
            let conn = db.pool.get()?;
            // BEGIN EXCLUSIVE serializes all concurrent policy checks
            conn.execute_batch("BEGIN EXCLUSIVE")?;

            let result = (|| -> Result<PolicyDecision, AppError> {
                // 0. Check global kill switch (NEW)
                let global = db.get_global_policy_sync(&conn)?;
                if global.kill_switch_active {
                    return Ok(PolicyDecision::Denied {
                        reason: format!("Emergency kill switch active: {}", global.kill_switch_reason),
                    });
                }

                // 0a. Check global minimum reserve balance (NEW)
                let current_balance = db.get_cached_balance_sync(&conn)?;
                if current_balance - amount < global.min_reserve_balance {
                    return Ok(PolicyDecision::Denied {
                        reason: format!(
                            "Would drop balance below minimum reserve of {}",
                            global.min_reserve_balance
                        ),
                    });
                }

                // 0b. Check global daily/weekly/monthly caps (NEW)
                let global_ledger = db.get_global_spending_ledger_sync(&conn)?;
                if global_ledger.daily_total() + amount > global.daily_cap && global.daily_cap > Decimal::ZERO {
                    return Ok(PolicyDecision::Denied {
                        reason: format!("Would exceed global daily cap of {}", global.daily_cap),
                    });
                }
                if global_ledger.weekly_total() + amount > global.weekly_cap && global.weekly_cap > Decimal::ZERO {
                    return Ok(PolicyDecision::Denied {
                        reason: format!("Would exceed global weekly cap of {}", global.weekly_cap),
                    });
                }
                if global_ledger.monthly_total() + amount > global.monthly_cap && global.monthly_cap > Decimal::ZERO {
                    return Ok(PolicyDecision::Denied {
                        reason: format!("Would exceed global monthly cap of {}", global.monthly_cap),
                    });
                }

                // 1. Check per-agent policy
                let policy = db.get_spending_policy_sync(&conn, &agent_id)?;
                let ledger = db.get_spending_ledger_sync(&conn, &agent_id)?;

                if amount > policy.per_tx_max {
                    return Ok(PolicyDecision::Denied {
                        reason: format!(
                            "Amount {} exceeds per-tx limit of {}",
                            amount, policy.per_tx_max
                        ),
                    });
                }

                // 2. Check daily cap
                let today_spent = ledger.daily_total();
                if today_spent + amount > policy.daily_cap {
                    return Ok(PolicyDecision::Denied {
                        reason: format!("Would exceed daily cap of {}", policy.daily_cap),
                    });
                }

                // 3. Check weekly cap
                let week_spent = ledger.weekly_total();
                if week_spent + amount > policy.weekly_cap {
                    return Ok(PolicyDecision::Denied {
                        reason: format!("Would exceed weekly cap of {}", policy.weekly_cap),
                    });
                }

                // 4. Check monthly cap
                let month_spent = ledger.monthly_total();
                if month_spent + amount > policy.monthly_cap {
                    return Ok(PolicyDecision::Denied {
                        reason: format!("Would exceed monthly cap of {}", policy.monthly_cap),
                    });
                }

                // 5. Check allowlist (if non-empty)
                if !policy.allowlist.is_empty()
                    && !policy.allowlist.contains(&recipient)
                {
                    return Ok(PolicyDecision::Denied {
                        reason: "Recipient not in allowlist".to_string(),
                    });
                }

                // 6. Auto-approve or require user approval
                if amount <= policy.auto_approve_max {
                    Ok(PolicyDecision::AutoApproved)
                } else {
                    Ok(PolicyDecision::RequiresApproval {
                        reason: format!(
                            "Amount {} exceeds auto-approve threshold of {}",
                            amount, policy.auto_approve_max
                        ),
                    })
                }
            })();

            match &result {
                Ok(_) => conn.execute_batch("COMMIT")?,
                Err(_) => conn.execute_batch("ROLLBACK")?,
            }
            result
        })
        .await?
    }
}
```

### 3.6 Transaction Processor

Orchestrates the full lifecycle of a transaction: policy check, approval (if needed), CLI execution, ledger update.

> **Async model (UPDATED):** `/v1/send` always returns `202 Accepted` with a `tx_id` and `"status": "executing"`. The agent polls `GET /v1/transactions/{tx_id}` for final status. Optionally, the agent can provide a `webhook_url` in the send request to receive a callback when the transaction completes.

> **Atomicity (UPDATED):** Transaction confirmation and ledger update are wrapped in a single `BEGIN EXCLUSIVE` SQLite transaction. If either fails, both roll back.

```rust
// core/tx_processor.rs (UPDATED)

pub struct TransactionProcessor {
    db: Arc<Database>,
    cli: Arc<dyn CliExecutable>,        // (UPDATED) trait object for mock support
    policy: Arc<SpendingPolicyEngine>,
    approvals: Arc<ApprovalManager>,
    notifications: Arc<NotificationManager>,
    event_bus: Arc<EventBus>,
}

/// (UPDATED) SendRequest uses Decimal, not String. Parsed at API boundary via serde.
pub struct SendRequest {
    pub to: String,
    pub amount: Decimal,              // (UPDATED) Decimal, not String
    pub asset: Option<String>,
    pub memo: Option<String>,
    pub category: Option<String>,
    pub description: Option<String>,  // (NEW) detailed description
    pub service_name: Option<String>, // (NEW) what service
    pub service_url: Option<String>,  // (NEW) service URL
    pub reason: Option<String>,       // (NEW) why the payment
    pub webhook_url: Option<String>,  // (NEW) callback URL for status updates
}

impl TransactionProcessor {
    /// Process a send request from an agent.
    /// ALWAYS returns 202 with tx_id. Execution happens asynchronously.
    pub async fn process_send(
        &self,
        agent_id: &str,
        request: SendRequest,
    ) -> Result<TransactionResult, AppError> {
        // Compute UTC period keys at creation time (not completion time)
        let now = Utc::now();
        let period_daily = format!("daily:{}", now.format("%Y-%m-%d"));
        let period_weekly = format!("weekly:{}", now.format("%G-W%V"));
        let period_monthly = format!("monthly:{}", now.format("%Y-%m"));

        // 1. Create transaction record in pending state
        let tx = self.db.create_transaction(Transaction {
            id: Uuid::new_v4().to_string(),
            agent_id: Some(agent_id.to_string()),
            tx_type: TxType::Send,
            amount: request.amount.to_string(),
            recipient: Some(request.to.clone()),
            status: TxStatus::Pending,
            memo: request.memo.unwrap_or_default(),
            description: request.description.unwrap_or_default(),
            service_name: request.service_name.unwrap_or_default(),
            service_url: request.service_url.unwrap_or_default(),
            reason: request.reason.unwrap_or_default(),
            webhook_url: request.webhook_url.clone(),
            period_daily: period_daily.clone(),
            period_weekly: period_weekly.clone(),
            period_monthly: period_monthly.clone(),
            ..Default::default()
        }).await?;

        // 2. Evaluate spending policy (inside BEGIN EXCLUSIVE)
        let decision = self.policy.evaluate(
            agent_id,
            request.amount,
            &request.to,
        ).await?;

        match decision {
            PolicyDecision::AutoApproved => {
                // Spawn async execution -- return 202 immediately
                let this = self.clone();
                let tx_clone = tx.clone();
                tokio::spawn(async move {
                    this.execute_send(&tx_clone).await;
                });
                Ok(TransactionResult::Accepted {
                    tx_id: tx.id,
                    status: "executing".to_string(),
                })
            }
            PolicyDecision::RequiresApproval { reason } => {
                self.db.update_tx_status(&tx.id, TxStatus::AwaitingApproval).await?;
                let approval = self.approvals.create_request(
                    agent_id,
                    ApprovalType::Transaction,
                    &tx,
                    &reason,
                ).await?;
                self.notifications.send(NotificationEvent::ApprovalRequired {
                    agent_id: agent_id.to_string(),
                    amount: request.amount.to_string(),
                    approval_id: approval.id.clone(),
                }).await?;
                self.event_bus.emit(Event::ApprovalCreated(approval.clone()));
                Ok(TransactionResult::Accepted {
                    tx_id: tx.id,
                    status: "awaiting_approval".to_string(),
                })
            }
            PolicyDecision::Denied { reason } => {
                self.db.update_tx_status(&tx.id, TxStatus::Denied).await?;
                self.event_bus.emit(Event::TransactionDenied(tx.id.clone()));
                Ok(TransactionResult::Denied { tx_id: tx.id, reason })
            }
        }
    }

    /// Execute the CLI send and update ledger atomically.
    /// Confirmation + ledger update wrapped in BEGIN EXCLUSIVE. (UPDATED)
    async fn execute_send(&self, tx: &Transaction) {
        self.db.update_tx_status(&tx.id, TxStatus::Executing).await.ok();

        let result = self.cli.run(AwalCommand::Send {
            to: tx.recipient.clone().unwrap(),
            amount: tx.amount.parse().unwrap(),
            asset: tx.asset.clone(),
        }).await;

        match result {
            Ok(output) => {
                let chain_hash = output.data.get("tx_hash")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                // Atomic: confirm tx + update both agent and global ledgers
                let confirmed = self.db.atomic_confirm_and_update_ledger(
                    &tx.id,
                    &chain_hash,
                    tx.agent_id.as_deref().unwrap(),
                    &tx.amount.parse().unwrap(),
                    &tx.period_daily,
                    &tx.period_weekly,
                    &tx.period_monthly,
                ).await;

                match confirmed {
                    Ok(_) => {
                        self.event_bus.emit(Event::TransactionConfirmed(tx.id.clone()));
                        // Fire webhook if provided
                        if let Some(url) = &tx.webhook_url {
                            self.fire_webhook(url, &tx.id, "confirmed").await;
                        }
                    }
                    Err(e) => {
                        self.db.fail_transaction(&tx.id, &e.to_string()).await.ok();
                        self.event_bus.emit(Event::TransactionFailed(tx.id.clone()));
                    }
                }
            }
            Err(e) => {
                self.db.fail_transaction(&tx.id, &e.to_string()).await.ok();
                self.event_bus.emit(Event::TransactionFailed(tx.id.clone()));
                if let Some(url) = &tx.webhook_url {
                    self.fire_webhook(url, &tx.id, "failed").await;
                }
            }
        }
    }

    /// Fire a webhook callback to the agent's specified URL.
    async fn fire_webhook(&self, url: &str, tx_id: &str, status: &str) {
        // Best-effort POST to webhook_url with tx status
        let _ = reqwest::Client::new()
            .post(url)
            .json(&serde_json::json!({ "tx_id": tx_id, "status": status }))
            .timeout(Duration::from_secs(5))
            .send()
            .await;
    }
}
```

### 3.7 Agent Registry

Manages agent lifecycle: registration, approval, token issuance, suspension.

> **Registration with invitation codes (UPDATED):** Agents must include a valid invitation code when registering. The user generates these codes in the UI. This replaces IP-based rate limiting entirely (which is meaningless on localhost).

> **Token delivery (UPDATED):** After user approval, the token is held in an encrypted cache for 5 minutes. The agent polls the status endpoint, which returns the token exactly once then deletes it. After the 5-minute window, the token is gone and the agent must re-register.

```rust
// core/agent_registry.rs (UPDATED)

pub struct AgentRegistry {
    db: Arc<Database>,
    notifications: Arc<NotificationManager>,
    event_bus: Arc<EventBus>,
}

/// (UPDATED) Rich registration request with metadata
pub struct AgentRegistrationRequest {
    pub name: String,
    pub description: Option<String>,
    pub purpose: Option<String>,          // (NEW) what the agent is built for
    pub agent_type: Option<String>,       // (NEW) e.g., "coding_assistant"
    pub capabilities: Option<Vec<String>>,// (NEW) ["send", "receive"]
    pub invitation_code: String,          // (NEW) required invitation code
}

impl AgentRegistry {
    /// Called when an agent self-registers via the API.
    pub async fn register(
        &self,
        request: AgentRegistrationRequest,
    ) -> Result<AgentRegistrationResult, AppError> {
        // (NEW) Validate invitation code
        let invitation = self.db.get_invitation_code(&request.invitation_code).await?;
        if invitation.is_none() {
            return Err(AppError::InvalidInvitationCode);
        }
        let invitation = invitation.unwrap();
        if invitation.use_count >= invitation.max_uses {
            return Err(AppError::InvitationCodeExpired);
        }
        if let Some(expires_at) = invitation.expires_at {
            if Utc::now().timestamp() > expires_at {
                return Err(AppError::InvitationCodeExpired);
            }
        }

        let agent = Agent {
            id: Uuid::new_v4().to_string(),
            name: request.name,
            description: request.description.unwrap_or_default(),
            purpose: request.purpose.unwrap_or_default(),
            agent_type: request.agent_type.unwrap_or_default(),
            capabilities: request.capabilities.unwrap_or_default(),
            invitation_code: request.invitation_code.clone(),
            status: AgentStatus::Pending,
            ..Default::default()
        };
        self.db.insert_agent(&agent).await?;

        // Mark invitation code as used
        self.db.use_invitation_code(&request.invitation_code, &agent.id).await?;

        // Create default spending policy (all zeros = nothing allowed until configured)
        self.db.insert_spending_policy(&SpendingPolicy {
            agent_id: agent.id.clone(),
            ..Default::default()
        }).await?;

        // Notify user
        self.notifications.send(NotificationEvent::AgentRegistered {
            agent_name: agent.name.clone(),
        }).await?;

        // Create approval request with expiration
        let now = Utc::now().timestamp();
        let approval = self.db.create_approval_request(ApprovalRequest {
            id: Uuid::new_v4().to_string(),
            agent_id: agent.id.clone(),
            request_type: ApprovalType::Registration,
            payload: serde_json::to_string(&agent)?,
            status: ApprovalStatus::Pending,
            expires_at: now + 86400,  // 24 hours for registration approvals
            ..Default::default()
        }).await?;

        self.event_bus.emit(Event::AgentRegistered(agent.id.clone()));

        Ok(AgentRegistrationResult {
            agent_id: agent.id,
            status: "pending".to_string(),
            message: "Registration submitted. Awaiting user approval. Poll /v1/agents/register/{id}/status for your token.".to_string(),
        })
    }

    /// Called when user approves a pending agent. Generates token, stores in encrypted delivery cache.
    pub async fn approve(&self, agent_id: &str) -> Result<AgentApprovalResult, AppError> {
        // Generate a secure random token
        let raw_token = format!("anb_{}", generate_secure_token(32));
        let token_hash = argon2_hash(&raw_token)?;
        let token_prefix = &raw_token[..12];

        self.db.activate_agent(agent_id, &token_hash, token_prefix).await?;

        // (UPDATED) Store token in encrypted delivery cache for 5 minutes
        let encrypted = encrypt_token(&raw_token)?;
        let now = Utc::now().timestamp();
        self.db.store_token_delivery(agent_id, &encrypted, now, now + 300).await?;

        self.event_bus.emit(Event::AgentApproved(agent_id.to_string()));

        // Token is also displayed to user in the UI (for manual delivery if needed)
        Ok(AgentApprovalResult {
            agent_id: agent_id.to_string(),
            token: raw_token,
        })
    }

    /// (NEW) Called when agent polls status endpoint after approval.
    /// Returns token exactly once, then deletes it.
    pub async fn retrieve_token(&self, agent_id: &str) -> Result<Option<String>, AppError> {
        let delivery = self.db.get_token_delivery(agent_id).await?;
        match delivery {
            Some(d) if !d.delivered && Utc::now().timestamp() <= d.expires_at => {
                let raw_token = decrypt_token(&d.encrypted_token)?;
                self.db.mark_token_delivered(agent_id).await?;
                self.db.delete_token_delivery(agent_id).await?;
                Ok(Some(raw_token))
            }
            _ => Ok(None), // Expired or already delivered
        }
    }
}
```

### 3.8 Notification System

Uses macOS native notifications via `tauri-plugin-notification`.

```rust
// core/notification.rs

pub enum NotificationEvent {
    TransactionConfirmed { agent_id: String, amount: String, recipient: String },
    TransactionFailed { agent_id: String, amount: String, error: String },
    ApprovalRequired { agent_id: String, amount: String, approval_id: String },
    AgentRegistered { agent_name: String },
    LimitIncreaseRequested { agent_id: String, requested: String },
}

pub struct NotificationManager {
    db: Arc<Database>,
    app_handle: tauri::AppHandle,
}

impl NotificationManager {
    pub async fn send(&self, event: NotificationEvent) -> Result<(), AppError> {
        let prefs = self.db.get_notification_preferences().await?;

        let should_send = match &event {
            NotificationEvent::TransactionConfirmed { amount, .. } => {
                prefs.on_all_tx
                    || (prefs.on_large_tx && amount.parse::<f64>()? >= prefs.large_tx_threshold)
            }
            NotificationEvent::TransactionFailed { .. } => prefs.on_errors,
            NotificationEvent::ApprovalRequired { .. } => true, // Always notify
            NotificationEvent::AgentRegistered { .. } => prefs.on_agent_registration,
            NotificationEvent::LimitIncreaseRequested { .. } => prefs.on_limit_requests,
        };

        if should_send {
            let (title, body) = event.to_notification_text();
            self.app_handle
                .notification()
                .builder()
                .title(&title)
                .body(&body)
                .show()?;
        }

        Ok(())
    }
}
```

### 3.9 MCP Server Implementation

> **Per-agent spawning (UPDATED):** The MCP server is spawned per-agent. The agent's bearer token is passed as a CLI argument or environment variable at spawn time. The server validates the token on startup and binds all operations to that agent's identity for the lifetime of the session.

The MCP server runs as a stdio-based server that Tauri spawns as a child process, or alternatively as an SSE server. It exposes tools that map 1:1 to the core service layer.

```rust
// api/mcp_server.rs (UPDATED)

pub struct McpServer {
    core: Arc<CoreServices>,
    agent_id: String,          // (NEW) bound to a single agent
    agent_token: String,       // (NEW) validated on startup
}

impl McpServer {
    /// Start MCP server for a specific agent.
    /// Token is passed via --token CLI arg or ANB_TOKEN env var.
    pub async fn start_stdio(core: Arc<CoreServices>, token: String) -> Result<(), AppError> {
        // Validate token on startup -- fail fast if invalid
        let agent_id = core.auth_service.validate_token(&token).await?;

        let server = McpServer { core, agent_id, agent_token: token };

        // Read JSON-RPC from stdin, write to stdout
        let stdin = tokio::io::stdin();
        let stdout = tokio::io::stdout();
        server.serve(stdin, stdout).await;
        Ok(())
    }
}
```

**MCP config example (~/.claude/mcp.json):**
```json
{
  "mcpServers": {
    "agent-neo-bank": {
      "command": "/path/to/agent-neo-bank-mcp",
      "args": ["--token", "anb_a3f8..."],
      "env": {}
    }
  }
}
```

**MCP Tool Definitions** (see Section 5.2 for full spec):
- `send_payment`
- `check_balance`
- `get_spending_limits`
- `request_limit_increase`
- `get_transaction_status`
- `list_my_transactions`
- `register_agent`

### 3.10 Unix Socket Server

```rust
// api/unix_socket.rs

pub async fn start_unix_socket(
    core: Arc<CoreServices>,
    path: &str,  // e.g., /tmp/agent-neo-bank.sock
) -> Result<(), AppError> {
    // Remove stale socket file if exists
    let _ = std::fs::remove_file(path);

    let listener = tokio::net::UnixListener::bind(path)?;

    // Set restrictive permissions (owner-only)
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;

    loop {
        let (stream, _) = listener.accept().await?;
        let core = core.clone();
        tokio::spawn(async move {
            // HTTP/1.1 over Unix socket using hyper
            handle_unix_connection(stream, core).await;
        });
    }
}
```

The Unix socket speaks the same HTTP/JSON protocol as the REST API. Agents connect with:
```
curl --unix-socket /tmp/agent-neo-bank.sock http://localhost/v1/send
```

### 3.11 Event Bus

Internal pub/sub for real-time frontend updates via Tauri events.

```rust
// core/event_bus.rs

pub enum Event {
    TransactionCreated(String),
    TransactionConfirmed(String),
    TransactionDenied(String),
    TransactionFailed(String),
    ApprovalCreated(ApprovalRequest),
    ApprovalResolved(String),
    AgentRegistered(String),
    AgentApproved(String),
    AgentSuspended(String),
    BalanceChanged,
}

pub struct EventBus {
    app_handle: tauri::AppHandle,
}

impl EventBus {
    pub fn emit(&self, event: Event) {
        let (name, payload) = event.to_tauri_event();
        let _ = self.app_handle.emit(&name, payload);
    }
}
```

Frontend listens via:
```typescript
import { listen } from '@tauri-apps/api/event';

listen('transaction-confirmed', (event) => {
  // Refresh transaction list
});
```

### 3.12 CoreServices Struct Definition **(NEW)**

The `CoreServices` struct holds all sub-services. It is stored as `Arc<CoreServices>` in Tauri state -- no wrapping `Mutex`. All methods take `&self`.

```rust
// core/services.rs

pub struct CoreServices {
    pub db: Arc<Database>,
    pub cli: Arc<dyn CliExecutable>,
    pub agent_registry: AgentRegistry,
    pub spending_policy: SpendingPolicyEngine,
    pub global_policy: GlobalPolicyEngine,
    pub tx_processor: TransactionProcessor,
    pub auth_service: AuthService,
    pub wallet_service: WalletService,
    pub approval_manager: ApprovalManager,
    pub notification_manager: NotificationManager,
    pub event_bus: EventBus,
    pub balance_cache: BalanceCache,
    pub invitation_manager: InvitationManager,
    pub config: AppConfig,
}

impl CoreServices {
    pub async fn new(
        db: Arc<Database>,
        cli: Arc<dyn CliExecutable>,
        config: AppConfig,
    ) -> Result<Self, AppError> {
        let event_bus = EventBus::new(/* app_handle passed separately */);
        let notification_manager = NotificationManager::new(db.clone());
        let balance_cache = BalanceCache::new(Duration::from_secs(30));

        let auth_service = AuthService::new(
            db.clone(),
            cli.clone(),
            Duration::from_secs(300), // 5-minute SHA-256 token cache TTL
        );

        let spending_policy = SpendingPolicyEngine::new(db.clone());
        let global_policy = GlobalPolicyEngine::new(db.clone());
        let approval_manager = ApprovalManager::new(db.clone(), notification_manager.clone());
        let invitation_manager = InvitationManager::new(db.clone());

        let tx_processor = TransactionProcessor::new(
            db.clone(),
            cli.clone(),
            spending_policy.clone(),
            approval_manager.clone(),
            notification_manager.clone(),
            event_bus.clone(),
        );

        let agent_registry = AgentRegistry::new(
            db.clone(),
            notification_manager.clone(),
            event_bus.clone(),
        );

        let wallet_service = WalletService::new(cli.clone(), balance_cache.clone());

        Ok(Self {
            db, cli, agent_registry, spending_policy, global_policy,
            tx_processor, auth_service, wallet_service, approval_manager,
            notification_manager, event_bus, balance_cache,
            invitation_manager, config,
        })
    }
}
```

> **No Mutex (UPDATED):** Sub-services use interior mutability only where needed. The SQLite connection pool (`r2d2::Pool`) handles its own concurrency. The balance cache uses `RwLock`. The auth token cache uses `RwLock`. Core service methods are all `&self`, never `&mut self`.

### 3.13 Auth Token Cache **(NEW)**

Two-tier authentication: argon2 for storage, SHA-256 for fast in-memory lookup.

```rust
// core/auth_service.rs (UPDATED)

pub struct AuthService {
    db: Arc<Database>,
    cli: Arc<dyn CliExecutable>,
    /// In-memory cache: SHA-256(bearer_token) -> agent_id, with 5-minute TTL
    token_cache: RwLock<HashMap<String, CachedAuth>>,
    cache_ttl: Duration,
}

struct CachedAuth {
    agent_id: String,
    cached_at: Instant,
}

impl AuthService {
    /// Validate a bearer token. Fast path: check SHA-256 cache.
    /// Slow path (cache miss): run argon2 verify against stored hashes.
    pub async fn validate_token(&self, raw_token: &str) -> Result<String, AppError> {
        let sha_hash = sha256_hex(raw_token);

        // Fast path: check in-memory cache
        {
            let cache = self.token_cache.read().await;
            if let Some(entry) = cache.get(&sha_hash) {
                if entry.cached_at.elapsed() < self.cache_ttl {
                    return Ok(entry.agent_id.clone());
                }
            }
        }

        // Slow path: load all active agent token hashes, argon2-verify
        let agents = self.db.get_active_agents_with_tokens().await?;
        for agent in agents {
            if argon2_verify(raw_token, &agent.api_token_hash)? {
                // Cache the result
                let mut cache = self.token_cache.write().await;
                cache.insert(sha_hash, CachedAuth {
                    agent_id: agent.id.clone(),
                    cached_at: Instant::now(),
                });
                return Ok(agent.id);
            }
        }

        Err(AppError::InvalidToken)
    }
}
```

> **Rationale:** Argon2 is intentionally slow (~100ms). Running it on every request creates a bottleneck. The SHA-256 cache provides O(1) lookups for repeat requests within the 5-minute TTL window. Argon2 only runs on cache miss (first request, token rotation, cache expiry).

### 3.14 Stale Approval Cleanup **(NEW)**

A periodic background task (every 5 minutes) expires old approval requests and fails their associated transactions.

```rust
// core/approval_manager.rs (UPDATED)

impl ApprovalManager {
    /// Run every 5 minutes. Expires approvals past their expires_at timestamp.
    pub async fn run_cleanup_loop(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(300));
        loop {
            interval.tick().await;
            self.expire_stale_approvals().await;
        }
    }

    async fn expire_stale_approvals(&self) {
        let now = Utc::now().timestamp();
        // Find all pending approvals where expires_at < now
        let expired = self.db.expire_approvals(now).await;
        for approval in expired.unwrap_or_default() {
            // Fail associated transactions
            if let Some(tx_id) = &approval.tx_id {
                self.db.fail_transaction(tx_id, "Approval request expired").await.ok();
            }
        }
    }
}
```

---

## 4. React Frontend Architecture

### 4.1 Page / Route Structure

```
/                          -> Redirect to /dashboard (if authed) or /onboarding
/onboarding                -> Onboarding (auth + fund flow)
/dashboard                 -> Dashboard (main view)
/agents                    -> Agent list
/agents/:id                -> Agent detail view
/transactions              -> Full transaction history
/approvals                 -> Pending approval queue
/fund                      -> Deposit / onramp page
/settings                  -> App settings
```

**Router:** `react-router-dom` v7 with a layout route for the Shell.

```
<Routes>
  <Route element={<PublicLayout />}>
    <Route path="/onboarding" element={<Onboarding />} />
  </Route>
  <Route element={<AuthGuard />}>
    <Route element={<Shell />}>
      <Route path="/dashboard" element={<Dashboard />} />
      <Route path="/agents" element={<AgentList />} />
      <Route path="/agents/:id" element={<AgentDetail />} />
      <Route path="/transactions" element={<Transactions />} />
      <Route path="/approvals" element={<Approvals />} />
      <Route path="/fund" element={<Fund />} />
      <Route path="/settings" element={<Settings />} />
    </Route>
  </Route>
</Routes>
```

### 4.2 Component Hierarchy

```
App
+-- PublicLayout
|   +-- Onboarding
|       +-- WelcomeStep
|       +-- EmailStep
|       +-- OtpStep
|       +-- FundStep (optional)
|
+-- AuthGuard
    +-- Shell
        +-- Sidebar (nav links, balance display)
        +-- Header (title, network badge, notification bell)
        +-- <Outlet /> (page content)
            |
            +-- Dashboard
            |   +-- BalanceCard
            |   +-- SpendingChart (by time period)
            |   +-- BudgetUtilization (bar chart per agent)
            |   +-- RecentTransactions
            |   +-- AgentStatusGrid
            |
            +-- AgentList
            |   +-- AgentCard (repeated)
            |
            +-- AgentDetail
            |   +-- Agent info header
            |   +-- SpendingLimitsEditor
            |   +-- AllowlistEditor
            |   +-- AgentActivityFeed
            |   +-- BudgetUtilization (this agent only)
            |
            +-- Transactions
            |   +-- FilterBar (date range, agent, type, status)
            |   +-- TransactionTable
            |       +-- TransactionRow (repeated)
            |
            +-- Approvals
            |   +-- ApprovalQueue
            |       +-- ApprovalCard (repeated)
            |
            +-- Fund
            |   +-- Wallet address display + copy
            |   +-- Coinbase Onramp iframe
            |
            +-- Settings
                +-- Network toggle
                +-- Notification preferences
                +-- API server status
                +-- Export data
```

### 4.3 State Management

**Zustand** for lightweight, TypeScript-friendly state management. Each domain gets its own store.

```typescript
// stores/authStore.ts
interface AuthState {
  isAuthenticated: boolean;
  email: string | null;
  checkAuth: () => Promise<void>;
  login: (email: string) => Promise<void>;
  verify: (email: string, otp: string) => Promise<void>;
  logout: () => Promise<void>;
}

// stores/agentStore.ts
interface AgentState {
  agents: Agent[];
  loading: boolean;
  fetchAgents: () => Promise<void>;
  approveAgent: (id: string) => Promise<AgentApprovalResult>;
  suspendAgent: (id: string) => Promise<void>;
  // ...
}

// stores/transactionStore.ts
interface TransactionState {
  transactions: Transaction[];
  filters: TransactionFilters;
  loading: boolean;
  fetchTransactions: (filters?: TransactionFilters) => Promise<void>;
  // ...
}
```

**Real-time updates:** Tauri event listeners in hooks update Zustand stores:

```typescript
// hooks/useTauriEvent.ts
export function useTauriEvent<T>(event: string, handler: (payload: T) => void) {
  useEffect(() => {
    const unlisten = listen<T>(event, (e) => handler(e.payload));
    return () => { unlisten.then(fn => fn()); };
  }, [event, handler]);
}
```

### 4.4 shadcn/ui Components

Core shadcn/ui components used throughout:

| Component | Usage |
|---|---|
| `Button` | All actions, with variants (default, destructive, outline, ghost) |
| `Card` | Dashboard cards, agent cards, approval cards |
| `Table` | Transaction history, agent list |
| `Badge` | Status indicators (active, pending, suspended, confirmed, failed) |
| `Dialog` | Confirmation dialogs (approve, deny, suspend) |
| `Sheet` | Transaction detail slide-over |
| `Input` / `Textarea` | Forms |
| `Select` | Filters, dropdowns |
| `Tabs` | Agent detail sections |
| `Avatar` | Agent icons |
| `Progress` | Budget utilization bars |
| `Toast` | Non-blocking notifications |
| `Separator` | Visual dividers |
| `DropdownMenu` | Context menus |
| `Switch` | Toggles (notifications, network) |
| `Skeleton` | Loading states |
| `Command` | Search/command palette |
| `Alert` | Warnings, errors |
| `ScrollArea` | Scrollable containers |

### 4.5 Key Screens

**Onboarding Flow:**
```
+--------------------------------------------------+
|                                                  |
|            Welcome to Agent Neo Bank             |
|                                                  |
|     A banking app for your AI agents.            |
|     You control the money.                       |
|     They do the work.                            |
|                                                  |
|            [ Get Started ]                       |
|                                                  |
+--------------------------------------------------+
         |
         v
+--------------------------------------------------+
|                                                  |
|            What's your email?                    |
|                                                  |
|     +------------------------------------+       |
|     | agent-banker@example.com           |       |
|     +------------------------------------+       |
|                                                  |
|     We'll send a one-time code.                  |
|                                                  |
|            [ Send Code ]                         |
|                                                  |
+--------------------------------------------------+
         |
         v
+--------------------------------------------------+
|                                                  |
|            Enter your code                       |
|                                                  |
|     +--+ +--+ +--+ +--+ +--+ +--+              |
|     |  | |  | |  | |  | |  | |  |              |
|     +--+ +--+ +--+ +--+ +--+ +--+              |
|                                                  |
|     Sent to agent-banker@example.com             |
|                                                  |
|            [ Verify ]                            |
|                                                  |
+--------------------------------------------------+
         |
         v
+--------------------------------------------------+
|                                                  |
|            You're in!                            |
|                                                  |
|     Your wallet address:                         |
|     0x7a3b...f42d  [Copy]                        |
|                                                  |
|     [ Fund via Coinbase ]  [ Fund Later ]        |
|                                                  |
+--------------------------------------------------+
```

**Dashboard:**
```
+------------------------------------------------------------------+
| [=] Agent Neo Bank              Base Sepolia    [bell]   [gear]  |
+----------+-------------------------------------------------------+
|          |                                                       |
| Dashboard|   +------------------+  +-------------------------+   |
| Agents   |   | Global Balance   |  | Budget Utilization      |   |
| Txns     |   | $1,247.83 USDC   |  | Agent A  [=====---] 62%|   |
| Approvals|   | +2.4% this week  |  | Agent B  [===-----] 38%|   |
| Fund     |   +------------------+  | Agent C  [=--------] 12%|   |
| Settings |                         +-------------------------+   |
|          |   +------------------------------------------------+  |
|   (2)    |   | Spending This Month            $823.47         |  |
| pending  |   |                                                |  |
|          |   | [Chart: daily spending line chart]              |  |
|          |   +------------------------------------------------+  |
|          |                                                       |
|          |   +------------------------------------------------+  |
|          |   | Recent Transactions                            |  |
|          |   +------+--------+----------+-------+------+------+  |
|          |   | Time | Agent  | To       | Amount| Type | Stat |  |
|          |   +------+--------+----------+-------+------+------+  |
|          |   | 2:34 | Claude | 0x4f2... | $5.00 | send | ok   |  |
|          |   | 2:12 | Claude | x402.dev | $0.50 | send | ok   |  |
|          |   | 1:48 | Devin  | 0x88a... | $12   | send | deny |  |
|          |   +------+--------+----------+-------+------+------+  |
+----------+-------------------------------------------------------+
```

**Agent Detail:**
```
+------------------------------------------------------------------+
| [<] Agent Detail: Claude Code                            [gear]  |
+----------+-------------------------------------------------------+
|          |                                                       |
| Nav      |   Status: Active          Since: Feb 24, 2026        |
|          |   Token: anb_a3f8...      Last Active: 2 min ago     |
|          |                                                       |
|          |   [Spending Limits]  [Activity]  [Allowlist]          |
|          |   +-------------------------------------------------+ |
|          |   | Per Transaction Max:  $25.00      [Edit]        | |
|          |   | Daily Cap:           $100.00      [Edit]        | |
|          |   | Weekly Cap:          $500.00      [Edit]        | |
|          |   | Monthly Cap:         $1,500.00    [Edit]        | |
|          |   | Auto-approve below:  $10.00       [Edit]        | |
|          |   +-------------------------------------------------+ |
|          |                                                       |
|          |   Budget Usage (This Month):                          |
|          |   [=================---------] $823 / $1,500 (55%)    |
|          |                                                       |
|          |   Recent Activity:                                    |
|          |   +-----------------------------------------------+   |
|          |   | 2:34pm  Sent $5.00 to 0x4f2...  (auto-approved)| |
|          |   | 2:12pm  Sent $0.50 to x402.dev  (auto-approved)| |
|          |   | 1:05pm  Sent $25.00 to 0x88a... (user approved)| |
|          |   +-----------------------------------------------+   |
|          |                                                       |
|          |   [ Suspend Agent ]                                   |
+----------+-------------------------------------------------------+
```

---

## 5. Agent Communication Protocol

### 5.1 REST API Endpoints

**Base URL:** `http://localhost:7402/v1`

**Authentication:** Bearer token in `Authorization` header.
Exception: `/v1/agents/register` requires an invitation code instead of a bearer token. **(UPDATED)**

#### Agent Registration **(UPDATED)**

```
POST /v1/agents/register
Content-Type: application/json

{
  "name": "Claude Code",
  "description": "AI coding assistant that helps with development tasks",
  "purpose": "Coding assistance and development automation",
  "agent_type": "coding_assistant",
  "capabilities": ["send", "receive"],
  "invitation_code": "INV-a8f3b2c1"        // (NEW) required
}

Response 201:
{
  "agent_id": "a1b2c3d4...",
  "status": "pending",
  "message": "Registration submitted. Awaiting user approval. Poll /v1/agents/register/{id}/status for your token."
}

Response 400 (invalid/expired invitation code):
{
  "error": "invalid_invitation_code",
  "message": "The invitation code is invalid or has already been used."
}
```

#### Check Registration Status **(UPDATED -- token delivery)**

```
GET /v1/agents/register/{agent_id}/status

Response 200 (still pending):
{
  "agent_id": "a1b2c3d4...",
  "status": "pending"
}

Response 200 (approved -- token returned ONCE then deleted):
{
  "agent_id": "a1b2c3d4...",
  "status": "active",
  "token": "anb_a3f8..."     // (UPDATED) returned exactly once, then deleted from cache
}

Response 200 (approved but token already delivered or expired):
{
  "agent_id": "a1b2c3d4...",
  "status": "active",
  "token": null,
  "message": "Token was already delivered or has expired. Contact the user to re-register."
}
```

#### Send Payment **(UPDATED -- always async 202)**

> **Design note:** `/v1/send` always returns `202 Accepted` with a `tx_id`. The agent polls `GET /v1/transactions/{tx_id}` for final status. Optionally, the agent can include a `webhook_url` to receive a callback.

```
POST /v1/send
Authorization: Bearer anb_a3f8...
Content-Type: application/json

{
  "to": "0x4f2a...9b3c",
  "amount": 5.00,                           // (UPDATED) Decimal, not String. Parsed via serde.
  "asset": "USDC",                          // optional, defaults to USDC
  "memo": "Payment for x402 API call",      // optional
  "category": "api_services",               // optional
  "description": "Paying for Claude API usage for code review task",  // (NEW)
  "service_name": "x402 API Gateway",       // (NEW)
  "service_url": "https://x402.dev",        // (NEW)
  "reason": "Need API access to complete code review",               // (NEW)
  "webhook_url": "http://localhost:8080/webhook/tx-status"           // (NEW) optional
}

Response 202 (always -- auto-approved, executing in background):
{
  "tx_id": "tx_...",
  "status": "executing",
  "message": "Transaction accepted. Poll GET /v1/transactions/{tx_id} for status."
}

Response 202 (queued for user approval):
{
  "tx_id": "tx_...",
  "status": "awaiting_approval",
  "message": "Transaction queued for user approval. Poll GET /v1/transactions/{tx_id} for status."
}

Response 403 (denied by policy -- immediate rejection):
{
  "error": "policy_denied",
  "message": "Amount exceeds daily cap of $100.00",
  "tx_id": "tx_..."
}

Response 403 (global kill switch active):
{
  "error": "kill_switch_active",
  "message": "Emergency kill switch is active. All agent operations are suspended."
}
```

> **Amount parsing (UPDATED):** The `SendRequest` struct uses `amount: Decimal` (from `rust_decimal`), not `String`. Deserialization happens at the API boundary via serde. Internal code never parses amount strings.

#### Check Balance **(UPDATED -- cached, per-agent visibility)**

> **Balance caching (NEW):** CLI responses are cached with 30-second TTL. One CLI call per TTL period regardless of how many agents poll.

> **Agent visibility (NEW):** Balance visibility is configurable per-agent in spending policy. Some agents don't need to know the wallet balance.

```
GET /v1/balance
Authorization: Bearer anb_a3f8...

Response 200 (balance visible):
{
  "balance": "1247.83",
  "asset": "USDC",
  "network": "base-sepolia",
  "address": "0x7a3b...f42d"
}

Response 200 (balance hidden for this agent):
{
  "balance": null,
  "balance_visible": false,
  "asset": "USDC",
  "network": "base-sepolia",
  "address": "0x7a3b...f42d",
  "message": "Balance visibility is disabled for this agent."
}
```

#### Get Spending Limits

```
GET /v1/spending/limits
Authorization: Bearer anb_a3f8...

Response 200:
{
  "per_tx_max": "25.00",
  "daily_cap": "100.00",
  "weekly_cap": "500.00",
  "monthly_cap": "1500.00",
  "auto_approve_max": "10.00",
  "usage": {
    "today": "52.30",
    "this_week": "312.80",
    "this_month": "823.47"
  },
  "allowlist": ["0x4f2a...9b3c", "x402.dev"]
}
```

#### Request Limit Increase

```
POST /v1/spending/request-increase
Authorization: Bearer anb_a3f8...
Content-Type: application/json

{
  "field": "daily_cap",
  "requested_value": "200.00",
  "reason": "Need to make more API calls for large refactoring task"
}

Response 202:
{
  "approval_id": "apr_...",
  "status": "pending",
  "message": "Limit increase request submitted for user review."
}
```

#### Get Transaction Status

```
GET /v1/transactions/{tx_id}
Authorization: Bearer anb_a3f8...

Response 200:
{
  "tx_id": "tx_...",
  "status": "confirmed",
  "amount": "5.00",
  "asset": "USDC",
  "to": "0x4f2a...9b3c",
  "chain_tx_hash": "0xabc...",
  "created_at": "2026-02-27T14:34:00Z",
  "category": "api_services",
  "memo": "Payment for x402 API call"
}
```

#### List My Transactions

```
GET /v1/transactions?limit=20&offset=0&type=send&status=confirmed
Authorization: Bearer anb_a3f8...

Response 200:
{
  "transactions": [...],
  "total": 47,
  "limit": 20,
  "offset": 0
}
```

#### List Agents **(NEW)**

```
GET /v1/agents?limit=20&offset=0
Authorization: Bearer anb_a3f8...

Response 200:
{
  "agents": [...],
  "total": 5,
  "limit": 20,
  "offset": 0
}
```

> **Pagination (UPDATED):** All list endpoints support `limit` and `offset` query parameters -- agents, approvals, and transactions.

#### List Approvals **(NEW)**

```
GET /v1/approvals?limit=20&offset=0&status=pending
Authorization: Bearer anb_a3f8...

Response 200:
{
  "approvals": [...],
  "total": 3,
  "limit": 20,
  "offset": 0
}
```

#### Generate Invitation Code **(NEW -- Tauri IPC only)**

```
// Tauri IPC command (not REST -- user-facing only)
invoke('generate_invitation_code', { label: "For Claude Code", expiresInHours: 24 })

Response:
{
  "code": "INV-a8f3b2c1",
  "expires_at": 1740700800,
  "label": "For Claude Code"
}
```

#### Global Policy **(NEW -- Tauri IPC only)**

```
// Read global policy
invoke('get_global_policy')

Response:
{
  "daily_cap": "500.00",
  "weekly_cap": "2000.00",
  "monthly_cap": "5000.00",
  "min_reserve_balance": "100.00",
  "kill_switch_active": false
}

// Update global policy
invoke('set_global_policy', {
  daily_cap: "500.00",
  weekly_cap: "2000.00",
  monthly_cap: "5000.00",
  min_reserve_balance: "100.00"
})

// Emergency kill switch
invoke('toggle_kill_switch', { active: true, reason: "Suspicious activity detected" })
```

#### Health Check (no auth)

```
GET /v1/health

Response 200:
{
  "status": "ok",
  "version": "0.1.0",
  "network": "base-sepolia",
  "mock_mode": false
}
```

### 5.2 MCP Tool Definitions

The MCP server exposes the following tools. Each tool maps to the same core service call as the REST endpoint.

```json
{
  "tools": [
    {
      "name": "send_payment",
      "description": "Send a USDC payment to a recipient address. Subject to your spending limits and approval policies.",
      "inputSchema": {
        "type": "object",
        "properties": {
          "to": { "type": "string", "description": "Recipient wallet address (0x...)" },
          "amount": { "type": "string", "description": "Amount in USDC (e.g., '5.00')" },
          "memo": { "type": "string", "description": "Optional memo describing the payment purpose" },
          "category": { "type": "string", "description": "Optional category (e.g., 'api_services', 'infrastructure')" }
        },
        "required": ["to", "amount"]
      }
    },
    {
      "name": "check_balance",
      "description": "Check the current wallet balance, network, and deposit address.",
      "inputSchema": { "type": "object", "properties": {} }
    },
    {
      "name": "get_spending_limits",
      "description": "View your current spending limits and how much you've used in each period.",
      "inputSchema": { "type": "object", "properties": {} }
    },
    {
      "name": "request_limit_increase",
      "description": "Request an increase to one of your spending limits. Requires user approval.",
      "inputSchema": {
        "type": "object",
        "properties": {
          "field": {
            "type": "string",
            "enum": ["per_tx_max", "daily_cap", "weekly_cap", "monthly_cap", "auto_approve_max"],
            "description": "Which limit to request an increase for"
          },
          "requested_value": { "type": "string", "description": "Requested new limit value in USDC" },
          "reason": { "type": "string", "description": "Why you need a higher limit" }
        },
        "required": ["field", "requested_value", "reason"]
      }
    },
    {
      "name": "get_transaction_status",
      "description": "Check the status of a specific transaction by its ID.",
      "inputSchema": {
        "type": "object",
        "properties": {
          "tx_id": { "type": "string", "description": "Transaction ID to look up" }
        },
        "required": ["tx_id"]
      }
    },
    {
      "name": "list_my_transactions",
      "description": "List your recent transactions with optional filters.",
      "inputSchema": {
        "type": "object",
        "properties": {
          "limit": { "type": "number", "description": "Max results (default 20)" },
          "type": { "type": "string", "enum": ["send", "receive", "earn"] },
          "status": { "type": "string", "enum": ["pending", "confirmed", "failed", "denied"] }
        }
      }
    }
  ]
}
```

### 5.3 Authentication Flow for Agents **(UPDATED)**

**Self-Registration Flow (with invitation codes and encrypted token delivery):**

```
Agent                          API Server              User (Tauri UI)
  |                               |                         |
  |                               |                         | User generates
  |                               |                         | invitation code
  |                               |                         | in Settings UI
  |                               |                         |
  |  POST /v1/agents/register     |                         |
  |  { name, description,         |                         |
  |    purpose, agent_type,        |                         |
  |    capabilities,               |                         |
  |    invitation_code }           |                         |
  |------------------------------>|                         |
  |                               |  Validate invitation    |
  |                               |  Store agent (pending)  |
  |                               |  Create approval req    |
  |                               |  Send OS notification   |
  |                               |------------------------>|
  |  201 { agent_id, "pending" }  |                         |
  |<------------------------------|                         |
  |                               |                         |
  |  (Agent polls status)         |                         |
  |  GET /v1/agents/register/     |     User clicks         |
  |       {id}/status             |     "Approve" + sets    |
  |------------------------------>|     spending limits     |
  |  200 { "pending" }            |     + balance visibility|
  |<------------------------------|<------------------------|
  |                               |                         |
  |                               |  Generate token         |
  |                               |  Store encrypted in     |
  |                               |  delivery cache (5 min) |
  |                               |  Display to user too    |
  |                               |                         |
  |  (Poll again, within 5 min)   |                         |
  |  GET /v1/agents/register/     |                         |
  |       {id}/status             |                         |
  |------------------------------>|                         |
  |  200 { "active", token }      |  Token returned ONCE    |
  |<------------------------------|  then deleted from cache |
  |                               |                         |
  |  (Agent stores token locally) |                         |
  |                               |                         |
  |  All future requests use:     |                         |
  |  Authorization: Bearer anb_...|                         |
```

**Token Format:** `anb_` prefix + 32 random alphanumeric characters.
**Token Storage:** Only the argon2 hash is stored in SQLite. The raw token is encrypted in a delivery cache for 5 minutes.

**Token Delivery (UPDATED):** The token is held encrypted for 5 minutes after user approval. The poll endpoint returns it exactly once then deletes it. After the 5-minute window, the token is gone and the agent must re-register. The user also sees the token in the UI for manual delivery.

**Token Validation (UPDATED):** Two-tier auth: the auth middleware first checks an in-memory SHA-256 cache (`token -> agent_id`, 5-minute TTL). On cache miss, it loads all active agent token hashes and runs argon2 verify. The SHA-256 cache is populated on first successful verify. This avoids running argon2 (~100ms) on every request.

### 5.4 Agent Discovery **(UPDATED)**

Agents discover the Neo Bank API through the **Agent Registration Skill** file (see Section 10) or documentation placed in well-known locations:

**Agent skill file (preferred for Claude Code agents):**
The file `skills/agent-neo-bank.md` (or `~/.claude/skills/agent-neo-bank.md`) provides comprehensive instructions for agents on how to register, authenticate, and use the API. See Section 10 for full contents.

**claude.md (simpler alternative for Claude Code agents):**
```markdown
## Agent Neo Bank

This workspace has access to Agent Neo Bank for making payments.

- API: http://localhost:7402/v1
- Auth: Bearer token in Authorization header
- Your token: <token here>
- Skill: See skills/agent-neo-bank.md for full registration and usage instructions.

To make a payment:
POST http://localhost:7402/v1/send
{ "to": "0x...", "amount": 5.00 }
```

**MCP config (~/.claude/mcp.json for Claude Code):**
```json
{
  "mcpServers": {
    "agent-neo-bank": {
      "command": "/path/to/agent-neo-bank-mcp",
      "args": ["--token", "anb_..."]
    }
  }
}
```

> **MCP auth (UPDATED):** The MCP server is spawned per-agent with the token passed as a CLI argument. The server validates the token on startup and binds all operations to that agent's identity.

---

## 6. Data Flow Diagrams

### 6.1 Agent Makes a Payment Request **(UPDATED -- async 202 model)**

```
  Agent                REST/MCP/Socket         Core Services           CLI           SQLite
    |                       |                       |                   |               |
    | POST /v1/send         |                       |                   |               |
    | {to, amount(Decimal), |                       |                   |               |
    |  description, service,|                       |                   |               |
    |  reason, webhook_url} |                       |                   |               |
    |---------------------->|                       |                   |               |
    |                       | Validate token        |                   |               |
    |                       | (SHA-256 cache first, |                   |               |
    |                       |  argon2 on miss)      |                   |               |
    |                       |---------------------->|                   |               |
    |                       |                       | Create TX record  |               |
    |                       |                       | (status=pending,  |               |
    |                       |                       |  UTC period keys) |               |
    |                       |                       |---------------------------------->|
    |                       |                       |                   |               |
    |                       |                       | BEGIN EXCLUSIVE   |               |
    |                       |                       | Check global      |               |
    |                       |                       |  policy (kill     |               |
    |                       |                       |  switch, reserve, |               |
    |                       |                       |  global caps)     |               |
    |                       |                       | Check agent       |               |
    |                       |                       |  policy + ledger  |               |
    |                       |                       | COMMIT            |               |
    |                       |                       |<----------------------------------|
    |                       |                       |                   |               |
    |           [if auto-approved -- async execution]                   |               |
    |  202 {tx_id,          |                       |                   |               |
    |   "executing"}        |                       |                   |               |
    |<----------------------|                       |                   |               |
    |                       |                       | (background task) |               |
    |                       |                       | awal send ...     |               |
    |                       |                       |------------------>|               |
    |                       |                       |   {tx_hash}       |               |
    |                       |                       |<------------------|               |
    |                       |                       | BEGIN EXCLUSIVE   |               |
    |                       |                       | Confirm TX +      |               |
    |                       |                       | Update agent +    |               |
    |                       |                       | global ledgers    |               |
    |                       |                       | COMMIT            |               |
    |                       |                       |---------------------------------->|
    |                       |                       | Emit event        |               |
    |                       |                       |---------> [frontend via Tauri]    |
    |                       |                       | Fire webhook      |               |
    |                       |                       |---------> [agent webhook_url]     |
    |                       |                       |                   |               |
    | (Agent polls status)  |                       |                   |               |
    | GET /v1/transactions/ |                       |                   |               |
    |     {tx_id}           |                       |                   |               |
    |---------------------->|                       |                   |               |
    | 200 {confirmed,       |                       |                   |               |
    |  chain_tx_hash}       |                       |                   |               |
    |<----------------------|                       |                   |               |
    |                       |                       |                   |               |
    |           [if requires approval]              |                   |               |
    |  202 {tx_id,          |                       |                   |               |
    |   "awaiting_approval"}|                       |                   |               |
    |<----------------------|                       |                   |               |
    |                       |                       | Create approval   |               |
    |                       |                       | (with expires_at) |               |
    |                       |                       |---------------------------------->|
    |                       |                       | OS notification   |               |
    |                       |                       |---------> [macOS notification]    |
    |                       |                       |                   |               |
    |                       |   [User approves in UI]                   |               |
    |                       |                       | Execute send      |               |
    |                       |                       |------------------>|               |
    |                       |                       |<------------------|               |
    |                       |                       | Atomic confirm +  |               |
    |                       |                       | ledger update     |               |
    |                       |                       |---------------------------------->|
```

### 6.2 Agent Self-Registers **(UPDATED -- invitation codes, encrypted token delivery)**

```
  Agent                REST API              Core Services            SQLite        User (UI)
    |                     |                       |                     |               |
    |                     |                       |                     |               |
    |                     |                       |                     |  User generates|
    |                     |                       |                     |  invitation   |
    |                     |                       |                     |  code in UI   |
    |                     |                       |                     |  "INV-a8f3..."  |
    |                     |                       |                     |               |
    | POST /register      |                       |                     |               |
    | {name, purpose,     |                       |                     |               |
    |  agent_type, desc,  |                       |                     |               |
    |  capabilities,      |                       |                     |               |
    |  invitation_code}   |                       |                     |               |
    |------------------->|                       |                     |               |
    |                     | register()            |                     |               |
    |                     |--------------------->|                     |               |
    |                     |                       | Validate invite     |               |
    |                     |                       | code                |               |
    |                     |                       |------------------->|               |
    |                     |                       | INSERT agent        |               |
    |                     |                       | (rich metadata)     |               |
    |                     |                       |------------------->|               |
    |                     |                       | Mark code used      |               |
    |                     |                       |------------------->|               |
    |                     |                       | INSERT policy       |               |
    |                     |                       | (all zeros)         |               |
    |                     |                       |------------------->|               |
    |                     |                       | INSERT approval_req |               |
    |                     |                       | (with expires_at)   |               |
    |                     |                       |------------------->|               |
    |                     |                       | OS notification     |               |
    |                     |                       |------------------------------>     |
    |                     |                       | Tauri event         |               |
    |                     |                       |----------------------------->[bell] |
    | 201 {pending}       |                       |                     |               |
    |<--------------------|                       |                     |               |
    |                     |                       |                     |               |
    |                     |                       |                     | User sees rich|
    |                     |                       |                     | agent info:   |
    |                     |                       |                     | name, purpose,|
    |                     |                       |                     | type, caps    |
    |                     |                       |                     |               |
    |                     |                       |                     | User sets     |
    |                     |                       |                     | limits +      |
    |                     |                       |                     | balance vis.  |
    |                     |                       |                     | clicks Approve|
    |                     |                       |<----------------------------[click] |
    |                     |                       | approve()           |               |
    |                     |                       | Generate token      |               |
    |                     |                       | Hash(argon2)+store  |               |
    |                     |                       | Encrypt + cache     |               |
    |                     |                       | (5 min TTL)         |               |
    |                     |                       |------------------->|               |
    |                     |                       | Show token to user  |               |
    |                     |                       |---------------------------->[modal] |
    |                     |                       |                     |               |
    | GET /register/      |                       |                     |               |
    |   {id}/status       |                       |                     |               |
    |------------------->|                       |                     |               |
    |                     |--------------------->|                     |               |
    |                     |                       | Decrypt token       |               |
    |                     |                       | Delete from cache   |               |
    |                     |                       |------------------->|               |
    | 200 {active, token} |  (token returned once, then deleted)       |               |
    |<--------------------|                       |                     |               |
```

### 6.3 User Funds Wallet

```
  User (UI)             Tauri IPC          Core Services        CLI / External
    |                       |                    |                    |
    | Navigate to /fund     |                    |                    |
    |                       |                    |                    |
    | invoke(get_address)   |                    |                    |
    |--------------------->|                    |                    |
    |                       |------------------->|                    |
    |                       |                    | awal wallet addr   |
    |                       |                    |------------------->|
    |                       |                    |<-------------------|
    |<---------------------|                    |                    |
    |                       |                    |                    |
    | [Display address]     |                    |                    |
    | "Send USDC to         |                    |                    |
    |  0x7a3b...f42d"       |                    |                    |
    |                       |                    |                    |
    | --- OR ---            |                    |                    |
    |                       |                    |                    |
    | invoke(get_onramp_url)|                    |                    |
    |--------------------->|                    |                    |
    |                       |------------------->|                    |
    |                       |                    | Build Coinbase     |
    |                       |                    | Onramp URL with    |
    |                       |                    | wallet address     |
    |<---------------------|                    |                    |
    |                       |                    |                    |
    | [Render Onramp iframe]|                    |                    |
    | User completes        |                    |                    |
    | purchase in widget    |                    |                    |
    |                       |                    |                    |
    | (Balance updates      |                    |                    |
    |  on next poll or      |                    |                    |
    |  manual refresh)      |                    |                    |
```

### 6.4 Agent Requests Limit Increase

```
  Agent              REST/MCP            Core Services           SQLite          User (UI)
    |                   |                      |                    |                |
    | POST /spending/   |                      |                    |                |
    | request-increase  |                      |                    |                |
    | {field, value,    |                      |                    |                |
    |  reason}          |                      |                    |                |
    |----------------->|                      |                    |                |
    |                   |--------------------->|                    |                |
    |                   |                      | INSERT approval    |                |
    |                   |                      | (type=limit_incr)  |                |
    |                   |                      |------------------>|                |
    |                   |                      | OS notification    |                |
    |                   |                      |----------------------------->[bell] |
    |                   |                      | Tauri event        |                |
    | 202 {pending}     |                      |---------------------------->[feed] |
    |<-----------------|                      |                    |                |
    |                   |                      |                    |                |
    |                   |                      |                    | User reviews   |
    |                   |                      |                    | in Approvals:  |
    |                   |                      |                    |                |
    |                   |                      |                    | "Claude Code   |
    |                   |                      |                    |  requests      |
    |                   |                      |                    |  daily_cap     |
    |                   |                      |                    |  $100 -> $200  |
    |                   |                      |                    |  Reason: ..."  |
    |                   |                      |                    |                |
    |                   |                      |                    | [Approve] [Deny]
    |                   |                      |<---------------------------[click]  |
    |                   |                      | UPDATE policy      |                |
    |                   |                      | (daily_cap=200)    |                |
    |                   |                      |------------------>|                |
    |                   |                      | UPDATE approval    |                |
    |                   |                      | (status=approved)  |                |
    |                   |                      |------------------>|                |
```

---

## 7. Security Model

### 7.1 Agent Token Scoping

- Each agent gets a unique, randomly generated API token.
- Tokens are prefixed `anb_` for easy identification.
- Only the argon2 hash is stored; the raw token is delivered via encrypted short-lived cache. **(UPDATED)**
- Token validation uses a two-tier cache (SHA-256 in-memory with 5min TTL, argon2 on cache miss). **(UPDATED)**
- An agent's token grants access ONLY to operations scoped to that agent:
  - Can send payments (within their limits)
  - Can check the global balance IF `balance_visible` is enabled for the agent **(UPDATED)**
  - Can view their own transactions
  - Can view their own spending limits
  - Can request limit increases
  - **Cannot** view other agents' data
  - **Cannot** modify their own limits directly
  - **Cannot** approve their own transactions

### 7.2 Tauri Permission Scoping **(UPDATED)**

Tauri v2 uses a capabilities system. We define minimal permissions:

> **Note (UPDATED):** `core:default` permits all registered Tauri IPC commands. Custom IPC commands (like `generate_invitation_code`, `toggle_kill_switch`, etc.) are automatically included. Add `tauri-plugin-clipboard-manager` to Cargo.toml dependencies.

```json
// capabilities/default.json (UPDATED)
{
  "identifier": "default",
  "description": "Default window capabilities",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "notification:default",
    "shell:allow-open",
    "clipboard-manager:allow-write",
    "clipboard-manager:allow-read"
  ]
}
```

**Denied by default:**
- `fs:*` (no arbitrary filesystem access)
- `shell:allow-execute` (only the CLI wrapper uses process spawning, via Rust directly)
- `http:*` (frontend does not make HTTP calls; all goes through Tauri IPC)

### 7.3 CLI Command Whitelisting

The CLI executor only allows a hardcoded set of commands. Any attempt to pass arbitrary arguments is rejected at the type level:

```rust
pub enum AwalCommand {
    AuthLogin { email: String },
    AuthVerify { email: String, otp: String },
    AuthStatus,
    AuthLogout,
    WalletBalance,
    WalletAddress,
    Send { to: String, amount: Decimal, asset: String },  // (UPDATED) Decimal type
    ConfigGet { key: String },
    ConfigSet { key: String, value: String },
}
```

- The `to` field is validated as a hex address before execution.
- The `amount` field is a `Decimal` -- validated as positive at the API boundary. Converted to string only when passed to CLI args. **(UPDATED)**
- The `asset` field is validated against a known list (`USDC`, `ETH`).
- No shell expansion or string interpolation: arguments are passed as a `Vec<String>` to `Command::args()`.

### 7.4 Rate Limiting **(UPDATED -- Phase 1, invitation-code-based)**

> **Moved to Phase 1 (UPDATED):** Basic rate limiting is included from day one, not deferred to Phase 4.

```
Transport       Limit                    Scope
-----------------------------------------------------------
REST API        60 requests/minute       Per agent token
MCP Server      30 tool calls/minute     Per session
Unix Socket     60 requests/minute       Per connection
Registration    Invitation code required (NEW)   Per code (one-time use)
```

**Registration rate limiting (UPDATED):** IP-based rate limiting is meaningless on localhost. Instead, the user generates one-time invitation codes in the UI. An agent must include a valid code in its registration request. This provides:
- User control over who can register
- Natural rate limiting (can't register without a code)
- Audit trail (which code was used by which agent)

Rate limit state for per-agent throttling is held in-memory (token bucket algorithm). Resets on app restart.

### 7.5 Additional Security Measures

| Measure | Implementation |
|---|---|
| Token rotation | User can regenerate an agent's token from the UI (invalidates old token) |
| Agent suspension | Immediately revokes all access; pending transactions are cancelled |
| Audit log immutability | Transaction records are INSERT-only; status changes create new rows in an audit trail (future enhancement) |
| No network exposure | Axum binds to `127.0.0.1` only; no external access |
| Socket permissions | Unix socket created with `0600` (owner read/write only) |
| Input validation | All amounts parsed as `rust_decimal::Decimal`; no floating-point math for money |
| CORS | REST API has no CORS headers (localhost only, no browser clients expected) |

---

## 8. Global Policy & Wallet Controls **(NEW)**

The Global Policy layer sits above all per-agent spending policies. It provides wallet-wide controls that apply regardless of individual agent limits.

### 8.1 Global Policy Engine

```rust
// core/global_policy.rs

pub struct GlobalPolicyEngine {
    db: Arc<Database>,
}

pub struct GlobalPolicy {
    pub daily_cap: Decimal,
    pub weekly_cap: Decimal,
    pub monthly_cap: Decimal,
    pub min_reserve_balance: Decimal,
    pub kill_switch_active: bool,
    pub kill_switch_reason: String,
}

impl GlobalPolicyEngine {
    /// Check global constraints before per-agent policy.
    /// Called inside BEGIN EXCLUSIVE transaction (see SpendingPolicyEngine).
    pub fn check_sync(
        &self,
        conn: &Connection,
        amount: Decimal,
    ) -> Result<(), PolicyDenied> {
        let policy = self.get_policy_sync(conn)?;

        // 1. Kill switch
        if policy.kill_switch_active {
            return Err(PolicyDenied::KillSwitchActive(policy.kill_switch_reason));
        }

        // 2. Minimum reserve
        let balance = self.get_cached_balance_sync(conn)?;
        if balance - amount < policy.min_reserve_balance {
            return Err(PolicyDenied::BelowReserve(policy.min_reserve_balance));
        }

        // 3. Global caps (0 = unlimited)
        let ledger = self.get_global_ledger_sync(conn)?;
        if policy.daily_cap > Decimal::ZERO && ledger.daily + amount > policy.daily_cap {
            return Err(PolicyDenied::GlobalDailyCapExceeded);
        }
        if policy.weekly_cap > Decimal::ZERO && ledger.weekly + amount > policy.weekly_cap {
            return Err(PolicyDenied::GlobalWeeklyCapExceeded);
        }
        if policy.monthly_cap > Decimal::ZERO && ledger.monthly + amount > policy.monthly_cap {
            return Err(PolicyDenied::GlobalMonthlyCapExceeded);
        }

        Ok(())
    }
}
```

### 8.2 User Interface

The Settings page includes a "Global Controls" section:

```
+--------------------------------------------------+
|  Global Wallet Controls                          |
|                                                  |
|  Daily Cap:           $500.00       [Edit]       |
|  Weekly Cap:          $2,000.00     [Edit]       |
|  Monthly Cap:         $5,000.00     [Edit]       |
|  Minimum Reserve:     $100.00       [Edit]       |
|                                                  |
|  Emergency Kill Switch:                          |
|  [ ] Suspend all agent operations                |
|                                                  |
|  Global Usage (Today): $127.30 / $500.00         |
|  [=====----] 25%                                 |
+--------------------------------------------------+
```

---

## 9. Transaction Monitor Service **(NEW)**

### 9.1 Overview

A lightweight cloud backend service that monitors the blockchain for incoming transactions to user wallets. This is necessary because:

- The Alchemy API key must NOT be exposed in the frontend/desktop app
- Chain polling needs to run continuously, even when the desktop app is closed
- Push notifications for incoming funds require a server-side component

### 9.2 Architecture

```
+-------------------+         +-------------------------+        +---------------+
| Desktop App       |         | Transaction Monitor     |        | Alchemy RPC   |
| (Tauri)           |<------->| Service (cloud)         |<------>| / Webhooks    |
|                   |  WS/WSS |                         |  HTTPS |               |
| - Registers wallet|         | - Watches addresses     |        | - eth_getLogs  |
|   address on      |         | - Detects incoming txs  |        | - newFilter    |
|   startup         |         | - Sends notifications   |        | - webhook subs |
| - Receives push   |         |   via WebSocket         |        |               |
|   notifications   |         | - No private keys       |        |               |
|   for incoming txs|         | - Read-only chain access |       |               |
+-------------------+         +-------------------------+        +---------------+
```

### 9.3 Components

**Cloud Service (Node.js or Rust):**
- Receives wallet address registrations from desktop apps
- Polls Alchemy for new transactions to watched addresses (or uses Alchemy webhooks)
- Sends real-time notifications to connected desktop apps via WebSocket
- Stores no sensitive data (just addresses to watch)
- Horizontally scalable (stateless workers)

**Desktop App Integration:**
- On startup, registers the user's wallet address with the monitor service
- Maintains a WebSocket connection for real-time incoming transaction notifications
- Falls back to periodic REST polling if WebSocket disconnects
- Displays OS notification when funds arrive

### 9.4 Development vs Production

| Concern | Development | Production |
|---|---|---|
| Chain polling | Direct Alchemy polling locally | Cloud service with Alchemy webhooks |
| API key location | `.env` file (local only) | Cloud service environment |
| Notification delivery | Local polling loop | WebSocket push from cloud |
| Address watching | Single address | Multi-tenant, many addresses |

> **For Phase 1/2:** Start with direct Alchemy polling in the Rust backend (API key in local `.env`). This is fine for development and single-user testing. The cloud service is built in Phase 3 for production readiness.

---

## 10. Agent Registration Skill **(NEW)**

### 10.1 Overview

Instead of agents hitting raw API endpoints, we provide a **Claude Code skill file** that serves as the agent's instruction manual for interacting with Agent Neo Bank. This skill file is placed at `skills/agent-neo-bank.md` in the project or at `~/.claude/skills/agent-neo-bank.md` globally.

### 10.2 Skill File Contents

The skill file instructs agents to:

1. **Register with rich identity metadata:**
   - `name`: Human-readable agent name
   - `purpose`: What the agent is built for
   - `agent_type`: Category (coding_assistant, research, infrastructure, etc.)
   - `capabilities`: What operations the agent needs
   - `invitation_code`: The one-time code provided by the user

2. **Include detailed transaction metadata:**
   - `description`: Detailed explanation of the transaction
   - `category`: Transaction category (api_services, infrastructure, data, etc.)
   - `service_name`: Name of the service being paid
   - `service_url`: URL of the service
   - `reason`: Why the agent needs to make this payment

3. **Follow the async transaction model:**
   - Submit via `POST /v1/send`
   - Always receive `202 Accepted`
   - Poll `GET /v1/transactions/{tx_id}` for final status
   - Optionally provide a `webhook_url` for callbacks

### 10.3 Example Skill File

```markdown
# Agent Neo Bank Skill

## Registration

To use Agent Neo Bank, you must first register:

1. You need an invitation code from the user. Ask for it if not provided.
2. Register by sending a POST request:

POST http://localhost:7402/v1/agents/register
Content-Type: application/json

{
  "name": "Your Agent Name",
  "purpose": "Brief description of what you do and why you need payment access",
  "agent_type": "coding_assistant",
  "description": "Detailed description of your capabilities and intended use",
  "capabilities": ["send"],
  "invitation_code": "INV-xxxxxxxx"
}

3. Poll for your token:
   GET http://localhost:7402/v1/agents/register/{agent_id}/status

4. Once approved, store the token and use it for all future requests.

## Making Payments

When making payments, ALWAYS include rich metadata so the user understands
what you are paying for and why:

POST http://localhost:7402/v1/send
Authorization: Bearer anb_...
Content-Type: application/json

{
  "to": "0x...",
  "amount": 5.00,
  "category": "api_services",
  "description": "Detailed description of what this payment is for",
  "service_name": "Name of the service",
  "service_url": "https://service.example.com",
  "reason": "Why you need to make this payment right now"
}

The response will ALWAYS be 202 Accepted. Poll the transaction status:
GET http://localhost:7402/v1/transactions/{tx_id}

## Checking Limits

Before making large payments, check your spending limits:
GET http://localhost:7402/v1/spending/limits

If you need higher limits, request an increase with a clear reason:
POST http://localhost:7402/v1/spending/request-increase
```

### 10.4 Dashboard Display

The rich metadata from agent registrations and transactions is displayed prominently in the dashboard:

- **Agent cards** show the agent's purpose, type, and capabilities
- **Transaction rows** show the service name, description, and reason
- **Budget charts** can be grouped by service/category
- Users see clear context about what their agents are doing and why

---

## 11. Build & Development

### 11.1 Dev Environment Setup

**Prerequisites:**
- Rust (stable, 1.77+)
- Node.js (20+)
- pnpm (package manager)
- Tauri CLI v2 (`cargo install tauri-cli --version "^2"`)
- Coinbase Agent Wallet CLI (`awal`) installed and on PATH

**Initial setup:**
```bash
# Clone and install
git clone <repo>
cd agent-neo-bank
pnpm install

# Initialize Tauri
cargo tauri init    # (already done in scaffolding)

# Set up shadcn/ui
pnpm dlx shadcn@latest init

# Development
cargo tauri dev     # Starts both Vite dev server and Rust backend
```

**Environment variables (.env, gitignored):**
```
VITE_API_PORT=7402
AWAL_BINARY_PATH=/usr/local/bin/awal
DEFAULT_NETWORK=base-sepolia
ANB_MOCK=false                          # (NEW) set to true for mock mode (no real CLI)
```

**Mock mode (NEW):** Set `ANB_MOCK=true` or pass `--mock` flag to replace the CLI executor with a mock that returns realistic fake data. This enables development and testing without a real `awal` session. Mock mode is indicated in the health check response and in the UI header.

### 11.2 Testing Strategy

See `docs/architecture/testing-specification.md` for the complete TDD testing specification, including:

- TDD methodology (red-green-refactor cycle, tests written before implementation)
- Test file conventions for Rust (inline `#[cfg(test)]` + `tests/` integration), React (colocated `*.test.tsx`), and API contract tests
- Concrete test cases (5-15 per component) with Given/When/Then descriptions
- 9 full integration test scenarios with numbered steps
- CI pipeline requirements with coverage thresholds (80% Rust, 70% React) and branch protection rules
- Shared test fixtures, helpers, and database setup/teardown patterns

### 11.3 Build / Release Pipeline

```
Development:
  cargo tauri dev              # Hot-reload frontend + Rust rebuild

Build:
  cargo tauri build            # Produces .dmg / .app for macOS

CI (GitHub Actions):
  1. cargo fmt --check
  2. cargo clippy -- -D warnings
  3. cargo test --lib --bins              # Rust unit tests
  4. cargo test --test '*'                # Rust integration tests
  5. cargo tarpaulin --out xml            # Coverage: fail if < 80%
  6. pnpm lint
  7. pnpm test -- --run                   # React unit tests (Vitest)
  8. pnpm test -- --run --coverage        # Coverage: fail if < 70%
  9. cargo tauri build
  10. Upload artifact (.dmg)

Branch Protection:
  - All CI jobs must pass before merge
  - At least 1 PR review approval required
  - Branch must be up to date with main
  - No direct pushes to main

Integration tests run against MockCliExecutor (no real blockchain in CI).
E2E tests run the full app in mock mode (ANB_MOCK=true) with Playwright.
See docs/architecture/testing-specification.md Section 5 for full details.

Release:
  - Tag-based releases (v0.1.0, v0.2.0, etc.)
  - GitHub Releases with .dmg attached
  - Tauri updater integration for auto-updates (future)
```

---

## 12. Implementation Phases **(UPDATED -- Phase 1 split, items moved)**

### Phase 1a: Plumbing **(NEW split)**
**Goal:** Scaffold, database, CLI wrapper with integration tests, auth flow, mock mode, basic shell.

> **TDD:** Tests for this phase's components must be written FIRST before implementation. See `docs/architecture/testing-specification.md` for all test cases. Start with test fixtures and helpers, then write failing tests for CLI wrapper, auth service, and database layer before implementing any of them.

| Task | Module | Details |
|---|---|---|
| Write test fixtures and helpers | `src-tauri/src/test_helpers.rs`, `src/test/helpers.ts`, `src/test/setup.ts` | Mock CLI outputs, test agent/transaction factories, `setup_test_db()`, `mock_cli_executor()`, React test setup with Tauri mocks. **This is the FIRST task -- all other implementation depends on it.** |
| Scaffold Tauri v2 + React + Vite | Project setup | `cargo tauri init`, Vite config, Tailwind v4 |
| Set up shadcn/ui | Frontend | Install base components: Button, Card, Input, Table, Badge, Dialog, Toast |
| SQLite database layer | `db/` | Full schema (incl. global_policy, invitation_codes, token_delivery tables), migrations, rusqlite connection pool, `spawn_blocking` wrapper |
| CLI wrapper with trait | `cli/` | `CliExecutable` trait, `RealCliExecutor`, `MockCliExecutor`, parser, all command types |
| CLI health check on startup | `main.rs` | Run `awal auth status`. If CLI missing: onboarding step. If session expired: re-auth redirect |
| Mock mode | `cli/executor.rs`, `config.rs` | `--mock` flag / `ANB_MOCK=true` env var. Mock returns realistic fake data |
| Auth flow (email OTP) | `commands::auth`, `core::auth_service` | Login, verify, logout, status |
| Two-tier token auth | `core::auth_service` | Argon2 for storage, SHA-256 in-memory cache (5min TTL) |
| Onboarding UI | `pages/Onboarding` | 4-step flow (welcome, email, OTP, address) |
| App shell + navigation | `components/layout/` | Sidebar, header, routing |
| Basic dashboard (placeholder) | `pages/Dashboard` | Balance display, empty states |
| CoreServices struct | `core/services.rs` | `Arc<CoreServices>` with all sub-services, `&self` methods, no Mutex |
| Integration tests for CLI wrapper | `tests/` | Test real and mock executors end-to-end |

**Deliverable:** App boots, user can authenticate, shell renders, CLI wrapper works (real and mock). Database is fully schemed. No agent functionality yet.

### Phase 1b: Agent Operations **(NEW split)**
**Goal:** Agent creation, spending policy, REST API, rate limiting, transaction history.

> **TDD:** Tests for this phase's components must be written FIRST before implementation. Write failing tests for spending policy engine, global policy engine, transaction processor, agent registry, invitation system, REST API endpoints, and rate limiter before writing any implementation code. See `docs/architecture/testing-specification.md` Sections 3.3-3.9 and 3.12 for all test cases.

| Task | Module | Details |
|---|---|---|
| Invitation code generation | `core/invitation.rs`, `commands/invitations.rs` | User generates codes in UI |
| Agent self-registration | `api/rest_routes`, `core/agent_registry` | POST /register with invitation code + rich metadata |
| Token delivery (encrypted cache) | `core/agent_registry` | 5-minute encrypted cache, poll-once-then-delete |
| Spending policy engine | `core::spending_policy` | Full validation logic, `BEGIN EXCLUSIVE` transactions |
| Global policy engine | `core::global_policy` | Global caps, min reserve, kill switch |
| Transaction processor (async) | `core::tx_processor` | Always 202 Accepted, async execution, webhook callback |
| Axum REST API | `api/rest_server` | `/v1/send`, `/v1/balance`, `/v1/health`, `/v1/transactions/{tx_id}` |
| Bearer token auth middleware | `api/auth_middleware` | Two-tier validation (SHA-256 cache + argon2 fallback) |
| Rate limiting | `api/rate_limiter.rs` | Invitation-code-based for registration, token bucket for API |
| Balance caching | `core::wallet_service` | 30s TTL, one CLI call per period |
| Agent balance visibility | `core::wallet_service` | Per-agent `balance_visible` flag |
| Amount parsing (Decimal) | `api/types.rs` | `SendRequest` uses `amount: Decimal`, parsed at API boundary |
| Transaction history UI | `pages/Transactions` | Table with basic filtering + pagination (limit/offset) |
| Stale approval cleanup | `core/approval_manager` | Background task every 5 min, `expires_at` column |
| Agent skill file | `skills/agent-neo-bank.md` | Registration and usage instructions for AI agents |
| Spending ledger (UTC) | `db/queries.rs` | Period determined at creation time, explicit UTC, first-tx upsert |
| Pagination everywhere | All list endpoints | `limit` and `offset` on agents, approvals, transactions |

**Deliverable:** Full agent lifecycle: register with invitation code, get approved with token delivery, send USDC (async 202 model), view transactions. Global policy controls active. Rate limiting from day one.

### Phase 2: Multi-Agent & Controls
**Goal:** Multiple agents, full spending controls, approval flow, MCP server.

> **TDD:** Tests for this phase's components must be written FIRST before implementation. Write failing tests for approval manager, MCP server, event bus, notification system, and all new UI components before implementation. See `docs/architecture/testing-specification.md` Sections 3.10-3.11 for MCP and approval test cases.

| Task | Module | Details |
|---|---|---|
| Agent list UI | `pages/AgentList` | Grid of agent cards with rich metadata (purpose, type, capabilities) |
| Agent detail UI (full) | `pages/AgentDetail` | Limits editor, allowlist, activity feed, balance visibility toggle |
| Approval queue | `core/approval_manager`, `pages/Approvals` | Create, list, resolve approvals with expiration |
| Approval UI | `components/approvals/` | Cards with approve/deny buttons, rich agent context |
| Limit increase requests | API + UI | Agent requests, user reviews |
| MCP server (per-agent) | `api/mcp_server`, `api/mcp_tools` | All tools, token via CLI arg, per-agent binding |
| Global policy UI | `pages/Settings` | Daily/weekly/monthly caps, min reserve, kill switch |
| Event bus + real-time updates | `core/event_bus` | Tauri events to frontend |
| OS notifications | `core/notification` | macOS native notifications |
| Notification preferences UI | `pages/Settings` | Toggle which events notify |
| Agent suspension/revocation | UI + core | Suspend from agent detail page |
| Budget utilization charts | `components/dashboard/` | Per-agent progress bars, by category |
| Invitation code management UI | `pages/Settings` | Generate, list, revoke codes |

**Deliverable:** Multiple agents can register, get approved, operate within limits. MCP server works for Claude Code (per-agent). User gets OS notifications and can manage approvals. Global wallet controls active.

### Phase 3: Receive, Earn, Onramp, Chain Monitoring
**Goal:** Full transaction types, funding flow, x402 integration, Transaction Monitor Service.

> **TDD:** Tests for this phase's components must be written FIRST before implementation. Write failing tests for incoming transaction detection, Unix socket server, transaction export, and new UI components before implementation.

| Task | Module | Details |
|---|---|---|
| Transaction Monitor Service | Cloud service | Alchemy polling/webhooks for incoming txs, WebSocket push to desktop |
| Receive transaction tracking | `core/tx_processor` + monitor | Detect incoming txs via monitor service |
| Earn tracking (x402) | `core/tx_processor` | Agent reports earnings via API |
| Coinbase Onramp integration | `pages/Fund`, `commands::onramp` | Embed widget with wallet address |
| Manual deposit flow | `pages/Fund` | Display address, copy, QR code |
| Unix domain socket server | `api/unix_socket` | Same protocol as REST, over socket |
| Spending breakdown charts | `components/dashboard/` | By agent, by category, by service, by time |
| Transaction export (CSV) | `commands::transactions` | Export with filters |
| Transaction search | `pages/Transactions` | Full-text search on memos, addresses, service names |
| Network toggle (sepolia/mainnet) | `pages/Settings` | With confirmation dialog |

**Deliverable:** Full transaction lifecycle (send/receive/earn). Users can fund via Onramp or manual deposit. Chain monitoring for incoming txs. Rich analytics in dashboard.

### Phase 4: Polish & Production
**Goal:** Production-ready, delightful UX, analytics.

> **TDD:** Tests for this phase's components must be written FIRST before implementation. Write failing tests for token rotation, error recovery, auto-updater, and any new features before implementation.

| Task | Module | Details |
|---|---|---|
| Spending analytics dashboard | `pages/Dashboard` | Time-series charts, comparisons, by service/category |
| Agent performance metrics | `pages/AgentDetail` | ROI tracking (earned vs spent) |
| Keyboard shortcuts | Frontend | Command palette (Cmd+K) |
| Dark mode | Tailwind config | System-aware theme toggle |
| Auto-updater | Tauri updater plugin | Check for updates on launch |
| Error recovery | All modules | Graceful CLI failures, retry logic |
| Token rotation | UI + core | Regenerate agent tokens |
| Backup/restore | DB utilities | Export/import SQLite database |
| Onboarding tour | Frontend | Interactive guide for new users |
| Production network support | Config | Mainnet toggle with safety warnings |
| Performance optimization | All | Query optimization, lazy loading |
| Production Transaction Monitor | Cloud service | Deploy cloud service, switch from local Alchemy to cloud proxy |

**Deliverable:** Polished, production-ready desktop app with full feature set.

---

## Appendix A: Key Dependencies

### Rust (Cargo.toml)

| Crate | Purpose |
|---|---|
| `tauri` v2 | Desktop app framework |
| `tokio` | Async runtime |
| `axum` | HTTP server for REST API |
| `rusqlite` + `r2d2` | SQLite with connection pooling (sync -- use `spawn_blocking`) |
| `serde` + `serde_json` | Serialization |
| `uuid` | UUID generation |
| `rust_decimal` | Precise decimal arithmetic for money **(UPDATED: used for SendRequest.amount)** |
| `argon2` | Password/token hashing (storage tier) |
| `sha2` | SHA-256 hashing (cache tier for fast token lookup) **(NEW)** |
| `aes-gcm` | AES encryption for token delivery cache **(NEW)** |
| `rand` | Secure random token generation |
| `chrono` | Date/time handling (all UTC) |
| `tracing` + `tracing-subscriber` | Structured logging |
| `thiserror` | Error type derivation |
| `reqwest` | HTTP client for webhook callbacks **(NEW)** |
| `tauri-plugin-notification` | OS notifications |
| `tauri-plugin-clipboard-manager` | Copy to clipboard **(UPDATED: add to Cargo.toml)** |

### Frontend (package.json)

| Package | Purpose |
|---|---|
| `react` + `react-dom` | UI framework |
| `react-router-dom` v7 | Routing |
| `@tauri-apps/api` v2 | Tauri frontend bindings |
| `zustand` | State management |
| `tailwindcss` v4 | Utility CSS |
| `@radix-ui/*` | Headless UI primitives (via shadcn) |
| `recharts` | Charts and graphs |
| `date-fns` | Date formatting |
| `clsx` + `tailwind-merge` | Class name utilities |
| `lucide-react` | Icon library |

---

## Appendix B: Configuration Defaults

```toml
# Default app_config entries (key-value in SQLite) (UPDATED)

[network]
current = "base-sepolia"       # base-sepolia | base-mainnet

[api]
rest_port = 7402
rest_host = "127.0.0.1"
unix_socket_path = "/tmp/agent-neo-bank.sock"
mcp_enabled = true

[security]
token_hash_algorithm = "argon2id"
token_cache_ttl_seconds = 300           # (NEW) SHA-256 cache TTL
rate_limit_requests_per_minute = 60
invitation_code_required = true          # (NEW) require invitation for registration

[defaults]
new_agent_per_tx_max = "0"     # Must be configured by user
new_agent_daily_cap = "0"
new_agent_weekly_cap = "0"
new_agent_monthly_cap = "0"
new_agent_auto_approve_max = "0"
new_agent_balance_visible = true         # (NEW) default: agents can see balance

[global_policy]                          # (NEW)
daily_cap = "0"                # 0 = unlimited
weekly_cap = "0"
monthly_cap = "0"
min_reserve_balance = "0"
kill_switch_active = false

[cache]                                  # (NEW)
balance_ttl_seconds = 30
approval_expiry_check_interval = 300     # 5 minutes
approval_default_expiry_hours = 24       # approval requests expire after 24h
token_delivery_ttl_seconds = 300         # token available for 5 minutes

[mock]                                   # (NEW)
enabled = false                # ANB_MOCK=true overrides this
```

---

## Appendix C: CLI Command Reference

Mapping of `awal` CLI commands used by the wrapper:

| Operation | CLI Command | Expected Output |
|---|---|---|
| Login | `awal auth login --email <email>` | "OTP sent to <email>" |
| Verify OTP | `awal auth verify --email <email> --otp <code>` | "Authenticated successfully" |
| Auth status | `awal auth status` | JSON with auth state |
| Logout | `awal auth logout` | "Logged out" |
| Get balance | `awal wallet balance` | JSON `{ "balance": "1247.83", "asset": "USDC" }` |
| Get address | `awal wallet address` | JSON `{ "address": "0x..." }` |
| Send | `awal send --to <addr> --amount <amt> --asset USDC` | JSON `{ "tx_hash": "0x..." }` |
| Auth status | `awal auth status` | JSON `{ "authenticated": true, "email": "..." }` **(NEW -- health check)** |
| Config get | `awal config get <key>` | Value string |
| Config set | `awal config set <key> <value>` | "Config updated" |

> **Note:** Actual CLI output formats should be verified against the current `awal` version and may require parser adjustments.
