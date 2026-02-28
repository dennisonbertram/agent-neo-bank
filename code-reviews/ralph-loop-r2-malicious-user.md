# Ralph Loop R2 -- Malicious User / UX Attacker Review

**Reviewer perspective**: Malicious User / UX Attacker
**Date**: 2026-02-27
**Scope**: React frontend pages and shared components (Tauri v2 desktop app)

---

## CRITICAL

### 1. Suspend Agent has no confirmation dialog -- wrong-agent action risk

**Where**: `src/pages/AgentDetail.tsx` lines 72-76, 249-256

**Issue**: The "Suspend Agent" button calls `handleSuspend` immediately on click with no confirmation dialog. If a user navigates to the wrong agent detail page (e.g., via stale browser history, or clicks the wrong card on the Agents list), a single mis-click permanently suspends an active agent. There is no undo mechanism visible. The `id` from `useParams` is used directly -- if a user has two tabs open on different agents and the URL changes, the suspend fires against whatever `id` is currently in the URL.

**Fix**: Add a confirmation modal that displays the agent name and ID before executing `invoke("suspend_agent", ...)`. Disable the button during the async operation and show a loading state.

---

### 2. Approval Approve/Deny buttons have no confirmation and no double-click protection

**Where**: `src/approvals/Approvals.tsx` lines 96-106, 248-264

**Issue**: Approving or denying a financial transaction (which may involve real token transfers) requires only a single click with zero confirmation. The `handleResolve` function has no loading/disabled state -- rapid double-clicks can fire duplicate `resolve_approval` calls. For transaction approvals displaying amounts like "500 USDC", a single mis-tap approves an irreversible on-chain transfer. The approval card shows a truncated recipient address (`truncateAddress`), so the user cannot fully verify the destination before approving.

**Fix**: (a) Add a confirmation step that shows the full recipient address and amount before approve/deny. (b) Add `disabled` state while the async call is in-flight. (c) Show full address (not truncated) on the confirmation dialog for transaction-type approvals.

---

### 3. Onboarding displays placeholder "0x..." as a real wallet address that users can copy

**Where**: `src/pages/Onboarding.tsx` line 20, `src/components/onboarding/FundStep.tsx` lines 12, 49

**Issue**: `walletAddress` is initialized to the string literal `"0x..."`. If the `auth_status` call fails or returns no address (lines 37-43, 61-65 of Onboarding.tsx), this placeholder is passed to `FundStep`. The `FundStep` component checks `addressReady = address !== "0x..." && address !== ""` to guard the copy button, but the raw `"0x..."` string is still displayed to the user in a `<code>` element styled identically to a real address. A user could manually copy this placeholder and attempt to send real funds to it, losing them permanently. The "Continue to Dashboard" button is disabled, but the address text is selectable.

**Fix**: Do not display any address text until a valid address is loaded. Show a loading skeleton or spinner instead of the placeholder string. Alternatively, display an explicit "Address not yet available" message.

---

### 4. Financial amounts displayed without proper decimal formatting -- materially misleading

**Where**: `src/pages/AgentDetail.tsx` lines 366-368, 413

**Issue**: In the Spending Limits display (line 367) and Activity feed (line 413), amounts are rendered as raw `${row.spent} / ${row.limit}` and `${tx.amount}` using string interpolation with no formatting. The `parseFloat` on lines 190-206 silently rounds or truncates high-precision values. For example, a budget spent of `"99.999999"` would display as `$99.999999` while the progress bar shows it as under the limit of `$100`, but the actual remaining budget is only $0.000001. Conversely, a value like `"0.100000000000000001"` would display misleadingly. The `formatCurrency` utility in `src/lib/format.ts` simply concatenates `amount + " " + asset` with no numeric formatting at all.

**Fix**: Use the `CurrencyDisplay` component (which properly formats to 2-6 decimals) consistently across all financial displays. Fix `formatCurrency` in `src/lib/format.ts` to actually parse and format the number. Add explicit rounding rules documented per asset type.

---

## HIGH

### 1. Revoke invitation code has no confirmation dialog

**Where**: `src/pages/Settings/InvitationCodes.tsx` lines 66-73, 129-136

**Issue**: Revoking an invitation code is a destructive, irreversible action. The revoke button fires immediately on click with no confirmation prompt. A mis-click in the codes table could revoke a code that an agent is about to use for registration, permanently blocking that agent's onboarding.

**Fix**: Add a confirmation dialog before calling `invoke("revoke_invitation_code", ...)`.

---

### 2. Approval errors are silently swallowed -- user thinks action succeeded

**Where**: `src/pages/Approvals.tsx` lines 96-106

**Issue**: The `handleResolve` catch block is empty (`// silently handle`). If the backend rejects an approve/deny (e.g., network error, already resolved, policy violation), the user sees no error feedback. The approval card disappears from the list after `loadApprovals()` re-fetches, making it appear the action succeeded when it may not have. For financial transactions, this means the user believes they approved a payment that never actually went through, or denied one that is still pending.

**Fix**: Display an error toast/banner when `resolve_approval` fails. Keep the approval card visible with its action buttons if the operation failed.

---

### 3. Remove-recipient button (X) in Allowed Recipients has no handler and no confirmation

**Where**: `src/pages/AgentDetail.tsx` lines 388-390

**Issue**: The X button next to each allowed recipient address has no `onClick` handler -- it is a completely non-functional button styled to look interactive. A user clicking it would expect the recipient to be removed from the allowlist but nothing happens. If/when a handler is added, there is no confirmation dialog for what is a security-critical action (removing an address from an allowlist changes which addresses the agent can send funds to).

**Fix**: Either wire up the handler with a confirmation dialog, or remove the button entirely until the feature is implemented. A non-functional destructive-looking button is worse than no button.

---

### 4. Dashboard always shows "No transactions yet" hardcoded empty state regardless of actual data

**Where**: `src/pages/Dashboard.tsx` lines 283-292

**Issue**: The "Recent Transactions" section always renders the static "No transactions yet" empty state. It never fetches or displays actual transaction data. A user with active agents making transactions sees a dashboard claiming there are no transactions, which is materially misleading. This could cause a user to believe their agents are not spending when they actually are.

**Fix**: Fetch recent transactions (e.g., `list_transactions` with a small limit) and display them. If the data is intentionally deferred, display "Coming soon" instead of a definitive "No transactions yet."

---

### 5. Global policy caps saved without validation -- user can set negative or zero limits

**Where**: `src/pages/Settings/GlobalPolicy.tsx` lines 22-42

**Issue**: The `handleCapChange` function directly sets whatever string the user types. `handleSaveCaps` sends it to the backend with no client-side validation. A user could set daily_cap to `"-100"` or `"0"` or `"abc"`. Setting caps to 0 could silently block all agent transactions without the user realizing the distinction from the kill switch. The `type="number"` attribute on the inputs does not prevent all invalid values (e.g., `"e"`, empty string, negative via arrow keys).

**Fix**: Add numeric validation (>= 0, must be a valid number) before saving, similar to the validation already implemented in `AgentDetail.tsx` lines 78-101. Show inline validation errors.

---

### 6. Fund page "Buy Crypto" button is clickable but non-functional (coming soon)

**Where**: `src/pages/Fund.tsx` lines 84-86

**Issue**: The "Continue to Coinbase" button is fully styled as an active, clickable primary button but has no `onClick` handler and does nothing. The "Coming soon" note is in tiny `text-xs` gray text below, easily missed. A user clicking the prominent CTA repeatedly would think the app is broken. This is especially misleading on a financial onramp page where users expect to transact.

**Fix**: Either disable the button with `disabled` attribute and `cursor-not-allowed` styling (matching the "Rotate Token" pattern in AgentDetail), or add an `onClick` that shows a clear "Coming soon" toast/modal.

---

### 7. QR code on Fund/Deposit page is a placeholder icon, not an actual QR code

**Where**: `src/pages/Fund.tsx` lines 129-134

**Issue**: The deposit tab shows a large QR-code-shaped placeholder (a dashed border box with a QrCode icon) alongside the text "Scan this QR code or copy the address above to send funds." This text instructs the user to scan a QR code that does not exist. A user relying on QR scanning to deposit funds would be confused and potentially blocked from funding their wallet via their preferred method.

**Fix**: Either generate a real QR code from the wallet address (using a library like `qrcode.react`), or remove the placeholder and QR-referencing text entirely until the feature is implemented.

---

### 8. Sidebar hardcodes user profile to "dennison" -- shows wrong identity

**Where**: `src/components/layout/Sidebar.tsx` lines 63-69

**Issue**: The sidebar shows a hardcoded user avatar initial "D" and username "dennison" with status "Connected". This is not fetched from any auth state. Any user of this app would see another person's name, which is confusing and undermines trust. In a financial application, displaying the wrong identity is a significant UX safety issue -- a user might think they are logged into the wrong account.

**Fix**: Fetch the actual authenticated user's name/email from auth state and display it. Show a loading state while auth info is being fetched.

---

## MEDIUM

### 1. MonoAddress copy button copies full address but displays truncated -- potential confusion

**Where**: `src/components/shared/MonoAddress.tsx` lines 14, 21

**Issue**: The component displays a truncated address (e.g., `0x1234...abcd`) but the copy button copies the full address. While this is arguably correct behavior, there is no tooltip or visual indication that the copied value differs from what is displayed. Users verifying by visual comparison would see a mismatch.

**Fix**: Add a tooltip or brief flash showing "Full address copied" to make the behavior explicit.

### 2. CurrencyDisplay shows ETH with up to 6 decimals but not full precision

**Where**: `src/components/shared/CurrencyDisplay.tsx` lines 6-9

**Issue**: ETH and WETH are capped at 6 decimal places (`maximumFractionDigits: 6`). ETH has 18 decimals of precision. While 6 decimals is reasonable for display, very small amounts (dust) could display as `$0.00` and be invisible to the user.

**Fix**: Consider showing a "< $0.01" indicator for non-zero amounts that round to zero at the display precision.

### 3. Export CSV button is non-functional

**Where**: `src/pages/Transactions.tsx` lines 220-224

**Issue**: The "Export CSV" button has no `onClick` handler. It looks fully functional but does nothing.

**Fix**: Disable the button or add a "Coming soon" indicator.

### 4. Agent cards on Dashboard are not clickable links

**Where**: `src/pages/Dashboard.tsx` lines 39-73

**Issue**: `AgentCard` components on the Dashboard are not wrapped in links to the agent detail page, unlike the cards on the Agents page. Users would expect to click through to see agent details.

**Fix**: Wrap `AgentCard` in a `<Link to={/agents/${agent.agent_id}}>`.

---

## LOW

### 1. Multiple `catch` blocks silently swallow errors with `// silently handle`

**Where**: Multiple files -- Approvals.tsx:103, Agents.tsx:17, InvitationCodes.tsx:43/62/72, GlobalPolicy.tsx:37/55

**Issue**: Consistent pattern of empty catch blocks. While not directly a UX attack vector, silent failures mean the user has no feedback when operations fail, creating a false sense of successful action.

**Fix**: Add error state and display error feedback for all user-initiated actions.

### 2. `Onboarding.handleComplete` uses `window.location.href` instead of React Router navigation

**Where**: `src/pages/Onboarding.tsx` lines 77-79

**Issue**: Full page reload loses any in-memory state. Not a security issue but a UX roughness.

**Fix**: Use `useNavigate()` from React Router.

### 3. ProgressBar does not handle edge case where value > max

**Where**: `src/components/shared/ProgressBar.tsx` line 18

**Issue**: The component caps at 100% via `Math.min`, which is correct, but when an agent exceeds their budget, the visual bar looks identical to being exactly at the limit with no over-budget indicator.

**Fix**: Consider adding a visual indicator (e.g., red pulsing border) when value exceeds max.

---

## Summary

CRITICAL: 4 | HIGH: 8 | APPROVED: NO

### Key themes requiring immediate attention:
1. **Missing confirmation dialogs on destructive/financial actions** (suspend agent, approve/deny transactions, revoke codes)
2. **Placeholder/hardcoded data displayed as real** (wallet address "0x...", username "dennison", static "no transactions" state, fake QR code)
3. **Financial amount formatting inconsistency** (raw string interpolation vs. CurrencyDisplay component, no rounding discipline)
4. **Silent error handling** throughout the codebase means users cannot distinguish success from failure on critical operations
