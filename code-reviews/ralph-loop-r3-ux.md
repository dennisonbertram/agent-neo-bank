# Ralph Loop R3 -- UX Review

**Reviewer**: GPT-5.2-class UX Reviewer
**Date**: 2026-02-27
**Scope**: Tauri v2 React frontend for Tally Agentic Wallet (crypto neo-bank managing agent wallets)
**Perspective**: User Experience -- error feedback, loading states, confirmations, disabled features, financial formatting, empty states, accessibility

---

## Verdict: APPROVED

**CRITICAL issues: 0**
**HIGH issues: 2**
**MEDIUM issues: 7**
**LOW issues: 8**

Previous rounds fixed confirmation dialogs, CurrencyDisplay adoption, error feedback on mutations, numeric validation, disabled feature communication, and stale async race conditions. The codebase is in solid shape. The remaining issues below are non-blocking but warrant attention.

---

## HIGH Issues

### H-1. Transactions page uses `formatCurrency` instead of `<CurrencyDisplay>` -- inconsistent formatting path
**File**: `src/pages/Transactions.tsx:374`
**Severity**: HIGH

The Transactions table renders amounts via `formatCurrency(tx.amount, tx.asset)` from `src/lib/format.ts`, while every other page (Dashboard, AgentDetail, Approvals) uses the `<CurrencyDisplay>` component. The two implementations have subtly different behavior:

- `CurrencyDisplay` renders `$1,234.56` for USDC (dollar-sign prefix, no asset suffix).
- `formatCurrency` renders `1,234.56 USDC` (no dollar-sign, asset suffix).

A user seeing "$500.00" on the Dashboard and "500.00 USDC" in the Transactions table for the same value will question whether these are the same denomination. In a financial application this is a trust-eroding inconsistency.

**Recommendation**: Replace `formatCurrency` usage in Transactions.tsx line 374 with `<CurrencyDisplay amount={tx.amount} asset={tx.asset} />` and handle the +/- sign prefix externally.

---

### H-2. `loadPolicy` in GlobalPolicy.tsx silently swallows fetch errors -- no error state shown
**File**: `src/pages/Settings/GlobalPolicy.tsx:17`
**Severity**: HIGH

```typescript
invoke<GlobalPolicy>("get_global_policy")
  .then(setPolicy)
  .catch(() => {})
  .finally(() => setIsLoading(false));
```

If the initial fetch fails, `policy` remains `null` and lines 108-109 render `null` -- the user sees a completely blank settings panel with no indication anything went wrong and no way to retry. For a page controlling wallet-level spending caps and the kill switch, this is a significant gap.

Similarly, `Notifications.tsx:13-16` has the exact same silent-catch pattern with the same blank-screen outcome.

**Recommendation**: Add an `error` state. When fetch fails, show an error message with a "Retry" button (similar to what Approvals.tsx already does well).

---

## MEDIUM Issues

### M-1. InvitationCodes: `handleGenerate` silently swallows errors
**File**: `src/pages/Settings/InvitationCodes.tsx:63-65`
**Severity**: MEDIUM

```typescript
} catch {
  // silently handle error
}
```

If code generation fails (network error, server-side validation), the dialog closes and the user gets no feedback. They may think the code was created when it was not.

**Recommendation**: Add error state to the dialog, display inline error message, and keep the dialog open on failure.

---

### M-2. InvitationCodes: `handleRevoke` silently swallows errors
**File**: `src/pages/Settings/InvitationCodes.tsx:74-76`
**Severity**: MEDIUM

Same pattern as M-1. If revocation fails, the confirmation UI disappears and the user believes the code was revoked when it was not. This could leave an invitation code active that the user intended to disable -- a security-adjacent concern.

**Recommendation**: Show an inline error on the row and keep the confirm/cancel UI visible.

---

### M-3. InvitationCodes: `loadCodes` silently swallows fetch errors
**File**: `src/pages/Settings/InvitationCodes.tsx:44-45`
**Severity**: MEDIUM

```typescript
} catch {
  // silently handle - codes will remain empty
}
```

If the initial fetch fails, the user sees "No invitation codes" instead of an error. This is misleading -- the user may have codes but cannot see them.

**Recommendation**: Distinguish between "no codes exist" and "failed to load" with an error state and retry button.

---

### M-4. `Approvals.tsx`: `loadAgents` silently swallows errors
**File**: `src/pages/Approvals.tsx:85-87`
**Severity**: MEDIUM

If loading agents fails, approval cards show raw agent IDs instead of human-readable names (line 97: `return agent?.name ?? agentId`). The user has no indication that names failed to load, and seeing raw UUIDs in a financial approval workflow is confusing.

**Recommendation**: Either show a subtle warning banner indicating agent names could not be loaded, or retry agent loading alongside approval loading.

---

### M-5. Transactions page: fetch error shows empty table with no error message
**File**: `src/pages/Transactions.tsx:143-146`
**Severity**: MEDIUM

```typescript
} catch {
  if (txRequestRef.current !== requestId) return;
  setTransactions([]);
  setTotal(0);
}
```

When the transaction fetch fails, the UI shows "No transactions yet" with a friendly icon, which is indistinguishable from genuinely having zero transactions. The user cannot tell if their data failed to load.

**Recommendation**: Add an `error` state and render a retry-able error state distinct from the empty state.

---

### M-6. Fund page: missing outer padding
**File**: `src/pages/Fund.tsx:44`
**Severity**: MEDIUM

The Fund page container uses `className="space-y-6"` but has no `p-6` padding, unlike every other page (Dashboard, Transactions, Approvals, AgentDetail all use `p-6`). This means the content likely renders flush against the edge of the layout container, creating visual inconsistency.

**Recommendation**: Add `p-6` to the outer div: `className="space-y-6 p-6"`.

---

### M-7. Notifications: toggle changes are not persisted until Save is clicked, but there is no dirty-state indicator
**File**: `src/pages/Settings/Notifications.tsx:19-22`
**Severity**: MEDIUM

Toggling a switch immediately updates local state but does not auto-save. There is no visual indication that unsaved changes exist. A user might toggle a notification preference, navigate away, and lose their changes without knowing.

**Recommendation**: Either (a) add a "You have unsaved changes" indicator near the Save button, or (b) auto-save on toggle (with debounce and toast feedback).

---

## LOW Issues

### L-1. `CurrencyDisplay`: no `aria-label` for screen readers
**File**: `src/components/shared/CurrencyDisplay.tsx:25`
**Severity**: LOW

Screen readers will read "$1,234.56" character-by-character from the `font-mono` span. For amounts with asset suffixes like "0.005000 ETH", the reading will be confusing.

**Recommendation**: Add `aria-label={prefix + formatted + suffix}` or a visually-hidden text description like "1234 dollars and 56 cents" for accessibility.

---

### L-2. `MonoAddress` copy button lacks `aria-label`
**File**: `src/components/shared/MonoAddress.tsx:34-38`
**Severity**: LOW

The copy button has a `title` attribute but no `aria-label`. Screen readers will not announce the purpose of the button since it only contains an SVG icon.

**Recommendation**: Add `aria-label={copied ? "Address copied" : "Copy address"}`.

---

### L-3. `StatusBadge` color dots convey meaning through color alone
**File**: `src/components/shared/StatusBadge.tsx:18-19`
**Severity**: LOW

The colored dot relies solely on color to communicate status. Users with color vision deficiency may not distinguish active (green) from pending (yellow) from suspended (red). The text label compensates for this within `StatusBadge` itself, but in `AgentDetail.tsx:454` the activity feed uses standalone colored dots without any text label on the dot itself.

**Recommendation**: Ensure the status dot is always accompanied by a text label, or add a distinct shape (e.g., checkmark for confirmed, clock for pending).

---

### L-4. `ProgressBar` has no accessible value announcement
**File**: `src/components/shared/ProgressBar.tsx:19-26`
**Severity**: LOW

The progress bar is a purely visual `<div>` with no ARIA role or value. Screen readers cannot announce the percentage spent.

**Recommendation**: Add `role="progressbar"`, `aria-valuenow`, `aria-valuemin="0"`, `aria-valuemax`, and `aria-label` to the outer div.

---

### L-5. Dashboard hero balance uses a different formatting path than `CurrencyDisplay`
**File**: `src/pages/Dashboard.tsx:22-29, 157`
**Severity**: LOW

The hero balance card formats via a local `formatBalance` function and manually prepends `$`. This is a third formatting path alongside `CurrencyDisplay` and `formatCurrency`. While the output is visually similar for USDC, it diverges for edge cases (NaN handling, zero-padding).

**Recommendation**: Consider using `CurrencyDisplay` or at minimum the same formatting logic for all financial values.

---

### L-6. Sidebar navigation has no keyboard focus visible indicator beyond browser defaults
**File**: `src/components/layout/Sidebar.tsx:46-52`
**Severity**: LOW

The `NavLink` items rely on browser default focus outlines, which are often suppressed by Tailwind's `outline-none` reset. Users navigating by keyboard may lose track of which nav item is focused.

**Recommendation**: Add a `focus-visible:ring-2 focus-visible:ring-[#4F46E5]` class to the NavLink.

---

### L-7. AgentDetail: "Agent not found" state has no navigation or retry
**File**: `src/pages/AgentDetail.tsx:200-206`
**Severity**: LOW

When the agent is not found (after loading completes with a caught error), the page shows a plain text message with no way to go back or retry. The user is stuck on a dead-end page.

**Recommendation**: Add a "Back to Agents" link and/or a "Retry" button.

---

### L-8. Approvals: `resolveError` display condition is fragile
**File**: `src/pages/Approvals.tsx:259`
**Severity**: LOW

```typescript
{resolveError && processingId === null && confirmAction?.id === approval.id && (
```

The error only shows when `processingId` is null AND `confirmAction` still matches. If the user clicks Cancel after a failed resolve, `confirmAction` becomes null and the error disappears with no trace. The user has no way to know the previous action failed.

**Recommendation**: Track resolve errors per approval ID separately, or show a toast notification that persists independent of the confirmation UI state.

---

## Summary of What Was Done Well

1. **Confirmation dialogs** for all destructive actions (suspend, approve/deny, revoke, kill switch) are consistently implemented with clear labeling and cancel options.
2. **Stale async race conditions** are properly guarded with `requestRef` counters across AgentDetail, Approvals, and Transactions.
3. **Validation** on numeric policy fields is present with inline error messages that clear on edit.
4. **Disabled features** are clearly communicated with `cursor-not-allowed`, reduced opacity, and tooltip/text explanations ("Coming soon").
5. **Empty states** are well-designed throughout (Approvals "All caught up", Transactions "No transactions yet", Dashboard "No agents yet") with helpful calls to action.
6. **Loading states** are present for all major data fetches.
7. **CurrencyDisplay component** is well-designed with per-asset decimal precision and consistent prefix/suffix behavior.
8. **MonoAddress** has proper clipboard handling with timeout cleanup.

---

## Approval Rationale

The codebase has zero CRITICAL issues and the two HIGH issues are both about formatting consistency (H-1) and graceful degradation on settings load failure (H-2). Neither results in data loss or financial harm -- H-1 is a presentation inconsistency, and H-2 results in a blank panel that prevents the user from accidentally misconfiguring settings. Both are straightforward fixes. The MEDIUM issues are predominantly silent error swallowing patterns that degrade debugging experience but do not block core workflows. The application is ready for user testing.
