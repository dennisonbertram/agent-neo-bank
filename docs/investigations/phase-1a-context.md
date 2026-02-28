# Phase 1a Consolidated Context

> **Generated:** 2026-02-27
> **Sources:** `architecture-plan.md` v2.0, `testing-specification.md` v1.0, CDP Agentic Wallet research (4 files)

---

## 1. Project Scaffold Requirements

### Tech Stack

| Layer | Technology | Version/Notes |
|---|---|---|
| Desktop framework | Tauri | v2 (capabilities-based permissions) |
| Frontend framework | React | With react-dom |
| Build tool | Vite | - |
| Language | TypeScript | Strict mode |
| CSS | Tailwind CSS | v4 |
| Component library | shadcn/ui | Radix UI primitives underneath |
| State management | Zustand | - |
| Routing | react-router-dom | v7 |
| Backend language | Rust | Via Tauri v2 |
| Database | SQLite | Via rusqlite + r2d2 connection pool |
| Async runtime | Tokio | All DB calls via `spawn_blocking` |
| HTTP server | Axum | Port 7402, for agent-facing REST API |
| Icons | lucide-react | - |
| Charts | recharts | - |
| Date formatting | date-fns | - |
| Class utilities | clsx + tailwind-merge | - |

### Rust Dependencies (Cargo.toml)

| Crate | Purpose |
|---|---|
| `tauri` v2 | Desktop app framework |
| `tokio` | Async runtime |
| `axum` | HTTP server for REST API |
| `rusqlite` + `r2d2` | SQLite with connection pooling (sync -- use `spawn_blocking`) |
| `serde` + `serde_json` | Serialization |
| `uuid` | UUID generation |
| `rust_decimal` | Precise decimal arithmetic for money |
| `argon2` | Password/token hashing (storage tier) |
| `sha2` | SHA-256 hashing (cache tier for fast token lookup) |
| `aes-gcm` | AES encryption for token delivery cache |
| `rand` | Secure random token generation |
| `chrono` | Date/time handling (all UTC) |
| `tracing` + `tracing-subscriber` | Structured logging |
| `thiserror` | Error type derivation |
| `reqwest` | HTTP client for webhook callbacks |
| `tauri-plugin-notification` | OS notifications |
| `tauri-plugin-clipboard-manager` | Copy to clipboard |

### Frontend Dependencies (package.json)

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

### shadcn/ui Base Components to Install

Button, Card, Input, Table, Badge, Dialog, Toast

### Directory Structure (Phase 1a relevant)

```
agent-neo-bank/
+-- src-tauri/
|   +-- Cargo.toml
|   +-- tauri.conf.json
|   +-- capabilities/
|   |   +-- default.json
|   |   +-- agent-api.json
|   +-- src/
|   |   +-- main.rs
|   |   +-- lib.rs
|   |   +-- commands/
|   |   |   +-- mod.rs
|   |   |   +-- auth.rs
|   |   |   +-- wallet.rs
|   |   |   +-- settings.rs
|   |   +-- core/
|   |   |   +-- mod.rs
|   |   |   +-- services.rs       # CoreServices struct
|   |   |   +-- auth_service.rs
|   |   |   +-- wallet_service.rs
|   |   +-- db/
|   |   |   +-- mod.rs
|   |   |   +-- schema.rs
|   |   |   +-- models.rs
|   |   |   +-- queries.rs
|   |   |   +-- migrations/
|   |   |       +-- 001_initial.sql
|   |   +-- cli/
|   |   |   +-- mod.rs
|   |   |   +-- executor.rs
|   |   |   +-- parser.rs
|   |   |   +-- commands.rs
|   |   +-- state/
|   |   |   +-- mod.rs
|   |   |   +-- app_state.rs
|   |   +-- error.rs
|   |   +-- config.rs
|   |   +-- test_helpers.rs       # Test fixtures (cfg(test) only)
+-- src/
|   +-- main.tsx
|   +-- App.tsx
|   +-- pages/
|   |   +-- Onboarding.tsx
|   |   +-- Dashboard.tsx
|   +-- components/
|   |   +-- layout/
|   |   |   +-- Sidebar.tsx
|   |   |   +-- Header.tsx
|   |   |   +-- Shell.tsx
|   |   +-- onboarding/
|   |   |   +-- EmailStep.tsx
|   |   |   +-- OtpStep.tsx
|   |   |   +-- FundStep.tsx
|   |   |   +-- WelcomeStep.tsx
|   |   +-- shared/
|   |       +-- CurrencyDisplay.tsx
|   |       +-- StatusBadge.tsx
|   |       +-- EmptyState.tsx
|   +-- hooks/
|   |   +-- useBalance.ts
|   |   +-- useTauriEvent.ts
|   |   +-- useInvoke.ts
|   +-- lib/
|   |   +-- tauri.ts
|   |   +-- format.ts
|   |   +-- constants.ts
|   +-- stores/
|   |   +-- authStore.ts
|   |   +-- settingsStore.ts
|   +-- types/
|   +-- test/
|       +-- helpers.ts
|       +-- setup.ts
+-- package.json
+-- tsconfig.json
+-- vite.config.ts
+-- tailwind.config.ts
+-- components.json
+-- index.html
```

---

## 2. Database Schema (All Tables, Exact Column Definitions)

All tables use `TEXT` for UUIDs and `INTEGER` for timestamps (Unix epoch seconds). All timestamps are explicitly UTC.

All spending-limit checks and ledger updates MUST be wrapped in `BEGIN EXCLUSIVE` transactions.

```sql
-- 001_initial.sql

-- Application configuration (key-value)
CREATE TABLE IF NOT EXISTS app_config (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- Agent registry
CREATE TABLE IF NOT EXISTS agents (
    id                TEXT PRIMARY KEY,           -- UUID v4
    name              TEXT NOT NULL,
    description       TEXT DEFAULT '',
    purpose           TEXT DEFAULT '',            -- what the agent is built for
    agent_type        TEXT DEFAULT '',            -- e.g., "coding_assistant", "research"
    capabilities      TEXT DEFAULT '[]',          -- JSON array: ["send", "receive"]
    status            TEXT NOT NULL DEFAULT 'pending',  -- pending | active | suspended | revoked
    api_token_hash    TEXT,                       -- argon2 hash of the agent's bearer token
    token_prefix      TEXT,                       -- first 8 chars for display (e.g., "anb_a3f8...")
    balance_visible   INTEGER NOT NULL DEFAULT 1, -- whether agent can see wallet balance
    invitation_code   TEXT,                       -- the invitation code used to register
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

-- Global policy -- wallet-level controls above all agent policies
CREATE TABLE IF NOT EXISTS global_policy (
    id                   TEXT PRIMARY KEY DEFAULT 'default',
    daily_cap            TEXT NOT NULL DEFAULT '0',       -- Global daily spending cap across all agents
    weekly_cap           TEXT NOT NULL DEFAULT '0',
    monthly_cap          TEXT NOT NULL DEFAULT '0',
    min_reserve_balance  TEXT NOT NULL DEFAULT '0',       -- Refuse txs that would drop below this
    kill_switch_active   INTEGER NOT NULL DEFAULT 0,      -- 1 = all agent operations suspended
    kill_switch_reason   TEXT DEFAULT '',
    updated_at           INTEGER NOT NULL
);

-- Global spending ledger -- aggregate across all agents
CREATE TABLE IF NOT EXISTS global_spending_ledger (
    period     TEXT PRIMARY KEY,               -- 'daily:2026-02-27' | 'weekly:2026-W09' | 'monthly:2026-02'
    total      TEXT NOT NULL DEFAULT '0',
    tx_count   INTEGER NOT NULL DEFAULT 0,
    updated_at INTEGER NOT NULL
);

-- Transactions
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
    description     TEXT DEFAULT '',              -- detailed description from agent
    service_name    TEXT DEFAULT '',              -- what service this payment is for
    service_url     TEXT DEFAULT '',              -- URL of the service
    reason          TEXT DEFAULT '',              -- why the agent needs this payment
    webhook_url     TEXT,                         -- optional callback URL for status updates
    error_message   TEXT,
    period_daily    TEXT,                         -- UTC period key at creation time
    period_weekly   TEXT,                         -- UTC period key at creation time
    period_monthly  TEXT,                         -- UTC period key at creation time
    created_at      INTEGER NOT NULL,
    updated_at      INTEGER NOT NULL
);

CREATE INDEX idx_tx_agent ON transactions(agent_id);
CREATE INDEX idx_tx_status ON transactions(status);
CREATE INDEX idx_tx_created ON transactions(created_at);
CREATE INDEX idx_tx_type ON transactions(tx_type);

-- Approval requests
CREATE TABLE IF NOT EXISTS approval_requests (
    id           TEXT PRIMARY KEY,              -- UUID v4
    agent_id     TEXT NOT NULL REFERENCES agents(id),
    request_type TEXT NOT NULL,                 -- transaction | limit_increase | registration
    payload      TEXT NOT NULL,                 -- JSON: the full request details
    status       TEXT NOT NULL DEFAULT 'pending', -- pending | approved | denied | expired
    tx_id        TEXT REFERENCES transactions(id), -- Links to tx if type=transaction
    expires_at   INTEGER NOT NULL,              -- auto-expire after this timestamp
    created_at   INTEGER NOT NULL,
    resolved_at  INTEGER,
    resolved_by  TEXT                           -- 'user' or 'auto'
);

CREATE INDEX idx_approval_status ON approval_requests(status);
CREATE INDEX idx_approval_agent ON approval_requests(agent_id);
CREATE INDEX idx_approval_expires ON approval_requests(expires_at);

-- Invitation codes -- user-generated codes for agent registration
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

-- Token delivery cache -- short-lived encrypted token storage
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
-- All reads and writes for a given agent MUST use BEGIN EXCLUSIVE transactions.
CREATE TABLE IF NOT EXISTS spending_ledger (
    agent_id   TEXT NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    period     TEXT NOT NULL,                  -- 'daily:2026-02-27' | 'weekly:2026-W09' | 'monthly:2026-02'
    total      TEXT NOT NULL DEFAULT '0',      -- Running total for this period
    tx_count   INTEGER NOT NULL DEFAULT 0,
    updated_at INTEGER NOT NULL,
    PRIMARY KEY (agent_id, period)
);
```

**Spending ledger upsert pattern:** `INSERT ... ON CONFLICT(agent_id, period) DO UPDATE SET total = total + ?1, tx_count = tx_count + 1`

**Period key format:** `daily:YYYY-MM-DD`, `weekly:YYYY-WNN`, `monthly:YYYY-MM` (all UTC)

---

## 3. CLI Wrapper Design

### Trait Definition

```rust
#[async_trait]
pub trait CliExecutable: Send + Sync {
    async fn run(&self, cmd: AwalCommand) -> Result<CliOutput, CliError>;
}
```

### Command Enum

```rust
pub enum AwalCommand {
    AuthLogin { email: String },
    AuthVerify { email: String, otp: String },
    AuthStatus,
    GetBalance,
    GetAddress,
    Send { to: String, amount: Decimal, asset: String },
}

impl AwalCommand {
    pub fn to_args(&self) -> Vec<String> { ... }
}
```

### CliOutput Struct

```rust
pub struct CliOutput {
    pub success: bool,
    pub data: serde_json::Value,  // Parsed JSON if available
    pub raw: String,               // Raw stdout
    pub stderr: String,
}
```

### Real CLI Executor

```rust
pub struct RealCliExecutor {
    binary_path: PathBuf,
    network: Network,
}
```

Uses `tokio::process::Command`, sets `AWAL_NETWORK` env var, captures stdout/stderr.

### Mock CLI Executor

```rust
pub struct MockCliExecutor {
    responses: HashMap<String, CliOutput>,
}
```

Returns canned responses. Activated via `ANB_MOCK=true` env var or `--mock` CLI flag.

### Whitelisted CLI Commands

Only these can be executed:
- `auth login`, `auth verify`, `auth status`, `auth logout`
- `wallet balance`, `wallet address`
- `send`
- `config get`, `config set`

### CDP Agentic Wallet CLI Commands and Output Formats

All commands use pattern: `npx awal@latest <command> [args] [options]`

Global option: `--json` for machine-readable JSON output.

| Command | Arguments | Output |
|---|---|---|
| `status [--json]` | None | Server health, auth status, email, wallet address |
| `auth login <email> [--json]` | email (required) | Returns `flowId` string |
| `auth verify <flowId> <otp> [--json]` | flowId, otp (required) | Auth confirmation |
| `balance [--chain <chain>] [--json]` | `--chain`: `base` or `base-sepolia` | Balance amount and asset |
| `address [--json]` | None | Ethereum-format address (0x...) |
| `show` | None | Opens companion window (browser UI) |
| `send <amount> <recipient> [--chain <chain>] [--json]` | amount, recipient (address or ENS) | Transaction confirmation with tx_hash |
| `trade <amount> <from> <to> [-s <slippage>] [--json]` | amount, from token, to token | Trade confirmation |
| `x402 bazaar search <query> [-k <n>] [--force-refresh] [--json]` | query | Search results |
| `x402 bazaar list [--network <network>] [--full] [--json]` | None | Service listings |
| `x402 details <url> [--json]` | url | Pricing and payment details |
| `x402 pay <url> [-X <method>] [-d <json>] [-q <params>] [--max-amount <n>] [--json]` | url | API response with payment |

**Amount format rules for `send`:**
- Dollar format: `"$5.00"` (quote the dollar sign)
- Decimal: `0.50`, `1.00`
- Whole number: `5` (interpreted as 5 USDC if <= 100)
- Atomic units: `1000000` (= $1.00 USDC; values > 100 without decimals are atomic)

**Token aliases for `trade`:**
- `usdc` -> `0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913` (6 decimals)
- `eth` -> `0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE` (18 decimals)
- `weth` -> `0x4200000000000000000000000000000000000006` (18 decimals)

**Chain support:**
- Base Mainnet: `eip155:8453`
- Base Sepolia: `eip155:84532` (testnet, no trading)

### Balance Caching

CLI balance responses cached with 30-second TTL via `tokio::sync::RwLock<Option<CachedBalance>>`. One CLI call per TTL period.

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
```

---

## 4. Auth Flow Design

### User Authentication (Email OTP via CLI)

1. User enters email in Onboarding UI
2. Frontend calls `invoke("auth_login", { email })` -> Tauri IPC
3. Rust calls `awal auth login <email>` via CLI wrapper
4. CLI sends OTP to email, returns `flowId`
5. User enters 6-digit OTP
6. Frontend calls `invoke("auth_verify", { email, otp })` -> Tauri IPC
7. Rust calls `awal auth verify <flowId> <otp>` via CLI wrapper
8. Session is established

### CLI Health Check on Startup

```rust
let cli: Arc<dyn CliExecutable> = if config.mock_mode {
    Arc::new(MockCliExecutor::new())
} else {
    let real_cli = RealCliExecutor::new(&config.awal_binary_path)?;
    match real_cli.run(AwalCommand::AuthStatus).await {
        Ok(output) if output.success => Arc::new(real_cli),
        Ok(_) => return Err(AppError::CliSessionExpired),   // redirect to re-auth
        Err(_) => return Err(AppError::CliNotFound),         // show onboarding step
    }
};
```

### Agent Token Authentication (Two-Tier)

**Storage tier:** Argon2 hash of bearer token stored in `agents.api_token_hash`.

**Cache tier:** SHA-256 hash of token kept in-memory `RwLock<HashMap<Sha256Hash, (AgentId, Instant)>>` with 5-minute TTL.

**Validation flow:**
1. Extract token from `Authorization: Bearer <token>` header
2. Compute SHA-256 of token, check in-memory cache
3. If cache hit and not expired: return agent_id (O(1))
4. If cache miss: query DB for all active agents, run argon2 verify against each hash
5. On match: add SHA-256 -> agent_id to cache with current timestamp
6. Only active agents are checked (suspended/revoked are rejected)

### Token Delivery (Post-Approval)

1. User approves agent in UI -> `approve(agent_id)` called
2. Generates `anb_<random 32 chars>` token
3. Stores argon2 hash in `agents.api_token_hash`
4. Stores AES-encrypted token in `token_delivery` table with 5-minute expiry
5. Agent polls `GET /v1/agents/register/{id}/status`
6. First poll within 5 minutes: returns token, marks as delivered, deletes from cache
7. Subsequent polls: returns `null` (already delivered or expired)

---

## 5. Test Fixtures Needed

### Rust Test Fixtures (`src-tauri/src/test_helpers.rs`)

#### Mock CLI Output Strings

```rust
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
```

#### Test Agent Factory

```rust
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
    let token_hash = format!("argon2_hash_of_{}", raw_token);
    let mut agent = create_test_agent(name, AgentStatus::Active);
    agent.api_token_hash = Some(token_hash);
    agent.token_prefix = Some(raw_token[..12].to_string());
    (agent, raw_token)
}
```

#### Test Transaction Factory

```rust
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
```

#### Test Helper Functions

```rust
pub fn setup_test_db() -> Arc<Database> {
    let db = Database::new_in_memory().expect("Failed to create in-memory DB");
    db.run_migrations().expect("Failed to run migrations");
    Arc::new(db)
}

pub fn setup_test_db_file() -> (Arc<Database>, tempfile::TempDir) {
    let tmp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let db_path = tmp_dir.path().join("test.db");
    let db = Database::new(db_path).expect("Failed to create file DB");
    db.run_migrations().expect("Failed to run migrations");
    (Arc::new(db), tmp_dir)
}

pub fn mock_cli_executor() -> Arc<MockCliExecutor> {
    let mut mock = MockCliExecutor::new();
    mock.set_response("get_balance", fixtures::mock_balance_output());
    mock.set_response("send", fixtures::mock_send_output("0xfake_tx_hash_123"));
    mock.set_response("auth_status", fixtures::mock_auth_status_authenticated());
    Arc::new(mock)
}

pub async fn setup_test_core_services() -> Arc<CoreServices> {
    let db = setup_test_db();
    let cli = mock_cli_executor();
    let config = AppConfig::default_test();
    Arc::new(CoreServices::new(db, cli, config).await.expect("Failed to create CoreServices"))
}

pub fn create_test_spending_policy(
    agent_id: &str,
    per_tx_max: &str,
    daily_cap: &str,
    weekly_cap: &str,
    monthly_cap: &str,
    auto_approve_max: &str,
) -> SpendingPolicy { ... }

pub fn create_test_invitation(code: &str, label: &str) -> InvitationCode { ... }
```

### React Test Fixtures

#### `src/test/setup.ts`

```typescript
import { vi } from 'vitest';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
  emit: vi.fn(),
}));
```

#### `src/test/helpers.ts`

```typescript
export function mockInvoke(responses: Record<string, unknown>) { ... }
export function createMockAgent(overrides: Partial<Agent> = {}): Agent { ... }
export function createMockTransaction(overrides: Partial<Transaction> = {}): Transaction { ... }
export function createMockSpendingPolicy(overrides: Partial<SpendingPolicy> = {}): SpendingPolicy { ... }
```

---

## 6. Mock Mode Requirements

**Activation:** `ANB_MOCK=true` env var OR `--mock` CLI flag.

**Behavior:**
- `MockCliExecutor` replaces `RealCliExecutor` -- no real CLI processes spawned
- Returns realistic fake data (canned balance, fake tx hashes, etc.)
- Health check endpoint returns `{ "mock_mode": true }`
- **All business logic still runs** -- spending policy engine, spending ledger, approval flow, invitation codes
- Only the CLI execution layer is mocked
- Still requires invitation codes and user approval for realistic testing

**Mock responses include:**
- Balance: `{ "balance": "1247.83", "asset": "USDC" }`
- Send: `{ "tx_hash": "0xfake_tx_hash_123" }`
- Auth status: `{ "authenticated": true, "email": "test@example.com" }`

---

## 7. App Shell Requirements

### Onboarding UI (4-Step Flow)

| Step | Component | Description |
|---|---|---|
| 1 | `WelcomeStep.tsx` | Welcome screen, app intro |
| 2 | `EmailStep.tsx` | Email input, triggers OTP via CLI |
| 3 | `OtpStep.tsx` | 6-digit OTP verification |
| 4 | `FundStep.tsx` | Display wallet address, fund instructions |

### App Shell Layout

| Component | Purpose |
|---|---|
| `Shell.tsx` | App shell wrapper, contains Sidebar + Header + content area |
| `Sidebar.tsx` | Navigation sidebar with route links |
| `Header.tsx` | Top bar with balance display |

### Routing (Phase 1a)

| Route | Page | Description |
|---|---|---|
| `/onboarding` | `Onboarding.tsx` | Auth + first-run setup (4-step flow) |
| `/` | `Dashboard.tsx` | Main dashboard with balance display, empty states |

### Dashboard (Placeholder for Phase 1a)

- Balance display (via `useBalance` hook + Tauri invoke)
- Empty states for agent grid, recent transactions, spending chart
- No agent functionality yet

### CoreServices Struct

```rust
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
```

Stored as `Arc<CoreServices>` in Tauri state -- no wrapping Mutex. All methods take `&self`.

---

## 8. Phase 1a Test Cases (Complete List)

### CLI Wrapper Tests (`cli/executor.rs`, `cli/parser.rs`, `cli/commands.rs`)

1. `test_cli_parse_balance_output_success` -- Parse `{"balance": "1247.83", "asset": "USDC"}` with exit 0
2. `test_cli_parse_send_output_with_tx_hash` -- Parse `{"tx_hash": "0xabc123..."}` with exit 0
3. `test_cli_parse_auth_status_authenticated` -- Parse `{"authenticated": true, "email": "user@example.com"}`
4. `test_cli_nonzero_exit_code_returns_error` -- Exit 1 with stderr returns `Err(CliError::CommandFailed)`
5. `test_cli_timeout_returns_error` -- Process timeout returns `Err(CliError::Timeout)`, child killed
6. `test_cli_session_expired_detected` -- `{"authenticated": false}` detected as expired
7. `test_cli_binary_not_found` -- Nonexistent binary path returns `Err(CliError::NotFound)`
8. `test_mock_executor_returns_canned_balance` -- MockCliExecutor returns preconfigured balance
9. `test_mock_executor_returns_canned_send` -- MockCliExecutor returns fake tx_hash
10. `test_mock_executor_returns_default_for_unknown_command` -- Default CliOutput for unconfigured commands
11. `test_cli_command_to_args_send` -- `Send` command produces correct args array
12. `test_cli_command_to_args_auth_login` -- `AuthLogin` command produces correct args array

### Auth Service Tests (`core/auth_service.rs`)

1. `test_auth_otp_login_calls_cli` -- Login delegates to CLI with correct email
2. `test_auth_otp_verify_success` -- Verify returns `AuthResult::Verified`
3. `test_auth_otp_verify_invalid_code` -- Invalid OTP returns `Err(AppError::InvalidOtp)`
4. `test_auth_token_validation_sha256_cache_hit` -- Cached token returns agent_id without argon2
5. `test_auth_token_validation_sha256_cache_miss_argon2_fallback` -- Cache miss falls through to argon2
6. `test_auth_token_validation_cache_expired_triggers_argon2` -- Expired cache entry triggers re-validation
7. `test_auth_token_validation_invalid_token` -- Bad token returns `Err(AppError::InvalidToken)`
8. `test_auth_token_validation_suspended_agent_rejected` -- Suspended agents rejected
9. `test_auth_cache_populated_after_first_validation` -- Cache populated after first argon2 verify
10. `test_auth_logout_clears_session` -- Logout invokes CLI and clears state

### Database Tests (`db/schema.rs`, `db/queries.rs`, `db/models.rs`)

- Schema creation (all tables created successfully)
- Migration runs idempotently
- CRUD operations for all tables
- Foreign key constraints enforced
- Index creation verified

### Onboarding UI Tests

- `EmailStep` -- renders email input, submits on enter, shows validation
- `OtpStep` -- renders 6-digit input, auto-advances, shows error on invalid
- `FundStep` -- displays address, copy button works
- `WelcomeStep` -- renders welcome text, advance button

### App Shell Tests

- `Shell` -- renders sidebar and content area
- `Sidebar` -- renders nav links, highlights active route
- `Header` -- displays balance when loaded, shows loading state
- `Dashboard` -- renders balance card, shows empty states

### Integration Tests Relevant to Phase 1a

- `src-tauri/tests/mock_mode.rs` -- Full app with MockCliExecutor (health check returns mock_mode: true, balance returns fake data, spending policy still runs)

---

## 9. Phase 1a Task List (from Architecture Plan)

| # | Task | Module | Details |
|---|---|---|---|
| 1 | Write test fixtures and helpers | `test_helpers.rs`, `src/test/` | **FIRST task** -- all others depend on it |
| 2 | Scaffold Tauri v2 + React + Vite | Project setup | `cargo tauri init`, Vite config, Tailwind v4 |
| 3 | Set up shadcn/ui | Frontend | Button, Card, Input, Table, Badge, Dialog, Toast |
| 4 | SQLite database layer | `db/` | Full schema, migrations, rusqlite pool, `spawn_blocking` |
| 5 | CLI wrapper with trait | `cli/` | `CliExecutable` trait, `RealCliExecutor`, `MockCliExecutor`, parser |
| 6 | CLI health check on startup | `main.rs` | Run `awal auth status`, handle missing/expired |
| 7 | Mock mode | `cli/executor.rs`, `config.rs` | `--mock` flag / `ANB_MOCK=true` |
| 8 | Auth flow (email OTP) | `commands::auth`, `core::auth_service` | Login, verify, logout, status |
| 9 | Two-tier token auth | `core::auth_service` | Argon2 storage + SHA-256 cache (5min TTL) |
| 10 | Onboarding UI | `pages/Onboarding` | 4-step flow |
| 11 | App shell + navigation | `components/layout/` | Sidebar, header, routing |
| 12 | Basic dashboard (placeholder) | `pages/Dashboard` | Balance display, empty states |
| 13 | CoreServices struct | `core/services.rs` | `Arc<CoreServices>`, all sub-services, `&self` methods |
| 14 | Integration tests for CLI wrapper | `tests/` | Test real and mock executors end-to-end |

**Phase 1a deliverable:** App boots, user can authenticate, shell renders, CLI wrapper works (real and mock). Database is fully schemed. No agent functionality yet.

---

## 10. Configuration Defaults

```toml
[network]
current = "base-sepolia"

[api]
rest_port = 7402
rest_host = "127.0.0.1"
unix_socket_path = "/tmp/tally-agentic-wallet.sock"
mcp_enabled = true

[security]
token_hash_algorithm = "argon2id"
token_cache_ttl_seconds = 300
rate_limit_requests_per_minute = 60
invitation_code_required = true

[defaults]
new_agent_per_tx_max = "0"
new_agent_daily_cap = "0"
new_agent_weekly_cap = "0"
new_agent_monthly_cap = "0"
new_agent_auto_approve_max = "0"
new_agent_balance_visible = true

[global_policy]
daily_cap = "0"          # 0 = unlimited
weekly_cap = "0"
monthly_cap = "0"
min_reserve_balance = "0"
kill_switch_active = false

[cache]
balance_ttl_seconds = 30
approval_expiry_check_interval = 300
approval_default_expiry_hours = 24
```
