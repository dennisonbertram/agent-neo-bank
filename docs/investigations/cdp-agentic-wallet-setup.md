# CDP Agentic Wallet - Setup & Authentication

> Research Date: 2026-02-27
> Source: https://docs.cdp.coinbase.com/agentic-wallet/quickstart

## Prerequisites

- **Node.js v24+** (required for `npx awal`)
- **Email address** for OTP authentication
- No API keys or CDP account required for basic usage
- For x402 mainnet facilitator: CDP account with `CDP_API_KEY_ID` and `CDP_API_KEY_SECRET`

## Installation

### Install Agent Skills

```bash
npx skills add coinbase/agentic-wallet-skills
```

This uses Vercel's Skills CLI to install all 8 wallet skills into your agent environment.

### CLI Tool

The CLI tool `awal` is invoked via `npx` - no global installation required:

```bash
npx awal@latest <command>
```

Always use `@latest` to ensure you have the most recent version.

## Authentication Flow

### Step 1: Check Status

```bash
npx awal@latest status
```

Returns server health and authentication state. Use `--json` for machine-readable output:

```bash
npx awal@latest status --json
```

### Step 2: Initiate Login

```bash
npx awal@latest auth login user@example.com
```

This sends a 6-digit OTP to the specified email and returns a `flowId`.

With JSON output:
```bash
npx awal@latest auth login user@example.com --json
```

### Step 3: Verify OTP

```bash
npx awal@latest auth verify <flowId> <otp>
```

Example:
```bash
npx awal@latest auth verify abc123-flow-id 482901
```

With JSON output:
```bash
npx awal@latest auth verify <flowId> <otp> --json
```

### Step 4: Confirm Authentication

```bash
npx awal@latest status
```

Should now show authenticated state with wallet address.

### Agent Email Access

If the agent has access to the user's email (e.g., via Gmail API or email MCP tool), it can read the OTP code directly and complete authentication autonomously. Otherwise, the agent must ask the user for the code.

## Post-Authentication Commands

### Check Balance

```bash
npx awal@latest balance
# Specify chain:
npx awal@latest balance --chain base
npx awal@latest balance --chain base-sepolia
```

### Get Wallet Address

```bash
npx awal@latest address
```

### Open Companion Window

```bash
npx awal@latest show
```

Opens the wallet companion UI for funding and visual management.

## Funding the Wallet

### Via Coinbase Onramp

1. Run `npx awal@latest show` to open the companion window
2. Click the "Fund" button
3. Select amount (preset: $10, $20, $50, or custom)
4. Choose payment method:
   - **Apple Pay** - Instant
   - **Coinbase account** - Transfer from existing account
   - **Debit card** - Instant
   - **ACH bank transfer** - 1-3 business days
5. Complete payment via Coinbase Pay browser interface
6. USDC deposits to wallet on Base network

### Via Direct Transfer

Send USDC directly to your wallet's Base network address:

```bash
# Get your address
npx awal@latest address
# Then send USDC to that address from any wallet/exchange
```

## Complete Quickstart Sequence

```bash
# 1. Check status
npx awal@latest status

# 2. Login
npx awal@latest auth login me@example.com

# 3. Verify (use flowId from step 2 and OTP from email)
npx awal@latest auth verify <flowId> <otp>

# 4. Check balance
npx awal@latest balance

# 5. Send USDC
npx awal@latest send 1 vitalik.eth

# 6. Trade tokens
npx awal@latest trade 5 usdc eth
```

## JSON Output

All commands support `--json` flag for programmatic parsing:

```bash
npx awal@latest status --json
npx awal@latest balance --json
npx awal@latest address --json
npx awal@latest send 1 vitalik.eth --json
npx awal@latest trade 1 usdc eth --json
```

## x402 Mainnet Setup (For Monetization)

If you want to use the CDP facilitator on mainnet for x402 services:

1. Create account at https://cdp.coinbase.com
2. Generate API credentials from the portal
3. Set environment variables:
   ```bash
   export CDP_API_KEY_ID=<your-key-id>
   export CDP_API_KEY_SECRET=<your-key-secret>
   ```
4. Use production facilitator URL (instead of x402.org testnet)
5. Use mainnet network identifiers:
   - Base: `eip155:8453`
   - Solana: `solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp`

## Testnet Setup

For development/testing, use Base Sepolia:

```bash
# Check testnet balance
npx awal@latest balance --chain base-sepolia

# Send on testnet
npx awal@latest send 1 0x1234...abcd --chain base-sepolia
```

Note: Token trading is **mainnet only** - not available on testnet.

## Troubleshooting

| Issue | Solution |
|-------|----------|
| "Not authenticated" | Run `npx awal auth login <email>` and verify OTP |
| OTP not received | Check spam folder; try again after a few minutes |
| Node.js version error | Upgrade to Node.js v24+ |
| Balance shows 0 | Fund wallet via Onramp or direct USDC transfer |
| Command not found | Use `npx awal@latest` (not `awal` directly) |
