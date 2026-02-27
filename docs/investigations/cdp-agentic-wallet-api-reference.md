# CDP Agentic Wallet - API / CLI Reference

> Research Date: 2026-02-27
> Sources: https://docs.cdp.coinbase.com/agentic-wallet/skills/*

## CLI Command Reference

All commands use the pattern: `npx awal@latest <command> [args] [options]`

Global option: `--json` returns machine-readable JSON output on all commands.

---

## Authentication Commands

### `status`

Check server health and authentication state.

```bash
npx awal@latest status [--json]
```

**Output**: Server health, authentication status, logged-in email, wallet address.

### `auth login <email>`

Initiate email OTP authentication.

```bash
npx awal@latest auth login <email> [--json]
```

**Parameters**:
- `email` (required): Email address for OTP delivery

**Returns**: `flowId` (string) - needed for the verify step

### `auth verify <flowId> <otp>`

Complete OTP verification.

```bash
npx awal@latest auth verify <flowId> <otp> [--json]
```

**Parameters**:
- `flowId` (required): Flow ID from `auth login` response
- `otp` (required): 6-digit code from email

---

## Wallet Commands

### `balance`

Check USDC wallet balance.

```bash
npx awal@latest balance [--chain <chain>] [--json]
```

**Options**:
- `--chain`: `base` (default) or `base-sepolia`

### `address`

Get the wallet's Base network address.

```bash
npx awal@latest address [--json]
```

**Returns**: Ethereum-format address (0x...)

### `show`

Open the wallet companion window (browser UI).

```bash
npx awal@latest show
```

Used for: Visual wallet management, funding via Coinbase Onramp.

---

## Transaction Commands

### `send <amount> <recipient>`

Send USDC to an Ethereum address or ENS name.

```bash
npx awal@latest send <amount> <recipient> [--chain <chain>] [--json]
```

**Parameters**:
- `amount` (required): Amount to send in one of these formats:
  - Dollar format: `"$5.00"` (quote the dollar sign in shell)
  - Decimal: `0.50`, `1.00`
  - Whole number: `5` (interpreted as 5 USDC if <= 100)
  - Atomic units: `1000000` (= $1.00 USDC; values > 100 without decimals are treated as atomic)
- `recipient` (required): One of:
  - Ethereum address: `0x1234...abcd`
  - ENS name: `vitalik.eth`

**Options**:
- `--chain`: `base` (default) or `base-sepolia`

**Amount Conversion Table**:

| Input | Interpreted As |
|-------|---------------|
| `"$5.00"` | 5.00 USDC |
| `5` | 5.00 USDC |
| `0.50` | 0.50 USDC |
| `1000000` | 1.00 USDC (atomic units, 6 decimals) |
| `500000` | 0.50 USDC (atomic units) |

**ENS Resolution**: Automatically resolves ENS names via Ethereum mainnet. Output shows both the ENS name and resolved address.

**Examples**:
```bash
npx awal@latest send "$5.00" 0x1234...abcd
npx awal@latest send 0.50 vitalik.eth
npx awal@latest send 1 0x1234...abcd --chain base-sepolia
npx awal@latest send 1 vitalik.eth --json
```

**Common Errors**:
- "Not authenticated" - Run auth login first
- "Insufficient funds" - Check balance and fund wallet
- "Invalid ENS" - Confirm ENS domain exists
- "Bad recipient format" - Use valid 0x address or ENS name

---

### `trade <amount> <from> <to>`

Swap tokens on Base mainnet.

```bash
npx awal@latest trade <amount> <from> <to> [-s <slippage>] [--json]
```

**Parameters**:
- `amount` (required): Amount of source token (same format rules as `send`)
- `from` (required): Source token - alias or contract address
- `to` (required): Destination token - alias or contract address

**Token Aliases**:

| Alias | Token | Decimals | Contract Address (Base) |
|-------|-------|----------|------------------------|
| `usdc` | USDC | 6 | `0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913` |
| `eth` | ETH | 18 | `0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE` |
| `weth` | WETH | 18 | `0x4200000000000000000000000000000000000006` |

**Options**:
- `-s, --slippage <n>`: Price slippage tolerance in basis points (100 = 1%). Default varies.

**Examples**:
```bash
npx awal@latest trade $1 usdc eth
npx awal@latest trade 0.01 eth usdc
npx awal@latest trade $5 usdc eth --slippage 200
npx awal@latest trade $1 usdc eth --json
# Using contract addresses directly:
npx awal@latest trade 100 0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913 0x4200000000000000000000000000000000000006
```

**Common Errors**:

| Error | Solution |
|-------|----------|
| "Not authenticated" | Run auth login |
| "Invalid token" | Check alias or use full 0x contract address |
| "Cannot trade a token to itself" | Use different from/to tokens |
| "TRANSFER_FROM_FAILED" | Check balance and token approvals |
| "No liquidity" | Reduce amount or try different pair |
| Decimal precision error | Match token's decimal places |

**Limitations**: Trading is **mainnet only** (Base). Not available on Base Sepolia testnet.

---

## x402 Commerce Commands

### `x402 bazaar search <query>`

Search the x402 bazaar for paid API services.

```bash
npx awal@latest x402 bazaar search <query> [-k <n>] [--force-refresh] [--json]
```

**Parameters**:
- `query` (required): Search keywords

**Options**:
- `-k <n>`: Number of results (default: 5)
- `--force-refresh`: Bypass 12-hour cache
- `--json`: JSON output

**No authentication required** for search.

**Cache**: Results cached locally at `~/.config/awal/bazaar/` with 12-hour auto-refresh.

### `x402 bazaar list`

List all available x402 services.

```bash
npx awal@latest x402 bazaar list [--network <network>] [--full] [--json]
```

**Options**:
- `--network`: Filter by `base` or `base-sepolia`
- `--full`: Show complete details
- `--json`: JSON output

### `x402 details <url>`

Inspect pricing and payment details for a specific x402 endpoint.

```bash
npx awal@latest x402 details <url> [--json]
```

**Returns**: Pricing, accepted payment schemes, network, API schemas. No payment required.

### `x402 pay <url>`

Make a paid API request to an x402 endpoint.

```bash
npx awal@latest x402 pay <url> [-X <method>] [-d <json>] [-q <params>] [-h <json>] [--max-amount <n>] [--correlation-id <id>] [--json]
```

**Parameters**:
- `url` (required): x402-enabled endpoint URL

**Options**:

| Option | Description |
|--------|-------------|
| `-X, --method <method>` | HTTP method (default: GET) |
| `-d, --data <json>` | JSON request body |
| `-q, --query <params>` | URL query parameters as JSON |
| `-h, --headers <json>` | Custom HTTP headers as JSON |
| `--max-amount <n>` | Maximum USDC spend in atomic units |
| `--correlation-id <id>` | Link related transactions |
| `--json` | JSON output |

**USDC Atomic Unit Reference**:

| Atomic Units | USDC Amount |
|-------------|-------------|
| 1,000,000 | $1.00 |
| 100,000 | $0.10 |
| 50,000 | $0.05 |
| 10,000 | $0.01 |
| 1,000 | $0.001 |

**Examples**:
```bash
# Basic GET
npx awal@latest x402 pay https://example.com/api/weather

# POST with body
npx awal@latest x402 pay https://example.com/api/sentiment -X POST -d '{"text": "I love this"}'

# With spending limit
npx awal@latest x402 pay https://example.com/api/data --max-amount 100000

# With query parameters
npx awal@latest x402 pay https://example.com/api/search -q '{"q": "bitcoin price"}'
```

**Requirements**: Must be authenticated with sufficient USDC balance. Target URL must be x402-enabled.

---

## Skills Reference

Each skill is a structured definition that teaches AI agents how to use the CLI commands above.

### Installed Skills (8 total)

| Skill ID | Purpose | Requires Auth | Requires Balance |
|----------|---------|---------------|-----------------|
| `authenticate-wallet` | Sign in via email OTP | No | No |
| `fund` | Add money via Coinbase Onramp | Yes | No |
| `send-usdc` | Send USDC to address/ENS | Yes | Yes |
| `trade` | Swap tokens on Base | Yes | Yes |
| `search-for-service` | Search x402 bazaar | No | No |
| `pay-for-service` | Pay for x402 API | Yes | Yes |
| `monetize-service` | Deploy paid API | Yes | No |
| `query-onchain-data` | Query Base data via CDP SQL API | Yes | Yes |

### Skill File Structure

Each skill is a `SKILL.md` file with:
- YAML frontmatter (name, description)
- Step-by-step instructions for the agent
- Specific `awal` commands to execute
- Tool permissions (what the agent can do without asking)

### Installation

```bash
npx skills add coinbase/agentic-wallet-skills
```

Skills activate automatically after installation and are available to the AI agent.
