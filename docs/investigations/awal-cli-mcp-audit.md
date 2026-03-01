# awal CLI vs MCP Tool Definitions Audit

**Date**: 2026-02-28
**CLI Version**: `npx awal@latest` (current)
**Files Audited**:
- `src-tauri/src/api/mcp_tools.rs` — MCP tool schemas
- `src-tauri/src/cli/commands.rs` — AwalCommand enum + to_args()
- `src-tauri/src/cli/executor.rs` — CLI execution layer

---

## Summary

- **13 MCP tools** defined
- **7 map to real awal CLI commands** (send, balance, address, trade, x402 pay, x402 bazaar list/search, x402 details)
- **4 are app-internal tools** (no CLI equivalent needed: get_spending_limits, request_limit_increase, register_agent, get_agent_info)
- **1 tool has an INVENTED parameter** (send_payment.memo)
- **1 tool has NO CLI equivalent** (get_transactions — awal has no history/transactions command)
- **Several CLI flags are MISSING** from MCP tools (trade slippage, x402 pay method/data/headers/max-amount, bazaar list network/full, bazaar search top-k)

---

## Tool-by-Tool Audit

### 1. `send_payment` → `awal send <amount> <recipient>`

CLI signature: `awal send [options] <amount> <recipient>`

| MCP Param | CLI Flag/Arg | Status | Notes |
|-----------|-------------|--------|-------|
| `to` | `<recipient>` (positional) | OK | Maps to positional arg |
| `amount` | `<amount>` (positional) | OK | Maps to positional arg |
| `asset` | — | **INVENTED** | CLI only sends USDC. There is no `--asset` flag. The `send` command description says "Send USDC to an address". |
| `memo` | — | **INVENTED** | No `--memo` flag exists in the CLI |
| — | `--chain <chain>` | **MISSING** | CLI supports `--chain` (default: base). AwalCommand has it but MCP tool doesn't expose it |

**Severity: MEDIUM** — `asset` and `memo` params will be silently ignored since AwalCommand::Send doesn't use them. The `asset` param is misleading since send only works with USDC.

### 2. `check_balance` → `awal balance`

CLI signature: `awal balance [options]`

| MCP Param | CLI Flag/Arg | Status | Notes |
|-----------|-------------|--------|-------|
| (none) | — | OK | No params needed for basic balance |
| — | `--asset <asset>` | **MISSING** | CLI can filter by asset (usdc, eth, weth) |
| — | `--chain <chain>` | **MISSING** | CLI can filter by chain. AwalCommand has `chain` but MCP tool doesn't expose it |

**Severity: LOW** — Returns all balances by default which is fine.

### 3. `get_address` → `awal address`

CLI signature: `awal address [options]`

| MCP Param | CLI Flag/Arg | Status | Notes |
|-----------|-------------|--------|-------|
| (none) | — | OK | No params needed |

**Severity: NONE** — Perfect match.

### 4. `trade_tokens` → `awal trade <amount> <from> <to>`

CLI signature: `awal trade [options] <amount> <from> <to>`

| MCP Param | CLI Flag/Arg | Status | Notes |
|-----------|-------------|--------|-------|
| `from_asset` | `<from>` (positional) | OK | Maps correctly. CLI also accepts contract addresses (0x...) |
| `to_asset` | `<to>` (positional) | OK | Maps correctly |
| `amount` | `<amount>` (positional) | OK | Maps correctly |
| — | `-c, --chain <chain>` | **MISSING** | CLI supports chain selection (default: base) |
| — | `-s, --slippage <bps>` | **MISSING** | CLI supports slippage tolerance in basis points (default: 100 = 1%) |

**Severity: MEDIUM** — Missing `slippage` is notable for a financial app. Agents can't control slippage tolerance.

### 5. `pay_x402` → `awal x402 pay <url>`

CLI signature: `awal x402 pay [options] <url>`

| MCP Param | CLI Flag/Arg | Status | Notes |
|-----------|-------------|--------|-------|
| `url` | `<url>` (positional) | OK | Maps correctly |
| `max_amount` | `--max-amount <amount>` | **PARTIAL** | CLI flag exists but note: CLI says "in USDC atomic units", MCP description says "optional safety cap" — units mismatch in docs. Also, AwalCommand::X402Pay only has `url`, does NOT pass max_amount to CLI. |
| — | `-X, --method <method>` | **MISSING** | HTTP method (GET, POST, PUT, DELETE, PATCH) |
| — | `-d, --data <json>` | **MISSING** | Request body as JSON |
| — | `-q, --query <params>` | **MISSING** | Query parameters as JSON |
| — | `-h, --headers <json>` | **MISSING** | Custom headers as JSON |
| — | `--correlation-id <id>` | **MISSING** | Correlation ID for grouping operations |

**Severity: HIGH** — `max_amount` is defined in MCP tool schema but **never passed to CLI** (AwalCommand::X402Pay only stores `url`). This means the safety cap is silently ignored. Also missing HTTP method/data means agents can only do GET requests.

### 6. `list_x402_services` → `awal x402 bazaar list`

CLI signature: `awal x402 bazaar list [options]`

| MCP Param | CLI Flag/Arg | Status | Notes |
|-----------|-------------|--------|-------|
| (none) | — | OK | Basic list works |
| — | `--network <network>` | **MISSING** | Filter by network |
| — | `--full` | **MISSING** | Return complete details |

**Severity: LOW** — Defaults are reasonable for basic usage.

### 7. `search_x402_services` → `awal x402 bazaar search <query>`

CLI signature: `awal x402 bazaar search [options] <query>`

| MCP Param | CLI Flag/Arg | Status | Notes |
|-----------|-------------|--------|-------|
| `query` | `<query>` (positional) | OK | Maps correctly |
| — | `-k, --top <n>` | **MISSING** | Number of results (default 5) |
| — | `--force-refresh` | **MISSING** | Force re-fetch before searching |

**Severity: LOW** — Defaults are reasonable.

### 8. `get_x402_details` → `awal x402 details <url>`

CLI signature: `awal x402 details [options] <url>`

| MCP Param | CLI Flag/Arg | Status | Notes |
|-----------|-------------|--------|-------|
| `url` | `<url>` (positional) | OK | Maps correctly |

**Severity: NONE** — Perfect match.

---

## App-Internal Tools (No CLI Equivalent Needed)

These tools are handled entirely by the Rust backend (SQLite database, spending policy engine) and do not call the awal CLI:

### 9. `get_spending_limits`
Internal tool — reads agent spending policy from DB. **OK, no CLI needed.**

### 10. `request_limit_increase`
Internal tool — creates a pending approval request in DB. **OK, no CLI needed.**

### 11. `register_agent`
Internal tool — registers agent with invitation code in DB. **OK, no CLI needed.**

### 12. `get_agent_info`
Internal tool — reads agent profile from DB. **OK, no CLI needed.**

### 13. `get_transactions`

| MCP Param | CLI Flag/Arg | Status | Notes |
|-----------|-------------|--------|-------|
| `limit` | — | N/A | |
| `status` | — | N/A | |

**NOTE**: `awal` has **NO** `transactions`, `history`, or `list` command. There is no way to fetch transaction history from the CLI. This tool is currently handled as app-internal (reads from local SQLite). This is **correct behavior** — the app tracks its own transaction log. However, this means the transaction history only includes transactions made through the app, not any made directly via the CLI.

---

## CLI Commands NOT Exposed as MCP Tools

| CLI Command | Description | Should We Add? |
|-------------|-------------|----------------|
| `awal status` | Server health + auth status | Maybe — useful for agents to check if wallet is healthy |
| `awal auth login` | Start email OTP | No — user-facing only |
| `awal auth verify` | Complete OTP | No — user-facing only |
| `awal show` | Show companion window | No — UI-only |

---

## Critical Issues (Action Required)

### CRITICAL: `pay_x402.max_amount` silently ignored
The MCP tool accepts `max_amount` but `AwalCommand::X402Pay` only stores `url`. The safety cap is never passed to the CLI. In a financial app, this means an agent could pay more than intended.

**Fix**: Add `max_amount: Option<String>` to `AwalCommand::X402Pay` and pass `--max-amount` flag when present.

### HIGH: `send_payment.asset` parameter is invented
The awal `send` command only sends USDC. There is no asset selection. If an agent passes `asset: "ETH"`, the tool will still send USDC. This is misleading.

**Fix**: Remove `asset` from the MCP tool schema, or add a note that send only supports USDC.

### HIGH: `send_payment.memo` parameter is invented
No `--memo` flag exists in the CLI. The parameter is silently ignored.

**Fix**: Remove `memo` from the MCP tool schema.

### MEDIUM: `trade_tokens` missing slippage control
Agents can't set slippage tolerance. Default is 100 bps (1%) which may be acceptable, but for large trades this could matter.

**Fix**: Add optional `slippage` param to MCP tool and `AwalCommand::Trade`.

### MEDIUM: `pay_x402` missing HTTP method/data/headers
Agents can only make GET requests to x402 services. POST endpoints are inaccessible.

**Fix**: Add `method`, `data`, `headers` params to MCP tool and `AwalCommand::X402Pay`.

---

## Arg Order Verification

Verified the positional argument order in `AwalCommand::to_args()` matches the CLI:

| Command | CLI Signature | Our Args | Match? |
|---------|--------------|----------|--------|
| send | `send <amount> <recipient>` | `["send", amount, to, "--json"]` | YES |
| trade | `trade <amount> <from> <to>` | `["trade", amount, from, to, "--json"]` | YES |
| x402 pay | `x402 pay <url>` | `["x402", "pay", url, "--json"]` | YES |
| x402 bazaar list | `x402 bazaar list` | `["x402", "bazaar", "list", "--json"]` | YES |
| x402 bazaar search | `x402 bazaar search <query>` | `["x402", "bazaar", "search", query, "--json"]` | YES |
| x402 details | `x402 details <url>` | `["x402", "details", url, "--json"]` | YES |
| auth login | `auth login <email>` | `["auth", "login", email, "--json"]` | YES |
| auth verify | `auth verify <flowId> <otp>` | `["auth", "verify", flowId, otp, "--json"]` | YES |
| balance | `balance` | `["balance", "--json"]` | YES |
| address | `address` | `["address", "--json"]` | YES |
| status | `status` | `["status", "--json"]` | YES |

All positional argument orders are correct.

---

## Auth Command Note

`AwalCommand::AuthLogout` generates `["auth", "logout", "--json"]` but the CLI `auth --help` only shows `login` and `verify` subcommands. There is no `logout` subcommand visible. This may be an undocumented command or may fail at runtime.
