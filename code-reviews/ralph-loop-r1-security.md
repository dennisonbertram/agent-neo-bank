# Security Review -- Round 1 (Attacker Perspective)

**Reviewer**: GPT-5.2 Security / Attacker
**Date**: 2026-02-27
**Scope**: React frontend pages, shared components, layout components (17 files)
**Context**: Tauri v2 desktop app -- all backend calls via `invoke()` IPC, no direct HTTP

---

## XSS

None found. No usage of `dangerouslySetInnerHTML`. All user-supplied strings (agent names, descriptions, addresses, error messages) are rendered via JSX text interpolation, which React auto-escapes. No dynamic `href` or `src` attributes constructed from user data. The codebase is clean on this axis.

---

## CRITICAL

### 1. Financial amounts displayed with raw string interpolation -- no formatting or precision control

**Where**: `src/pages/AgentDetail.tsx` lines 367-368, 413
**Issue**: Spending limits and transaction amounts are displayed as raw `${row.spent} / ${row.limit}` and `${tx.amount}` without any formatting. The `row.spent` and `row.limit` values come from `parseFloat()` on backend strings. JavaScript `parseFloat` followed by default `toString()` can produce results like `0.30000000000000004` for common decimal operations. In a financial application, a user seeing `$0.30000000000000004 / $100` could be confused or, worse, misinterpret their remaining budget. This also applies to the activity feed column where `tx.amount` (a raw backend string) is displayed with a `$` prefix but zero formatting.

**Fix**: Use a consistent formatting function (like `CurrencyDisplay` or `toLocaleString` with fixed decimals) for ALL financial values. Never display raw `parseFloat` results or unformatted strings with a `$` prefix. Specifically:
- Line 367-368: Replace `${row.spent} / ${row.limit}` with a formatter that applies fixed decimal precision.
- Line 413: Replace `${tx.amount}` with `formatCurrency(tx.amount, tx.asset)` or `CurrencyDisplay`.

### 2. `formatCurrency` is a no-op -- returns raw string with no numeric formatting

**Where**: `src/lib/format.ts` line 3-5
**Issue**: `formatCurrency(amount, asset)` just returns `` `${amount} ${asset}` `` -- it does zero numeric formatting, no locale formatting, no decimal precision clamping. This is used in `Transactions.tsx` line 374 to display financial amounts. If the backend returns `"0.100000000000000001"` or `"1e-7"`, those strings are displayed verbatim to the user prefixed with the send/receive sign. An attacker controlling an agent that creates transactions with carefully crafted amount strings could display misleading values (e.g., a string like `"100 USDC to attacker\nActual: 0.001"` if the backend doesn't sanitize -- though React escapes newlines in rendering, the raw unformatted string is still the core issue).

**Fix**: `formatCurrency` must parse the string to a number and format with appropriate decimal precision for the asset type. Example:
```ts
export function formatCurrency(amount: string, asset = "USDC"): string {
  const num = parseFloat(amount);
  if (isNaN(num)) return `-- ${asset}`;
  const decimals = asset === "USDC" ? 2 : 6;
  return `${num.toLocaleString("en-US", { minimumFractionDigits: 2, maximumFractionDigits: decimals })} ${asset}`;
}
```

### 3. Approval action has no confirmation gate -- single click approves transactions

**Where**: `src/pages/Approvals.tsx` lines 96-106, 250-251
**Issue**: The `handleResolve` function immediately invokes `resolve_approval` on click with no confirmation dialog, no undo window, and errors are silently swallowed. An accidental click on "Approve" for a high-value transaction sends it through immediately. The silent error catch on line 103-105 means if the approval partially succeeds or the backend errors, the user gets zero feedback -- they may believe an approval was denied when it was actually approved, or vice versa.

**Fix**:
1. Add a confirmation dialog before approving/denying, especially for transactions. Show the amount, recipient, and agent name in the confirmation.
2. Surface errors from `handleResolve` to the user via a toast or inline error state rather than swallowing them silently.
3. Consider adding optimistic UI with rollback or at minimum a loading/disabled state on the buttons during the invoke call to prevent double-clicks.

---

## HIGH

### 1. Inconsistent financial formatting across the application -- three different strategies

**Where**: Multiple files
**Issue**: The app uses at least four different approaches to format financial values:
- `CurrencyDisplay` component (`src/components/shared/CurrencyDisplay.tsx`) -- uses `toLocaleString` with asset-specific decimals. This is the best implementation.
- `formatBalance` in `Dashboard.tsx` -- uses `toLocaleString` with 2 decimal places, ignores asset type.
- `formatCurrency` in `src/lib/format.ts` -- no formatting at all, returns raw string.
- Raw string interpolation in `AgentDetail.tsx` -- `${row.spent}` with no formatting.

This inconsistency means the same amount could display differently on different screens. A user checking a $1,000.50 balance on the Dashboard sees `$1,000.50` but the same value on AgentDetail might show as `$1000.5` or `$1000.4999999999999`. This undermines trust and could cause confusion about actual balances.

**Fix**: Standardize on `CurrencyDisplay` (or its underlying logic) as the single source of truth for all financial formatting. Remove or deprecate `formatCurrency` from `format.ts` and `formatBalance` from `Dashboard.tsx`. Audit every `${}` interpolation that displays money.

### 2. Spending policy edit sends raw string values to backend without numeric coercion

**Where**: `src/pages/AgentDetail.tsx` lines 104-108, 116-127
**Issue**: `handleSaveLimits` validates that fields are valid numbers but then sends `editPolicy` to the backend with the raw string values (line 106: `invoke("update_agent_spending_policy", { policy: editPolicy })`). The `handleEditChange` function on line 118 stores `e.target.value` (a string) directly. So the policy object sent to the backend has string values like `"100.00"` rather than numbers. While the validation checks `parseFloat`, it does not coerce the values. If the backend expects numeric types, this could silently fail or store incorrect values. If the backend accepts strings, values like `"100.00"` and `"100"` represent the same amount but could cause comparison issues.

**Fix**: Before sending to the backend, coerce validated string values to numbers:
```ts
const coerced = {
  ...editPolicy,
  per_tx_max: parseFloat(editPolicy.per_tx_max).toString(),
  daily_cap: parseFloat(editPolicy.daily_cap).toString(),
  // etc.
};
await invoke("update_agent_spending_policy", { policy: coerced });
```

### 3. Silent error swallowing throughout the codebase hides failures from the user

**Where**: `src/pages/Approvals.tsx` lines 81-83, 103-105; `src/pages/Agents.tsx` line 17; `src/pages/Dashboard.tsx` lines 108-109; `src/pages/Fund.tsx` lines 19-25
**Issue**: Multiple critical operations silently catch and discard errors:
- `loadAgents` in Approvals -- if agents fail to load, approval cards show raw agent IDs instead of names, potentially confusing users into approving for the wrong agent.
- `handleResolve` in Approvals -- approve/deny errors are silently swallowed. User has no idea if their action succeeded.
- `list_agents` in Agents.tsx -- empty catch discards error, shows empty state indistinguishable from "no agents exist."
- `fetchBudgets` in Dashboard -- budget fetch failure shows nothing, user may think they have no budget utilization.

**Fix**: At minimum, surface errors for all user-initiated actions (approve, deny, save). For background data fetches, distinguish between "loading", "no data", and "error" states so the user knows when data is missing vs. unavailable.

---

## MEDIUM

### 1. Onboarding uses `window.location.href` for navigation instead of React Router

**Where**: `src/pages/Onboarding.tsx` line 79
**Issue**: `handleComplete` uses `window.location.href = "/"` which causes a full page reload, losing all React state. In a Tauri app this also triggers a full webview reload. Should use React Router's `useNavigate()` for SPA navigation.

### 2. Clipboard write exposes full wallet address to system clipboard without user awareness

**Where**: `src/components/shared/MonoAddress.tsx` line 22, `src/pages/Fund.tsx` line 34, `src/components/onboarding/FundStep.tsx` line 21
**Issue**: Clicking the copy button writes the full wallet address to the system clipboard with no indication of what was copied beyond a brief checkmark icon. The address remains in the clipboard indefinitely. Other apps can read it. While this is standard UX for crypto apps, there is no clipboard clearing mechanism.

### 3. No rate limiting or debounce on approval button clicks

**Where**: `src/pages/Approvals.tsx` lines 248-264
**Issue**: The approve/deny buttons have no disabled state during the async operation. Rapid double-clicking could fire `resolve_approval` twice. While the backend should be idempotent, this is defense-in-depth.

### 4. `handleSuspend` has no confirmation and no loading/error state

**Where**: `src/pages/AgentDetail.tsx` lines 72-76
**Issue**: Suspending an agent is a significant action but has no confirmation dialog, no loading indicator during the operation, and no error handling. A misclick suspends the agent immediately.

### 5. Dashboard "Recent Transactions" section is hardcoded empty

**Where**: `src/pages/Dashboard.tsx` lines 270-292
**Issue**: The recent transactions section always shows the "No transactions yet" empty state. It never fetches or displays actual recent transactions. This could mislead users into thinking no transactions have occurred when agents are actively spending.

---

## LOW

### 1. Sidebar has hardcoded user profile ("dennison", "D" avatar)

**Where**: `src/components/layout/Sidebar.tsx` lines 63-69
**Issue**: The user profile section is hardcoded. Should fetch from auth state.

### 2. `CurrencyDisplay` falls back to raw string display for NaN amounts

**Where**: `src/components/shared/CurrencyDisplay.tsx` lines 13-15
**Issue**: When `parseFloat(amount)` returns NaN, the component renders the raw amount string in a monospace span. This is a reasonable fallback but could display unexpected content if the backend sends malformed data.

### 3. Onboarding wallet address initialized to placeholder `"0x..."`

**Where**: `src/pages/Onboarding.tsx` line 20
**Issue**: The wallet address is initialized to the string `"0x..."`. If `auth_status` fails silently, this placeholder is passed to `FundStep` which checks `address !== "0x..."` to determine readiness. This coupling between a magic string and component logic is fragile.

### 4. Export CSV button is non-functional

**Where**: `src/pages/Transactions.tsx` lines 220-223
**Issue**: The "Export CSV" button has no `onClick` handler. This is a UI stub, not a security issue, but could confuse users.

---

**CRITICAL: 3 | HIGH: 3 | APPROVED: NO**
