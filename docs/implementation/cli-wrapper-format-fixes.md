# CLI Wrapper Format Fixes

## Summary

Fixed all 7 CLI wrapper format mismatches between the parser/mock layer and real `awal` CLI output. All changes followed strict TDD: tests written first, verified to fail, then implementation fixed.

## Bugs Fixed

### Bug 1: Balance format (multi-asset)
- **Before:** Parser expected `{"balance": "1247.83", "asset": "USDC"}`
- **After:** Parser handles `{"address": "...", "chain": "...", "balances": {"USDC": {"raw": "...", "formatted": "...", "decimals": N}, ...}, "timestamp": "..."}`
- New types: `BalanceResponse`, `AssetBalance` in `parser.rs`
- `CachedBalance` now holds full multi-asset data
- API `BalanceResponse` returns USDC from map for backward compat + all assets

### Bug 2: Auth status nested format
- **Before:** `data["authenticated"]` at top level
- **After:** `data["auth"]["authenticated"]` (with fallback to flat format)
- Updated `parse_auth_status()` and `auth_service.rs::check_status()`

### Bug 3: Address bare string
- **Before:** Expected `{"address": "0x..."}`
- **After:** Handles bare JSON string `"0x..."` (with fallback to object format)
- New function: `parse_address()`
- Updated `wallet_service.rs::get_address()`

### Bug 4: Missing --chain support
- `GetBalance` changed to `GetBalance { chain: Option<String> }`
- `Send` includes `chain: Option<String>`
- `to_args()` appends `--chain <value>` when present

### Bug 5: Send command no --asset flag
- Removed `asset: String` from `AwalCommand::Send`
- Send is USDC-only, `--asset` flag never existed in real CLI
- Replaced with `chain: Option<String>`

### Bug 6: Auth login/verify format
- Login: `{"flowId": "...", "message": "..."}` (was `{"flow_id": "..."}`)
- Verify: `{"success": true, "message": "..."}` (was `{"verified": true}`)
- New types: `LoginResponse`, `VerifyResponse`
- New functions: `parse_login_response()`, `parse_verify_response()`

### Bug 7: AuthVerify passes email instead of flowId
- Renamed `AuthVerify { email, otp }` to `AuthVerify { flow_id, otp }`
- `auth_service.rs::verify()` now passes stored `flow_id` instead of `email`

## Files Modified

### Rust (src-tauri/src/)
- `cli/commands.rs` - AwalCommand enum + to_args() + tests
- `cli/parser.rs` - New types, rewritten parsers, new functions, 12 new tests
- `cli/executor.rs` - MockCliExecutor defaults match real formats, 4 new tests
- `cli/mod.rs` - Updated exports
- `core/wallet_service.rs` - CachedBalance, BalanceResponse, get_address(), 3 new tests
- `core/auth_service.rs` - check_status(), verify() uses flow_id
- `core/tx_processor.rs` - Send command uses chain instead of asset
- `state/app_state.rs` - Updated tests for new formats

### TypeScript (src/)
- `types/index.ts` - Added AssetBalance, updated BalanceResponse
- `hooks/useBalance.ts` - Returns multi-asset balances

## Test Results

- 326 total tests pass across all test suites (unit + integration)
- All integration tests pass (mock_mode, cli_failure_recovery, spending_limits, token_delivery, mcp_e2e, mcp_integration)
- Zero regressions
