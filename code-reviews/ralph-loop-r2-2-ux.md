# Ralph Loop Round 2 - UX / Malicious User Review

**Reviewer perspective**: Malicious User / UX Attacker
**Date**: 2026-02-27
**Round**: 2

---

## CRITICAL

### C-1: Approvals page displays raw amount/asset without CurrencyDisplay formatting

**File**: `src/pages/Approvals.tsx`, lines 127-129

The `renderPayloadDetails` function for transaction approvals renders `data.amount` and `data.asset` as raw strings directly:

```tsx
<p className="text-xl font-semibold text-[#1A1A1A]">
  {data.amount} {data.asset}
</p>
```

This bypasses `CurrencyDisplay` and `formatCurrency` entirely. A malicious agent could submit a transaction approval with `amount: "0.001000000000000001"` and the user sees the raw unformatted string. More critically, the amount is parsed from a JSON payload string (`parsePayload`) with no validation -- a crafted payload like `{"amount": "1000", "asset": "<script>alert(1)</script>"}` would render the script tag name literally (React escapes, so no XSS, but the user sees garbage). The real issue: the user makes approve/deny decisions on financial amounts that are not formatted consistently with the rest of the app, which could cause them to approve the wrong amount. For example, `"100"` USDC vs `"100"` ETH look identical here but are vastly different values (~$250k vs $100).

**Fix**: Use `CurrencyDisplay` (or `formatCurrency`) with the parsed asset, and validate that `data.asset` is one of the known assets (USDC, ETH, WETH) before displaying.

### C-2: AgentDetail activity feed omits asset, always shows amounts as USD

**File**: `src/pages/AgentDetail.tsx`, line 450

```tsx
<CurrencyDisplay amount={tx.amount} />
```

The `Transaction` type has an `asset` field, but `CurrencyDisplay` is called without passing `asset`. Since `CurrencyDisplay` defaults to USDC formatting (2 decimals, `$` prefix) when no asset is provided, an ETH transaction of `1.5` would display as `$1.50` instead of `1.500000 ETH`. The user sees materially wrong financial data -- an ETH transaction worth ~$3,750 would appear as "$1.50".

**Fix**: Pass `asset={tx.asset}` to `CurrencyDisplay`:
```tsx
<CurrencyDisplay amount={tx.amount} asset={tx.asset} />
```

---

## HIGH

### H-1: AgentDetail spending limit progress bars omit asset context

**File**: `src/pages/AgentDetail.tsx`, lines 407

```tsx
<CurrencyDisplay amount={String(row.spent)} /> / <CurrencyDisplay amount={String(row.limit)} />
```

All spending policy amounts are displayed using `CurrencyDisplay` without an `asset` prop, so they always render with a `$` prefix and 2 decimal places. While spending policies are likely USDC-denominated today, there is no label or context confirming this. If the backend ever supports multi-asset policies, these would silently display incorrect formatting. More immediately, a user comparing this section to the activity feed (which also lacks asset) cannot distinguish asset types.

**Severity reasoning**: HIGH rather than CRITICAL because spending policies are currently single-asset (USDC), but it's misleading since there's no explicit "USDC" label and it breaks if multi-asset support is added.

**Fix**: Either pass `asset="USDC"` explicitly, or add a "(USDC)" label to the Spending Limits section header.

### H-2: GlobalPolicy save silently swallows errors -- user thinks caps are saved when they are not

**File**: `src/pages/Settings/GlobalPolicy.tsx`, lines 67-68

```tsx
} catch {
  // handle error
}
```

When `update_global_policy` fails (network error, backend validation rejection, etc.), the error is silently swallowed. The `isSaving` spinner stops, and the user sees the form return to its normal state with no error message. The user reasonably believes their global spending caps were saved successfully, but they were not. This is a significant safety gap: the user may lower caps to restrict a rogue agent, believe the change took effect, and walk away -- while the old (higher) caps remain active.

**Fix**: Add a `saveError` state (like AgentDetail already has) and display an error banner when the save fails.

### H-3: GlobalPolicy kill switch toggle silently swallows errors

**File**: `src/pages/Settings/GlobalPolicy.tsx`, lines 85-86 and 98-99

Both `handleToggleKillSwitch` (deactivation) and `handleConfirmKillSwitch` (activation) silently swallow errors. If the kill switch activation fails, the confirmation dialog closes (line 96: `setShowKillConfirm(false)` is in the try block so it won't execute on error, but `loadPolicy()` also fails silently) and the user may not realize the switch didn't actually engage. For an emergency control, silent failure is a HIGH severity issue.

**Fix**: Display an error message in the kill switch section when the invoke call fails. Keep the confirmation dialog open on error so the user can retry.

### H-4: Suspend agent error silently swallowed -- user thinks agent is suspended when it may not be

**File**: `src/pages/AgentDetail.tsx`, lines 82-83

```tsx
} catch {
  // Error suspending agent
}
```

When `suspend_agent` fails, the confirmation dialog closes (via `setShowSuspendConfirm(false)` on line 80 -- this is in the try block so it won't execute on error), and `isSuspending` returns to false. However there is no error feedback to the user. The confirmation UI disappears (because `isSuspending` is false and the button re-enables), but the agent remains active. The user needs to notice that the status badge didn't change, which is easy to miss.

**Fix**: Add a `suspendError` state and display an error message near the suspend button when the operation fails. Keep `showSuspendConfirm` true on error so the user can retry.

---

## MEDIUM

### M-1: Approvals page `resolveError` display condition is fragile

**File**: `src/pages/Approvals.tsx`, lines 258

```tsx
{resolveError && processingId === null && confirmAction?.id === approval.id && (
```

The error is only visible when `processingId === null` AND `confirmAction` still points to this approval. If the user clicks "Cancel" after a failed resolve (line 280: `setConfirmAction(null)`), the error disappears and cannot be seen again. The user may not realize the approve/deny action failed. Consider persisting the error independently of the confirm action state, or clearing it only on explicit dismiss.

### M-2: Approvals transaction amounts show no currency symbol or formatting

**File**: `src/pages/Approvals.tsx`, line 128

Even setting aside the CRITICAL issue (C-1) about using raw strings, the `limit_increase` payload (lines 144-153) also displays `proposed_daily` and `proposed_monthly` as raw strings with no currency formatting or asset indicator. The user sees "1000" with no indication whether that's $1,000 USDC or 1,000 ETH.

### M-3: Dashboard `formatBalance` duplicates CurrencyDisplay logic

**File**: `src/pages/Dashboard.tsx`, lines 22-28

`formatBalance` is a local function that formats the hero balance with `toLocaleString`. This duplicates and deviates from `CurrencyDisplay` and `formatCurrency`. The hero balance always shows 2 decimal places regardless of asset. If the primary balance ever changes from USDC, the formatting will be wrong.

### M-4: Agents page silently swallows list_agents error

**File**: `src/pages/Agents.tsx`, line 17

```tsx
.catch(() => {})
```

If `list_agents` fails, the page shows the empty state ("No agents yet") rather than an error state. The user may think they have no agents when actually the request failed. Should display an error state with a retry option.

### M-5: Onboarding wallet address placeholder "0x..." displayed to user

**File**: `src/pages/Onboarding.tsx`, line 22 and `src/components/onboarding/FundStep.tsx`, line 49

The initial `walletAddress` state is `"0x..."` and this value is rendered directly in the FundStep via `<code>{address}</code>`. While the copy button and continue button are disabled when the address is `"0x..."`, the user still sees the literal string "0x..." in the address field, which could be confusing or appear broken.

---

## LOW

### L-1: No loading indicator on Approvals page initial load

**File**: `src/pages/Approvals.tsx`

When `isLoading` is true, the page renders the approvals map over an empty array (no items). There's no explicit loading skeleton or spinner. The "All caught up!" empty state flashes briefly before data loads.

### L-2: InvitationCodes generate dialog has no error feedback

**File**: `src/pages/Settings/InvitationCodes.tsx`, lines 63-64

`handleGenerate` silently catches errors. If code generation fails, the dialog closes and the user sees no feedback.

### L-3: Transaction table search is client-side only on current page

**File**: `src/pages/Transactions.tsx`, lines 202-213

The search filter only applies to the currently loaded page of transactions (up to 20). The search input gives no indication that it only searches the current page, potentially causing users to miss transactions on other pages.

### L-4: AgentDetail Cancel edit doesn't exist

**File**: `src/pages/AgentDetail.tsx`

When editing spending limits, there's no "Cancel" button to discard changes. The only options are "Save" or navigating away. This is a minor UX gap -- the user may want to cancel an edit without saving.

---

CRITICAL: 2 HIGH: 4 APPROVED: NO
