# Ralph Loop Round 2 - Review 3: Protocol Correctness / Logic

**Reviewer perspective**: Correctness -- logic, edge cases, invariants
**Date**: 2026-02-27
**Files reviewed**: AgentDetail, Transactions, Approvals, Dashboard, Onboarding, Fund, Agents, GlobalPolicy, InvitationCodes, CurrencyDisplay, MonoAddress, ProgressBar, FundStep, format.ts

---

## CRITICAL

None found.

## HIGH

None found.

## MEDIUM

### M-1: `pendingCount` in Approvals counts from filtered list, not total pending

**File**: `src/pages/Approvals.tsx` (line 118)
**Description**: `pendingCount` is derived from `approvals.filter(a => a.status === "pending").length`. When the filter is set to `"all"`, this correctly counts pending items from the full list. However, when the filter is `"pending"`, the backend already filters to pending-only, so the count is accurate. But if the backend `list_approvals` with `status: "pending"` ever returns stale resolved items (race with resolution), the count could be transiently wrong. This is minor since the count self-corrects on next load, but worth noting.
**Severity**: MEDIUM (cosmetic inaccuracy possible during concurrent resolution)

### M-2: `agentMap` in Transactions is recomputed on every render

**File**: `src/pages/Transactions.tsx` (line 119)
**Description**: `const agentMap = new Map(agents.map(...))` is computed outside of `useMemo`, so it is reconstructed on every render. With many agents and frequent re-renders this is wasteful. Not a correctness bug but an unnecessary allocation pattern.
**Severity**: MEDIUM (performance, not a logic bug)

### M-3: GlobalPolicy `handleSaveCaps` does not update local state after save

**File**: `src/pages/Settings/GlobalPolicy.tsx` (lines 55-71)
**Description**: After a successful `invoke("update_global_policy", ...)`, the local `policy` state is not refreshed. The user sees the old (unsaved) state object which happens to match what was saved, but `updated_at` and any server-side normalization are not reflected. If the backend modifies values (e.g., rounding), the UI will be out of sync until the page is revisited.
**Severity**: MEDIUM (stale local state after save)

### M-4: GlobalPolicy kill switch toggle/deactivate has no loading guard

**File**: `src/pages/Settings/GlobalPolicy.tsx` (lines 74-88, 90-101)
**Description**: `handleToggleKillSwitch` (deactivation path) and `handleConfirmKillSwitch` do not set any loading/disabled state, so the user can double-click and fire multiple `invoke` calls. The confirm activation button is also not disabled during the async operation.
**Severity**: MEDIUM (double-submit possible)

### M-5: Approvals `resolveError` display condition may hide the error

**File**: `src/pages/Approvals.tsx` (line 258)
**Description**: The error is shown only when `processingId === null && confirmAction?.id === approval.id`. After `handleResolve` completes with an error, `processingId` is set to `null` in the `finally` block, but `confirmAction` is NOT cleared on error (it is only cleared on success at line 107). This means the error IS displayed correctly. However, the condition `processingId === null` is redundant given the flow -- it works, but if `processingId` were set to another approval's ID (impossible in current flow since buttons are disabled), the error would be hidden. Current code is correct but fragile.
**Severity**: MEDIUM (fragile but currently correct)

## LOW

### L-1: `formatDate` in AgentDetail treats `timestamp === 0` as "Never"

**File**: `src/pages/AgentDetail.tsx` (lines 159-162)
**Description**: The condition `if (!timestamp)` is falsy for `0`, so a Unix timestamp of `0` (Jan 1 1970) would display as "Never" rather than the actual date. In practice, a `created_at` of `0` is unlikely but the guard is imprecise. A strict `null` check (`timestamp === null`) would be more correct.
**Severity**: LOW

### L-2: `MonoAddress` does not guard against very short addresses

**File**: `src/components/shared/MonoAddress.tsx` (line 14)
**Description**: `address.slice(0, 6)` and `address.slice(-4)` can produce overlapping or nonsensical output for addresses shorter than 10 characters. The `full` prop defaults to `false`, so a 5-character address would display as e.g., `0x123...x123`. Not a crash, just visually odd. All real Ethereum addresses are 42 chars so this is unlikely.
**Severity**: LOW

### L-3: Transactions search is client-side only, applied to current page

**File**: `src/pages/Transactions.tsx` (lines 202-213)
**Description**: The `searchQuery` filter is applied only to the current page of `transactions` (max 20). Matching transactions on other pages are invisible to the user. The "No matching transactions on this page" empty state partially communicates this, but users may expect a global search.
**Severity**: LOW (UX expectation mismatch, not a bug)

### L-4: InvitationCodes loading skeleton shows nothing

**File**: `src/pages/Settings/InvitationCodes.tsx` (lines 97-101)
**Description**: While `isLoading` is true and `codes.length === 0`, the component falls through to the table rendering path with an empty `<tbody>`. This shows the table headers with no rows rather than a loading indicator. Harmless but slightly jarring.
**Severity**: LOW (no loading state shown)

### L-5: Dashboard `formatBalance` used only for hero display; CurrencyDisplay used elsewhere

**File**: `src/pages/Dashboard.tsx` (lines 22-29)
**Description**: The Dashboard defines its own `formatBalance` that always uses 2 decimal places, while `CurrencyDisplay` is the canonical component used elsewhere. The hero balance is USDC-specific so 2 decimals is correct, but having two formatting paths could diverge over time.
**Severity**: LOW (consistency)

---

CRITICAL: 0 HIGH: 0 APPROVED: YES
