# Mock Mode Implementation Report

## Summary

Implemented mock mode activation via `ANB_MOCK` environment variable. When set to `"true"` or `"1"`, the app uses `MockCliExecutor` with pre-loaded default responses and an in-memory SQLite database. All business logic still runs -- only the CLI execution layer is mocked.

## Changes

### `src-tauri/src/cli/executor.rs`
- Added `MockCliExecutor::with_defaults()` -- creates a mock executor pre-loaded with standard responses for all supported commands:
  - `auth_status` -> authenticated with test@example.com
  - `auth_login` -> flow_id: "mock-flow-123"
  - `auth_verify` -> success
  - `auth_logout` -> success
  - `get_balance` -> {"balance": "1247.83", "asset": "USDC"}
  - `get_address` -> {"address": "0xMockWalletAddress123"}
  - `send` -> {"tx_hash": "0xmock_tx_hash_abc123"}
- Added 5 new tests for `with_defaults()` method

### `src-tauri/src/config.rs`
- Added `db_path: String` field to `AppConfig`
- Added `AppConfig::from_env()` -- reads `ANB_MOCK` env var to set `mock_mode`
- Updated `Default` impl to include `db_path: "agent-neo-bank.db"`
- Updated `default_test()` to include `db_path: ":memory:"`
- Added 8 new tests (env var parsing, defaults, db_path)
- Used `Mutex` to serialize env var tests to avoid race conditions

### `src-tauri/src/state/app_state.rs`
- Full implementation of `AppState` with constructor that creates appropriate CLI executor and DB based on config
- Added `cli: Arc<dyn CliExecutable>` field alongside existing `auth_service` and `db`
- `AppState::new(config)` -- creates mock or real executor based on `config.mock_mode`
- `AppState::new_mock()` -- convenience for tests
- Creates `AuthService` wired to the CLI executor and DB
- Added 8 new tests covering mock mode balance, auth, address, send, and full startup

### `src-tauri/src/lib.rs`
- Updated `run()` to use `AppConfig::from_env()` instead of `AppConfig::default()`
- Uses `AppState::new(config)` for centralized state creation
- In non-mock mode, resolves `db_path` relative to Tauri app data dir
- Removed manual CLI executor and auth service wiring (now in `AppState::new`)

## Test Results

All 75 tests pass:
- 5 new executor tests (with_defaults variants)
- 8 new config tests (env var, defaults, db_path)
- 8 new app_state tests (mock mode, full startup, auth service integration)
- All 54 pre-existing tests continue to pass

## How to Use

```bash
# Run in mock mode
ANB_MOCK=true cargo tauri dev

# Run in real mode (default)
cargo tauri dev
```
