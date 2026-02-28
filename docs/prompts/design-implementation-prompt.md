# Design Implementation: UI Polish

```
Implement the Tally Agentic Wallet design system and polish all existing screens using the swarm skill.

## Context
- Design spec: docs/design/standalone-design-prompt.md (THE source of truth for all visual decisions)
- Design brief: docs/design/design-brief.md
- Architecture: docs/architecture/architecture-plan.md
- Phases 1a, 1b, 2, 2.5 complete. All backend functionality working. App runs on localhost:1420.
- Current UI uses default shadcn dark zinc theme — needs to become warm light premium theme.
- Pencil designs exist for all 11 screens. This prompt implements them in React.

## Current Frontend Structure
- Framework: React + Vite + TypeScript + Tailwind CSS v4 + shadcn/ui
- Router: react-router-dom (Routes in src/App.tsx)
- Layout: Shell.tsx wraps sidebar routes, Onboarding is standalone
- Existing pages: Dashboard, Agents, AgentDetail, Transactions, Approvals, Settings, Onboarding
- Missing pages: Fund (will be built in Phase 3, NOT this prompt)
- UI components: src/components/ui/ (shadcn: badge, button, card, dialog, input, sonner, table)
- Dashboard components: src/components/dashboard/ (AgentBudgets, BudgetProgress, GlobalBudget)

## Important: Preserve All Existing Functionality
This is a VISUAL polish pass. All existing Tauri invoke() calls, state management, data fetching, and backend integration must remain intact. We are reskinning, not rebuilding.

## Design System to Implement

### CSS Theme (src/index.css) — THE FIRST TASK
Replace the dark zinc shadcn defaults with the warm light theme:

**Light mode variables:**
- --background: #FAFAF9 (warm off-white)
- --foreground: #1A1A1A
- --card: #FFFFFF
- --card-foreground: #1A1A1A
- --primary: #4F46E5 (indigo-600)
- --primary-foreground: #FFFFFF
- --secondary: #FFFFFF (white bg buttons)
- --secondary-foreground: #1A1A1A
- --muted: #F9FAFB
- --muted-foreground: #6B7280
- --accent: #EEF2FF (indigo-50)
- --accent-foreground: #4F46E5
- --destructive: #EF4444
- --border: #E8E5E0 (warm gray)
- --input: #E8E5E0
- --ring: #6366F1
- --radius: 0.75rem (12px for cards)
- --sidebar-background: #FFFFFF
- --sidebar-foreground: #1A1A1A
- --sidebar-primary: #4F46E5
- --sidebar-accent: #EEF2FF
- --sidebar-border: #E8E5E0

**Additional custom properties needed:**
- --color-text-secondary: #6B7280
- --color-text-tertiary: #9CA3AF
- --color-border-subtle: #F0EDE8
- --color-success: #10B981
- --color-success-bg: #ECFDF5
- --color-warning: #F59E0B
- --color-warning-bg: #FFFBEB
- --color-danger: #EF4444
- --color-danger-bg: #FEF2F2

### Typography
- Primary font: Inter (import from Google Fonts or fontsource)
- Mono font: JetBrains Mono (for wallet addresses, tx hashes, amounts)
- Add `font-feature-settings: "tnum"` for tabular numerals on monetary values

### Icon Set
- Use Lucide icons throughout (already installed via lucide-react)
- Default size: 20px, stroke: 1.5px

## Wave 1 (parallelize — foundation + independent components)

1. **CSS Theme Update** (`src/index.css`, font imports)
   - Replace all shadcn dark zinc variables with warm light theme above
   - Import Inter and JetBrains Mono fonts
   - Add custom utility classes: `.font-mono-tabular` for monetary values
   - Add card shadow: `shadow-sm` = `0 1px 3px rgba(0,0,0,0.06), 0 1px 2px rgba(0,0,0,0.04)`
   - Tests: existing component tests still pass, no visual regressions in Playwright

2. **Sidebar Redesign** (`src/components/layout/Sidebar.tsx`)
   - White background (#FFFFFF), right border (#E8E5E0)
   - 240px width (currently 60/w-60 — update to w-60 = 240px)
   - App logo at top: "Tally Agentic Wallet" with small icon (or just styled text for now)
   - 6 nav items: Dashboard, Agents, Transactions, Approvals, Fund, Settings
   - Active item: indigo background (#EEF2FF), indigo text (#4F46E5), semibold
   - Inactive: #6B7280 text, hover → #F9FAFB background
   - Icons: 20px Lucide icons (LayoutDashboard, Bot, ArrowUpDown, CheckCircle, Wallet, Settings)
   - Add "Fund" nav item (route to /fund — placeholder page for now)
   - Bottom: user profile area with email initial avatar
   - Tests: nav items render, active state works, Fund link present

3. **Shell/Header Update** (`src/components/layout/Shell.tsx`, `Header.tsx`)
   - Content area: max-width 1200px, centered, padding 32px
   - Background: #FAFAF9
   - Remove any dark-theme header styling
   - Tests: Shell renders content correctly

4. **Shared UI Components** (new: `src/components/shared/`)
   - `StatusBadge.tsx`: Pill shape (rounded-full), 6px colored dot + text. Props: status (active/pending/suspended/revoked)
   - `ProgressBar.tsx`: 6px height, full-round radius, configurable color. Props: value, max, color
   - `MonoAddress.tsx`: Truncated address (first 6 + last 4), mono font, copy button with tooltip
   - `GradientCard.tsx`: The hero balance card component — gradient bg, white text, parallax tilt on hover
   - Tests: each component renders with props, copy button works

## Wave 2 (depends on Wave 1 — CSS theme must be in place)

5. **Dashboard Redesign** (`src/pages/Dashboard.tsx`, new components in `src/components/dashboard/`)
   - **Balance Card (HERO)**: GradientCard component. Background: `linear-gradient(135deg, #4F46E5, #7C3AED, #6366F1)`. Credit-card shape, max-width 480px, min-height 220px, 16px radius.
     - Top-left: truncated wallet address (mono, 12px, 80% opacity) + copy icon
     - Center-left: balance in 48px bold (hero number), white
     - Below: "USDC" label (12px, 80% opacity)
     - Secondary balances: ETH + WETH amounts (14px, 70% opacity)
     - Bottom: "Fund Wallet" button (white bg, indigo text)
     - Hover: subtle shadow deepens, slight translateY(-2px)
   - **Quick Actions Row**: Horizontal row of pill chips: "Send", "Fund", "Invite Agent", "Settings". Each: icon + label, rounded-full, white bg, border, px-4 py-2.
   - **Your Agents Section**: "Your Agents" header + "View all" link. 3-column grid of agent cards with: 32px circle icon, name, purpose (truncated), status dot, spending progress bar, "$X / $Y". Add agent card with dashed border + icon.
   - **Recent Transactions Section**: "Recent Transactions" + "View all". 5-row list: 36px icon circle, agent name, description, amount (green/red), timestamp.
   - **Empty states**: Illustration placeholder + friendly copy + CTA for both agents and transactions.
   - IMPORTANT: Preserve all existing invoke() calls and state management. Wrap existing data in new visual components.
   - Tests: balance card renders with data, quick actions clickable, agent cards show data, transaction list renders

6. **Onboarding Redesign** (`src/pages/Onboarding.tsx`, `src/components/onboarding/`)
   - Full-screen centered layout, NO sidebar/Shell
   - Soft radial gradient background: #EEF2FF fading to #FAFAF9
   - **Welcome step**: centered card (max-width 440px), logo, "Give your AI agents spending power" headline (24px semibold), subtext, "Get Started" primary button (48px height, full width), "Powered by Coinbase Agent Wallet" footer
   - **Email step**: back button, "Connect your wallet" title, email input (48px height), display name input, "Send Verification Code" button, legal text
   - **OTP step**: "Check your email" title, email display, 6 individual digit inputs (48x56px each, 12px radius, 2px border), auto-advance, "Resend code" link, "Verify" button
   - IMPORTANT: Preserve existing onboarding state machine and Tauri invoke() calls
   - Tests: all 3 steps render, navigation works, form submission works

## Wave 3 (depends on Wave 2)

7. **Agents List Redesign** (`src/pages/Agents.tsx`)
   - Page header: "Agents" (24px semibold) + "Generate Invitation Code" primary button + search input
   - Filter tabs: All | Active | Pending | Suspended (with count badges). Active tab has indigo underline.
   - Responsive grid: 3 columns desktop, 2 medium, 1 narrow
   - Agent cards: 20px padding, 12px radius, status dot, spending progress bar, purpose subtitle
   - Dashed "Add Agent" card as last item
   - Tests: filter tabs work, grid renders, search filters

8. **Agent Detail Redesign** (`src/pages/AgentDetail.tsx`)
   - Breadcrumb: "Agents > {name}"
   - Header: 48px icon + name (24px) + StatusBadge + created date
   - Action buttons: "Suspend Agent" (red outline), "Rotate Token" (secondary)
   - **Card 1 — Spending Limits**: 4 rows (per-tx, daily, weekly, monthly) with ProgressBar + "$X/$Y". Edit toggle.
   - **Card 2 — Allowed Recipients**: Address list with MonoAddress + remove buttons. Add input at bottom.
   - **Card 3 — Activity Feed**: Timeline with vertical line + colored dots. Entry: icon, amount, address, status, timestamp.
   - Tests: all 3 cards render, edit mode works, activity shows

9. **Transactions Redesign** (`src/pages/Transactions.tsx`)
   - **Filter bar** in card: date range, agent dropdown, type toggles (All/Send/Receive/Earn), status toggles, search
   - **Data table**: sticky header (#F9FAFB), 52px rows, 7 columns (Date, Agent, Type, Amount, Recipient, Status, Description)
   - Type badges: Send=indigo pill, Receive=green, Earn=amber
   - Amount: mono font, green for positive, red for negative
   - **Pagination**: "Showing X-Y of Z", page size selector, prev/next buttons
   - "Export CSV" secondary button in header
   - Tests: table renders with data, filters work, pagination works

10. **Approvals Redesign** (`src/pages/Approvals.tsx`)
    - Header: "Approvals" + "X pending" amber subtitle
    - Vertical stack of cards with 3px left border in #F59E0B
    - Each card: 40px agent icon, name, timestamp, amount (20px semibold), recipient (mono), reason (quoted), Approve (green) + Deny (red outline) buttons
    - Empty state: checkmark-shield illustration, "All caught up!"
    - Tests: cards render, approve/deny trigger invoke(), empty state shows

11. **Settings Redesign** (`src/pages/Settings.tsx` and sub-pages)
    - 5 vertically stacked cards with 24px gap:
      - **Global Spending Limits**: 3 ProgressBar rows + reserve balance input + kill switch (red toggle)
      - **Notifications**: 5 toggle rows with labels and descriptions
      - **Invitation Codes**: Generate button + table (code mono, status badge, date, agent, revoke)
      - **Network**: Testnet/Mainnet selector cards side-by-side, network dot indicator
      - **Wallet Info**: Full address with copy, email, session status
    - Tests: toggles work, kill switch confirmation, code generation

## Wave 4 (depends on Wave 3)

12. **Micro-interactions & Polish**
    - Balance card: hover shadow deepens + translateY(-2px), transition 200ms
    - Agent cards: hover lift (translateY(-2px) + shadow), press scale(0.98)
    - Buttons: hover background shift (200ms), press scale(0.98) (100ms)
    - Copy button: icon swaps to checkmark, "Copied!" tooltip, revert after 2s
    - Status badge pending dot: pulse animation (opacity 1→0.4, 2s loop)
    - Skeleton loading: shimmer gradient sweep, 1.5s loop
    - Toast notifications: slide from right, colored left border
    - `prefers-reduced-motion`: disable transforms, zero durations
    - Tests: animations don't break layout, reduced motion respected

13. **Fund Placeholder Page** (`src/pages/Fund.tsx`)
    - Simple placeholder: "Fund Wallet — Coming Soon" with the correct layout structure
    - Two tab placeholders: "Buy Crypto" and "Deposit"
    - This will be fully implemented in Phase 3
    - Add route to App.tsx
    - Tests: page renders, tabs switch

14. **Visual Regression & Integration Tests**
    - Run ALL existing tests — nothing may break
    - Update any Playwright selectors that changed due to restyling
    - Screenshot comparison for key screens (if Playwright visual testing available)
    - Verify all Tauri invoke() calls still work
    - Verify routing still works with new Shell/Sidebar

## Rules
- STRICT: Preserve all existing backend integration (invoke() calls, state, data fetching)
- This is a VISUAL pass — if a component already fetches data correctly, wrap it in new styling
- Use existing shadcn/ui components where possible (Card, Button, Input, Badge, Dialog, Table)
- Add new shadcn components via `npx shadcn@latest add <component>` if needed (e.g., tabs, toggle, progress)
- All monetary values must use JetBrains Mono with tabular numerals
- All wallet addresses use MonoAddress component (truncated, copy button)
- Use worktree isolation for all code-writing agents
- Commit after each passing wave
- The standalone design prompt (docs/design/standalone-design-prompt.md) is the VISUAL REFERENCE — match it

## Deliverable
Premium, consumer-grade UI matching the design system: warm off-white background, indigo primary, gradient hero balance card, styled data tables, progress bars, status badges, micro-interactions. All existing functionality preserved. Full test coverage maintained.
```
