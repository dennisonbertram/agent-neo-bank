# CLI Wrapper Implementation Report

> **Date:** 2026-02-27
> **Task:** #4 - CLI wrapper module with trait, executors, and parser

## Summary

Fully implemented the CLI wrapper module at `src-tauri/src/cli/` with all tests passing (22 tests).

## Files Modified

### `src-tauri/src/cli/commands.rs`
- **AwalCommand enum**: AuthLogin, AuthVerify, AuthStatus, AuthLogout, GetBalance, GetAddress, Send
- **to_args()**: Converts each variant to CLI argument vectors (always appends `--json`)
- **command_key()**: Returns string key for MockCliExecutor response lookup
- **Tests (7)**: All `to_args()` variants + `command_key()` validation

### `src-tauri/src/cli/executor.rs`
- **CliOutput**: `{ success, data, raw, stderr }` with Serialize/Deserialize
- **CliError**: Structured enum with `CommandFailed { stderr, exit_code }`, `Timeout`, `NotFound`, `ParseError`, `SessionExpired` -- all derive `thiserror::Error`
- **CliExecutable trait**: `async fn run(&self, cmd: AwalCommand) -> Result<CliOutput, CliError>`
- **RealCliExecutor**: Uses `tokio::process::Command`, sets `AWAL_NETWORK` env var, 30s timeout, kills child on timeout, parses JSON stdout
  - `new()` validates binary exists via `which`
  - `new_unchecked()` skips validation (for test scenarios)
- **MockCliExecutor**: Thread-safe `RwLock<HashMap<String, CliOutput>>`, returns default `{ success: true, data: {} }` for unknown commands
- **Tests (5)**: Canned balance, canned send, unknown command default, binary not found (via `new()`), spawn failure (via `new_unchecked()`)

### `src-tauri/src/cli/parser.rs`
- **parse_balance()**: Extracts `(Decimal, String)` from `{"balance": "...", "asset": "..."}`
- **parse_send_result()**: Extracts tx_hash string from `{"tx_hash": "..."}`
- **parse_auth_status()**: Returns `AuthStatusResult { authenticated, email, wallet_address }`, detects session expiry when `authenticated: false`
- **AuthStatusResult struct**: `{ authenticated: bool, email: Option<String>, wallet_address: Option<String> }`
- **Tests (10)**: Balance success, balance missing field, balance invalid decimal, send success, send missing hash, auth authenticated, auth unauthenticated (SessionExpired), auth with wallet address, nonzero exit code, session expired detection

### `src-tauri/src/cli/mod.rs`
- Re-exports all public types: `AwalCommand`, `CliError`, `CliExecutable`, `CliOutput`, `MockCliExecutor`, `RealCliExecutor`, `parse_auth_status`, `parse_balance`, `parse_send_result`, `AuthStatusResult`

## Test Results

```
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured
```

All 22 CLI tests pass. `cargo check` passes with no errors.

## Design Decisions

1. **CliError::CommandFailed** uses structured `{ stderr, exit_code }` fields instead of a single string, enabling callers to inspect exit codes
2. **MockCliExecutor** uses `RwLock` instead of plain `HashMap` for thread safety (can `set_response` after construction)
3. **RealCliExecutor** has `binary` + `args_prefix` fields, supporting the `npx awal@latest` pattern (`binary: "npx"`, `args_prefix: ["awal@latest"]`)
4. **parse_auth_status** returns `Err(CliError::SessionExpired)` when `authenticated: false`, matching the health check flow in the architecture plan
