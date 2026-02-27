# CDP Agentic Wallet - Overview

> Research Date: 2026-02-27
> Source: https://docs.cdp.coinbase.com/agentic-wallet/welcome

## What is Agentic Wallet?

Agentic Wallet is a Coinbase product that gives AI agents a standalone, self-custody wallet to hold and spend stablecoins (USDC) or trade for other tokens on Base. It is designed specifically for agent-to-agent commerce and AI-driven financial operations.

Key characteristics:
- **Self-custody wallet** controlled by the agent (not a custodial solution)
- **Private keys remain in Coinbase infrastructure** - agents never have direct access to keys
- **CLI/MCP-based interface** - agents interact via the `awal` CLI tool, not an imported SDK
- **Base network only** - all operations occur on Coinbase's Base L2
- **USDC-centric** - primary currency is USDC with ability to trade to ETH/WETH

## AgentKit vs Agentic Wallet

These are two distinct Coinbase products. Understanding the difference is critical:

| Aspect | AgentKit | Agentic Wallet |
|--------|----------|----------------|
| Type | SDK/toolkit imported into agent code | Standalone wallet accessed via CLI/MCP |
| Integration | Import packages, call functions | Shell out to `npx awal` commands |
| Capabilities | Full onchain (deploy contracts, NFTs, etc.) | Wallet operations only (send, trade, fund) |
| Networks | Multi-network (EVM chains + Solana) | Base only |
| Key Management | Multiple wallet providers (CDP, Viem, Privy, etc.) | Email OTP authentication, keys in Coinbase infra |
| Use Case | Building onchain agents with full blockchain access | Giving agents a payment wallet for commerce |

**For our project**: Agentic Wallet is the simpler, more focused option if we just need agents to hold/send/receive USDC and trade tokens. AgentKit is for deeper blockchain integration.

## Core Architecture

### Components

1. **`awal` CLI** - Command-line tool that agents invoke via `npx awal@latest <command>`
2. **Agent Skills** - Pre-built capability definitions that tell AI agents how to use the CLI
3. **x402 Protocol** - HTTP-native payment protocol for agent-to-agent commerce
4. **Coinbase Infrastructure** - Backend handling key management, transaction signing, KYT screening

### How It Works

```
AI Agent
  |
  |-- invokes --> npx awal@latest <command>
  |                    |
  |                    |-- authenticates via email OTP
  |                    |-- sends USDC to addresses/ENS names
  |                    |-- trades tokens on Base
  |                    |-- pays for x402 services
  |                    |-- searches x402 bazaar
  |
  |-- receives --> JSON/text responses from CLI
```

### Authentication Model

- Email OTP-based authentication (no API keys needed)
- Agent initiates login with `npx awal auth login <email>`
- 6-digit OTP sent to email
- Agent verifies with `npx awal auth verify <flowId> <otp>`
- If the agent has access to the user's email, it can read the OTP directly
- Session persists after authentication

### Security Model

1. **Private keys never exposed** - Keys remain in Coinbase infrastructure
2. **Spending guardrails** - Configurable spending limits enforced pre-transaction
3. **KYT (Know Your Transaction) screening** - Blocks high-risk interactions automatically
4. **Email OTP auth** - No raw private keys or API secrets for the agent to leak

## Supported Operations

| Operation | Description | Command |
|-----------|-------------|---------|
| Status | Check server health and auth state | `npx awal status` |
| Login | Initiate email OTP auth | `npx awal auth login <email>` |
| Verify | Complete OTP verification | `npx awal auth verify <flowId> <otp>` |
| Balance | Check USDC balance | `npx awal balance` |
| Address | Get wallet address | `npx awal address` |
| Send | Send USDC to address/ENS | `npx awal send <amount> <recipient>` |
| Trade | Swap tokens on Base | `npx awal trade <amount> <from> <to>` |
| Fund | Open Coinbase Onramp | `npx awal show` |
| Search | Find x402 paid services | `npx awal x402 bazaar search <query>` |
| Pay | Pay for x402 service | `npx awal x402 pay <url>` |

## Supported Chains and Tokens

### Chain Support
- **Base Mainnet** (`eip155:8453`) - Production
- **Base Sepolia** (`eip155:84532`) - Testnet

### Token Support

| Token | Decimals | Contract Address (Base Mainnet) |
|-------|----------|-------------------------------|
| USDC | 6 | `0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913` |
| ETH | 18 | `0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE` |
| WETH | 18 | `0x4200000000000000000000000000000000000006` |

Custom token contract addresses can also be used for trading.

## x402 Protocol

x402 is an HTTP-native payment protocol that enables agent-to-agent commerce. Key aspects:

- Extends HTTP 402 (Payment Required) status code
- Stateless, HTTP-native, blockchain-agnostic design
- Enables machine-to-machine API payments
- Agents can both consume and provide paid services

### x402 Network Support

The x402 protocol itself supports broader networks than Agentic Wallet:

**CDP Facilitator (Production):**
- Base mainnet (`eip155:8453`)
- Base Sepolia (`eip155:84532`)
- Solana mainnet
- Solana devnet

**EVM Reference SDK:**
- Any EVM-compatible chain with EIP-3009 token support (Ethereum, Optimism, Polygon, Arbitrum, Avalanche, etc.)

**Solana Reference SDK:**
- SPL Token Program tokens (v1 and v2)
- Token2022 program tokens (v2 only)

### x402 Payment Flow

1. Client sends HTTP request to paid endpoint
2. Server returns HTTP 402 with payment requirements in header
3. Client constructs and signs USDC payment
4. Client retransmits request with `PAYMENT-SIGNATURE` header
5. Server validates payment (directly or via facilitator)
6. Server processes request and settles payment on-chain
7. Server returns 200 OK with response and `PAYMENT-RESPONSE` header

## Use Cases

1. **Pay-per-call APIs** - Agents pay USDC for API access via x402
2. **Gasless payments** - USDC transfers without gas fees on Base
3. **Agent-to-agent commerce** - Agents buying/selling services from each other
4. **Spending limits** - Configurable per-session and per-transaction limits
5. **Tips and donations** - Send USDC to any address or ENS name
6. **Token trading** - Swap between USDC, ETH, WETH on Base

## Skills Repository

- **GitHub**: https://github.com/coinbase/agentic-wallet-skills
- **License**: MIT
- **Installation**: `npx skills add coinbase/agentic-wallet-skills`
- **Skills format**: Each skill is a folder with a `SKILL.md` file containing YAML frontmatter and instructions
- Uses Vercel's Skills CLI for installation

## Key Limitations

1. **Base only** - No multi-chain support (unlike AgentKit)
2. **USDC-centric** - Primary stablecoin; can trade to ETH/WETH but limited token universe
3. **CLI-based** - Must shell out to `npx awal` commands; no native SDK import
4. **Email OTP** - Requires email access for authentication
5. **Trading on mainnet only** - Token trading not available on testnet
6. **Three tokens only** - Built-in aliases for USDC, ETH, WETH (custom addresses possible)
