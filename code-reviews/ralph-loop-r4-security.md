# Security Review: Ralph Loop R4 -- Attacker-Focused Frontend Audit

**Reviewer**: GPT-5.2-class Security Reviewer
**Date**: 2026-02-27
**Scope**: Tauri v2 React frontend -- 12 files (pages, shared components, format utils)
**Context**: Desktop app, no HTTP endpoints, no cookies, no SSR. Rust backend via `invoke()`.
**Round**: 4 (progressive fixes applied in R1-R3)

---

## Verdict: APPROVED

**CRITICAL issues: 0**
**HIGH issues: 0**
**MEDIUM issues: 2**
**LOW issues: 3**

---

## Summary

Round 4 finds this frontend in strong shape. The codebase correctly applies React's built-in XSS protection throughout -- zero instances of `dangerouslySetInnerHTML`, `eval()`, `Function()`, or `innerHTML` manipulation exist in any reviewed file. All destructive/mutating operations (suspend agent, approve/deny requests, kill switch activation, code revocation) are gated behind two-step confirmation dialogs. Race conditions in async data fetching are guarded by `useRef` request counters in every page that loads data. Financial values are validated client-side before submission, and displayed through the `CurrencyDisplay` component with proper formatting. `setTimeout` handles in `Fund.tsx` and `MonoAddress.tsx` are correctly cleaned up on unmount.

The trust boundary is correctly placed: the Rust backend via `invoke()` handles all wallet operations, blockchain interactions, and policy enforcement. The frontend is a presentation and input validation layer, which is the correct architecture for a Tauri desktop app.

The remaining findings are minor and none represent exploitable attack vectors in this context.

---

## Detailed Findings

### MEDIUM-1: Silent error swallowing on InvitationCodes mutations (CARRIED FROM R3, UNFIXED)

**Files**:
- `src/pages/Settings/InvitationCodes.tsx:63-65` -- `handleGenerate` catch block is empty
- `src/pages/Settings/InvitationCodes.tsx:73-75` -- `handleRevoke` catch block is empty

**Impact**: If invitation code generation or revocation fails, the user receives no feedback. The dialog closes and `loadCodes()` re-fetches, making it appear the operation succeeded. A user could believe a code was revoked when the backend actually rejected the operation, leaving the code active for unauthorized agent registration.

This is the most security-relevant silent failure in the codebase because invitation codes gate agent registration -- an attacker with a code that was "revoked" (but actually wasn't) could still register a rogue agent.

**Severity**: MEDIUM -- the backend is the enforcement layer, and this is a UI feedback gap, not a bypass. But it misrepresents security-critical state to the operator.

**Recommendation**: Add error state and display for both `handleGenerate` and `handleRevoke`:
```tsx
const [generateError, setGenerateError] = useState<string | null>(null);

const handleGenerate = async () => {
  setGenerateError(null);
  try {
    await invoke<InvitationCode>("generate_invitation_code", { label });
    setLabel("");
    setIsDialogOpen(false);
    await loadCodes();
  } catch (err) {
    setGenerateError(err instanceof Error ? err.message : String(err));
  }
};
```

---

### MEDIUM-2: `parseFloat` normalization before `invoke()` can silently alter financial values (CARRIED FROM R3, UNFIXED)

**Files**:
- `src/pages/AgentDetail.tsx:120-126` -- `parseFloat(editPolicy.per_tx_max).toString()`
- `src/pages/Settings/GlobalPolicy.tsx:67-70` -- same pattern for caps
- `src/pages/Settings/Notifications.tsx:49-50` -- threshold normalization

**Impact**: The pattern `parseFloat(value).toString()` is used to "normalize" values before sending to the backend. This strips trailing zeros (`"1.00"` becomes `"1"`), and for edge-case inputs near the limits of float64 precision, could alter the value. Example: `"9999999999999999.99"` would lose the `.99` due to float64 limitations.

In practice, for typical USDC policy caps (sub-million, 2 decimal places), this is safe. But the normalization is unnecessary -- the backend's Rust decimal parser is more precise than JavaScript's `parseFloat`. Sending the raw validated string would be strictly better.

**Severity**: MEDIUM -- no real-world fund loss expected, but the architectural principle of not mutating financial values in the less-precise layer (JS) before sending to the more-precise layer (Rust) matters for a crypto application.

**Recommendation**: After validation confirms the string is a valid non-negative number, send the raw string to the backend without the `parseFloat().toString()` round-trip. If canonical formatting is desired, let the backend return the normalized form.

---

### LOW-1: `loadAgents` in Approvals.tsx silently swallows errors

**File**: `src/pages/Approvals.tsx:85-87`

**Impact**: If agent list fails to load, approval cards show raw agent IDs instead of names. This is a graceful degradation, not a security issue, but the user has no indication that agent names are unavailable due to an error vs. simply not being set.

**Severity**: LOW

---

### LOW-2: Client-side search in Transactions.tsx operates only on current page

**File**: `src/pages/Transactions.tsx:203-214`

**Impact**: The search filters only the 20 transactions on the current page, not all transactions. The UI says "Search transactions..." which could mislead users into thinking they've searched all transactions when they've only searched the current page. An operator looking for a specific suspicious transaction might miss it if it's on a different page.

**Severity**: LOW -- this is a UX/completeness issue. The UI does show "X of Y on this page" when search is active (line 406), which partially mitigates the confusion.

---

### LOW-3: `GlobalPolicy.loadPolicy` is not wrapped in useCallback and lacks race condition guard

**File**: `src/pages/Settings/GlobalPolicy.tsx:15-23`

**Impact**: Unlike other pages (AgentDetail, Approvals, Transactions) which use `useRef` request counters to prevent stale data from late-resolving promises, `GlobalPolicy.loadPolicy` is a plain function without such a guard. Similarly, `Notifications.loadPrefs` lacks this pattern. In practice, these pages don't have parameters that change (no route params, no filters), so a race condition is unlikely -- the function is only called on mount and on retry. But it's an inconsistency with the otherwise disciplined pattern used elsewhere.

**Severity**: LOW -- no practical exploit path since the load is triggered only on mount.

---

## Previously Fixed Issues (Verified Resolved)

The following issues from earlier rounds were verified as properly addressed:

1. **XSS via dangerouslySetInnerHTML** -- Confirmed: zero instances across all files. All dynamic content rendered via JSX text expressions.
2. **Race conditions in data fetching** -- Confirmed: `useRef` request counters in `AgentDetail`, `Approvals`, `Transactions`.
3. **Missing confirmation dialogs on destructive actions** -- Confirmed: suspend agent, approve/deny, kill switch activation, and code revocation all have two-step confirmation flows.
4. **Error feedback on mutations** -- Confirmed: suspend, save policy, resolve approval, kill switch, and save notifications all surface errors to the user.
5. **CurrencyDisplay for financial formatting** -- Confirmed: component exists and is used consistently for monetary values across all pages.
6. **Numeric validation on policy inputs** -- Confirmed: `validateField` in AgentDetail, validation loop in GlobalPolicy, threshold validation in Notifications.
7. **setTimeout cleanup on unmount** -- Confirmed: `Fund.tsx` and `MonoAddress.tsx` both clean up timers via `useEffect` return.
8. **Load error states with retry** -- Confirmed: GlobalPolicy and Notifications show error states with retry buttons.

---

## Architecture Assessment

The security architecture is sound for a Tauri desktop application:

- **Trust boundary**: All blockchain/wallet operations are in Rust. The frontend is untrusted input collection + display.
- **No web attack surface**: No cookies, no CORS, no SSR, no HTTP server. The Tauri IPC (`invoke()`) is the only communication channel.
- **Input validation**: Client-side validation exists for UX, but the backend must (and presumably does) re-validate all inputs.
- **No sensitive data in frontend state**: Wallet addresses are displayed but all signing/key operations are backend-only.

---

## Conclusion

This frontend is approved for its role as a presentation layer over a Rust backend. The two remaining MEDIUM findings are carried over from R3 and represent minor gaps in error feedback and an unnecessary precision-lossy normalization step -- neither is exploitable. The codebase demonstrates consistent, disciplined patterns for async safety, input validation, destructive action confirmation, and XSS prevention.
