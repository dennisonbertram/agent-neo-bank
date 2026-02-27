# CLI Wrapper Validation: Mock vs Real `awal` CLI Output

**Date**: 2026-02-27
**Authenticated as**: dennison@dennisonbertram.com
**Wallet Address**: 0x72AE334bfbaAB69350EB4f5c5EfBac5697C504B4
**AWAL Version**: 1.0.0 (npm: awal@latest)

---

## 1. Real CLI Output for Each Command

### 1.1 `npx awal@latest status --json`

**Exit Code**: 0
**Stdout (JSON)**:
```json
{
  "server": {
    "running": true,
    "pid": 11705
  },
  "auth": {
    "authenticated": true,
    "email": "dennison@dennisonbertram.com"
  }
}
```

**Key observation**: The real output has a nested structure with `server` and `auth` top-level keys. The `authenticated` and `email` fields are inside `auth`, NOT at the top level.

---

### 1.2 `npx awal@latest balance --json`

**Exit Code**: 0
**Stdout (JSON)**:
```json
{
  "address": "0x72AE334bfbaAB69350EB4f5c5EfBac5697C504B4",
  "chain": "Base",
  "balances": {
    "USDC": {
      "raw": "20000000",
      "formatted": "20.00",
      "decimals": 6
    },
    "ETH": {
      "raw": "100000001000000000",
      "formatted": "0.10",
      "decimals": 18
    },
    "WETH": {
      "raw": "100000001000000000",
      "formatted": "0.10",
      "decimals": 18
    }
  },
  "timestamp": "2026-02-27T20:47:28.494Z"
}
```

**Key observation**: Real output returns ALL balances (USDC, ETH, WETH) in a `balances` map with `raw`/`formatted`/`decimals` per asset. There is NO flat `{"balance": "...", "asset": "..."}` structure.

---

### 1.3 `npx awal@latest balance --chain base-sepolia --json`

**Exit Code**: 0
**Stdout (JSON)**:
```json
{
  "address": "0x72AE334bfbaAB69350EB4f5c5EfBac5697C504B4",
  "chain": "Base Sepolia",
  "balances": {
    "USDC": {
      "raw": "0",
      "formatted": "0.00",
      "decimals": 6
    },
    "ETH": {
      "raw": "200000000000000000",
      "formatted": "0.20",
      "decimals": 18
    },
    "WETH": {
      "raw": "200000000000000000",
      "formatted": "0.20",
      "decimals": 18
    }
  },
  "timestamp": "2026-02-27T20:47:29.915Z"
}
```

**Key observation**: Same structure as mainnet balance. Chain name is "Base Sepolia" (display name), not "base-sepolia" (CLI arg).

---

### 1.4 `npx awal@latest address --json`

**Exit Code**: 0
**Stdout (JSON)**:
```json
"0x72AE334bfbaAB69350EB4f5c5EfBac5697C504B4"
```

**Key observation**: Returns a bare JSON string, NOT an object. There is no `{"address": "..."}` wrapper.

---

### 1.5 `npx awal@latest show --json`

**Exit Code**: 1
**Stderr**: `error: unknown option '--json'`

**Key observation**: `show` does not support `--json`. It is a GUI command to display the companion wallet window.

---

## 2. Error Cases

### 2.1 `npx awal@latest send --to 0xinvalid --amount 1 --json`

**Exit Code**: 1
**Stderr**: `error: unknown option '--to'`

**Key observation**: `send` uses positional arguments, not named flags. Correct syntax is:
```
awal send <amount> <recipient> [--chain <chain>] [--json]
```

### 2.2 `npx awal@latest balance --chain invalid-chain --json`

**Exit Code**: 1
**Stdout (non-JSON)**:
```
- Fetching balances...
✖ Failed to fetch balances
Bridge communication error: Cannot read properties of undefined (reading 'name')

This may indicate a configuration issue. Try restarting the wallet.
```

**Key observation**: Error output is plain text even with `--json` flag. Exit code 1. The error includes Unicode cross mark and a generic bridge error.

---

## 3. Testnet Send Attempt

### 3.1 `npx awal@latest send 0.001 0x000000000000000000000000000000000000dEaD --chain base-sepolia --json`

**Exit Code**: 1
**Stdout (non-JSON)**:
```
- Preparing transaction...
Transaction failed: Insufficient USDC balance. You have 0.00 USDC but need $0.001.
```

**Key observations**:
- `send` is USDC-only. There is no `--asset` flag. Cannot send ETH/WETH via `send`.
- Insufficient balance error is plain text, not JSON, even with `--json` flag.
- The error references USDC specifically, confirming it's hardcoded to USDC.

### 3.2 Post-send balance check (`balance --chain base-sepolia --json`)

Balance unchanged (as expected since send failed):
```json
{
  "address": "0x72AE334bfbaAB69350EB4f5c5EfBac5697C504B4",
  "chain": "Base Sepolia",
  "balances": {
    "USDC": { "raw": "0", "formatted": "0.00", "decimals": 6 },
    "ETH": { "raw": "200000000000000000", "formatted": "0.20", "decimals": 18 },
    "WETH": { "raw": "200000000000000000", "formatted": "0.20", "decimals": 18 }
  },
  "timestamp": "2026-02-27T20:48:10.932Z"
}
```

---

## 4. CLI Wrapper Code Analysis

### Files Reviewed

| File | Purpose |
|------|---------|
| `src-tauri/src/cli/commands.rs` | `AwalCommand` enum and `to_args()` serialization |
| `src-tauri/src/cli/executor.rs` | `RealCliExecutor`, `MockCliExecutor`, `CliOutput` |
| `src-tauri/src/cli/parser.rs` | `parse_balance()`, `parse_send_result()`, `parse_auth_status()` |
| `src-tauri/src/core/wallet_service.rs` | `WalletService` with `BalanceCache` |

---

## 5. Discrepancies Between Mock/Parser and Real CLI

### CRITICAL: Balance Format Mismatch

**Mock expects**:
```json
{"balance": "1247.83", "asset": "USDC"}
```

**Real CLI returns**:
```json
{
  "address": "0x72AE334bfbaAB69350EB4f5c5EfBac5697C504B4",
  "chain": "Base",
  "balances": {
    "USDC": { "raw": "20000000", "formatted": "20.00", "decimals": 6 },
    "ETH": { "raw": "100000001000000000", "formatted": "0.10", "decimals": 18 },
    "WETH": { "raw": "100000001000000000", "formatted": "0.10", "decimals": 18 }
  },
  "timestamp": "2026-02-27T20:47:28.494Z"
}
```

**Impact**: `parse_balance()` in `parser.rs` looks for `output.data["balance"]` which does NOT exist in real output. `wallet_service.rs` also looks for `output.data["balance"]`. Both will fail with `ParseError("Missing 'balance' field")` when connected to the real CLI.

**The parser needs to extract from** `output.data["balances"]["USDC"]["formatted"]` (or iterate all assets).

---

### CRITICAL: Auth Status Format Mismatch

**Mock expects**:
```json
{"authenticated": true, "email": "test@example.com"}
```

**Real CLI returns**:
```json
{
  "server": { "running": true, "pid": 11705 },
  "auth": { "authenticated": true, "email": "dennison@dennisonbertram.com" }
}
```

**Impact**: `parse_auth_status()` looks for `output.data["authenticated"]` at the top level. Real output has it nested under `output.data["auth"]["authenticated"]`. The parser will incorrectly treat an authenticated user as unauthenticated (returning `SessionExpired`).

---

### CRITICAL: Address Format Mismatch

**Mock expects**:
```json
{"address": "0xMockWalletAddress123"}
```

**Real CLI returns** (bare string):
```json
"0x72AE334bfbaAB69350EB4f5c5EfBac5697C504B4"
```

**Impact**: `wallet_service.rs` `get_address()` looks for `output.data["address"]` which will be `null` since the real output is a bare string, not an object. The `RealCliExecutor` will parse the bare string as `serde_json::Value::String(...)`, so `output.data` will be a `Value::String`, not a `Value::Object`. Accessing `output.data["address"]` on a `Value::String` returns `Value::Null`.

---

### HIGH: Send Command Argument Format Wrong

**Code generates** (`commands.rs` line 39-41):
```
send 5.00 0xRecipient --json
```

**Real CLI expects**:
```
send <amount> <recipient> [--chain <chain>] --json
```

The positional arg order matches (amount first, recipient second), so **the positional format is actually correct**. However:

1. The `asset` field in `AwalCommand::Send` is ignored (comment says "asset is always USDC for now") -- this matches reality since `send` is USDC-only.
2. There is no `--chain` support in the command builder, so sends will always go to mainnet Base.

---

### HIGH: Send Response Format Mismatch

**Mock expects**:
```json
{"tx_hash": "0xabc123"}
```

**Real CLI returns** (for insufficient balance error):
```
Transaction failed: Insufficient USDC balance. You have 0.00 USDC but need $0.001.
```

We could not capture a successful send response (no testnet USDC). The mock's `{"tx_hash": "..."}` format is unverified against real output. This needs to be tested with a funded wallet.

---

### MEDIUM: No Multi-Chain Support in Commands

The `AwalCommand::GetBalance` variant has no chain parameter. Real CLI supports `--chain base-sepolia` etc. The wrapper can only query the default chain (Base mainnet).

---

### MEDIUM: No `trade` Command Support

Real CLI supports `awal trade <amount> <from> <to>` for token swaps. The wrapper has no `AwalCommand::Trade` variant.

---

### LOW: Error Responses Are Not JSON

Real CLI errors are always plain text even with `--json` flag. The `RealCliExecutor` handles this correctly -- on non-zero exit, it returns `CliError::CommandFailed` with the stderr/stdout. However, the mock never exercises error paths since it always returns `Ok(...)`.

---

## 6. Recommendations

### P0 - Fix Immediately (Will Break Against Real CLI)

1. **Fix `parse_balance()`** to handle the real multi-asset balance format:
   ```rust
   // Instead of: output.data["balance"].as_str()
   // Use: output.data["balances"]["USDC"]["formatted"].as_str()
   // Or better: return all balances as a HashMap<String, BalanceInfo>
   ```

2. **Fix `parse_auth_status()`** to read from nested `auth` object:
   ```rust
   // Instead of: output.data["authenticated"]
   // Use: output.data["auth"]["authenticated"]
   // And: output.data["auth"]["email"]
   ```

3. **Fix `get_address()` parsing** to handle bare string response:
   ```rust
   // Instead of: output.data["address"].as_str()
   // Use: output.data.as_str() (the whole value is the address string)
   ```

4. **Update MockCliExecutor defaults** to match real formats:
   - `auth_status` mock should use `{"server": {...}, "auth": {...}}` structure
   - `get_balance` mock should use `{"address": "...", "chain": "...", "balances": {...}}` structure
   - `get_address` mock should return a bare string: `"0xMockWalletAddress123"`

### P1 - Add Missing Functionality

5. **Add `--chain` support to `GetBalance`** command variant:
   ```rust
   GetBalance { chain: Option<String> }
   ```

6. **Add `--chain` support to `Send`** command variant for testnet sends.

7. **Add `Trade` command** variant for token swaps.

8. **Store multi-asset balances** instead of single asset in `BalanceCache`.

### P2 - Improve Robustness

9. **Add error response mocks** to `MockCliExecutor` for testing error paths.

10. **Test with real successful send** once testnet USDC is available, to verify the tx response JSON format.

11. **Handle spinner text in stdout**: Some commands output spinner text (`- Fetching balances...`) to stdout before JSON. The JSON parser in `RealCliExecutor` uses `serde_json::from_str(&stdout)` which will fail if spinner text is mixed in. Consider stripping non-JSON prefixes or parsing only the last JSON block.

---

## 7. Summary Table

| Aspect | Mock/Parser Assumes | Real CLI Returns | Status |
|--------|-------------------|-----------------|--------|
| Balance format | `{"balance": "...", "asset": "..."}` | `{"balances": {"USDC": {...}, "ETH": {...}, ...}}` | **BROKEN** |
| Auth status format | `{"authenticated": true, "email": "..."}` | `{"server": {...}, "auth": {"authenticated": true, "email": "..."}}` | **BROKEN** |
| Address format | `{"address": "0x..."}` | `"0x..."` (bare string) | **BROKEN** |
| Send args | `send <amount> <recipient> --json` | `send <amount> <recipient> --json` | OK |
| Send response | `{"tx_hash": "..."}` | Unknown (could not test success) | **UNVERIFIED** |
| Chain support | Not supported | `--chain base-sepolia` | **MISSING** |
| Trade command | Not implemented | `trade <amount> <from> <to>` | **MISSING** |
| Error format | N/A (no error mocks) | Plain text, not JSON | OK (handled by executor) |
