# Agent Neo Bank -- Standalone Design Prompt

> Copy-paste this entire prompt into any AI design tool (Figma AI, v0, Bolt, another Claude session, etc.) to generate the Agent Neo Bank UI.

---

## Product Overview

Agent Neo Bank is a desktop fintech app that gives AI agents autonomous spending power. Users set up a crypto wallet (Coinbase Agent Wallet), define budgets, and let AI agents (like Claude Code) pay for services with human-controlled guardrails. Think of it as a banking app where your "employees" are AI agents. Target users are non-technical to semi-technical people who expect a consumer-grade financial app -- NOT a developer tool or admin panel.

---

## Design Direction

**Aesthetic:** Premium, clean, warm. A modern neo-bank crossed with Apple's clarity and OpenAI's softness. Minimal but never hollow -- every pixel serves a purpose.

**References:**
- **Apple.com** -- Extreme whitespace, typographic hierarchy, restrained power
- **OpenAI** -- Generous rounded corners, card-based layouts, subtle gradients, modern warmth
- **Mercury / Revolut** -- Financial dashboard patterns, balance cards, transaction tables
- **Linear** -- Sidebar navigation, keyboard-first feel, snappy transitions

**Principles:**
1. Clarity over cleverness -- instantly understandable screens
2. Consumer banking feel, not a dev tool -- think Apple Wallet, not Swagger UI
3. Bright with restraint -- color is for status and hierarchy, most UI is neutral
4. Trust through structure -- consistent spacing, aligned elements, predictable layouts
5. Progressive disclosure -- essential info first, details on demand

---

## Design System

### Colors (Light Mode)
- **Background:** `#FAFAF9` (warm off-white)
- **Surface/Cards:** `#FFFFFF`
- **Border:** `#E8E5E0` (warm gray) | Subtle: `#F0EDE8`
- **Text Primary:** `#1A1A1A` | Secondary: `#6B7280` | Tertiary: `#9CA3AF`
- **Primary:** `#4F46E5` (indigo-600) | Hover: `#4338CA` | Light: `#EEF2FF`
- **Balance Card Gradient:** `linear-gradient(135deg, #4F46E5 0%, #7C3AED 50%, #6366F1 100%)`
- **Success:** `#10B981` | Background: `#ECFDF5`
- **Warning:** `#F59E0B` | Background: `#FFFBEB`
- **Danger:** `#EF4444` | Background: `#FEF2F2`
- **Focus Ring:** `#6366F1`, 2px, 2px offset

### Colors (Dark Mode)
- **Background:** `#0F1117` (deep navy) | Surface: `#1A1D27` | Raised: `#232736`
- **Border:** `#2D3142` | Text Primary: `#F1F1F4` | Secondary: `#9CA3AF`
- **Balance Card Gradient:** `linear-gradient(135deg, #6366F1 0%, #8B5CF6 50%, #7C3AED 100%)`
- Semantic colors keep same hex; tint backgrounds become 10-15% opacity on dark surface

### Typography
- **Primary Font:** Inter (weights: 400, 500, 600, 700)
- **Mono Font:** JetBrains Mono (wallet addresses, tx hashes)
- **Scale:** Display: 48/36/30px bold | Headings: 24/20/16/14px semibold | Body: 16/14/12px regular | Label: 13px medium (0.02em tracking) | Mono: 14/12px
- **Hero balance** uses 48px bold with -0.02em tracking and tabular numerals. One display-size number per screen max.

### Spacing & Layout
- **Base grid:** 4px. Every margin, padding, gap is a multiple of 4.
- **Sidebar:** 240px expanded, 72px collapsed, `#FFFFFF` background, right border
- **Content area:** max-width 1200px, centered, padding 32px (48px bottom)
- **Card padding:** 24px. Card gap in grids: 16px. Card radius: 12px. Card shadow: `0 1px 3px rgba(0,0,0,0.06), 0 1px 2px rgba(0,0,0,0.04)`
- **Button radius:** 8px. Button height: 40px default, 32px small, 48px large.
- **Section gap:** 32px between page sections.

### Component Patterns
- **Buttons:** Primary (#4F46E5 bg, white text), Secondary (white bg, border), Ghost (transparent), Danger (#DC2626), Success (#059669). All 8px radius, 13px medium text, `scale(0.98)` on press.
- **Inputs:** 40px height (48px for onboarding), 8px radius, 1px border, focus ring on focus.
- **Status Badges:** Pill shape (`border-radius: 9999px`), 6px colored dot + text. Active=green, Pending=yellow (pulsing dot), Suspended=red.
- **Cards:** White bg, 1px `#F0EDE8` border, 12px radius, subtle shadow. Hover: translateY(-2px) + deeper shadow.
- **Toast Notifications:** Top-right, 360px max, 12px radius, slide-in from right. Colored left border (green/amber/red).
- **Icons:** Lucide icon set, 20px default, 1.5px stroke.

---

## Screens

### 1. Welcome / Onboarding
Full-screen centered layout, NO sidebar. Content in a centered card (max-width 440px) over a soft radial gradient background using `#EEF2FF` fading into `#FAFAF9`.

**Elements top to bottom:**
- App logo icon (48px)
- 24px gap
- Headline: "Give your AI agents spending power" -- 24px semibold, centered
- 12px gap
- Subtext: "Set up a wallet, define budgets, and let your AI agents pay for services autonomously -- with guardrails you control." -- 16px regular, `#6B7280`, centered
- 8px gap
- "Set up in 2 minutes" -- 12px, `#9CA3AF`
- 32px gap
- "Get Started" button -- primary, large (48px), full width
- 24px gap
- "Powered by Coinbase Agent Wallet" -- 12px, `#9CA3AF`, with subtle Coinbase icon

### 2. Email Input
Full-screen centered card (440px), no sidebar. Soft gradient background.

- Back button (ghost, top-left of card area)
- "Connect your wallet" -- 24px semibold, centered
- "Enter your email to set up your Agent Wallet. We'll send you a verification code." -- 14px, `#6B7280`
- 24px gap
- Email input (48px height) with label "Email address"
- 12px gap
- Display name input (40px) with label "Display name (optional)"
- 24px gap
- "Send Verification Code" button -- primary, large, full width
- 16px gap
- Legal text: "By continuing, you agree to the Coinbase Agent Wallet terms." -- 12px, `#9CA3AF`
- Collapsible "What happens next?" section with numbered steps

**Loading state:** Button shows spinner + "Sending..."

### 3. OTP Verification
Full-screen centered card (440px), no sidebar.

- Back button (ghost)
- "Check your email" -- 24px semibold, centered
- Shows submitted email address -- 14px, `#6B7280`
- 32px gap
- **6 individual digit inputs** in a row: each 48x56px, 12px radius, 2px border, 30px bold centered font, 8px gap between. Focused input gets indigo border. Filled inputs get `#EEF2FF` background. Auto-advance on entry, auto-submit on 6th digit.
- 16px gap
- "Didn't receive it?" + "Resend code" link (with 60s cooldown)
- 24px gap
- "Verify" button -- primary, large, full width
- "Code expires in 4:32" -- 12px countdown

**Error state:** Inputs shake horizontally (3 cycles), borders turn red, error message below.
**Success:** Inputs flash green bg, checkmark overlay, smooth transition to dashboard.

### 4. Dashboard (THE Main Screen)
App shell with sidebar. This is the most important screen.

**Section 1 -- Page Header:** "Dashboard" or "Welcome, {name}!" -- 24px semibold

**Section 2 -- Balance Card (HERO ELEMENT):**
Credit-card-style element, max-width 480px, min-height 220px, 16px radius. Background: indigo-to-violet gradient. All text white.
- Top-left: truncated wallet address (mono, 12px, 80% opacity) + copy icon
- Center-left: **$20.00** in 48px bold (the hero number, counts up from 0 on load)
- Below balance: "USDC" label (12px, 80% opacity)
- Below that: secondary balances "0.10 ETH . 0.10 WETH" (14px, 70% opacity)
- Bottom-center: "Fund Wallet" button (white bg, indigo text, 8px radius)
- Hover: subtle 3D parallax tilt (max 3deg), shadow deepens, diagonal shine effect shifts
- Shadow: `0 10px 15px rgba(0,0,0,0.06), 0 4px 6px rgba(0,0,0,0.04)`

**Section 3 -- Quick Actions:**
Horizontal row of pill chips: "Send", "Fund", "Invite Agent", "Settings". Each has a 20px icon + label, full-round radius, white bg, border, 12px 16px padding.

**Section 4 -- Your Agents:**
Header: "Your Agents" + "View all" link. 3-column grid of Agent Cards.
Each agent card: 20px padding, 12px radius. Shows 32px circle icon, agent name (16px semibold), purpose line (12px gray, truncated), status dot (8px, colored), spending progress bar (6px height, full-round), "$12 / $50" text. Hover: lift 2px + shadow.
**Empty state:** Illustration, "No agents yet", "Generate an invitation code to let an AI agent connect to your wallet.", CTA button.
Last card slot: dashed border "Add Agent" card with + icon.

**Section 5 -- Recent Transactions:**
Header: "Recent Transactions" + "View all" link. List of 5 rows.
Each row: 56px height, 36px circle icon (colored bg + icon), agent name (14px medium), description (12px gray), amount (14px, green for positive/red for negative), timestamp (12px, light gray). Subtle dividers between rows.
**Empty state:** "No transactions yet."

### 5. Agent List
Sidebar + page header "Agents" with "Generate Invitation Code" primary button + search input.
Filter tabs below header: All, Active, Pending, Suspended (with count badges). Active tab has indigo underline.
Content: responsive grid of Agent Cards (3 cols desktop, 2 medium, 1 narrow).

### 6. Agent Detail
Breadcrumb: "Agents > Agent Name". Header: 48px icon + name (24px) + status badge. Subtitle: purpose + created date. Action buttons: "Suspend Agent" (danger outline), "Rotate Token" (secondary).

**Card 1 -- Spending Limits:** Four rows (per-transaction, daily, weekly, monthly) each with label, progress bar, "$X / $Y" display. Edit toggle reveals input fields. Auto-approve threshold input.

**Card 2 -- Allowed Recipients:** List of addresses with remove buttons. Add input + button at bottom. Empty: "All recipients allowed."

**Card 3 -- Activity Feed:** Timeline with vertical line + dots. Each entry: type icon, amount, recipient address (truncated mono), status badge, timestamp. "Load more" at bottom.

### 7. Transactions
Full-width data table. Header: "Transactions" + "Export CSV" button.
**Filter bar:** Date range picker, agent dropdown (multi-select), type toggles (All/Send/Receive/Earn), status toggles (All/Completed/Pending/Failed), search input.
**Table columns:** Date, Agent, Type, Amount, Recipient/Service, Status, Description. Sortable by Date/Amount/Agent. Sticky header row with `#F9FAFB` bg. 52px row height, hover highlights. Pagination bottom-right: "Showing 1-25 of 142", page size selector, prev/next.

### 8. Approvals Queue
Header: "Approvals" with "X pending" subtitle. Vertical stack of approval cards (16px gap).
Each card: 24px padding, 3px left border in `#F59E0B` (pending accent). Shows 40px agent icon, agent name, timestamp, requested amount (20px semibold), recipient address (mono), quoted reason. Expandable details section. "Approve" (green) and "Deny" (red outline) buttons, 12px gap.
**Empty state:** Checkmark-shield illustration, "All caught up!", celebratory tone.

### 9. Settings
Vertical stack of settings cards:

**Global Spending Limits:** Daily/weekly/monthly cap inputs with utilization progress bars. Minimum reserve balance input. Emergency kill switch: large red toggle "Disable all agent spending" with confirmation modal.

**Notifications:** Toggle rows for: transaction completed, approval needed, budget threshold (80%), agent suspended, incoming deposit. Each row has label, description, and switch.

**Invitation Codes:** "Generate Code" button. Table: code (mono font), status badge (Active/Used/Expired/Revoked), created date, linked agent, revoke button.

**Network:** Toggle between Testnet (Sepolia) and Mainnet (Base) with network indicator dot. Mainnet switch triggers safety checklist modal.

**Wallet Info:** Full address with copy button, connected email, session status.

### 10. Fund Wallet
Tab layout: "Buy Crypto" | "Deposit".
**Buy tab:** Coinbase Onramp widget placeholder with branding. Note about card/bank transfer.
**Deposit tab:** "Your Wallet Address" header, full address + copy button, large centered QR code (200px), supported tokens note (USDC, ETH, WETH on Base/Sepolia), network badge.

### 11. Spending Analytics
Header: "Spending Breakdown" + time range pills (7d, 30d, 90d).
**Row 1 (2 columns):** Horizontal bar chart (spending by agent) + donut chart (spending by category).
**Row 2 (full width):** Line chart -- daily spending trend.
**Row 3:** "Top Services" table -- service name, total spent, tx count, last used.
Chart styling: primary gradient colors for fills, 12px gray axis labels, dashed `#F0EDE8` grid lines, tooltip cards with shadow.

---

## Micro-interactions

- **Balance card:** 3D parallax tilt on hover (max 3deg), shadow deepens, diagonal shine effect shifts
- **Balance number:** Counts up from 0 to value over 500ms with easing on data load
- **Agent cards:** translateY(-2px) + shadow increase on hover (200ms ease), scale(0.98) on click
- **Buttons:** Background shift on hover (200ms), scale(0.98) on press (100ms)
- **Approve/Deny:** Flash colored background (150ms), then card fades out
- **Pending badge dot:** Pulses opacity 1.0 to 0.4 (2s loop)
- **Page transitions:** Content fades out (100ms) + slide left, new fades in (200ms) + slide from right
- **Toasts:** Slide in from right + fade (300ms ease-out), slide out + fade (200ms)
- **Modals:** Backdrop fades in (200ms), content scales 0.95 to 1.0 + fades (250ms)
- **Sidebar collapse:** Width animates 200ms ease, text fades, icons stay centered
- **Copy button:** Icon swaps to checkmark, tooltip "Copied!", reverts after 2s
- **Skeleton loading:** Shimmer gradient sweep left-to-right, 1.5s loop, `#F3F4F6` to `#E5E7EB`
- **OTP error:** Inputs shake horizontally (3 cycles, 4px, 300ms total)
- Respect `prefers-reduced-motion`: disable transforms, zero durations

---

## Key UX Notes

1. **Balance card is THE hero** -- credit-card-style with gradient, it should feel like a premium physical card. One big number per screen.
2. **Consumer app, not dev tool** -- if it looks like a Swagger UI or admin dashboard, it is wrong. Think Apple Wallet, Revolut, Mercury.
3. **Permission controls must be intuitive** -- spending limits use visual progress bars, not just numbers. Kill switch is prominent and red.
4. **Beautiful data visualization** -- charts use gradient fills, smooth animations, clear labels. No ugly default chart styles.
5. **Empty states are designed moments** -- each has an illustration, friendly copy, and a contextual CTA. Never just "No data."
6. **Dark mode is a first-class citizen** -- deep navy backgrounds, slightly brighter gradients, same semantic colors at reduced opacity tints.
7. **Onboarding is full-screen and focused** -- no sidebar, no chrome, just the centered card flow. Minimal friction.
8. **All monetary values use tabular/monospace numerals** for clean alignment in tables and lists.
9. **Navigation:** 6 items in sidebar (Dashboard, Agents, Transactions, Approvals, Fund, Settings) using Lucide icons. Active item gets indigo background tint.
10. **Wallet addresses** always truncated (first 6 + last 4 chars) with copy button, displayed in mono font.
