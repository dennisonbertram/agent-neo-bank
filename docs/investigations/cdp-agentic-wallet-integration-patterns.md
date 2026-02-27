# CDP Agentic Wallet - Integration Patterns

> Research Date: 2026-02-27
> Sources: All Agentic Wallet and x402 documentation pages

## Integration Architecture Options

There are three primary ways to integrate Agentic Wallet capabilities into an AI agent system:

### Option 1: Agent Skills (Recommended for AI Agents)

Install pre-built skills that teach your AI agent how to use the wallet CLI.

```bash
npx skills add coinbase/agentic-wallet-skills
```

The agent then receives natural language instructions for each capability. Example prompts the agent can handle:
- "Sign in to my wallet with me@email.com"
- "Send 10 USDC to barmstrong.eth"
- "Buy $5 of ETH"
- "Search for weather API services"
- "Pay for sentiment analysis at this URL"

**Best for**: Claude, ChatGPT, or other LLM agents with skills/tool support.

### Option 2: Direct CLI Integration

Shell out to `npx awal@latest` commands from your application code.

```typescript
import { execSync } from 'child_process';

function awalCommand(cmd: string): string {
  const result = execSync(`npx awal@latest ${cmd} --json`, {
    encoding: 'utf-8',
    timeout: 30000,
  });
  return JSON.parse(result);
}

// Example usage
const status = awalCommand('status');
const balance = awalCommand('balance');
const sendResult = awalCommand('send 5 vitalik.eth');
```

**Best for**: Custom agent frameworks, programmatic integrations, automation scripts.

### Option 3: x402 SDK Integration (For Payment Only)

Use the x402 client SDK directly for paying for services, without the `awal` CLI.

```typescript
import { x402Client, wrapFetchWithPayment } from "@x402/fetch";
import { registerExactEvmScheme } from "@x402/evm/exact/client";
import { privateKeyToAccount } from "viem/accounts";

const signer = privateKeyToAccount(process.env.EVM_PRIVATE_KEY);
const client = new x402Client();
registerExactEvmScheme(client, { signer });

const fetchWithPayment = wrapFetchWithPayment(fetch, client);
const response = await fetchWithPayment("https://api.example.com/paid-endpoint");
```

**Best for**: Direct programmatic x402 payments without the full Agentic Wallet.

---

## Pattern 1: Autonomous Agent with Wallet

An agent that can authenticate, fund, and transact independently.

### Architecture

```
User
  |
  |-- "Send $5 USDC to alice.eth"
  |
Agent (Claude/GPT/etc.)
  |
  |-- checks auth: npx awal status --json
  |-- if not authed: npx awal auth login <email>
  |     |-- reads OTP from email (if has email access)
  |     |-- npx awal auth verify <flowId> <otp>
  |-- checks balance: npx awal balance --json
  |-- if insufficient: prompts user to fund
  |-- executes: npx awal send 5 alice.eth --json
  |-- returns confirmation to user
```

### Implementation Notes

- Agent needs shell/bash execution capability
- All commands support `--json` for reliable parsing
- Agent should always check status before operations
- Agent should always check balance before send/trade
- If agent has email access (Gmail MCP, etc.), it can complete auth autonomously

---

## Pattern 2: Agent-to-Agent Commerce via x402

Two agents transacting with each other - one provides a service, the other pays for it.

### Seller Agent: Creating a Paid API

```javascript
// x402-server/index.js
const express = require("express");
const { paymentMiddleware } = require("x402-express");

const app = express();

const payTo = "0xYourWalletAddress"; // from: npx awal address

const routes = {
  "GET /api/weather": {
    price: "$0.01",
    network: "base",
    config: {
      description: "Get current weather data for a city",
      inputSchema: {
        type: "object",
        properties: {
          city: { type: "string", description: "City name" }
        }
      },
      outputSchema: {
        type: "object",
        properties: {
          temp: { type: "number" },
          conditions: { type: "string" }
        }
      }
    }
  },
  "POST /api/sentiment": {
    price: "$0.05",
    network: "base",
    config: {
      description: "Analyze sentiment of text",
      inputSchema: {
        type: "object",
        properties: {
          text: { type: "string" }
        }
      }
    }
  }
};

// Free endpoints go BEFORE the middleware
app.get("/health", (req, res) => res.json({ status: "ok" }));

// Apply payment middleware
app.use(paymentMiddleware(payTo, routes));

// Protected endpoints
app.get("/api/weather", (req, res) => {
  res.json({ temp: 72, conditions: "sunny" });
});

app.post("/api/sentiment", express.json(), (req, res) => {
  res.json({ sentiment: "positive", score: 0.95 });
});

app.listen(3000);
```

**Setup:**
```bash
mkdir x402-server && cd x402-server
npm init -y
npm install express x402-express
node index.js
```

### Buyer Agent: Consuming Paid APIs

```bash
# Discover services
npx awal@latest x402 bazaar search "weather API"

# Check pricing
npx awal@latest x402 details https://weather-api.example.com/api/weather

# Pay and call
npx awal@latest x402 pay https://weather-api.example.com/api/weather -q '{"city": "NYC"}'

# POST with body
npx awal@latest x402 pay https://sentiment.example.com/api/sentiment \
  -X POST \
  -d '{"text": "This product is amazing"}' \
  --max-amount 100000
```

### Pricing Guidelines for Services

| Service Type | Suggested Price Range |
|-------------|----------------------|
| Simple lookup / static data | $0.001 - $0.01 |
| API enrichment / data processing | $0.01 - $0.10 |
| Computational queries | $0.10 - $0.50 |
| AI inference / generation | $0.05 - $1.00 |

---

## Pattern 3: MCP Integration

Using AgentKit's MCP extension for deeper blockchain integration beyond Agentic Wallet.

### Setup

```bash
npm install @coinbase/agentkit-model-context-protocol @coinbase/agentkit @modelcontextprotocol/sdk
```

### Environment Variables

```bash
CDP_API_KEY_NAME=<your-key-name>
CDP_API_KEY_PRIVATE_KEY=<your-private-key>
```

### Server Implementation

```typescript
import { AgentKit } from "@coinbase/agentkit";
import { getMcpTools } from "@coinbase/agentkit-model-context-protocol";
import { Server } from "@modelcontextprotocol/sdk/server/index.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";

// Initialize AgentKit
const agentKit = await AgentKit.from({
  cdpApiKeyName: process.env.CDP_API_KEY_NAME,
  cdpApiKeyPrivateKey: process.env.CDP_API_KEY_PRIVATE_KEY,
});

// Get MCP tools
const tools = await getMcpTools(agentKit);

// Create MCP server
const server = new Server(
  { name: "agentkit-mcp", version: "1.0.0" },
  { capabilities: { tools: {} } }
);

// Register tool handlers
server.setRequestHandler(ListToolsRequestSchema, async () => ({
  tools: tools.map(t => ({ name: t.name, description: t.description, inputSchema: t.inputSchema }))
}));

server.setRequestHandler(CallToolRequestSchema, async (request) => {
  const tool = tools.find(t => t.name === request.params.name);
  const result = await tool.execute(request.params.arguments);
  return { content: [{ type: "text", text: JSON.stringify(result) }] };
});

// Start
const transport = new StdioServerTransport();
await server.connect(transport);
```

**Note**: This is the AgentKit MCP approach, which is separate from Agentic Wallet. It provides broader blockchain capabilities but requires CDP API keys and a different setup.

---

## Pattern 4: x402 SDK (Programmatic Buyer)

For applications that need to programmatically pay for x402 services without the `awal` CLI.

### Node.js with fetch

```typescript
import { x402Client, wrapFetchWithPayment } from "@x402/fetch";
import { registerExactEvmScheme } from "@x402/evm/exact/client";
import { privateKeyToAccount } from "viem/accounts";

// Setup
const signer = privateKeyToAccount(process.env.EVM_PRIVATE_KEY as `0x${string}`);
const client = new x402Client();
registerExactEvmScheme(client, { signer });
const fetchWithPayment = wrapFetchWithPayment(fetch, client);

// Use like normal fetch - payments handled automatically
const response = await fetchWithPayment("https://api.example.com/paid-endpoint");
const data = await response.json();
```

### Node.js with axios

```typescript
import { createPaymentInterceptor } from "@x402/axios";
// Similar pattern - intercepts 402 responses and retries with payment
```

### Python (async with httpx)

```python
from x402.clients.httpx import x402HttpxClient
import httpx

async with httpx.AsyncClient() as base_client:
    async with x402HttpxClient(base_client) as client:
        response = await client.get("https://api.example.com/paid-endpoint")
```

### Python (sync with requests)

```python
from x402.clients.requests import x402_requests
import requests

with requests.Session() as session:
    with x402_requests(session) as client:
        response = client.get("https://api.example.com/paid-endpoint")
```

### Go

```go
import "github.com/coinbase/x402/go"

httpClient := x402.WrapHTTPClient(client)
resp, err := httpClient.Get("https://api.example.com/paid-endpoint")
```

### Multi-Network Support

Register multiple blockchain schemes for cross-chain payment:

```typescript
const client = new x402Client();
registerExactEvmScheme(client, { signer: evmSigner });
registerExactSvmScheme(client, { signer: svmSigner });
// Client auto-selects correct scheme based on 402 response
```

---

## Pattern 5: x402 SDK (Programmatic Seller)

For building paid API servers using x402 middleware.

### Express.js

```bash
npm install @x402/express @x402/evm @x402/core
```

### Next.js

```bash
npm install @x402/next @x402/evm @x402/core
```

### Hono

```bash
npm install @x402/hono @x402/evm @x402/core
```

### Python FastAPI

```bash
pip install "x402[fastapi]"
```

### Python Flask

```bash
pip install "x402[flask]"
```

### Go (Gin)

```bash
go get github.com/coinbase/x402/go
```

---

## Testnet vs Mainnet Configuration

### Testnet (Development)

- **Network**: Base Sepolia (`eip155:84532`)
- **Facilitator**: `https://www.x402.org/facilitator`
- **USDC contract**: `0x036CbD53842c5426634e7929541eC2318f3dCF7e`
- **No CDP account required**
- Trading not available on testnet

### Mainnet (Production)

- **Network**: Base Mainnet (`eip155:8453`)
- **Facilitator**: CDP Facilitator (requires CDP account)
- **USDC contract**: `0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913`
- **Required env vars**: `CDP_API_KEY_ID`, `CDP_API_KEY_SECRET`

---

## Error Handling Patterns

### CLI Error Handling (TypeScript)

```typescript
import { execSync } from 'child_process';

interface AwalResult {
  success: boolean;
  data?: any;
  error?: string;
}

function awalCommand(cmd: string): AwalResult {
  try {
    const result = execSync(`npx awal@latest ${cmd} --json`, {
      encoding: 'utf-8',
      timeout: 30000,
    });
    return { success: true, data: JSON.parse(result) };
  } catch (error: any) {
    return { success: false, error: error.message };
  }
}

// Pre-flight checks
async function ensureReady(): Promise<boolean> {
  const status = awalCommand('status');
  if (!status.success || !status.data?.authenticated) {
    return false; // Need to authenticate
  }

  const balance = awalCommand('balance');
  if (!balance.success || balance.data?.balance <= 0) {
    return false; // Need to fund
  }

  return true;
}
```

### x402 Error Handling

```typescript
try {
  const response = await fetchWithPayment("https://api.example.com/endpoint");
  const data = await response.json();
} catch (error) {
  if (error.message.includes("No scheme registered")) {
    // Unsupported blockchain network
  } else if (error.message.includes("Payment already attempted")) {
    // Retry failed
  } else if (error.message.includes("Insufficient funds")) {
    // Need more USDC
  }
}
```

---

## Service Discovery Pattern

Agents can autonomously discover and use services:

```bash
# 1. Search for what you need
npx awal@latest x402 bazaar search "sentiment analysis" --json

# 2. Inspect the best result
npx awal@latest x402 details https://sentiment.example.com/api/analyze --json

# 3. Check if affordable
npx awal@latest balance --json

# 4. Pay and use
npx awal@latest x402 pay https://sentiment.example.com/api/analyze \
  -X POST \
  -d '{"text": "Great product"}' \
  --max-amount 50000 \
  --json
```

**Bazaar cache**: Results stored at `~/.config/awal/bazaar/` with 12-hour auto-refresh. Use `--force-refresh` to bypass.

---

## Deployment Checklist for x402 Services

1. Get your wallet address: `npx awal@latest address`
2. Install dependencies: `npm install express x402-express`
3. Configure routes with prices and descriptions
4. Place payment middleware AFTER free endpoints
5. Test with `curl -i` (should return 402) and `npx awal x402 pay` (should return 200)
6. For mainnet: configure CDP credentials and production facilitator
7. Register service in x402 bazaar for discoverability by other agents

---

## Key Integration Decisions

| Decision | Agentic Wallet (awal CLI) | AgentKit SDK | x402 SDK Direct |
|----------|--------------------------|-------------|----------------|
| Setup Complexity | Low (npx, email auth) | Medium (CDP keys, SDK) | Medium (private key, SDK) |
| Chain Support | Base only | Multi-chain EVM + Solana | Any EVM + Solana |
| Capabilities | Send, trade, x402 | Full onchain (contracts, NFTs, etc.) | Payment only |
| Agent Integration | Skills CLI | MCP server or SDK import | Code import |
| Key Management | Coinbase manages keys | Multiple providers | You manage keys |
| Best For | Simple agent wallets | Full blockchain agents | Programmatic payments |
