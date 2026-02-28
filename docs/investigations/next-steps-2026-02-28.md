# Tally Agentic Wallet — Next Steps Investigation

**Date**: 2026-02-28
**Investigator**: Claude Code research agent

---

## Current State Summary

### What Is Built

**Rust Backend** (fully implemented, 10 command modules):
- `auth.rs` — login, verify OTP, status, logout
- `agents.rs` — list, get, get_policy, update_policy, suspend, revoke, get_transactions
- `transactions.rs` — list (paginated), get single
- `wallet.rs` — get_balance, get_address
- `budget.rs` — agent summaries, global summary
- `settings.rs` — global policy CRUD, kill switch
- `approvals.rs` — list, get, resolve
- `notifications.rs` — get/update preferences
- `invitation_codes.rs` — list, generate, revoke

**Frontend** (all 11 screens exist, design system complete):

| Screen | File | Status |
|--------|------|--------|
| Onboarding (4 slides) | `src/pages/Onboarding.tsx` | Working |
| Install Skill | `src/pages/InstallSkill.tsx` | Working (expand/collapse fixed) |
| Connect Coinbase | `src/pages/ConnectCoinbase.tsx` | Working |
| Verify OTP | `src/pages/VerifyOtp.tsx` | Working (OTP rendering fixed) |
| Home Dashboard | `src/pages/Home.tsx` | Working (placeholder data) |
| Add Funds | `src/pages/AddFunds.tsx` | Working (placeholder data) |
| Agents List | `src/pages/AgentsList.tsx` | Working (placeholder data) |
| Agent Detail | `src/pages/AgentDetail.tsx` | Working (placeholder data) |
| Transaction Detail | `src/pages/TransactionDetail.tsx` | Working (placeholder data) |
| Settings | `src/pages/Settings.tsx` | Working |
| Stats | `src/pages/Stats.tsx` | Placeholder page |

**Design System** (complete):
- `src/styles/tokens.css` — all CSS custom properties
- `src/styles/globals.css` — resets, typography scale, animations
- `src/index.css` — Tailwind v4 theme integration
- 9 UI components: Button, InputGroup, OtpInput, ProgressBar, SegmentControl, StatusPill, Stepper, SuccessCheck, Toggle
- 3 layout components: BottomNav, ScreenHeader, TopBar
- 3 agent components: AgentCard, AgentPillRow, AgentAvatar
- 2 transaction components: TransactionItem, MetaCard
- 1 icon component: AppLogo

**State Management** (3 Zustand stores):
- `authStore.ts` — auth state, login/logout, status check
- `walletStore.ts` — address, balances
- `agentStore.ts` — agents list, budget summaries

**Typed API Layer**: `src/lib/tauri.ts` wraps all Tauri invoke calls with TypeScript types covering auth, wallet, agents, transactions, approvals, invitations, notifications, budget, and settings.

### What Works End-to-End (E2E tested 2026-02-28)
- 9 of 11 test suites fully passing
- All navigation and routing working
- All screens render correctly with placeholder data
- OTP input and expand/collapse bugs were fixed in the latest swarm session

---

## Uncommitted Work That Needs Committing

There is a **large uncommitted changeset** (69 files changed, 574 insertions, 10,604 deletions):

### Staged Changes (ready to commit)
- `src-tauri/tauri.conf.json` — window resized to 390x844, non-resizable

### Unstaged Modifications
- `src/App.tsx` — full route tree with 11 routes, `DefaultRedirect`, auth check on mount
- `src/components/ui/button.tsx` — cva-based button replacing old shadcn button
- `src/index.css` — Tailwind v4 + design system tokens
- `src/lib/tauri.ts` — typed API wrapper
- `src/pages/AgentDetail.tsx` — redesigned for new design system
- `src/pages/Onboarding.tsx` — 4-slide welcome flow
- `src/pages/Settings.tsx` — redesigned settings
- `src/stores/authStore.ts` — Zustand store with flowId cleanup

### Deleted Files (~45 files)
The old Phase 2 UI (shadcn-based dashboard, sidebar layout, desktop-first components) has been completely removed. This includes old components in `dashboard/`, `onboarding/`, `shared/`, old pages (`Dashboard`, `Agents`, `Approvals`, `Fund`, `Transactions`, Settings sub-pages), old hooks, and old utilities.

### Untracked New Files (~30+ files)
All the new design system infrastructure: styles, UI components, layout components, agent/transaction components, icons, new pages (Home, AddFunds, AgentsList, ConnectCoinbase, VerifyOtp, InstallSkill, TransactionDetail, Stats), stores, mock data, and docs.

### Also Untracked
- `.mcp.json` — MCP server config
- `code-reviews/` — 10 Ralph Loop review files from the bugfix swarm
- `docs/design/` — implementation plan, product brief, screen designs, system docs

**Recommendation**: Commit in logical groups:
1. Design system foundation (styles, tokens, index.css, tauri.conf.json)
2. Shared components (ui/, layout/, agent/, transaction/, icons/)
3. Stores and API layer (stores/, lib/tauri.ts, lib/cn.ts, data/)
4. Pages and routing (all pages + App.tsx)
5. Docs and reviews (docs/, code-reviews/)

Or, if the old UI is intentionally replaced wholesale, a single commit with message like "Replace desktop dashboard UI with mobile-first design system" is acceptable.

---

## Remaining Gaps Between Frontend and Backend

### 1. All Screens Use Placeholder Data (HIGHEST PRIORITY)
Every post-auth screen (Home, Agents, AgentDetail, TransactionDetail, AddFunds, Settings) imports from `src/data/placeholder_data.json` instead of calling the Tauri backend. The typed API layer (`src/lib/tauri.ts`) is fully defined but **not wired into any page components**.

### 2. Missing Backend Commands
| Command | Needed By | Status |
|---------|-----------|--------|
| `install_skill` | InstallSkill page | Not implemented in Rust |
| `resume_agent` (or equivalent) | AgentDetail unpause | Not implemented |

### 3. Stats Page Is Empty
`src/pages/Stats.tsx` is a placeholder. No design spec exists for what statistics to show. The bottom nav has a Stats tab pointing to it.

### 4. Approval Flow Not Surfaced in UI
The backend has full `approvals.rs` (list, get, resolve) and the API layer has `tauriApi.approvals`, but no UI screen exists for viewing/resolving approval requests. The product brief mentions approval thresholds but no dedicated approvals screen.

### 5. Invitation Codes Not Surfaced
Backend has `invitation_codes.rs` with generate/list/revoke. API layer has `tauriApi.invitations`. No UI exists.

### 6. QR Code Not Implemented
AddFunds page shows a placeholder box instead of an actual QR code. Need to add `qrcode.react` dependency.

### 7. "View on Explorer" Not Wired
TransactionDetail references opening BaseScan but `@tauri-apps/plugin-shell` may not be installed for the `open()` API.

### 8. No Loading/Error States
Pages don't show loading spinners, skeleton screens, or error handling for failed Tauri invocations.

---

## Pre-Existing Architectural Issues (from Ralph Loop Reviews)

### HIGH: Client-Side Auth Is UX-Only, Not Security
**Source**: Ralph Round 3, Pass 1 (Adversarial)

`ProtectedRoute` in `App.tsx` checks `isAuthenticated` from Zustand. Anyone with JS console access can flip this boolean. This is inherent to SPA architecture and cannot be fixed at the frontend alone.

**Mitigation needed**: Every Tauri command handler in Rust must independently verify the auth session. Route guards should be treated as UX convenience, not security boundaries.

### HIGH: `checkAuthStatus` Fail-Open in Browser Mode
**Source**: Ralph Round 3, Pass 3 (Correctness)

When running outside Tauri (browser dev mode), `checkAuthStatus` silently returns without changing state if `__TAURI_INTERNALS__` is not present. This is intentional for visual testing but means the auth loading state is never resolved, and there is no loading indicator while auth is being checked.

**Mitigation needed**: Add an `isLoading` state to authStore that is `true` during `checkAuthStatus` and render a splash/loading screen until it resolves.

### MEDIUM: `flowId` Not Cleared on Successful Auth
**Source**: Ralph Round 2/3 reviews

When `checkAuthStatus` succeeds, `flowId` is not cleared. Stale `flowId` could cause confusion if the user logs out and re-authenticates.

**Status**: Partially addressed (cleared on failure and logout, but not on success path).

---

## Recommended Next Priorities (Ordered)

### Priority 1: Commit All Uncommitted Work
The working tree has ~70 files changed including a complete UI rewrite. This needs to be committed immediately to avoid losing work. The swarm bugfixes doc notes "Committed and pushed" is still unchecked.

### Priority 2: Wire Real Tauri Backend Calls Into Pages
Replace placeholder data with actual `tauriApi.*` calls. This is the single largest gap — every screen is rendering static JSON. Approach per the implementation plan's Wave 3/4 structure:
- **Wave 3 first**: Home (`get_balance`, `get_address`, `list_transactions`, `list_agents`), AgentsList, AddFunds
- **Wave 4 next**: AgentDetail (policy CRUD, suspend), TransactionDetail, Settings (notification prefs, logout)

### Priority 3: Add Loading and Error States
Every page that calls Tauri invoke needs:
- Loading indicator (skeleton or spinner) while awaiting response
- Error handling with user-visible feedback (toast via sonner)
- Auth expiry detection and redirect to onboarding

### Priority 4: Fix Auth Loading State
Add `isLoading` boolean to `authStore` and render a splash screen in `App.tsx` until `checkAuthStatus` resolves. Currently there is a flash of the wrong route on app startup.

### Priority 5: Implement `install_skill` Backend Command
The InstallSkill page currently just transitions to success state without doing anything. Need a Rust command that writes the necessary `claude.md` and `agents.md` files to enable agent wallet access.

### Priority 6: Add QR Code to Add Funds
Install `qrcode.react` and render actual QR code from the wallet address.

### Priority 7: Design and Implement Stats Page
The Stats tab exists in the bottom nav but the page is a placeholder. Decide what analytics to show (spending over time, per-agent breakdown, transaction volume) and implement.

### Priority 8: Surface Approval Queue in UI
The backend fully supports approval workflows. Add a UI for viewing pending approvals and approving/rejecting them. Could be a tab on the Home screen or a dedicated route.

### Priority 9: Harden Auth Architecture
- Ensure all Rust command handlers verify auth independently
- Add `isLoading` splash screen to prevent route flash
- Clear `flowId` on successful authentication
- Consider adding OTP input accessibility attributes (`autoComplete="one-time-code"`)

### Priority 10: E2E Re-Test After Backend Wiring
Once real Tauri calls replace placeholder data, run a full E2E test pass in the Tauri app (not just browser) to verify the complete flow from onboarding through authenticated screens.

---

## Files Referenced

- `/Users/dennisonbertram/Develop/apps/agent-neo-bank/docs/design/implementation-plan.md`
- `/Users/dennisonbertram/Develop/apps/agent-neo-bank/docs/design/product-brief.md`
- `/Users/dennisonbertram/Develop/apps/agent-neo-bank/docs/testing/e2e-test-results.md`
- `/Users/dennisonbertram/Develop/apps/agent-neo-bank/docs/process/swarms/e2e-bugfixes-2026-02-28.md`
- `/Users/dennisonbertram/Develop/apps/agent-neo-bank/src/App.tsx`
- `/Users/dennisonbertram/Develop/apps/agent-neo-bank/src/lib/tauri.ts`
- `/Users/dennisonbertram/Develop/apps/agent-neo-bank/src/stores/authStore.ts`
- `/Users/dennisonbertram/Develop/apps/agent-neo-bank/src/data/placeholder_data.json`
- `/Users/dennisonbertram/Develop/apps/agent-neo-bank/code-reviews/e2e-bugfixes-ralph-r3-pass1-20260228-074314.md`
