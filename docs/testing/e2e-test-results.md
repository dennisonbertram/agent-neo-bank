# E2E Test Results — Tally Agentic Wallet Frontend

**Date**: 2026-02-28
**Environment**: Browser (localhost:1420), 390x844px viewport
**Auth bypass**: `isAuthenticated: true` in authStore for protected route testing (reverted after)
**Tester**: Chrome browser automation (mcp__claude-in-chrome)

---

## Summary

| Suite | Name | Result | Issues |
|-------|------|--------|--------|
| 1 | Onboarding | ALL PASS | — |
| 2 | Install Skill | PASS with issue | Expand/collapse not working |
| 3 | Connect Coinbase | PASS | Expected Tauri error in browser |
| 4 | Verify OTP | ISSUE | OTP inputs not rendering (0 in DOM) |
| 5 | Home Dashboard | ALL PASS | — |
| 6 | Add Funds | ALL PASS | — |
| 7 | Agents List | ALL PASS | — |
| 8 | Agent Detail | ALL PASS | — |
| 9 | Transaction Detail | ALL PASS | — |
| 10 | Settings | ALL PASS | — |
| 11 | Navigation & Routing | ALL PASS | Stats tab has no route |

**Overall: 9/11 suites fully passing, 2 with known issues**

---

## Suite 1: Onboarding (4/4 PASS)

- [x] Slide 1: "Your Agents, Your Rules" — icon, title, subtitle render
- [x] Slide 2: "Fund Once, Spend Smart" — swipe/dot navigation works
- [x] Slide 3: "Stay in Control" — dot indicator updates
- [x] Slide 4: "Connect & Go" — CTA changes to "Get Started", navigation dots show 4th active

## Suite 2: Install Skill (3/4 PASS)

- [x] Page renders: skill icon, "Coinbase Wallet" title, description
- [x] "Confirm Installation" button → success state with checkmark
- [x] Success state shows "Skill Installed" with "Continue" button
- [ ] **ISSUE**: "What changes?" expand/collapse section not responding to clicks

## Suite 3: Connect Coinbase (3/3 PASS)

- [x] Email input field renders, "Send code" button initially disabled
- [x] Typing email activates "Send code" button
- [x] Clicking "Send code" produces expected Tauri error in browser (`Cannot read properties of undefined (reading 'invoke')`)

## Suite 4: Verify OTP (2/3 PASS — 1 CRITICAL ISSUE)

- [x] Page title "Enter verification code" renders
- [x] Countdown timer and back button work
- [ ] **CRITICAL**: OTP input boxes not rendering at all — 0 `<input>` elements found in DOM. Component fails to mount, not just a visibility issue.
  - **GitHub Issue**: #1 (needs update — more severe than originally reported)

## Suite 5: Home Dashboard (5/5 PASS)

- [x] Top bar: "DB" avatar, "Tally Wallet" title, bell icon
- [x] Balance card: "$20.32", "BASE" network badge, wallet address `0x72AE...04B4`, "0.10 ETH 20.00 USDC"
- [x] Quick action buttons: "+ Add Funds", "Agents"
- [x] Segment control: "Overview" / "Agents" tabs
- [x] Activity list: 5 transactions with agent icons, tags (API FEE, GAS, SWAP, APPROVAL), amounts

## Suite 6: Add Funds (4/4 PASS)

- [x] QR code placeholder renders
- [x] Warning pill: "Only send assets on Base network"
- [x] Wallet address displayed with Copy button
- [x] "Buy with Card" button (disabled), Close button navigates back to /home

## Suite 7: Agents List (4/4 PASS)

- [x] Three agent cards: Research Agent (ACTIVE), Deploy Bot (ACTIVE), Treasury (PENDING)
- [x] Segment control "Active" filter: shows 2 agents, hides Treasury
- [x] Segment control "Archived" filter: shows empty state
- [x] Card click navigates to agent detail (`/agents/agent-research-001`)

## Suite 8: Agent Detail (6/6 PASS)

- [x] Header: back button, ACTIVE status pill, "Research Agent" name, description
- [x] Daily Spend card: "$3.50 / $30.00", progress bar (12%), "RESET IN 14H"
- [x] Spending Controls: Daily Limit stepper (increment $25→$30 works, percentage updates reactively to 14%)
- [x] Per Transaction stepper: $5.00
- [x] Approval Threshold toggle present
- [x] Agent History: 3 transactions (Arxiv API Call, Cross-Chain Query, Metadata Storage), Filter button, Save Changes button

## Suite 9: Transaction Detail (6/6 PASS)

- [x] Back button with "Details" label
- [x] Transaction amount: "-2.50 USDC", date "February 27, 2025 • 7:00 PM"
- [x] Agent info: icon, "Research Agent", "Verified Agent" badge
- [x] Agent Metadata table: Category (API Fee), Purpose (OpenAI GPT-4 API call), Request ID (REQ_-001)
- [x] Cost Breakdown: Service Fee (-2.50 USDC), Network Fee ($0.00)
- [x] Notes: "Market analysis batch job", "View on Explorer" link

## Suite 10: Settings (6/6 PASS)

- [x] "Home" back link
- [x] User profile: "DB" avatar, "Dennison Bertram", email
- [x] NOTIFICATIONS section: 5 toggle rows (Agent Requests, Transaction Completed, Approval Required, Daily Limit Reached, Low Balance)
- [x] Toggle interaction: clicking OFF→ON works (Transaction Completed tested)
- [x] ACCOUNT & SECURITY: "Reset Coinbase Connection" (red), "Export Wallet History"
- [x] Version: v0.1.0 (Base Mainnet)

## Suite 11: Navigation & Routing (7/7 PASS)

- [x] Bottom nav "Home" tab → `/home`
- [x] Bottom nav "Agents" tab → `/agents`
- [x] Bottom nav "Settings" tab → `/settings`
- [x] Home "Add Funds" shortcut → `/add-funds`
- [x] Home "Agents" shortcut → `/agents`
- [x] Transaction row click → `/transactions/tx-001`
- [x] Invalid route `/nonexistent-page` → redirects to `/onboarding` (catch-all)

**Notes:**
- Stats tab navigates to `/home` (no dedicated route yet)
- FAB (+) button has no action (placeholder)

---

## Issues Found

### Critical
1. **OTP inputs not rendering** (GitHub #1) — `OtpInput` component produces 0 `<input>` elements in DOM. Needs investigation — likely a component mount failure, not a CSS visibility issue.

### Minor
2. **Install Skill expand/collapse broken** — "What changes?" section doesn't respond to clicks. Needs new GitHub issue.
3. **Stats tab has no route** — Bottom nav "Stats" falls through to `/home`. Low priority.
4. **FAB (+) button no-op** — Center button in bottom nav does nothing. Needs route assignment.

---

## Post-Test Cleanup

- [x] Reverted `authStore.ts`: `isAuthenticated` set back to `false`, `email` set to `null`
