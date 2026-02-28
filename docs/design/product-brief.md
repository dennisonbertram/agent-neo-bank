# Agent Neo Bank — Product & Design Brief

## Product Concept

An agent-first wallet desktop app (mobile-style UI in Tauri) that:

- Connects once to **Coinbase** using **email + one-time password**
- Displays a **Base-native** wallet balance (USDC/ETH on Base)
- Lets **local agents** (running on the user's machine) request spending access that appears in-app automatically
- Lets the user set **spending limits** (daily/per-transaction), require approvals over thresholds, and pause agents
- Shows a transparent transaction feed where each transaction includes **agent name + purpose metadata**
- Provides full transaction detail with structured agent-populated metadata + explorer link

The onboarding includes an **"Install skill locally"** step that updates local files `claude.md` and `agents.md` so agents can request wallet access and attach transaction metadata.

**Important: The end user never interacts with the CLI or terminal.** Everything happens in the desktop app window.

---

## Visual Style

- Premium fintech: neutral backgrounds, subtle card surfaces, rounded corners, soft shadows, generous spacing
- Strong typography scale: big balance, clear section headings, readable transaction rows
- Calm fintech minimalism — clean, trustworthy, iOS-first feeling
- Explicit states: loading, empty, pending agent access, disabled "Coming Soon"
- Trust cues: clear warnings for wrong network, clear permission language, limit/approval visibility
- Fixed mobile-size window (feels like a phone app, not a desktop app)

---

## Screens & Flows

### 1. Welcome / Onboarding (4 slides)

Horizontal paging with "Continue" CTA.

- **Slide 1**: "A wallet built for agents" — Connect once, let local agents pay for tools/services/onchain actions
- **Slide 2**: "Hold USDC + ETH on Base" — Fast, low-fee. Deposit from Coinbase in seconds
- **Slide 3**: "Agents spend with limits you control" — Set caps, require approvals, pause anytime
- **Slide 4**: "Every action has a reason" — Agents attach metadata to each transaction

Final slide CTA: **"Get set up"**

---

### 2. Setup Step A — Install Skill Locally

**Title:** Install the local skill
**Body:** "This enables your agents to request wallet access and send metadata to the app."

- Primary: **Install skill locally**
- Secondary: **Learn what this changes**

Expandable "What changes?" panel:
- "Updates local config files (`claude.md`, `agents.md`) to register the wallet skill and permissions."
- "No funds move without your limits and rules."
- "You can uninstall or reset anytime from Settings."

**Success state:**
- Title: "Skill installed"
- Body: "Your agents can now request access. Next: connect Coinbase."
- CTA: **Continue**

---

### 3. Setup Step B — Connect Coinbase (Email + OTP)

**Connect screen**: Email input → "Send code"

**OTP screen**: 6-digit OTP input, "Resend code" → success transitions to main app

---

### 4. Home (Post-Login)

- **Top balance card**: large balance, USDC/ETH breakdown, "Base" network indicator, address snippet
- Actions: **Add Funds**, **Agents**
- **Transaction list** below
- Empty state: "No transactions yet" + "Once your agents are connected, their actions will show up here with clear explanations and details." + "Add funds" CTA
- Top-right: **Settings gear** + circular avatar/initials

---

### 5. Add Funds

- Copyable smart contract wallet address
- QR code
- Deposit guidance: "Deposit USDC or ETH on Base to this wallet"
- Warning: "Deposits must be on Base. Deposits from other networks may be lost."
- "Copy address" (primary), "Share QR" (secondary)
- Disabled: "Buy with card (Coming Soon)"

---

### 6. Agents List

Self-populated when local agents request access via installed skill.

List rows: agent name, status pill (Active/Pending/Paused), spending summary

Sample agents:
1. Research Runner — Active — $25/day • $10/tx — "Approval over $10"
2. Deploy Bot — Active — $100/day • $50/tx — "Approval over $50"
3. Data Buyer — Pending — "Requested access from local machine"
4. Treasury Watcher — Paused — $0/day — "Spending paused by user"

Empty state: "No agents connected yet" + "Agents appear here when they request access from your machine."

---

### 7. Agent Detail — Spending Controls

- Header: agent name, status, description
- Spending controls: daily limit, per-tx limit
- Toggle: "Require approval for transactions over $X"
- Toggle: "Pause agent spending"
- CTA: "Update limits"
- Agent transactions list (filtered)

---

### 8. Transaction Feed

Each row shows:
- Label (merchant/protocol or action label)
- Amount (USDC/ETH)
- Timestamp
- Agent name
- One-line purpose summary

Sample rows:
1. $6.50 USDC — API usage — Research Runner — "Paid for search credits" — Today 3:14 PM
2. 0.0032 ETH — Onchain action — Deploy Bot — "Deployed contract + verified on explorer" — Yesterday
3. $18.00 USDC — Data purchase — Data Buyer — "Purchased price feed snapshot" — Feb 25
4. $2.10 USDC — Tooling — Treasury Watcher — "Webhook relay fee for alerts" — Feb 24

---

### 9. Transaction Detail (Deep Transparency)

- Amount, asset, status, timestamp
- From/To addresses (short + expandable)
- Network: Base, fees/gas
- **Agent-provided metadata** section:
  - Category, Purpose, Notes, Cost breakdown, Request ID / Trace ID
  - Relevant links (invoice, internal trace, explorer)
- "View on Explorer" button (Base explorer)

---

### 10. Settings

- Profile header: email + avatar/initials
- Reset Coinbase connection
- Notifications (toggles): agent requests access, transaction completed, approval required, daily limit reached, low balance
- Manage local skill: Reinstall / Uninstall
- About / Support

---

## Component Specs Needed

- Buttons (primary, secondary, disabled/coming-soon)
- Cards (balance card, surface cards)
- List rows (agent row, transaction row)
- Steppers/sliders (spending limits)
- Toggles
- Empty states
- Status pills (Active, Pending, Paused)
- Warning banners
- QR/address module
- Bottom navigation bar
- OTP input (6-digit)
