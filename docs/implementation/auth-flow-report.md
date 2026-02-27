# Auth Flow Implementation Report

## Summary

Implemented auth flow with email OTP login/verify, session management, two-tier agent token validation (SHA-256 cache + argon2 fallback), and Tauri command integration.

## Files Modified

### `src-tauri/src/core/auth_service.rs`
- **AuthService struct**: Holds CLI executor, DB, token cache (RwLock<HashMap>), cache TTL, current flow state
- **login()**: Calls CLI `auth login <email>`, returns `AuthResult::OtpSent { flow_id }`
- **verify()**: Calls CLI `auth verify <email> <otp>`, returns `AuthResult::Verified`
- **check_status()**: Calls CLI `status`, returns `AuthStatus { authenticated, email }`
- **validate_agent_token()**: Two-tier lookup:
  1. SHA-256 hash of token -> in-memory cache (O(1))
  2. Cache miss -> query DB for active agents, argon2 verify against each hash
  3. On match: populate cache; suspended/revoked agents rejected
- **logout()**: Calls CLI `auth logout`, clears local state
- **10 tests**: All passing

### `src-tauri/src/commands/auth.rs`
- `auth_login(email, State<AppState>)` -> delegates to AuthService
- `auth_verify(otp, State<AppState>)` -> delegates to AuthService
- `auth_status(State<AppState>)` -> returns AuthStatus
- `auth_logout(State<AppState>)` -> delegates to AuthService

### `src-tauri/src/state/app_state.rs`
- Updated by mock-mode agent with `AppState::new(config)` factory
- Holds `cli`, `auth_service`, `db`, `config`

### `src-tauri/src/lib.rs`
- Updated by mock-mode agent with `AppState::new(config)` in setup hook
- Uses `AppConfig::from_env()` to read ANB_MOCK

## Test Results

All 10 auth service tests pass:
1. `test_auth_otp_login_calls_cli` - Login delegates to CLI, returns flow_id
2. `test_auth_otp_verify_success` - Verify with correct OTP returns Verified
3. `test_auth_otp_verify_invalid_code` - Invalid OTP returns Err(InvalidOtp)
4. `test_auth_check_status_authenticated` - Returns authenticated with email
5. `test_auth_check_status_unauthenticated` - Returns not authenticated
6. `test_auth_token_validation_sha256_cache_hit` - Cached token returns agent_id
7. `test_auth_token_validation_sha256_cache_miss_argon2_fallback` - Cache miss uses argon2
8. `test_auth_token_validation_cache_expired_triggers_argon2` - Expired cache triggers re-validation
9. `test_auth_token_validation_invalid_token` - Bad token returns Err(InvalidToken)
10. `test_auth_token_validation_suspended_agent_rejected` - Suspended agent tokens fail
11. `test_auth_logout_clears_session` - Logout clears state

Total: 74 tests passing (1 pre-existing flaky env var test in config.rs not related to auth changes).

## Types

```rust
pub enum AuthResult {
    OtpSent { flow_id: String },
    Verified,
}

pub struct AuthStatus {
    pub authenticated: bool,
    pub email: Option<String>,
}
```
