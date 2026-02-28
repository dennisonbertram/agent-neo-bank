# Backend Wiring Spec — 2026-02-28

## Overview
Replace placeholder data imports with real Tauri backend calls across all pages. Add loading/error states.

## Pattern
Each page currently does:
```tsx
import placeholderData from '../data/placeholder_data.json'
const wallet = placeholderData.wallet
```

Replace with:
```tsx
import { useEffect, useState } from 'react'
import { tauriApi } from '../lib/tauri'

const [wallet, setWallet] = useState<BalanceResponse | null>(null)
const [loading, setLoading] = useState(true)
const [error, setError] = useState<string | null>(null)

useEffect(() => {
  tauriApi.wallet.getBalance()
    .then(setWallet)
    .catch(e => setError(e.message || 'Failed to load'))
    .finally(() => setLoading(false))
}, [])
```

For browser (non-Tauri) fallback, catch invoke errors and use placeholder data.

## Browser Fallback Strategy
Since the app can run in browser mode for testing, wrap Tauri calls:
```tsx
async function safeTauriCall<T>(fn: () => Promise<T>, fallback: T): Promise<T> {
  try { return await fn() }
  catch { return fallback }
}
```

Place this helper in `src/lib/tauri.ts`.

## Wave 1 — Core pages (parallel, no file conflicts)

### Task 1: Home page (`src/pages/Home.tsx`)
- `tauriApi.wallet.getBalance()` → balance card
- `tauriApi.wallet.getAddress()` → wallet address
- `tauriApi.transactions.list({ limit: 5, offset: 0 })` → activity feed
- `tauriApi.agents.list()` → agent pills in "Agents" tab
- `tauriApi.budget.getAgentSummaries()` → agent daily spend
- Add loading skeleton + error state

### Task 2: Agents List (`src/pages/AgentsList.tsx`)
- `tauriApi.agents.list()` → agent cards
- `tauriApi.budget.getAgentSummaries()` → budget info per card
- Add loading skeleton + error state

### Task 3: Agent Detail (`src/pages/AgentDetail.tsx`)
- `tauriApi.agents.get(agentId)` → agent info
- `tauriApi.agents.getPolicy(agentId)` → spending controls
- `tauriApi.agents.getTransactions(agentId, 5)` → history
- `tauriApi.agents.updatePolicy(policy)` → save changes
- Add loading skeleton + error state

### Task 4: Settings (`src/pages/Settings.tsx`)
- `tauriApi.notifications.getPreferences()` → toggle states
- `tauriApi.notifications.updatePreferences(prefs)` → toggle saves
- `tauriApi.auth.logout()` → Reset Coinbase Connection
- Add loading state

## Wave 2 — Secondary pages + shared helper

### Task 5: Transaction Detail (`src/pages/TransactionDetail.tsx`)
- `tauriApi.transactions.get(txId)` → all transaction fields
- `tauriApi.agents.get(tx.agent_id)` → agent name/info
- Add loading state

### Task 6: Add Funds (`src/pages/AddFunds.tsx`)
- `tauriApi.wallet.getAddress()` → wallet address display + QR
- Keep placeholder for Buy with Card (not implemented)
- Add loading state

### Task 7: Auth flow wiring (`src/pages/ConnectCoinbase.tsx`, `src/pages/VerifyOtp.tsx`)
- `tauriApi.auth.login(email)` → send OTP
- `tauriApi.auth.verify(otp)` → verify OTP
- Wire up authStore.setAuthenticated on success
- Navigate to /home on success

### Task 8: Shared helper + auth loading state
- Add `safeTauriCall` to `src/lib/tauri.ts`
- Add `isLoading` to authStore
- Add splash screen in App.tsx while auth checking
