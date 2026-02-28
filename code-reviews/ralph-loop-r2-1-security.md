# Ralph Loop Round 2 - Security / Attacker Review

**Date**: 2026-02-27
**Reviewer Perspective**: Security / Attacker
**Round**: 2
**Files Reviewed**: AgentDetail.tsx, Transactions.tsx, Approvals.tsx, Dashboard.tsx, Onboarding.tsx, Fund.tsx, Agents.tsx, Settings.tsx, Settings/GlobalPolicy.tsx, Settings/InvitationCodes.tsx, CurrencyDisplay.tsx, MonoAddress.tsx, FundStep.tsx, Sidebar.tsx, format.ts, Notifications.tsx

---

## CRITICAL

None found.

## HIGH

**H-1: Approvals page does not use CurrencyDisplay for financial amounts (Approvals.tsx:127-128)**

The Approvals page renders `data.amount` and `data.asset` directly as raw strings from parsed JSON payload (`{data.amount} {data.asset}`). Unlike the rest of the app which uses `CurrencyDisplay` or `formatCurrency` for consistent, locale-aware, asset-decimal-aware formatting, this renders the raw backend value. An agent could submit a transaction approval request with an amount like `"1000.000000000000000001"` or `"1e5"` and the user would see that raw string rather than a normalized financial display. This creates a material financial display inconsistency -- the user might approve a transaction showing `100000` while the rest of the app would show `100,000.00 USDC`. For a security-critical approval action, the displayed amount must use the same formatting pipeline as the rest of the app.

**File**: `/Users/dennisonbertram/Develop/apps/agent-neo-bank/src/pages/Approvals.tsx`, lines 127-128
```tsx
// Current - raw string display:
{data.amount} {data.asset}

// Should use CurrencyDisplay:
<CurrencyDisplay amount={data.amount} asset={data.asset} />
```

Similarly, `data.proposed_daily` and `data.proposed_monthly` on lines 145 and 149 display raw limit values without formatting.

**H-2: Notifications large_tx_threshold has no numeric validation (Notifications.tsx:108-117)**

The `large_tx_threshold` field accepts arbitrary string input and sends it directly to the backend via `invoke("update_notification_preferences", { prefs })` without any validation. A user could enter negative values, empty string, `NaN`-producing text, or extremely large/small numbers. Unlike GlobalPolicy which validates all numeric fields before save, Notifications skips validation entirely. If the backend uses this threshold for comparison against transaction amounts, a malformed value (e.g., `"-1"`, `"abc"`, `""`) could cause all transactions to trigger (or never trigger) large-tx notifications, effectively disabling a security monitoring feature.

**File**: `/Users/dennisonbertram/Develop/apps/agent-neo-bank/src/pages/Settings/Notifications.tsx`, lines 108-117, 22-31

## MEDIUM

**M-1: AgentDetail spending policy save does not validate hierarchical cap consistency (AgentDetail.tsx:96-134)**

The spending policy edit validates that each field is a non-negative number, but does not validate that `per_tx_max <= daily_cap <= weekly_cap <= monthly_cap`. A user could set `per_tx_max: 1000` and `daily_cap: 100`, creating a logically impossible policy. While the backend should enforce this, the frontend silently accepts contradictory limits, potentially creating a false sense of security if the backend also lacks this check.

**File**: `/Users/dennisonbertram/Develop/apps/agent-neo-bank/src/pages/AgentDetail.tsx`, lines 96-134

**M-2: GlobalPolicy save does not validate cap hierarchy (GlobalPolicy.tsx:35-71)**

Same issue as M-1 but for global policy: `daily_cap`, `weekly_cap`, `monthly_cap` are validated individually but not against each other. A daily cap higher than a monthly cap is logically wrong and could confuse enforcement.

**File**: `/Users/dennisonbertram/Develop/apps/agent-neo-bank/src/pages/Settings/GlobalPolicy.tsx`, lines 35-71

**M-3: GlobalPolicy save error is silently swallowed (GlobalPolicy.tsx:67-68)**

When `invoke("update_global_policy")` fails, the error is caught and ignored. The user sees the button return to "Save Caps" with no feedback that the save failed. The user may believe their security policy was updated when it was not. AgentDetail correctly shows `saveError` -- GlobalPolicy should do the same.

**File**: `/Users/dennisonbertram/Develop/apps/agent-neo-bank/src/pages/Settings/GlobalPolicy.tsx`, lines 67-68

**M-4: Kill switch toggle error is silently swallowed (GlobalPolicy.tsx:83-87, 98-100)**

Both `handleToggleKillSwitch` (deactivation path) and `handleConfirmKillSwitch` (activation path) silently swallow errors. If the kill switch activation fails, the user sees the dialog close and may believe the kill switch is active when it is not. For an emergency security feature, error feedback is essential.

**File**: `/Users/dennisonbertram/Develop/apps/agent-neo-bank/src/pages/Settings/GlobalPolicy.tsx`, lines 83-87, 98-100

**M-5: Notifications save error is silently swallowed (Notifications.tsx:27-28)**

Same pattern as M-3/M-4. If saving notification preferences fails, the user gets no feedback and may believe their monitoring settings are active when they are not.

**File**: `/Users/dennisonbertram/Develop/apps/agent-neo-bank/src/pages/Settings/Notifications.tsx`, lines 27-28

**M-6: Approvals resolveError persists across different approval items (Approvals.tsx:258)**

The `resolveError` display condition checks `processingId === null && confirmAction?.id === approval.id`, but after a failed resolve attempt, if the user opens the confirm dialog for a *different* approval, `resolveError` from the previous attempt is not cleared. The stale error could confuse the user. The `setConfirmAction` calls on lines 291 and 299 do not clear `resolveError`.

**File**: `/Users/dennisonbertram/Develop/apps/agent-neo-bank/src/pages/Approvals.tsx`, lines 258, 291, 299

## LOW

**L-1: InvitationCodes generate error is silently swallowed (InvitationCodes.tsx:63-64)**

If `generate_invitation_code` fails, the dialog closes and no feedback is shown. The user may believe a code was generated when it was not.

**File**: `/Users/dennisonbertram/Develop/apps/agent-neo-bank/src/pages/Settings/InvitationCodes.tsx`, lines 63-64

**L-2: Dashboard formatBalance uses parseFloat which loses precision for large numbers (Dashboard.tsx:22-29)**

`parseFloat` converts to IEEE 754 double, losing precision beyond ~15-16 significant digits. For very large balance values this could display an incorrect amount. `CurrencyDisplay` has the same limitation but at least formats consistently. This is low severity because wallet balances at 15+ significant digits are unlikely in practice.

**File**: `/Users/dennisonbertram/Develop/apps/agent-neo-bank/src/pages/Dashboard.tsx`, lines 22-29

**L-3: Agents list does not handle fetch error visually (Agents.tsx:17)**

If `list_agents` fails, the error is swallowed and the user sees "No agents yet" rather than an error state. This could mask connectivity issues.

**File**: `/Users/dennisonbertram/Develop/apps/agent-neo-bank/src/pages/Agents.tsx`, line 17

**L-4: Onboarding FundStep allows "Continue to Dashboard" without verifying funds received (FundStep.tsx:67-73)**

The button is enabled as long as an address is loaded, regardless of whether any funds have actually been deposited. The user can proceed to the dashboard with a zero-balance wallet. While this is a UX choice, it means the "Fund your wallet" step does not actually verify funding.

**File**: `/Users/dennisonbertram/Develop/apps/agent-neo-bank/src/components/onboarding/FundStep.tsx`, lines 67-73

---

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0     |
| HIGH     | 2     |
| MEDIUM   | 6     |
| LOW      | 4     |

CRITICAL: 0 HIGH: 2 APPROVED: NO

### Required fixes for approval:
1. **H-1**: Use `CurrencyDisplay` (or `formatCurrency`) in Approvals page for transaction amounts and proposed limits
2. **H-2**: Add numeric validation to `large_tx_threshold` before saving notification preferences (same pattern as GlobalPolicy validation)
