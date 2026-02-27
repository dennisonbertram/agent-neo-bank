# AWAL CLI Real Output Investigation

**Date**: 2026-02-27
**AWAL Version**: 1.0.0 (npm package: awal@2.0.3)
**Node.js Version**: v22.21.1 (docs say v24+ required, but works fine on v22)
**Platform**: macOS Darwin 24.1.0

## Environment Notes

- Port 1420: Running a Vite dev server for agent-neo-bank project (`node .../node_modules/.bin/vite`)
- The AWAL CLI spawns a background wallet server process (visible in `status` output as PID)
- Node v24+ requirement appears to be soft -- all commands work on v22.21.1

---

## Command: `npx awal@latest --version`

**Exit Code**: 0
**Stdout**:
```
1.0.0
```

---

## Command: `npx awal@latest --help`

**Exit Code**: 0
**Stdout**:
```
Usage: awal [options] [command]

Coinbase Wallet CLI for payments and crypto

Options:
  -V, --version                         output the version number
  -h, --help                            display help for command

Commands:
  status [options]                      Check wallet server health and
                                        authentication status
  balance [options]                     Get wallet balances (USDC, ETH, WETH)
  address [options]                     Get wallet address
  show                                  Show and focus the wallet companion
                                        window
  x402                                  X402 payment protocol commands
  auth                                  Authentication commands
  send [options] <amount> <recipient>   Send USDC to an address
  trade [options] <amount> <from> <to>  Swap tokens on Base network via CDP Swap API.

  Examples:
    npx awal trade $1 usdc eth          # Swap $1 USDC for ETH
    npx awal trade 0.50 usdc eth        # Swap $0.50 USDC for ETH
    npx awal trade 0.01 eth usdc        # Swap 0.01 ETH for USDC
    npx awal trade 100 0x... 0x...      # Swap using contract addresses
  help [command]                        display help for command
```

---

## Command: `npx awal@latest status --json`

**Exit Code**: 0
**Spinner**: `- Checking status...` (stderr, animated)
**Stdout (JSON)**:
```json
{
  "server": {
    "running": true,
    "pid": 11705
  },
  "auth": {
    "authenticated": false
  }
}
```

**Notes**:
- Without `--json`, the status command hangs/takes very long with just a spinner
- The `--json` flag is essential for programmatic use
- Server appears to auto-start when any awal command is run

---

## Command: `npx awal@latest balance --json` (unauthenticated)

**Exit Code**: 1
**Spinner**: `- Fetching balances...` (stderr)
**Stdout**:
```
✖ Failed to fetch balances
Authentication required.

Sign in using one of:
  1. Email OTP:
     npx awal auth login <your-email>
     npx awal auth verify <flow-id> <6-digit-code>

  2. Wallet UI:
     npx awal show
```

**Notes**:
- Even with `--json`, the error output is NOT JSON -- it's plain text
- The checkmark/cross symbols are Unicode: ✖ (U+2716)
- Exit code is 1 for auth failures

---

## Command: `npx awal@latest address --json` (unauthenticated)

**Exit Code**: 1
**Spinner**: `- Fetching wallet address...` (stderr)
**Stdout**:
```
✖ Failed to fetch address
Authentication required.

Sign in using one of:
  1. Email OTP:
     npx awal auth login <your-email>
     npx awal auth verify <flow-id> <6-digit-code>

  2. Wallet UI:
     npx awal show
```

---

## Command: `npx awal@latest send 0.01 0x0000...0000 --json` (unauthenticated)

**Exit Code**: 1
**Spinner**: `- Preparing transaction...` (stderr)
**Stdout**:
```
✖ Transaction failed
Authentication required.

Sign in using one of:
  1. Email OTP:
     npx awal auth login <your-email>
     npx awal auth verify <flow-id> <6-digit-code>

  2. Wallet UI:
     npx awal show
```

---

## Command: `npx awal@latest trade 1 usdc eth --json` (unauthenticated)

**Exit Code**: 1
**Spinner**: `- Preparing swap...` (stderr)
**Stdout**:
```
✖ Swap failed
Authentication required.

Sign in using one of:
  1. Email OTP:
     npx awal auth login <your-email>
     npx awal auth verify <flow-id> <6-digit-code>

  2. Wallet UI:
     npx awal show
```

---

## Auth Flow

### Command: `npx awal@latest auth login test@example.com --json`

**Exit Code**: 0
**Spinner**: `- Sending verification code...` (stderr)
**Stdout (JSON)**:
```json
{
  "flowId": "8137a783-2e95-4c6c-b3cc-0df36bebebb9",
  "message": "Verification code sent to test@example.com. Ask the user for the 6-digit code from their email."
}
```

**Notes**:
- This actually sends a verification email (even to test@example.com)
- The flowId is a UUID v4 format
- The message field includes instructions for the agent calling it
- Non-interactive -- no prompts, email is a positional argument

### Command: `npx awal@latest auth verify <flowId> <otp> --json`

**Exit Code**: 1 (with invalid/expired flow)
**Spinner**: `- Verifying code...` (stderr)
**Stdout**:
```
✖ Verification failed
Bridge communication error: Verification code expired. Please call sign_in_with_email again to get a new code.

This may indicate a configuration issue. Try restarting the wallet.
```

**Notes**:
- Error output is plain text even with `--json`
- flowId and otp are positional arguments (not interactive)
- The error mentions "Bridge communication error" suggesting internal RPC to wallet server

---

## Auth Subcommand Help

### `npx awal@latest auth --help`
```
Usage: awal auth [options] [command]

Authentication commands

Options:
  -h, --help                       display help for command

Commands:
  login [options] <email>          Start email OTP authentication
  verify [options] <flowId> <otp>  Complete email OTP verification
  help [command]                   display help for command
```

### `npx awal@latest auth login --help`
```
Usage: awal auth login [options] <email>

Start email OTP authentication

Arguments:
  email       Email address

Options:
  --json      Output as JSON
  -h, --help  display help for command
```

### `npx awal@latest auth verify --help`
```
Usage: awal auth verify [options] <flowId> <otp>

Complete email OTP verification

Arguments:
  flowId      Flow ID from login command
  otp         6-digit verification code

Options:
  --json      Output as JSON
  -h, --help  display help for command
```

---

## X402 Commands

### `npx awal@latest x402 --help`
```
Usage: awal x402 [options] [command]

X402 payment protocol commands

Options:
  -h, --help               display help for command

Commands:
  bazaar                   Browse X402 payment-enabled services
  pay [options] <url>      Make HTTP request with X402 payment handling
  details [options] <url>  Discover X402 payment requirements for an endpoint
                           (auto-detects HTTP method)
  help [command]           display help for command
```

### `npx awal@latest x402 pay --help`
```
Usage: awal x402 pay [options] <url>

Make HTTP request with X402 payment handling

Arguments:
  url                    Full URL of the X402-enabled endpoint

Options:
  -X, --method <method>  HTTP method (GET, POST, PUT, DELETE, PATCH) (default: "GET")
  -d, --data <json>      Request body as JSON
  -q, --query <params>   Query parameters as JSON
  -h, --headers <json>   Custom headers as JSON
  --max-amount <amount>  Maximum amount per request in USDC atomic units
  --correlation-id <id>  Correlation ID to group related operations
  --json                 Output as JSON
  --help                 display help for command
```

### `npx awal@latest x402 details --help`
```
Usage: awal x402 details [options] <url>

Discover X402 payment requirements for an endpoint (auto-detects HTTP method)

Arguments:
  url         Full URL of the X402-enabled endpoint

Options:
  --json      Output as JSON
  -h, --help  display help for command
```

### `npx awal@latest x402 bazaar list --json` (sample)

**Exit Code**: 0
**Spinner**: `- Fetching bazaar resources...` (stderr)
**Stdout**: Large JSON with `items` array. Sample entries:

```json
{
  "items": [
    {
      "resource": "https://public.zapper.xyz/x402/account-identity",
      "description": "Get social identity (ENS, Farcaster, Lens, Basenames) for a specific address.",
      "maxAmountRequired": "1100",
      "network": "base",
      "scheme": "exact"
    },
    {
      "resource": "https://public.zapper.xyz/x402/nft-collection-metadata",
      "description": "Get NFT collection data including market stats, holders, events, and a sample of NFTs.",
      "maxAmountRequired": "1100",
      "network": "base",
      "scheme": "exact"
    },
    {
      "resource": "https://public.zapper.xyz/x402/token-price",
      "description": "Get real-time token price and market cap",
      "maxAmountRequired": "1100",
      "network": "base",
      "scheme": "exact"
    },
    {
      "resource": "https://api.nansen.ai/api/v1/smart-money/netflow",
      "description": "Get Smart Money Netflow Data",
      "maxAmountRequired": "50000",
      "network": "eip155:8453",
      "scheme": "exact"
    }
  ]
}
```

**Note**: Full bazaar list is ~234KB JSON with many providers (Zapper, Nansen, etc.)

---

## Other Command Help

### `npx awal@latest balance --help`
```
Usage: awal balance [options]

Get wallet balances (USDC, ETH, WETH)

Options:
  --asset <asset>  Show specific asset only (usdc, eth, weth)
  --chain <chain>  Blockchain network (e.g., base, base-sepolia)
  --json           Output as JSON
  -h, --help       display help for command
```

### `npx awal@latest send --help`
```
Usage: awal send [options] <amount> <recipient>

Send USDC to an address

Arguments:
  amount           Amount to send (e.g., "$0.01", "0.01", or atomic units)
  recipient        Recipient address (0x...) or ENS domain

Options:
  --chain <chain>  Blockchain network (default: base) (default: "base")
  --json           Output as JSON
  -h, --help       display help for command
```

### `npx awal@latest trade --help`
```
Usage: awal trade [options] <amount> <from> <to>

Swap tokens on Base network via CDP Swap API.

Examples:
  npx awal trade $1 usdc eth          # Swap $1 USDC for ETH
  npx awal trade 0.50 usdc eth        # Swap $0.50 USDC for ETH
  npx awal trade 0.01 eth usdc        # Swap 0.01 ETH for USDC
  npx awal trade 100 0x... 0x...      # Swap using contract addresses

Arguments:
  amount                Amount to swap (e.g., "$1.00", "0.50", "500000")
  from                  Source token: alias (usdc, eth, weth) or contract address (0x...)
  to                    Destination token: alias (usdc, eth, weth) or contract address (0x...)

Options:
  -c, --chain <chain>   Blockchain network (default: "base")
  -s, --slippage <bps>  Slippage tolerance in basis points (100 = 1%) (default: "100")
  --json                Output result as JSON
  -h, --help            display help for command
```

---

## Summary of Patterns for MockCliExecutor

### Output Format Patterns

1. **Success with JSON**: When `--json` is used and command succeeds, stdout is valid JSON
2. **Error output**: Always plain text even with `--json` flag, starts with `✖` symbol
3. **Spinners**: Animated spinner text on stderr (e.g., `- Checking status...`, `- Fetching balances...`)
4. **Auth errors**: Consistent multi-line format with sign-in instructions

### Spinner Messages by Command
| Command | Spinner Text |
|---------|-------------|
| status | `Checking status...` |
| balance | `Fetching balances...` |
| address | `Fetching wallet address...` |
| send | `Preparing transaction...` |
| trade | `Preparing swap...` |
| auth login | `Sending verification code...` |
| auth verify | `Verifying code...` |
| x402 bazaar list | `Fetching bazaar resources...` |

### Error Message Patterns (unauthenticated)
| Command | Error Header |
|---------|-------------|
| balance | `✖ Failed to fetch balances` |
| address | `✖ Failed to fetch address` |
| send | `✖ Transaction failed` |
| trade | `✖ Swap failed` |
| auth verify | `✖ Verification failed` |

### Auth Error Block (appears after error header for auth-required commands)
```
Authentication required.

Sign in using one of:
  1. Email OTP:
     npx awal auth login <your-email>
     npx awal auth verify <flow-id> <6-digit-code>

  2. Wallet UI:
     npx awal show
```

### Exit Codes
- **0**: Success (status, auth login, help, version, bazaar list)
- **1**: Failure (auth required, verification failed, transaction failed)

### Key Observations
1. `auth login` is NOT interactive -- email is a positional arg, returns JSON with flowId
2. `auth verify` is NOT interactive -- flowId and OTP are positional args
3. The `--json` flag is available on most commands but errors are always plain text
4. The wallet server auto-starts and runs as a background process
5. `status` without `--json` can hang; always use `--json` for programmatic access
6. `send` defaults to `--chain base`, supports ENS domains
7. `trade` supports token aliases (usdc, eth, weth) or contract addresses
8. `balance` supports `--chain base-sepolia` for testnet
9. `x402 pay` supports full HTTP methods, custom headers, query params, and request bodies
