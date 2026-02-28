# Agent Neo Bank — Frontend Implementation Plan

**Date**: 2026-02-27
**Tech Stack**: Tauri v2, React 19, TypeScript, Tailwind CSS v4, React Router v7, Zustand v5
**Window**: Fixed 390×844px (no resizing, no phone bezel in the app itself)

---

## Current State

The project has:
- `src/main.tsx` — entry point with BrowserRouter + App shell (imports Inter + JetBrains Mono fonts)
- `src/types/index.ts` — complete TypeScript types mirroring Rust models
- `src-tauri/src/commands/` — all backend commands implemented: `auth`, `agents`, `transactions`, `budget`, `settings`, `approvals`, `wallet`
- `package.json` — all dependencies already installed: React Router v7, Zustand v5, Tailwind v4, lucide-react, date-fns, clsx, tailwind-merge, sonner, class-variance-authority

What does NOT exist yet:
- `src/App.tsx`
- Any page/screen components
- `src/styles/tokens.css`
- `src/styles/globals.css` (or `src/index.css` design system baseline)
- Any store files
- Any component files

---

## Section 1: Project Setup

### 1.1 Tailwind v4 Configuration

Tailwind v4 is already installed via `@tailwindcss/vite`. Verify `vite.config.ts` has the plugin. No `tailwind.config.js` is needed — v4 uses CSS-first configuration via `@theme` in the CSS file.

**`src/index.css`** must be the entry point that:
1. Imports `@tailwindcss/vite` directives
2. Declares all custom properties via `@theme`
3. Applies global resets

The design tokens map to Tailwind utilities via `@theme` so that classes like `bg-primary`, `text-primary`, `rounded-pill`, `shadow-subtle` are available.

### 1.2 Tauri Window Configuration

Update `src-tauri/tauri.conf.json` — the `app.windows` block currently sets 1200×800 with `resizable: true`. Change to:

```json
{
  "title": "Agent Neo Bank",
  "width": 390,
  "height": 844,
  "resizable": false,
  "fullscreen": false,
  "decorations": true,
  "transparent": false,
  "center": true
}
```

This makes the Tauri window itself be the phone. No `border-radius` or `box-shadow` on `.app-container` — those are design prototype artifacts only.

### 1.3 Font Setup

Already imported in `main.tsx`:
- `@fontsource/inter` (400, 500, 600, 700) — primary UI font (replaces `-apple-system` stack)
- `@fontsource/jetbrains-mono` (400) — monospace for addresses, IDs, file names

The design spec calls for `-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto...` — in the Tauri app, Inter is a good substitute that gives a consistent cross-platform feel.

---

## Section 2: Design System Layer

### 2.1 `src/styles/tokens.css`

All CSS custom properties from the design specs. This file is `@import`-ed into `src/index.css`.

```css
/* src/styles/tokens.css */

:root {
  /* === COLORS === */

  /* Backgrounds */
  --bg-primary: #FFFFFF;
  --bg-secondary: #F8F9FA;
  --surface-hover: #F2F2F7;

  /* Text */
  --text-primary: #111111;
  --text-secondary: #8E8E93;
  --text-tertiary: #C7C7CC;

  /* Accents (solid) */
  --accent-green: #8FB5AA;
  --accent-yellow: #F2D48C;
  --accent-terracotta: #D9A58B;
  --accent-blue: #BCCCDC;

  /* Accent dims (15% opacity — for badge fills, avatar backgrounds) */
  --accent-green-dim: rgba(143, 181, 170, 0.15);
  --accent-yellow-dim: rgba(242, 212, 140, 0.15);
  --accent-terracotta-dim: rgba(217, 165, 139, 0.15);
  --accent-blue-dim: rgba(188, 204, 220, 0.15);

  /* Semantic text on dim backgrounds */
  --status-active-text: #4A6E65;
  --status-pending-text: #8F7843;
  --status-paused-text: #8F6652;

  /* Semantic */
  --color-danger: #E5484D;
  --color-link: #0052FF;      /* Base blue — "View All", network badge */
  --color-positive: #4A6E65;  /* Positive/incoming amounts */

  --black: #000000;
  --white: #FFFFFF;

  /* === SPACING === */
  --space-xs: 4px;
  --space-sm: 8px;
  --space-md: 16px;
  --space-lg: 24px;
  --space-xl: 32px;

  /* === BORDER RADIUS === */
  --radius-sm: 12px;    /* OTP boxes, icon containers */
  --radius-md: 20px;    /* Input groups, skill card, agent card */
  --radius-lg: 32px;    /* Balance card */
  --radius-pill: 999px; /* Buttons, segment control, status pills */

  /* === SHADOWS === */
  --shadow-subtle: 0 4px 24px rgba(0, 0, 0, 0.04);
  --shadow-float: 0 8px 32px rgba(0, 0, 0, 0.08);
  --shadow-fab: 0 8px 24px rgba(0, 0, 0, 0.15);

  /* === TYPOGRAPHY === */
  --font-sans: "Inter", -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
  --font-mono: "JetBrains Mono", "SF Mono", "Menlo", monospace;

  /* === ANIMATION === */
  --transition-fast: 0.1s ease;
  --transition-base: 0.2s ease;
  --transition-slow: 0.4s ease;
}
```

### 2.2 `src/styles/globals.css`

```css
/* src/styles/globals.css */

/* Box model and tap highlight reset */
*, *::before, *::after {
  box-sizing: border-box;
  -webkit-tap-highlight-color: transparent;
}

/* Typography reset */
h1, h2, h3, h4, h5, h6, p {
  margin: 0;
}

/* App container — fills the Tauri window exactly */
#root {
  width: 390px;
  height: 844px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  background-color: var(--bg-primary);
  font-family: var(--font-sans);
  color: var(--text-primary);
  position: relative;
}

/* Screen scroll containers */
.screen-scroll {
  flex: 1;
  overflow-y: auto;
  -webkit-overflow-scrolling: touch;
  /* Hide scrollbar visually (mobile-like) */
  scrollbar-width: none;
}
.screen-scroll::-webkit-scrollbar { display: none; }

/* Standard screen padding */
.screen-pad {
  padding: 60px 24px 100px 24px;
}

/* Detail screen padding (no bottom nav) */
.screen-pad-detail {
  padding: 60px 24px 40px 24px;
}

/* Entry animations */
@keyframes fadeInUp {
  from { opacity: 0; transform: translateY(10px); }
  to   { opacity: 1; transform: translateY(0); }
}
@keyframes slideUp {
  from { opacity: 0; transform: translateY(20px); }
  to   { opacity: 1; transform: translateY(0); }
}

.animate-in {
  animation: fadeInUp 0.4s ease forwards;
}
.animate-slide-up {
  animation: slideUp 0.5s ease forwards;
}

/* Typography scale classes */
.text-display {
  font-size: 42px;
  font-weight: 600;
  letter-spacing: -1px;
  line-height: 1.1;
  color: var(--text-primary);
}

.text-title {
  font-size: 22px;
  font-weight: 600;
  letter-spacing: -0.5px;
  color: var(--text-primary);
}

.text-subtitle {
  font-size: 17px;
  font-weight: 500;
  letter-spacing: -0.3px;
  color: var(--text-primary);
}

.text-body {
  font-size: 15px;
  font-weight: 400;
  line-height: 1.5;
  color: var(--text-secondary);
}

.text-caption {
  font-size: 12px;
  font-weight: 500;
  letter-spacing: 0.5px;
  text-transform: uppercase;
  color: var(--text-secondary);
}

.text-mono {
  font-family: var(--font-mono);
  font-size: 13px;
}

/* Status pill classes */
.status-active  { background: var(--accent-green-dim);      color: var(--status-active-text); }
.status-pending { background: var(--accent-yellow-dim);     color: var(--status-pending-text); }
.status-paused  { background: var(--accent-terracotta-dim); color: var(--status-paused-text); }
```

### 2.3 `src/index.css` (entry)

```css
@import "@tailwindcss";

@import "./styles/tokens.css";
@import "./styles/globals.css";
```

### 2.4 Tailwind v4 Theme Extension

Add to `src/index.css` after the `@import "@tailwindcss"` line:

```css
@theme {
  --color-bg-primary: var(--bg-primary);
  --color-bg-secondary: var(--bg-secondary);
  --color-surface-hover: var(--surface-hover);
  --color-text-primary: var(--text-primary);
  --color-text-secondary: var(--text-secondary);
  --color-text-tertiary: var(--text-tertiary);
  --color-accent-green: var(--accent-green);
  --color-accent-yellow: var(--accent-yellow);
  --color-accent-terracotta: var(--accent-terracotta);
  --color-accent-blue: var(--accent-blue);
  --color-danger: var(--color-danger);
  --color-link: var(--color-link);

  --radius-sm: var(--radius-sm);
  --radius-md: var(--radius-md);
  --radius-lg: var(--radius-lg);
  --radius-pill: var(--radius-pill);

  --shadow-subtle: var(--shadow-subtle);
  --shadow-float: var(--shadow-float);
  --shadow-fab: var(--shadow-fab);

  --font-sans: var(--font-sans);
  --font-mono: var(--font-mono);

  --spacing-xs: var(--space-xs);
  --spacing-sm: var(--space-sm);
  --spacing-md: var(--space-md);
  --spacing-lg: var(--space-lg);
  --spacing-xl: var(--space-xl);
}
```

### 2.5 Shared Component File Paths

All shared components live in `src/components/`. Organize by category:

```
src/components/
├── ui/
│   ├── Button.tsx            — btn-primary, btn-outline, btn-sm, btn-action variants
│   ├── InputGroup.tsx        — labeled input field (email input, wallet address display)
│   ├── OtpInput.tsx          — 6-digit OTP grid with keyboard handling
│   ├── StatusPill.tsx        — active/pending/paused status badge
│   ├── Toggle.tsx            — toggle switch (two size variants: sm=agent-detail, lg=settings)
│   ├── ProgressBar.tsx       — spending progress bar (track + fill + labels)
│   ├── Stepper.tsx           — increment/decrement control for spending limits
│   ├── SegmentControl.tsx    — pill-shaped tab switcher
│   └── SuccessCheck.tsx      — green circle with checkmark (install success)
├── layout/
│   ├── AppShell.tsx          — root layout wrapper (positions bottom nav, handles scroll)
│   ├── BottomNav.tsx         — 4-tab or 5-tab+FAB bottom navigation bar
│   ├── ScreenHeader.tsx      — sticky header with back button and optional right element
│   └── TopBar.tsx            — home screen top bar (avatar + wallet name + bell)
├── agent/
│   ├── AgentCard.tsx         — agent list card (icon + name + status + spending bar)
│   ├── AgentPillRow.tsx      — DNA-style pill row on the home screen (Research/Deploy/Treasury)
│   └── AgentAvatar.tsx       — 44×44 rounded square with accent color + icon
├── transaction/
│   ├── TransactionItem.tsx   — transaction row (avatar + name + tag + amount)
│   └── MetaCard.tsx          — key-value card used in transaction detail
└── icons/
    └── AppLogo.tsx           — hexagon+circle SVG in green rounded square
```

#### Button Component (`src/components/ui/Button.tsx`)

Uses `class-variance-authority` for variants:

```typescript
// variants: primary | outline | action | sm-outline
// primary: black bg, white text, 56px h, pill radius, full width
// outline: transparent, 1px border tertiary, text-primary, 56px h, pill radius, full width
// action: bg-secondary, text-primary, 52px h, radius 16px, flex 1
// sm-outline: outline but 36px h, auto width, px-4, 13px font
```

#### InputGroup Component (`src/components/ui/InputGroup.tsx`)

```typescript
// Props: label, children (input element), className
// Renders: bg-secondary rounded-md p-4 container with label above
```

#### OtpInput Component (`src/components/ui/OtpInput.tsx`)

```typescript
// Props: length=6, value, onChange, onComplete
// Renders: flex row of 6 digit boxes (48×56px, bg-secondary, radius-sm)
// Handles: single character per box, auto-advance, backspace-to-previous
// Uses: hidden actual <input> or individual inputs with ref forwarding
```

#### StatusPill Component (`src/components/ui/StatusPill.tsx`)

```typescript
// Props: status: "active" | "pending" | "paused" | "running"
// Renders: span with appropriate status-* class + uppercase label
// Size: padding 4px 10px, border-radius 8px, 11px/700
```

#### BottomNav Component (`src/layout/BottomNav.tsx`)

```typescript
// Props: variant: "main" | "agents", activeTab: string, onNavigate
// "main" variant: 4 tabs (Home | History | Add-FAB | Stats | Settings)
//   — FAB center position with -28px margin-top (or -40px per settings screen)
// Maps tab clicks to router.navigate() calls
// Positioned: absolute bottom-0, 84px height, frosted glass
```

---

## Section 3: Screen Implementation Order

### Screen 1: Welcome / Onboarding Slides

**File**: `src/pages/Onboarding.tsx`
**Route**: `/onboarding` (or `/` when not authenticated)

**Key components used**:
- `AppLogo` (60×60 green rounded square with hexagon SVG)
- Onboarding slide with `.animate-slide-up` entry
- Indicator dots (active = 24×6px pill, inactive = 6×6px circle)
- `Button` (primary, full-width)

**Layout**:
- Full-height flex column with content centered vertically (`justify-content: center`)
- CTA button is absolutely positioned at bottom (`bottom: 50px, left/right: 40px`)
- Single slide (design shows one welcome slide before transitioning to install-skill flow)

**Tauri invoke calls**: None — pure UI

**State**: Local `useState` for slide index if multi-slide, otherwise navigate directly

**Notes**: The design prototype shows one welcome slide + "Get set up" CTA. The product brief describes 4 slides. Implement 4 slides with a pager/horizontal scroll or step indicator. All slides share the same CTA region. Last slide CTA reads "Get set up".

---

### Screen 2: Install Skill Locally

**File**: `src/pages/InstallSkill.tsx`
**Route**: `/setup/install`

**Key components used**:
- Icon badge (48×48, green-dim bg, package SVG)
- SkillCard with expandable "What changes?" panel (collapsible via `useState`)
- FileChange rows (`claude.md` + `agents.md` with status tags)
- `Button` primary ("Install skill locally") + cancel
- `SuccessCheck` (for success state)

**Two states**: `install` | `success` — controlled by `useState<'install' | 'success'>`

**Tauri invoke calls**:
```typescript
// When "Install skill locally" is clicked:
invoke('install_skill')  // NOTE: this command may need to be added to Rust backend
// Falls back to: writes to local claude.md / agents.md via file system plugin
```

**Note**: The Rust backend does not yet have an `install_skill` command. This is a placeholder for Wave 1 investigation. For now, clicking "Install" can immediately transition to success state and navigate to `/setup/connect`.

**Navigation**: On success state "Continue" → `/setup/connect`

---

### Screen 3: Connect Coinbase (Email Input)

**File**: `src/pages/ConnectCoinbase.tsx`
**Route**: `/setup/connect`

**Key components used**:
- `Button` sm-outline (back button "← Back")
- `InputGroup` (email input, prefilled placeholder)
- `Button` primary ("Send Code")
- Local skill status section (shows `agents.md` and `claude.md` as "UPDATED")

**Tauri invoke calls**:
```typescript
// On "Send Code":
const result = await invoke<{ status: string; flow_id?: string }>('auth_login', { email })
// On success (status === "otp_sent"): navigate('/setup/verify')
// Store flow_id in auth store for the verify step
```

**State**: Local `useState` for email value, `useState` for loading/error

---

### Screen 4: Verify OTP

**File**: `src/pages/VerifyOtp.tsx`
**Route**: `/setup/verify`

**Key components used**:
- `OtpInput` (6-digit grid, auto-focus, auto-advance)
- `Button` primary ("Verify")
- "Resend code" text link

**Tauri invoke calls**:
```typescript
// On "Verify":
const result = await invoke<{ status: string }>('auth_verify', { otp })
// On status === "verified":
//   1. Check auth status
//   2. Navigate to '/home'
//   3. Set global auth state to authenticated

// On "Resend code":
await invoke('auth_login', { email })  // Re-trigger OTP send
```

**State**: OTP value in local state, loading state, error message

---

### Screen 5: Home / Dashboard

**File**: `src/pages/Home.tsx`
**Route**: `/home`
**Auth required**: Yes

**Key components used**:
- `TopBar` (avatar + wallet name + bell icon)
- `BalanceCard` (black card, radial glow, network badge, address pill, token holdings)
- `Button` action×2 ("Add Funds" | "Agents")
- `SegmentControl` ("Overview" | "Agents") — controls which sub-section shows
- `AgentPillRow`×3 (Research, Deploy Bot, Treasury — the DNA pill row)
- `TransactionItem`×N (recent activity feed)
- `BottomNav` variant="main" activeTab="home"

**Tauri invoke calls**:
```typescript
// On mount:
const balance = await invoke<BalanceResponse>('get_balance')
const address = await invoke<AddressResponse>('get_address')
const { transactions } = await invoke<ListTransactionsResponse>('list_transactions', {
  limit: 10, offset: 0
})
const agents = await invoke<Agent[]>('list_agents')
const budgetSummaries = await invoke<AgentBudgetSummary[]>('get_agent_budget_summaries')
```

**Layout**:
- Screen has `overflow-y: auto`, `padding: 60px 24px 100px 24px`
- `TopBar` is NOT sticky in this screen — it scrolls with content (design shows absolute positioning at top)
- Actually per spec: `#main-header` is `position: absolute; top: 0` with gradient fade
- Therefore: use `position: sticky top-0` on the header with `z-index: 10`, `bg: linear-gradient(to bottom, white 80%, transparent)`
- Content starts at `padding-top: 80px` to clear the absolute header

**Empty state**: Show "No transactions yet" + description + "Add funds" CTA when `transactions.length === 0`

---

### Screen 6: Add Funds

**File**: `src/pages/AddFunds.tsx`
**Route**: `/add-funds`
**Auth required**: Yes

**Key components used**:
- QR code placeholder (200×200 bg-secondary box with grid icon + dashed border)
- Warning pill (yellow-subtle, "Send only USDC or ETH on Base")
- `InputGroup` (wallet address display + copy icon)
- `Button` secondary disabled ("Buy with Card (Coming Soon)")
- `Button` outline ("Close")

**Tauri invoke calls**:
```typescript
// On mount:
const { address } = await invoke<AddressResponse>('get_address')

// On "Copy address" click:
import { writeText } from '@tauri-apps/plugin-clipboard-manager'
await writeText(address)
```

**Note**: QR code generation — use a library like `qrcode.react` (add to package.json) or render a placeholder for Wave 2, implement real QR in Wave 3.

---

### Screen 7: Agents List

**File**: `src/pages/AgentsList.tsx`
**Route**: `/agents`
**Auth required**: Yes

**Key components used**:
- Header (title "My Agents" + settings gear button)
- `SegmentControl` ("Active" | "All Agents" | "Archived")
- `AgentCard`×N (one per agent from `list_agents`)
- `BottomNav` variant="agents" activeTab="agents" (includes FAB)

**Tauri invoke calls**:
```typescript
// On mount:
const agents = await invoke<Agent[]>('list_agents')
const budgetSummaries = await invoke<AgentBudgetSummary[]>('get_agent_budget_summaries')

// Combine agent + budget data for each card
```

**Filtering**: Segment control filters by `agent.status`:
- "Active" → `status === "active"`
- "All Agents" → all statuses
- "Archived" → `status === "revoked"`

**Empty state**: "No agents connected yet" + description text

**Navigation**: Tap agent card → `/agents/:agentId`

---

### Screen 8: Agent Detail

**File**: `src/pages/AgentDetail.tsx`
**Route**: `/agents/:agentId`
**Auth required**: Yes

**Key components used**:
- `ScreenHeader` (sticky, back button + "Running" status badge)
- Agent identity block (caption + display name + description)
- Daily Spend card (bg-secondary, radius 24px, amount + `ProgressBar` + pause toggle)
- Spending Controls section: `Stepper`×2 (daily limit, per-tx limit) + `Toggle` (approval threshold)
- Agent History: `TransactionItem`×N
- `Button` primary ("Save Changes")

**Tauri invoke calls**:
```typescript
// On mount:
const agent = await invoke<Agent>('get_agent', { agentId })
const policy = await invoke<SpendingPolicy>('get_agent_spending_policy', { agentId })
const budgets = await invoke<AgentBudgetSummary[]>('get_agent_budget_summaries')
const { transactions } = await invoke<ListTransactionsResponse>('list_transactions', {
  limit: 10, offset: 0, agentId
})

// On "Save Changes":
await invoke('update_agent_spending_policy', { policy: updatedPolicy })

// On pause toggle:
await invoke('suspend_agent', { agentId })    // when toggling ON (pause)
// Reactivation not yet mapped — may need a resume_agent command or update status directly

// Note: agent status mapping: design "Paused" = Rust "suspended"
```

**Local state**: `dailyLimit`, `perTxLimit`, `requireApproval`, `isPaused` — initialized from loaded policy, updated on Save

---

### Screen 9: Transaction Detail

**File**: `src/pages/TransactionDetail.tsx`
**Route**: `/transactions/:txId`
**Auth required**: Yes

**Key components used**:
- Back button (text + chevron icon, "Details")
- Amount hero block (42px display, USDC/ETH suffix, timestamp)
- Agent identity row (48×48 agent avatar + name + "Verified Agent" tag)
- `MetaCard` × 3 (Agent Metadata, Cost Breakdown, Notes)
- `Button` outline ("View on Explorer" — opens Base explorer URL)

**Tauri invoke calls**:
```typescript
// On mount:
const tx = await invoke<Transaction>('get_transaction', { txId })
const agent = tx.agent_id
  ? await invoke<Agent>('get_agent', { agentId: tx.agent_id })
  : null

// On "View on Explorer":
import { open } from '@tauri-apps/plugin-shell'
await open(`https://basescan.org/tx/${tx.chain_tx_hash}`)
// Note: @tauri-apps/plugin-shell may need to be added to dependencies
```

**Metadata display**: The `tx.description`, `tx.category`, `tx.reason`, `tx.service_name`, `tx.service_url` fields map to the Agent Metadata card. The `tx.memo` maps to the Notes section.

---

### Screen 10: Settings

**File**: `src/pages/Settings.tsx`
**Route**: `/settings`
**Auth required**: Yes

**Key components used**:
- Profile header (64px avatar with initials + name + email)
- Settings groups with `Toggle` (settings variant: 50×30px) for notifications
- Chevron rows for "Reset Coinbase Connection" and "Export Wallet History"
- `BottomNav` variant="main" activeTab="settings"
- Version string at bottom

**Tauri invoke calls**:
```typescript
// On mount:
const authStatus = await invoke<AuthStatusResponse>('auth_status')
const notifPrefs = await invoke<NotificationPreferences>('get_notification_preferences')

// On toggle change:
await invoke('update_notification_preferences', { prefs: updatedPrefs })

// On "Reset Coinbase Connection":
await invoke('auth_logout')
// Then navigate to '/onboarding' and clear auth store

// Note: get_notification_preferences and update_notification_preferences
// commands need to be confirmed/added in the notifications.rs command file
```

**Danger action**: "Reset Coinbase Connection" should show a confirmation step before calling `auth_logout`.

---

## Section 4: Routing Structure

**File**: `src/App.tsx`

```typescript
// src/App.tsx

import { Routes, Route, Navigate } from 'react-router-dom'
import { useAuthStore } from './stores/authStore'

// Pages
import Onboarding from './pages/Onboarding'
import InstallSkill from './pages/InstallSkill'
import ConnectCoinbase from './pages/ConnectCoinbase'
import VerifyOtp from './pages/VerifyOtp'
import Home from './pages/Home'
import AddFunds from './pages/AddFunds'
import AgentsList from './pages/AgentsList'
import AgentDetail from './pages/AgentDetail'
import TransactionDetail from './pages/TransactionDetail'
import Settings from './pages/Settings'

function ProtectedRoute({ children }: { children: React.ReactNode }) {
  const { isAuthenticated } = useAuthStore()
  if (!isAuthenticated) return <Navigate to="/onboarding" replace />
  return <>{children}</>
}

export function App() {
  return (
    <Routes>
      {/* Onboarding flow — unauthenticated */}
      <Route path="/onboarding" element={<Onboarding />} />
      <Route path="/setup/install" element={<InstallSkill />} />
      <Route path="/setup/connect" element={<ConnectCoinbase />} />
      <Route path="/setup/verify" element={<VerifyOtp />} />

      {/* Main app — requires auth */}
      <Route path="/home" element={<ProtectedRoute><Home /></ProtectedRoute>} />
      <Route path="/add-funds" element={<ProtectedRoute><AddFunds /></ProtectedRoute>} />
      <Route path="/agents" element={<ProtectedRoute><AgentsList /></ProtectedRoute>} />
      <Route path="/agents/:agentId" element={<ProtectedRoute><AgentDetail /></ProtectedRoute>} />
      <Route path="/transactions/:txId" element={<ProtectedRoute><TransactionDetail /></ProtectedRoute>} />
      <Route path="/settings" element={<ProtectedRoute><Settings /></ProtectedRoute>} />

      {/* Default redirect */}
      <Route path="/" element={<Navigate to="/onboarding" replace />} />
      <Route path="*" element={<Navigate to="/onboarding" replace />} />
    </Routes>
  )
}
```

### Navigation Patterns

| Screen Type | Back Pattern | Bottom Nav |
|---|---|---|
| Onboarding / Setup | None or inline "← Back" button | No |
| Home | N/A (root) | Yes — HOME active |
| Add Funds | Close button → `/home` | Yes (shown per design) |
| Agents List | N/A (root) | Yes — AGENTS active |
| Agent Detail | Back chevron button → `-1` in history | No |
| Transaction Detail | "Details" text+chevron → `-1` in history | No |
| Settings | N/A (root) | Yes — SETTINGS active |

---

## Section 5: State Management

### Approach

Use **Zustand v5** (already installed) for global state. Three stores cover all needs:

### 5.1 `src/stores/authStore.ts`

```typescript
interface AuthState {
  isAuthenticated: boolean
  email: string | null
  flowId: string | null  // OTP flow ID from auth_login response

  // Actions
  setAuthenticated: (email: string) => void
  setFlowId: (id: string) => void
  logout: () => void
  checkAuthStatus: () => Promise<void>  // calls auth_status on startup
}
```

- `checkAuthStatus()` is called once on app load (in `App.tsx` `useEffect`) to restore session
- `isAuthenticated` drives the `ProtectedRoute` guard
- `flowId` is needed by the verify screen to pass context

### 5.2 `src/stores/walletStore.ts`

```typescript
interface WalletState {
  address: string | null
  balances: Record<string, AssetBalance> | null
  totalBalance: string | null
  isLoading: boolean

  // Actions
  fetchBalance: () => Promise<void>
  fetchAddress: () => Promise<void>
}
```

- Loaded once on Home mount, cached in store
- Re-fetched on pull-to-refresh or when returning from Add Funds

### 5.3 `src/stores/agentStore.ts`

```typescript
interface AgentState {
  agents: Agent[]
  budgetSummaries: AgentBudgetSummary[]
  isLoading: boolean
  lastFetched: number | null

  // Actions
  fetchAgents: () => Promise<void>           // list_agents + get_agent_budget_summaries
  suspendAgent: (agentId: string) => Promise<void>
  revokeAgent: (agentId: string) => Promise<void>
  updatePolicy: (policy: SpendingPolicy) => Promise<void>
}
```

- Agents list is loaded on AgentsList mount and cached
- Individual agent data is fetched on AgentDetail mount (not cached globally — too granular)

### Why Not More Stores?

- **Transactions**: Fetched per-screen with `useEffect`, not cached globally (pagination makes global cache complex)
- **Approvals**: Fetched on demand, not a persistent UI concept in current screens
- **Settings/GlobalPolicy**: Fetched on Settings mount, local component state handles edits

---

## Section 6: Tauri Backend Integration

### Complete Invoke Map

| Screen | Invoke Call | Command | Parameters |
|---|---|---|---|
| App startup | `auth_status` | `auth.rs` | none |
| ConnectCoinbase | `auth_login` | `auth.rs` | `email: String` |
| VerifyOtp | `auth_verify` | `auth.rs` | `otp: String` |
| Settings | `auth_logout` | `auth.rs` | none |
| Home | `get_balance` | `wallet.rs` | none |
| Home, AddFunds | `get_address` | `wallet.rs` | none |
| Home | `list_transactions` | `transactions.rs` | `limit, offset` |
| Home | `list_agents` | `agents.rs` | none |
| Home | `get_agent_budget_summaries` | `budget.rs` | none |
| AgentsList | `list_agents` | `agents.rs` | none |
| AgentsList | `get_agent_budget_summaries` | `budget.rs` | none |
| AgentDetail | `get_agent` | `agents.rs` | `agent_id: String` |
| AgentDetail | `get_agent_spending_policy` | `agents.rs` | `agent_id: String` |
| AgentDetail | `get_agent_transactions` | `agents.rs` | `agent_id, limit` |
| AgentDetail | `update_agent_spending_policy` | `agents.rs` | `policy: SpendingPolicy` |
| AgentDetail | `suspend_agent` | `agents.rs` | `agent_id: String` |
| TransactionDetail | `get_transaction` | `transactions.rs` | `tx_id: String` |
| TransactionDetail | `get_agent` | `agents.rs` | `agent_id: String` |
| Settings | `get_global_policy` | `settings.rs` | none |
| Settings | `update_global_policy` | `settings.rs` | `policy: GlobalPolicy` |

### Missing Commands to Add

The following commands are referenced in the design but do not yet exist in the Rust backend:

1. **`get_notification_preferences`** — needed by Settings screen to load toggle states
2. **`update_notification_preferences`** — needed by Settings screen to save toggle states
3. **`install_skill`** — needed by InstallSkill screen to write `claude.md` / `agents.md`

The `notifications.rs` command file exists — check if these commands are stubbed there. If not, they need to be added.

### Auth Flow Sequence

```
App loads
  └── invoke('auth_status')
        ├── { authenticated: true, email: "..." }
        │     └── setAuthenticated(email) → navigate('/home')
        └── { authenticated: false }
              └── navigate('/onboarding')

Onboarding → InstallSkill → ConnectCoinbase
  └── invoke('auth_login', { email })
        └── { status: 'otp_sent', flow_id: '...' }
              └── setFlowId(flow_id) → navigate('/setup/verify')

VerifyOtp
  └── invoke('auth_verify', { otp })
        └── { status: 'verified' }
              └── setAuthenticated(email) → navigate('/home')
```

### Type Safety Wrapper

Create `src/lib/tauri.ts` to wrap all invoke calls with typed return values:

```typescript
// src/lib/tauri.ts
import { invoke } from '@tauri-apps/api/core'
import type { Agent, SpendingPolicy, Transaction, ... } from '../types'

export const tauriApi = {
  auth: {
    login: (email: string) => invoke<{ status: string; flow_id?: string }>('auth_login', { email }),
    verify: (otp: string) => invoke<{ status: string }>('auth_verify', { otp }),
    status: () => invoke<AuthStatusResponse>('auth_status'),
    logout: () => invoke<void>('auth_logout'),
  },
  wallet: {
    getBalance: () => invoke<BalanceResponse>('get_balance'),
    getAddress: () => invoke<AddressResponse>('get_address'),
  },
  agents: {
    list: () => invoke<Agent[]>('list_agents'),
    get: (agentId: string) => invoke<Agent>('get_agent', { agentId }),
    getPolicy: (agentId: string) => invoke<SpendingPolicy>('get_agent_spending_policy', { agentId }),
    updatePolicy: (policy: SpendingPolicy) => invoke<void>('update_agent_spending_policy', { policy }),
    suspend: (agentId: string) => invoke<void>('suspend_agent', { agentId }),
    revoke: (agentId: string) => invoke<void>('revoke_agent', { agentId }),
    getTransactions: (agentId: string, limit?: number) =>
      invoke<Transaction[]>('get_agent_transactions', { agentId, limit }),
  },
  transactions: {
    list: (params: { limit: number; offset: number; agentId?: string; status?: string }) =>
      invoke<ListTransactionsResponse>('list_transactions', {
        limit: params.limit,
        offset: params.offset,
        agentId: params.agentId ?? null,
        status: params.status ?? null,
      }),
    get: (txId: string) => invoke<Transaction>('get_transaction', { txId }),
  },
  budget: {
    getAgentSummaries: () => invoke<AgentBudgetSummary[]>('get_agent_budget_summaries'),
    getGlobalSummary: () => invoke<GlobalBudgetSummary>('get_global_budget_summary'),
  },
  settings: {
    getGlobalPolicy: () => invoke<GlobalPolicy>('get_global_policy'),
    updateGlobalPolicy: (policy: GlobalPolicy) => invoke<void>('update_global_policy', { policy }),
    toggleKillSwitch: (active: boolean, reason?: string) =>
      invoke<void>('toggle_kill_switch', { active, reason }),
  },
}
```

---

## Section 7: Implementation Waves

### Wave 1: Foundation

**Objective**: Get the project building with all design system infrastructure in place. No Tauri calls — static/hardcoded data only.

**Files to create**:

```
src-tauri/tauri.conf.json         UPDATE: window 390×844, resizable: false
src/index.css                     NEW: Tailwind v4 entry + @theme
src/styles/tokens.css             NEW: all CSS custom properties
src/styles/globals.css            NEW: resets, typography scale, animation classes
src/App.tsx                       NEW: route tree (all routes stubbed to placeholder)
src/lib/tauri.ts                  NEW: typed invoke wrappers
src/stores/authStore.ts           NEW: auth Zustand store
src/stores/walletStore.ts         NEW: wallet Zustand store
src/stores/agentStore.ts          NEW: agent Zustand store
src/components/ui/Button.tsx      NEW: all button variants
src/components/ui/InputGroup.tsx  NEW
src/components/ui/OtpInput.tsx    NEW
src/components/ui/StatusPill.tsx  NEW
src/components/ui/Toggle.tsx      NEW (sm + lg variants)
src/components/ui/ProgressBar.tsx NEW
src/components/ui/Stepper.tsx     NEW
src/components/ui/SegmentControl.tsx NEW
src/components/ui/SuccessCheck.tsx NEW
src/components/layout/BottomNav.tsx NEW
src/components/layout/ScreenHeader.tsx NEW
src/components/layout/TopBar.tsx  NEW
src/components/icons/AppLogo.tsx  NEW
```

**Verification**: `npm run dev` builds and shows blank screen at correct 390×844 window size. All component files import without TypeScript errors.

---

### Wave 2: Auth Flow (Onboarding → Connect → Verify)

**Objective**: Complete onboarding experience end-to-end with real Tauri auth calls.

**Files to create**:

```
src/pages/Onboarding.tsx          NEW: 4-slide welcome with indicators + "Get set up" CTA
src/pages/InstallSkill.tsx        NEW: install state + success state (stub invoke for now)
src/pages/ConnectCoinbase.tsx     NEW: email input + local skill status display
src/pages/VerifyOtp.tsx           NEW: 6-digit OTP with real auth_verify invoke
```

**Key behaviors**:
- App startup: `useEffect` in `App.tsx` calls `auth_status` → routes to `/home` or `/onboarding`
- `ConnectCoinbase` calls `auth_login` → stores `flow_id` → navigates to verify
- `VerifyOtp` calls `auth_verify` → on success sets auth store → navigates to `/home`
- `VerifyOtp` "Resend code" re-calls `auth_login`

**Agent component files to create** (needed by Wave 3 but no harm in Wave 2):
```
src/components/agent/AgentCard.tsx     NEW
src/components/agent/AgentPillRow.tsx  NEW
src/components/agent/AgentAvatar.tsx   NEW
src/components/transaction/TransactionItem.tsx NEW
src/components/transaction/MetaCard.tsx       NEW
```

---

### Wave 3: Main App (Home + Agents List + Add Funds)

**Objective**: Core post-auth screens with real data from Tauri backend.

**Files to create**:

```
src/pages/Home.tsx          NEW: balance card + action buttons + agent pills + transaction feed
src/pages/AddFunds.tsx      NEW: QR placeholder + copy address + warning + disabled card buy
src/pages/AgentsList.tsx    NEW: segment control + agent cards + FAB nav
```

**Tauri calls wired up**:
- Home: `get_balance`, `get_address`, `list_transactions`, `list_agents`, `get_agent_budget_summaries`
- AddFunds: `get_address` + clipboard write
- AgentsList: `list_agents`, `get_agent_budget_summaries`

**Store integration**:
- Home and AgentsList both use `agentStore.fetchAgents()` — call once, cache in store
- `walletStore.fetchBalance()` + `walletStore.fetchAddress()` called on Home mount

**Empty states**:
- Home: when `transactions.length === 0` show empty state
- AgentsList: when filtered list is empty show segment-specific empty state

---

### Wave 4: Detail Screens + Settings

**Objective**: Drill-down screens and settings with full CRUD.

**Files to create**:

```
src/pages/AgentDetail.tsx         NEW: all controls + save changes + agent history
src/pages/TransactionDetail.tsx   NEW: full transparency view
src/pages/Settings.tsx            NEW: notifications + account section + bottom nav
```

**Tauri calls wired up**:
- AgentDetail: `get_agent`, `get_agent_spending_policy`, `get_agent_transactions`, `update_agent_spending_policy`, `suspend_agent`
- TransactionDetail: `get_transaction`, `get_agent`
- Settings: `auth_status`, `get_global_policy`, `auth_logout`

**Polish tasks for Wave 4**:
- Wire up transaction "View on Explorer" with `@tauri-apps/plugin-shell` `open()`
- Add `sonner` toast notifications for save confirmations, copy success
- Add loading states (skeleton or spinner) for all `invoke` calls
- Handle error states from Tauri (network failures, auth expired)
- Implement approval required notifications via Tauri event listener

---

## Section 8: CSS Class Conventions

### Naming Strategy

Since Tailwind v4 is used, prefer utility classes for layout and spacing. Use semantic class names only for complex, repeated patterns not easily expressed in utilities.

| Pattern | Implementation |
|---|---|
| Screen container | `<div className="screen-scroll screen-pad">` (global CSS classes) |
| Balance card | Tailwind utilities inline: `bg-black rounded-[32px] p-8 relative overflow-hidden` |
| Agent card | Component with internal structure, uses `bg-[var(--bg-secondary)] rounded-[20px] p-5` |
| Status pill | `<StatusPill status="active" />` (component handles class selection) |
| Bottom nav | `<BottomNav />` (component handles all nav logic) |
| Button | `<Button variant="primary">` (cva-based component) |

### Key Design Decisions

1. **`rounded-[Xpx]` vs design tokens**: Use Tailwind arbitrary values for border radii that match the exact spec values. The `@theme` block maps `--radius-sm` → `rounded-sm` etc.

2. **Color utilities**: Use `bg-[var(--bg-secondary)]` pattern for cases where the Tailwind theme class isn't clear enough. The `@theme` block should expose `bg-bg-secondary` but arbitrary values are a safe fallback.

3. **Absolute positioning in screens**: The `BottomNav` and `TopBar` use `position: absolute` within the `#root` container (which is `position: relative`). This matches the design spec exactly.

4. **No phone bezel**: Unlike the HTML prototypes, the Tauri window IS the phone. Never apply `box-shadow: 0 0 0 10px #000` or `border-radius: 40px` to the root container.

---

## Section 9: Key Implementation Notes

### Design Inconsistencies to Resolve

The design specs contain some minor inconsistencies across screens that need a canonical decision:

| Issue | Design Spec Says | Recommended Resolution |
|---|---|---|
| Bottom nav FAB margin-top | `-28px` (agents-list) vs `-40px` (settings) | Use `-28px` consistently (half of 56px FAB) |
| Toggle class name | `active` (agent-detail) vs `on` (settings) | Use controlled React state, no CSS class hack |
| Nav icon size | `22px` (dashboard) vs `24px` (agents-list, settings) | Use `24px` consistently |
| text-title size | `20px` (dashboard), `22px` (settings), `24px` (agents), `28px` (onboarding2) | Context-specific: use correct size per screen |
| Toggle track size | `44×24` (agent-detail) vs `50×30` (settings) | Use settings size (`50×30`) everywhere for consistency |

### Status Mapping (Design vs Rust)

| Design Label | Rust AgentStatus |
|---|---|
| Active | `"active"` |
| Pending | `"pending"` |
| Paused | `"suspended"` |
| (revoked — not shown in design) | `"revoked"` |

### Amount Formatting

- USDC amounts: use `Intl.NumberFormat` with currency style
- ETH amounts: show up to 4 decimal places
- Positive amounts (incoming): color `#4A6E65`
- Negative amounts (outgoing): color `#111111` (text-primary — neutral, not red)

### Transaction Metadata

The `Transaction` type has structured fields (`category`, `description`, `reason`, `memo`, `service_name`, `service_url`) that map to the "Agent Metadata" section in the transaction detail screen:

```
category    → "Category" row
reason      → "Purpose" row
memo        → "Notes" section
service_name → referenced in description
```

The cost breakdown shown in the design is part of `tx.description` or a parsed JSON from `tx.metadata` — this mapping needs to be confirmed with the Rust `Transaction` model's intended use of each field.

---

## File Tree Summary

```
src/
├── main.tsx                          EXISTING (keep as-is)
├── index.css                         CREATE (Tailwind v4 entry)
├── App.tsx                           CREATE (route tree)
├── types/
│   └── index.ts                      EXISTING (keep as-is)
├── styles/
│   ├── tokens.css                    CREATE
│   └── globals.css                   CREATE
├── lib/
│   └── tauri.ts                      CREATE (typed invoke wrappers)
├── stores/
│   ├── authStore.ts                  CREATE
│   ├── walletStore.ts                CREATE
│   └── agentStore.ts                 CREATE
├── components/
│   ├── ui/
│   │   ├── Button.tsx
│   │   ├── InputGroup.tsx
│   │   ├── OtpInput.tsx
│   │   ├── StatusPill.tsx
│   │   ├── Toggle.tsx
│   │   ├── ProgressBar.tsx
│   │   ├── Stepper.tsx
│   │   ├── SegmentControl.tsx
│   │   └── SuccessCheck.tsx
│   ├── layout/
│   │   ├── BottomNav.tsx
│   │   ├── ScreenHeader.tsx
│   │   └── TopBar.tsx
│   ├── agent/
│   │   ├── AgentCard.tsx
│   │   ├── AgentPillRow.tsx
│   │   └── AgentAvatar.tsx
│   ├── transaction/
│   │   ├── TransactionItem.tsx
│   │   └── MetaCard.tsx
│   └── icons/
│       └── AppLogo.tsx
└── pages/
    ├── Onboarding.tsx                 Wave 2
    ├── InstallSkill.tsx               Wave 2
    ├── ConnectCoinbase.tsx            Wave 2
    ├── VerifyOtp.tsx                  Wave 2
    ├── Home.tsx                       Wave 3
    ├── AddFunds.tsx                   Wave 3
    ├── AgentsList.tsx                 Wave 3
    ├── AgentDetail.tsx                Wave 4
    ├── TransactionDetail.tsx          Wave 4
    └── Settings.tsx                   Wave 4

src-tauri/
└── tauri.conf.json                    UPDATE: window 390×844, resizable: false
```
