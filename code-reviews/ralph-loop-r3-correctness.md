# Ralph Loop R3 -- Protocol Correctness / Logic Review (Re-review)

**Reviewer perspective**: Protocol Correctness / Logic
**Date**: 2026-02-27
**Round**: 3 (re-review after R2 fixes)
**Files reviewed**: 12 files across `src/pages/`, `src/components/shared/`, `src/lib/`

## Previously fixed items (confirmed resolved)

- Stale async race conditions: `useRef` counters added in `AgentDetail`, `Approvals`, `Transactions` -- CONFIRMED
- `CurrencyDisplay` asset-aware decimals: per-asset decimal map applied -- CONFIRMED
- `parseFloat` normalization before save in `AgentDetail.handleSaveLimits` (line 120-126) and `GlobalPolicy.handleSaveCaps` (line 61-66) -- CONFIRMED
- Numeric validation with `validateField` in `AgentDetail` and equivalent in `GlobalPolicy` -- CONFIRMED
- `setTimeout` cleanup on unmount in `Fund` and `MonoAddress` via `timerRef` -- CONFIRMED
- Confirmation dialogs added for suspend (`AgentDetail`), kill switch (`GlobalPolicy`), revoke (`InvitationCodes`), approve/deny (`Approvals`) -- CONFIRMED
- `Onboarding.handleComplete` now uses `useNavigate()` instead of `window.location.href` -- CONFIRMED
- `Approvals.handleResolve` has `processingId` guard and disabled buttons -- CONFIRMED
- `AgentDetail` spending display now uses `CurrencyDisplay` component instead of raw template literals -- CONFIRMED

---

## Verdict: APPROVED

**CRITICAL: 0 | HIGH: 2 (non-blocking, documented below)**

The codebase has addressed all previously identified CRITICAL issues. The remaining HIGH findings are architectural trade-offs and informational inaccuracies rather than data corruption or financial calculation bugs. They should be addressed in a follow-up iteration but do not block this round.

---

## HIGH

### 1. `Transactions` pagination metadata is misleading when client-side search is active

**Where**: `src/pages/Transactions.tsx:171-174, 288-312, 401-426`

**Issue**: The pagination footer shows `Showing {showStart}-{showEnd} of {total}` where `total` is the server-side count, but the rendered table shows `filteredTransactions` (client-side filtered). When `searchQuery` is non-empty:
- The row count on screen may be 2, but the footer says "Showing 1-20 of 150"
- Clicking "Next" fetches the next server page, which may have 0 matching results
- The empty-state at line 300 says "No matching transactions on this page" which is correct messaging, but the pagination counters above/below are still wrong

This is not data corruption but produces incorrect UI that could cause user confusion about missing transactions.

**Recommendation**: When `searchQuery` is non-empty, either (a) hide the server-side pagination counts, (b) replace with `filteredTransactions.length` of `transactions.length` on this page, or (c) move search server-side. The empty-state messaging at line 300-310 is already good -- extend that pattern to the pagination bar.

### 2. `Approvals.pendingCount` may undercount when viewing "all" with server-side limits

**Where**: `src/pages/Approvals.tsx:119`

**Issue**: `pendingCount` is `approvals.filter(a => a.status === "pending").length`. When `filter === "all"`, the backend returns all statuses. If the backend has pagination or result limits (not visible from frontend code alone), the count could be less than actual pending. The count is displayed prominently at line 178.

This is informational -- if the backend returns all results without pagination (which appears likely from the `list_approvals` call with no limit/offset), this is a non-issue. Flagging for awareness.

**Recommendation**: Confirm backend returns unbounded results, or fetch pending count as a separate scalar.

---

## MEDIUM

### 1. `GlobalPolicy.loadPolicy` lacks race condition guard

**Where**: `src/pages/Settings/GlobalPolicy.tsx:14-19`

**Issue**: Unlike `AgentDetail` and `Transactions` which use `useRef` request counters, `loadPolicy` uses a bare `.then()` chain with no stale-response guard. In React StrictMode (dev), the component mounts twice, firing two `get_global_policy` calls. The slower one wins. This is unlikely to cause visible issues since both calls return the same data, but it breaks the pattern established in other files.

**Recommendation**: Add a `useRef` guard or convert to the same `useCallback` + `requestRef` pattern used elsewhere.

### 2. `GlobalPolicy.handleSaveCaps` sets `updated_at` on the client side

**Where**: `src/pages/Settings/GlobalPolicy.tsx:67`

**Issue**: `updated_at: Math.floor(Date.now() / 1000)` is set client-side. If the client clock is skewed (common in desktop apps, VMs, or after sleep/wake), this timestamp will be incorrect. The backend should be the source of truth for timestamps.

**Recommendation**: Remove client-side `updated_at` and let the backend set it, or at minimum do not rely on this value for conflict detection.

### 3. `Notifications.handleToggle` applies boolean toggle to non-boolean field

**Where**: `src/pages/Settings/Notifications.tsx:19-21`

**Issue**: `handleToggle` does `!prefs[key]` for any key of `NotificationPreferences`. The type includes `large_tx_threshold: string` and `id: string`. If `handleToggle` were accidentally called with `"large_tx_threshold"` or `"id"`, it would toggle a truthy string to `false`, corrupting the preference object. Currently, only boolean keys are wired to toggle buttons (line 54-85), but the function signature accepts any key.

**Recommendation**: Narrow the type: `key: keyof Pick<NotificationPreferences, 'enabled' | 'on_all_tx' | 'on_large_tx' | 'on_errors' | 'on_limit_requests' | 'on_agent_registration'>`.

### 4. `Dashboard.fetchBudgets` has no stale-response guard

**Where**: `src/pages/Dashboard.tsx:99-113`

**Issue**: Same pattern gap as `GlobalPolicy`. No `requestRef` counter. In StrictMode double-mount or if `fetchBudgets` is called from a refresh button in the future, the slower response will overwrite the newer one.

**Recommendation**: Add `requestRef` guard.

### 5. `AgentDetail` overwrites entire policy including `auto_approve_max` and `allowlist` on save

**Where**: `src/pages/AgentDetail.tsx:118-130`

**Issue**: `handleSaveLimits` sends the full `normalizedPolicy` (spread from `editPolicy`) to `update_agent_spending_policy`. This includes `auto_approve_max` and `allowlist` which are NOT editable in the current UI. If another admin modifies these fields between the page load and save, the stale values from the original fetch will overwrite the updated backend values. This is a classic lost-update problem.

**Recommendation**: Send only the four editable cap fields, or implement optimistic concurrency (e.g., `updated_at` check).

### 6. `Fund` page calls two different Tauri commands for wallet address

**Where**: `src/pages/Fund.tsx:16-27`

**Issue**: First tries `get_wallet_address`, falls back to `auth_status`. The `Dashboard` uses `get_address` (yet another command). Three different commands for the same data across the app. If any are deprecated, the fallback chain may silently return different address formats.

**Recommendation**: Standardize on a single command. The `Dashboard` uses `get_address` returning `AddressResponse`; `Fund` uses `get_wallet_address` returning `string`. Pick one.

---

## LOW

### 1. `Transactions.agentMap` recomputed every render

**Where**: `src/pages/Transactions.tsx:119`

**Issue**: `const agentMap = new Map(agents.map(...))` runs on every render. Should use `useMemo`.

**Fix**: `const agentMap = useMemo(() => new Map(agents.map(a => [a.id, a.name])), [agents]);`

### 2. Duplicated `truncateAddress` in Approvals

**Where**: `src/pages/Approvals.tsx:45-48`

**Issue**: Local `truncateAddress` duplicates `@/lib/format.truncateAddress` with slightly different slice offsets (6/4 vs configurable chars+2/chars).

**Fix**: Use the shared utility.

### 3. `formatCurrency` in `format.ts` always appends asset name, `CurrencyDisplay` uses `$` prefix

**Where**: `src/lib/format.ts:9-18` vs `src/components/shared/CurrencyDisplay.tsx:12-25`

**Issue**: Two different formatting strategies for the same data. `formatCurrency("100", "USDC")` returns `"100.00 USDC"` while `CurrencyDisplay` renders `$100.00`. Transaction table uses `formatCurrency` (showing "100.00 USDC") while AgentDetail uses `CurrencyDisplay` (showing "$100.00"). Inconsistent user experience.

**Fix**: Unify formatting. For USDC, pick either `$X.XX` or `X.XX USDC` and use it everywhere.

### 4. `CurrencyDisplay` uses `parseFloat` for financial display

**Where**: `src/components/shared/CurrencyDisplay.tsx:13`

**Issue**: `parseFloat` on financial strings can lose precision for very large values (>2^53). `toLocaleString` with `maximumFractionDigits` handles rounding for typical values, but amounts exceeding ~9 quadrillion (unlikely but architecturally unbounded) would silently lose precision. Documented as accepted risk.

### 5. `InvitationCodes` silently swallows generate/revoke errors

**Where**: `src/pages/Settings/InvitationCodes.tsx:63-65, 74-76`

**Issue**: Both `handleGenerate` and `handleRevoke` catch blocks are empty. User gets no feedback if code generation or revocation fails.

**Fix**: Add error state and display feedback.

### 6. `MonoAddress` truncation overlaps for short addresses

**Where**: `src/components/shared/MonoAddress.tsx:14`

**Issue**: `address.slice(0, 6) + "..." + address.slice(-4)` for an 8-char address produces overlapping content (e.g., `"0x123456"` becomes `"0x1234...3456"` which is 14 chars, longer than the original). The `full` prop bypass exists but default truncation is fragile for non-Ethereum addresses.

**Fix**: Add length guard: `full || address.length <= 13 ? address : truncated`.

---

## Summary

| Severity | Count | Blocking? |
|----------|-------|-----------|
| CRITICAL | 0 | -- |
| HIGH | 2 | No (informational UI inaccuracy, not data corruption) |
| MEDIUM | 6 | No |
| LOW | 6 | No |

**Verdict: APPROVED**

The R2 fixes successfully addressed all CRITICAL issues (policy normalization, stale async guards, financial display). The two remaining HIGH issues are UI-level informational inaccuracies (pagination counts during client-side search, and pending count accuracy) -- neither causes data corruption, state corruption, or financial calculation errors. They should be tracked for the next iteration.

Key strengths observed:
- Consistent `useRef` race condition guards across data-fetching pages
- Proper validation + normalization pipeline before backend writes
- Confirmation dialogs on destructive actions
- `setTimeout` cleanup preventing memory leaks
- Asset-aware decimal formatting in `CurrencyDisplay`
