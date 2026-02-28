# Ralph Loop R3 -- Protocol Correctness / Logic Review

**Reviewer perspective**: Protocol Correctness / Logic
**Date**: 2026-02-27
**Files reviewed**: 11 files across `src/pages/`, `src/components/shared/`, `src/components/onboarding/`

---

## CRITICAL

### 1. Spending policy validation casts string fields through `parseFloat` but saves raw string -- type mismatch can commit invalid data

**Where**: `src/pages/AgentDetail.tsx` lines 78--113 (`validateField` + `handleSaveLimits`)

**Issue**: `validateField` receives the SpendingPolicy field value cast as `string` via `editPolicy[field] as string` (line 95). However, `SpendingPolicy` fields (`per_tx_max`, `daily_cap`, etc.) are typed as `string` and the `<Input type="number">` binds `e.target.value` (a string) into the policy via `handleEditChange`. The validation calls `parseFloat(value)` and checks `>= 0`, but the raw string (e.g. `"10.300000000001"`, `"1e5"`, `"-0"`) is what gets sent to the backend via `invoke("update_agent_spending_policy", { policy: editPolicy })`. If the backend expects a specific numeric format (e.g. decimal string without scientific notation), the frontend can commit malformed values. Critically, `parseFloat("-0")` passes the `>= 0` check (since `-0 >= 0` is `true` in JS), and `"1e5"` parses as 100000 but may be stored literally.

**Fix**: After validation, normalize the field values to a canonical decimal string (e.g. `parseFloat(value).toFixed(2)` or a BigDecimal library) before saving. Reject scientific notation and negative zero explicitly.

### 2. `handleEditChange` stores raw input string but `validateField` is called on save with potentially stale type assumption

**Where**: `src/pages/AgentDetail.tsx` lines 90--97

**Issue**: `fieldsToValidate` iterates over `SpendingPolicy` keys and accesses `editPolicy[field] as string`. The `as string` cast is unchecked. If any of these fields were ever set to a non-string value (e.g., a number from the backend response), `validateField` would call `parseFloat` on a number coerced to string, which works, but the `value.trim() === ""` check would fail silently on `"undefined"` if the field were truly missing. More critically, `auto_approve_max` and `allowlist` are also SpendingPolicy fields that are NOT validated but ARE sent to the backend in the full `editPolicy` object. If the user edits only caps, the `auto_approve_max` from the original policy is passed through unvalidated, and if the backend policy object was mutated server-side between load and save, stale `auto_approve_max`/`allowlist` values overwrite the current backend state.

**Fix**: Either send only the changed fields to the backend (PATCH semantics), or re-fetch the policy before saving and merge only the user-edited fields. Validate `auto_approve_max` alongside the cap fields.

---

## HIGH

### 1. `Approvals.handleResolve` has no optimistic lock or double-click guard -- can double-approve/deny

**Where**: `src/pages/Approvals.tsx` lines 96--106

**Issue**: `handleResolve` is an `async` function called directly from button `onClick`. There is no `isSaving` guard or button disabled state. If the user double-clicks "Approve", two `invoke("resolve_approval", ...)` calls fire concurrently. The first succeeds; the second may fail (benign) or, if the backend is not idempotent, may corrupt state. The buttons also have no `disabled` prop set during the async operation.

**Fix**: Add a `processingId` state that tracks which approval is being resolved. Disable both Approve/Deny buttons for that approval while the request is in-flight. Clear on completion.

### 2. `Transactions` pagination total count is wrong when client-side search is active

**Where**: `src/pages/Transactions.tsx` lines 171--174, 401--426

**Issue**: The pagination UI shows `Showing {showStart}-{showEnd} of {total}` where `total` is the server-side total, but `filteredTransactions` is the client-side-filtered subset of the current page. When `searchQuery` is active, the user sees "Showing 1-20 of 150" but only 3 rows are rendered (those matching the search). The pagination controls still allow navigating through all 150 server-side results, but the page counts and "Showing X-Y" are misleading. Navigating to page 2 might show 0 results if the search term only matches items on page 1.

**Fix**: Either move search to the server side, or when client-side search is active, display `filteredTransactions.length` as the count and disable server-side pagination (or clearly indicate that search only applies to the current page).

### 3. `CurrencyDisplay` uses floating-point arithmetic for financial amounts

**Where**: `src/components/shared/CurrencyDisplay.tsx` lines 13, 20--23

**Issue**: `parseFloat(amount)` converts the string amount to a JS float, then `toLocaleString` formats it. For values like `"0.1"` + `"0.2"`, floating-point representation can produce `0.30000000000000004`. While `toLocaleString` with `maximumFractionDigits` typically rounds this correctly, edge cases exist for large values near `Number.MAX_SAFE_INTEGER / 1e6` where precision is lost entirely. For a financial application, this is a design-level risk.

**Fix**: Use a decimal arithmetic library (e.g. `decimal.js`, `bignumber.js`) or keep amounts as string-formatted values from the backend and only do formatting without float conversion. At minimum, document the precision limits.

### 4. `AgentDetail` spending display uses `parseFloat` on financial strings without precision control

**Where**: `src/pages/AgentDetail.tsx` lines 186--208, 366--369

**Issue**: `policyRows` builds `limit` and `spent` values via `parseFloat(policy.per_tx_max) || 0`. These floats are then displayed as `${row.spent} / ${row.limit}` using JS template literals, which use `Number.toString()` -- this produces outputs like `$0.30000000000000004 / $100`. No `toFixed()` or locale formatting is applied.

**Fix**: Use `toLocaleString` or `toFixed(2)` for display, or use the `CurrencyDisplay` component for consistency. Better yet, use a decimal library.

### 5. `Onboarding` uses `window.location.href` for navigation -- breaks SPA routing and loses React state

**Where**: `src/pages/Onboarding.tsx` line 79

**Issue**: `handleComplete` sets `window.location.href = "/"`, which causes a full page reload in a Tauri/React SPA. This destroys all React state, unmounts the entire app, and re-initializes everything. In a Tauri app, this may also reset auth state held in memory. The rest of the app uses `react-router-dom` for navigation.

**Fix**: Use `useNavigate()` from react-router-dom: `const navigate = useNavigate(); navigate("/");`

### 6. Dashboard "Recent Transactions" section is hardcoded empty -- never fetches data

**Where**: `src/pages/Dashboard.tsx` lines 270--292

**Issue**: The "Recent Transactions" section always shows "No transactions yet" regardless of whether transactions exist. No fetch is performed. The dashboard fetches agent budgets but not recent transactions. This is a logic gap -- the UI implies transactions will appear dynamically, but they never will.

**Fix**: Add a `fetchRecentTransactions` call (similar to `AgentDetail`) and render actual transaction data, or clearly label this as a placeholder/coming-soon feature.

### 7. `Approvals.pendingCount` counts client-side filtered results, not actual pending count

**Where**: `src/pages/Approvals.tsx` line 108

**Issue**: `pendingCount` is computed as `approvals.filter(a => a.status === "pending").length`. When `filter === "pending"`, the server only returns pending items, so `pendingCount === approvals.length` (correct). But when `filter === "all"`, the server returns all approvals, and `pendingCount` shows the count of pending ones in the returned set. If the server paginates or limits results, this count may be less than the actual total pending count in the system.

**Fix**: Fetch the pending count separately from the server, or always include it in the response metadata.

---

## MEDIUM

### 1. `MonoAddress` truncates addresses shorter than 10 characters incorrectly

**Where**: `src/components/shared/MonoAddress.tsx` line 14

**Issue**: `address.slice(0, 6)` + `"..."` + `address.slice(-4)` produces a string longer than the original for addresses shorter than 10 characters (e.g., `"0x1234"` becomes `"0x1234...1234"` -- the slices overlap). The `full` prop bypass exists but the truncation logic itself is fragile.

**Fix**: Add a length guard: `const displayAddress = full || address.length <= 10 ? address : ...`

### 2. `AgentDetail.formatDate` treats `0` as falsy

**Where**: `src/pages/AgentDetail.tsx` lines 139--142

**Issue**: `if (!timestamp)` is true for `timestamp === 0`, which represents the Unix epoch (Jan 1, 1970). While unlikely in practice, this would display "Never" for a valid timestamp.

**Fix**: Use `if (timestamp === null || timestamp === undefined)`.

### 3. `Agents` page does not handle fetch errors visibly

**Where**: `src/pages/Agents.tsx` lines 14--19

**Issue**: The `.catch(() => {})` swallows all errors silently. If the backend is down, the user sees "No agents yet" instead of an error message, which is misleading.

**Fix**: Set an error state and display a retry option.

### 4. `Fund` page calls two different Tauri commands for the same data

**Where**: `src/pages/Fund.tsx` lines 16--27

**Issue**: First tries `get_wallet_address`, then falls back to `auth_status`. This dual-path approach is fragile and the two commands may return different address formats. If the first command is deprecated or renamed, this silently falls through.

**Fix**: Standardize on a single command for wallet address retrieval, or document the fallback chain explicitly.

### 5. `Approvals` filter state not reflected in URL

**Where**: `src/pages/Approvals.tsx` line 52

**Issue**: The `filter` state (`"pending"` vs `"all"`) is held in component state. Refreshing the page always resets to `"pending"`. If a user shares a link or navigates back, the filter context is lost.

**Fix**: Sync filter state to URL search params via `useSearchParams()`.

### 6. `Dashboard.fetchBudgets` has no race condition guard

**Where**: `src/pages/Dashboard.tsx` lines 99--113

**Issue**: Unlike `AgentDetail` and `Transactions` which use `requestRef` for stale-response protection, `fetchBudgets` has no guard. If `fetchBudgets` is somehow called twice (e.g., StrictMode double-mount in dev), both responses will be applied, with the slower one winning regardless of which was initiated last.

**Fix**: Add a `requestRef` pattern consistent with other pages, or use an AbortController.

---

## LOW

### 1. `Transactions.agentMap` is recomputed on every render

**Where**: `src/pages/Transactions.tsx` line 119

**Issue**: `const agentMap = new Map(agents.map(...))` runs on every render. Should be memoized with `useMemo`.

**Fix**: `const agentMap = useMemo(() => new Map(agents.map(a => [a.id, a.name])), [agents]);`

### 2. `Approvals` has a duplicated `truncateAddress` function

**Where**: `src/pages/Approvals.tsx` lines 44--47

**Issue**: This duplicates the same function from `@/lib/format`. Slightly different behavior (hardcoded 4 chars vs configurable).

**Fix**: Use the shared `truncateAddress` from `@/lib/format`.

### 3. Inconsistent error handling patterns across pages

**Where**: Multiple files

**Issue**: `AgentDetail` shows errors on save, `Approvals` silently swallows resolve errors, `Agents` silently swallows fetch errors, `Dashboard` silently swallows budget errors. There is no consistent error boundary or notification pattern.

**Fix**: Adopt a consistent error notification system (toast/snackbar pattern).

### 4. `ProgressBar` does not handle negative values

**Where**: `src/components/shared/ProgressBar.tsx` line 18

**Issue**: If `value` is negative, `Math.min((value / max) * 100, 100)` produces a negative percentage, which renders a zero-width bar (CSS `width: -X%` collapses). Not a crash, but semantically incorrect.

**Fix**: Clamp: `Math.max(0, Math.min((value / max) * 100, 100))`

### 5. `AgentDetail` "Allowed Recipients" X button has no handler

**Where**: `src/pages/AgentDetail.tsx` lines 388--390

**Issue**: The X button next to each allowlist address has no `onClick` handler. It renders as a clickable button that does nothing.

**Fix**: Either add a remove handler or hide the button until the feature is implemented.

---

CRITICAL: 2 HIGH: 7 APPROVED: NO
