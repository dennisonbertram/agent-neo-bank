# CLI Wrapper Codebase - Current State Investigation

**Date**: 2026-02-27
**Scope**: Full exploration of CLI wrapper layer, core services, test infrastructure, and frontend types

---

## Summary

The CLI wrapper layer is a well-structured abstraction over the `awal` CLI binary. It uses a trait-based executor pattern (`CliExecutable`) that enables both real process execution and mock testing. The architecture follows: **Commands** (enum) -> **Executor** (trait impl) -> **Parser** (response parsing) -> **Services** (WalletService, AuthService). Balance caching is embedded directly in `WalletService` (no separate `balance_cache.rs` or `session_manager.rs` files exist).

---

## File Inventory

| File | Status | Lines |
|------|--------|-------|
| `src-tauri/src/cli/mod.rs` | EXISTS | 7 |
| `src-tauri/src/cli/commands.rs` | EXISTS | 119 |
| `src-tauri/src/cli/executor.rs` | EXISTS | 424 |
| `src-tauri/src/cli/parser.rs` | EXISTS | 235 |
| `src-tauri/src/core/mod.rs` | EXISTS | 11 |
| `src-tauri/src/core/wallet_service.rs` | EXISTS | 416 |
| `src-tauri/src/core/auth_service.rs` | EXISTS | 582 |
| `src-tauri/src/core/services.rs` | EXISTS | 22 (stub) |
| `src-tauri/src/core/session_manager.rs` | DOES NOT EXIST | - |
| `src-tauri/src/core/balance_cache.rs` | DOES NOT EXIST | - |
| `src-tauri/tests/common/mod.rs` | EXISTS | 179 |
| `src-tauri/tests/cli_failure_recovery.rs` | EXISTS | 371 |
| `src-tauri/tests/mock_mode.rs` | EXISTS | 319 |
| `src/types/index.ts` | EXISTS | 158 |
| `src/hooks/useBalance.ts` | EXISTS | 34 |
| `src/lib/format.ts` | EXISTS | 10 |
| `src/lib/tauri.ts` | EXISTS | 3 |

---

## 1. CLI Module (`src-tauri/src/cli/`)

### 1.1 `mod.rs` - Module Structure

```rust
pub mod commands;
pub mod executor;
pub mod parser;

pub use commands::AwalCommand;
pub use executor::{CliError, CliExecutable, CliOutput, MockCliExecutor, RealCliExecutor};
pub use parser::{parse_auth_status, parse_balance, parse_send_result, AuthStatusResult};
```

Three submodules with all key types re-exported at the `cli` level.

### 1.2 `commands.rs` - Command Enum (Full Content)

```rust
use rust_decimal::Decimal;

/// Commands that can be executed via the awal CLI.
/// Only whitelisted commands are allowed.
#[derive(Debug, Clone)]
pub enum AwalCommand {
    AuthLogin { email: String },
    AuthVerify { email: String, otp: String },
    AuthStatus,
    AuthLogout,
    GetBalance,
    GetAddress,
    Send { to: String, amount: Decimal, asset: String },
}

impl AwalCommand {
    /// Convert the command to CLI argument strings.
    pub fn to_args(&self) -> Vec<String> {
        match self {
            AwalCommand::AuthLogin { email } => {
                vec!["auth".into(), "login".into(), email.clone(), "--json".into()]
            }
            AwalCommand::AuthVerify { email, otp } => {
                vec!["auth".into(), "verify".into(), email.clone(), otp.clone(), "--json".into()]
            }
            AwalCommand::AuthStatus => {
                vec!["status".into(), "--json".into()]
            }
            AwalCommand::AuthLogout => {
                vec!["auth".into(), "logout".into(), "--json".into()]
            }
            AwalCommand::GetBalance => {
                vec!["balance".into(), "--json".into()]
            }
            AwalCommand::GetAddress => {
                vec!["address".into(), "--json".into()]
            }
            AwalCommand::Send { to, amount, asset } => {
                let _ = asset; // asset is always USDC for now
                vec!["send".into(), amount.to_string(), to.clone(), "--json".into()]
            }
        }
    }

    /// Key used by MockCliExecutor for response lookup.
    pub fn command_key(&self) -> &str {
        match self {
            Self::AuthLogin { .. } => "auth_login",
            Self::AuthVerify { .. } => "auth_verify",
            Self::AuthStatus => "auth_status",
            Self::AuthLogout => "auth_logout",
            Self::GetBalance => "get_balance",
            Self::GetAddress => "get_address",
            Self::Send { .. } => "send",
        }
    }
}
```

**Key observations**:
- 7 whitelisted commands covering auth, wallet, and send operations
- All commands append `--json` for JSON output mode
- `Send` ignores the `asset` field (hardcoded USDC assumption)
- `command_key()` provides string keys for mock response lookup
- Uses `rust_decimal::Decimal` for amounts (not f64)

**Tests** (7 tests): Verify `to_args()` output for all variants and `command_key()` values.

### 1.3 `executor.rs` - Executor Trait + Real/Mock Implementations (Full Content)

```rust
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

use super::commands::AwalCommand;

// CliOutput
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliOutput {
    pub success: bool,
    pub data: serde_json::Value,
    pub raw: String,
    pub stderr: String,
}

// CliError
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("Command failed (exit {exit_code:?}): {stderr}")]
    CommandFailed { stderr: String, exit_code: Option<i32> },
    #[error("Command timed out")]
    Timeout,
    #[error("CLI binary not found: {0}")]
    NotFound(String),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Session expired")]
    SessionExpired,
}

// CliExecutable trait
#[async_trait]
pub trait CliExecutable: Send + Sync {
    async fn run(&self, cmd: AwalCommand) -> Result<CliOutput, CliError>;
}
```

**RealCliExecutor**:
- Spawns real `tokio::process::Command` with configurable binary path, prefix args, network env var
- `new()` validates binary exists via `which`; `new_unchecked()` skips validation
- 30-second default timeout via `tokio::time::timeout`
- Sets `AWAL_NETWORK` environment variable
- Attempts JSON parse of stdout; falls back to `{"raw": "<stdout>"}` wrapper
- Non-zero exit codes return `CliError::CommandFailed`

**MockCliExecutor**:
- `HashMap<String, CliOutput>` behind `RwLock` for thread-safe canned responses
- `with_defaults()` pre-loads responses for all 7 commands
- Unknown commands return empty `{}` success response (not an error)
- Used extensively in tests via `CliExecutable` trait

**Tests** (10 tests): Cover mock canned responses, defaults, unknown commands, binary-not-found, and spawn failures.

### 1.4 `parser.rs` - Response Parsers (Full Content)

```rust
pub struct AuthStatusResult {
    pub authenticated: bool,
    pub email: Option<String>,
    pub wallet_address: Option<String>,
}

pub fn parse_balance(output: &CliOutput) -> Result<(Decimal, String), CliError> { ... }
pub fn parse_send_result(output: &CliOutput) -> Result<String, CliError> { ... }
pub fn parse_auth_status(output: &CliOutput) -> Result<AuthStatusResult, CliError> { ... }
```

**Key observations**:
- `parse_balance`: Extracts `balance` (Decimal) and `asset` (defaults to "USDC") from `output.data`
- `parse_send_result`: Extracts `tx_hash` string from `output.data`
- `parse_auth_status`: Returns `SessionExpired` error if `authenticated == false`; checks both `wallet_address` and `address` fields
- All parsers check `output.success` first and return `CommandFailed` on failure

**Tests** (9 tests): Cover success paths, missing fields, invalid decimals, non-zero exit codes, and session expiry detection.

---

## 2. Core Module (`src-tauri/src/core/`)

### 2.1 `mod.rs` - Module Structure

```rust
pub mod agent_registry;
pub mod approval_manager;
pub mod event_bus;
pub mod services;
pub mod auth_service;
pub mod global_policy;
pub mod invitation;
pub mod spending_policy;
pub mod tx_processor;
pub mod notification;
pub mod wallet_service;
```

11 submodules. **No `session_manager` or `balance_cache` module** -- caching is embedded in `wallet_service.rs`.

### 2.2 `wallet_service.rs` - Wallet Service with Embedded Balance Cache (Full Content)

```rust
// CachedBalance
pub struct CachedBalance {
    pub balance: Decimal,
    pub asset: String,
    pub fetched_at: Instant,
}

// BalanceCache
pub struct BalanceCache {
    cache: RwLock<Option<CachedBalance>>,
    ttl: Duration,
}
```

**BalanceCache** implements a read-lock-first, write-lock-with-double-check pattern:
1. Fast path: read lock, return if within TTL
2. Slow path: write lock, double-check (prevents thundering herd), call CLI, cache result
3. `invalidate()` clears cache forcing next call to CLI

**WalletService**:
```rust
pub struct WalletService {
    cli: Arc<dyn CliExecutable>,
    db: Arc<Database>,
    cache: BalanceCache,
}
```

Methods:
- `get_balance()` -- cache-aware balance fetch
- `get_balance_for_agent(agent_id)` -- checks per-agent `balance_visible` flag via DB lookup before returning balance
- `get_address()` -- direct CLI call (no caching), extracts `address` field

**BalanceResponse** (returned to frontend):
```rust
pub struct BalanceResponse {
    pub balance: Option<String>,
    pub asset: Option<String>,
    pub balance_visible: bool,
    pub cached: bool,
}
```

**Tests** (8 tests): Cache hit/miss, TTL expiry, concurrent access (10 tasks, CLI called only once), agent visibility (hidden/visible), get_address, cache invalidation, nonexistent agent.

### 2.3 `auth_service.rs` - Authentication Service (Full Content)

**AuthService**:
```rust
pub struct AuthService {
    cli: Arc<dyn CliExecutable>,
    db: Arc<Database>,
    token_cache: RwLock<HashMap<String, CachedToken>>,
    cache_ttl: Duration,
    current_flow_id: RwLock<Option<String>>,
    current_email: RwLock<Option<String>>,
}
```

Methods:
- `login(email)` -- calls CLI `auth login`, stores flow_id/email for verify step
- `verify(otp)` -- calls CLI `auth verify` using stored email, clears flow_id on success
- `check_status()` -- calls CLI `status`, returns authenticated + email
- `validate_agent_token(token)` -- two-tier lookup:
  1. SHA-256 hash -> in-memory cache (fast path)
  2. Cache miss -> argon2 verify against all active agents in DB (slow path)
- `logout()` -- calls CLI `auth logout`, clears local state

**Tests** (9 tests): OTP login/verify, status check, token validation (cache hit, argon2 fallback, expired cache, invalid token, suspended agent), logout.

### 2.4 `services.rs` - CoreServices (Stub)

```rust
pub struct CoreServices {
    pub config: AppConfig,
}
```

Placeholder with TODO comments listing planned fields (db, cli, all service types). Not yet implemented.

---

## 3. Test Infrastructure

### 3.1 `tests/common/mod.rs` - Integration Test Helpers

Key functions:
- `create_test_app()` -- builds full Axum app with `MockCliExecutor::with_defaults()` and in-memory SQLite
- `create_test_app_with_config(config)` -- custom config variant
- `create_test_app_with_db_and_config(db, config)` -- existing DB variant
- `create_test_app_with_db_config_and_cli(db, config, cli)` -- fully custom, used for switchable CLI tests
- `bearer_request(method, uri, token, body)` -- builds authenticated HTTP request
- `body_json(response)` -- parses response body to `serde_json::Value`
- `register_approve_and_get_token(state, inv_code, name)` -- full agent registration lifecycle
- `register_agent_with_policy(state, inv_code, name, per_tx_max, ...)` -- registration + custom spending policy

**AppStateAxum** contains: `db`, `auth_service`, `agent_registry`, `tx_processor`, `wallet_service`, `rate_limiter`, `config`.

### 3.2 `tests/cli_failure_recovery.rs` - CLI Failure Integration Tests

**SwitchableCliExecutor**: Wraps `MockCliExecutor`, toggles send failures at runtime via `AtomicBool`.

3 tests:
1. CLI failure marks tx as "failed" with error_message
2. CLI failure does NOT update spending ledger (failed tx doesn't count against limits)
3. CLI failure then retry succeeds after `set_send_fails(false)`

All use `wait_for_tx_status()` polling instead of sleep.

### 3.3 `tests/mock_mode.rs` - Mock Mode Integration Tests

5 tests:
1. Health endpoint reports `mock_mode: true`
2. Balance returns fake data in mock mode
3. Full send lifecycle (register, approve, send, poll -> confirmed)
4. Non-mock mode health check reports `mock_mode: false`
5. Multiple sends accumulate correctly; spending policy still enforced in mock mode

### 3.4 Other Integration Test Files (not read in detail)

- `tests/agent_lifecycle.rs`
- `tests/approval_flow.rs`
- `tests/concurrent_transactions.rs`
- `tests/global_policy.rs`
- `tests/kill_switch.rs`
- `tests/kill_switch_integration.rs`
- `tests/limit_increase.rs`
- `tests/mcp_integration.rs`
- `tests/mcp_e2e.rs`
- `tests/spending_limits.rs`
- `tests/token_delivery.rs`

---

## 4. Frontend Types (`src/`)

### 4.1 `src/types/index.ts` - TypeScript Type Definitions

Key balance/wallet related types:

```typescript
export interface BalanceResponse {
  balance: string;
  asset: string;
}

export interface AddressResponse {
  address: string;
}

export interface AuthStatusResponse {
  authenticated: boolean;
  email?: string;
}

export interface Agent {
  // ... 16 fields including:
  balance_visible: boolean;
  // ...
}
```

Also defines: `AgentStatus`, `TxStatus`, `TxType`, `ApprovalRequestType`, `ApprovalStatus`, `SpendingPolicy`, `GlobalPolicy`, `Transaction`, `ApprovalRequest`, `InvitationCode`, `TokenDelivery`, `NotificationPreferences`, `SpendingLedger`, `AgentBudgetSummary`, `GlobalBudgetSummary`.

### 4.2 `src/hooks/useBalance.ts` - Balance React Hook

```typescript
export function useBalance(): UseBalanceReturn {
  // Calls invoke<{ balance: string; asset: string }>("get_balance")
  // Returns { balance, isLoading, error, refetch }
}
```

Simple Tauri invoke wrapper with loading/error state.

### 4.3 `src/lib/format.ts` - Formatting Utilities

```typescript
export function formatCurrency(amount: string, asset = "USDC"): string
export function truncateAddress(address: string, chars = 4): string
```

### 4.4 `src/lib/tauri.ts` - Tauri Re-export

```typescript
export { invoke } from "@tauri-apps/api/core";
```

---

## 5. Architecture Observations

### Data Flow
```
Frontend (useBalance hook)
  -> Tauri invoke("get_balance")
    -> WalletService.get_balance_for_agent()
      -> BalanceCache.get_or_fetch()
        -> CliExecutable.run(AwalCommand::GetBalance)
          -> [RealCliExecutor spawns `awal balance --json`]
          -> [MockCliExecutor returns canned response]
        -> parser::parse_balance() [used by parser module, but WalletService parses inline]
```

### Key Design Patterns
1. **Trait-based abstraction**: `CliExecutable` trait enables seamless real/mock switching
2. **Double-check locking**: BalanceCache prevents thundering herd on concurrent requests
3. **Two-tier auth caching**: SHA-256 fast path + argon2 slow path for token validation
4. **Background execution**: Sends return 202 immediately; tx status is polled
5. **Decimal arithmetic**: `rust_decimal::Decimal` throughout (no floating-point)
6. **Per-agent visibility**: `balance_visible` flag controls balance exposure per agent

### Notable Gaps
1. **No `session_manager.rs`**: Session state is managed within `AuthService` (flow_id, email)
2. **No `balance_cache.rs`**: Balance caching is embedded in `wallet_service.rs`
3. **`CoreServices` is a stub**: Only holds `AppConfig`; actual service wiring is in `tests/common/mod.rs` `AppStateAxum`
4. **`Send` ignores asset**: The `asset` field in `AwalCommand::Send` is unused (`let _ = asset`)
5. **Parser module partially redundant**: `WalletService` parses balance inline rather than using `parse_balance()` from `parser.rs`
6. **No retry logic**: Failed CLI calls are not retried automatically; the caller must retry

### Test Coverage Summary
- `cli/commands.rs`: 7 unit tests
- `cli/executor.rs`: 10 unit tests (mock + real executor)
- `cli/parser.rs`: 9 unit tests
- `core/wallet_service.rs`: 8 unit tests (cache, visibility, concurrency)
- `core/auth_service.rs`: 9 unit tests (OTP, token validation, argon2)
- `tests/cli_failure_recovery.rs`: 3 integration tests
- `tests/mock_mode.rs`: 5 integration tests
- **Total**: 51 tests across CLI/wallet/auth modules
