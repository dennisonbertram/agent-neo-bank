# Security Review: Ralph Loop R3 -- Attacker-Focused Frontend Audit

**Reviewer**: GPT-5.2-class Security Reviewer
**Date**: 2026-02-27
**Scope**: Tauri v2 React frontend -- 13 files
**Context**: Desktop app, no HTTP endpoints, no cookies, no SSR. Rust backend via `invoke()`.

---

## Verdict: APPROVED

**CRITICAL issues: 0**
**HIGH issues: 0**
**MEDIUM issues: 3**
**LOW issues: 5**

---

## Summary

This frontend is well-structured for a Tauri desktop application. There are zero XSS vectors -- React's JSX escaping handles all user-supplied content correctly, and no `dangerouslySetInnerHTML` is used anywhere in the reviewed files. All mutating operations (suspend, approve/deny, kill switch, save policy) have confirmation dialogs. Race conditions in data fetching are mitigated via `requestRef` counters. Financial values are validated before submission. The Rust backend is the true trust boundary, and the frontend correctly delegates all sensitive operations to it.

No genuinely exploitable CRITICAL or HIGH issues were found in this Tauri desktop context.

---

## Detailed Findings

### MEDIUM-1: Silent error swallowing hides failures on critical operations

**Files**:
- `src/pages/Settings/InvitationCodes.tsx:63-65` -- `handleGenerate` catch block is empty
- `src/pages/Settings/InvitationCodes.tsx:73-75` -- `handleRevoke` catch block is empty
- `src/pages/Approvals.tsx:85-87` -- `loadAgents` catch silently swallows
- `src/pages/Settings/GlobalPolicy.tsx:17` -- `loadPolicy` catch is `() => {}`
- `src/pages/Settings/Notifications.tsx:15` -- initial load catch is `() => {}`

**Impact**: If invitation code generation fails (e.g., backend rejects a duplicate label, or a revocation fails due to a race), the user gets zero feedback. The dialog closes and `loadCodes()` runs, making it appear the operation succeeded when it did not. For InvitationCodes specifically, a user could believe they revoked a code when revocation actually failed, leaving the code active for unauthorized agent registration.

**Severity**: MEDIUM -- no financial loss, but security-relevant state (invitation code lifecycle) could be misrepresented to the user.

**Recommendation**: Add `setSaveError(...)` or equivalent toast for `handleGenerate` and `handleRevoke`. For data-loading failures (`loadPolicy`, `loadAgents`, notification prefs), at minimum set an error state so the user knows data may be stale.

---

### MEDIUM-2: `parseFloat` used for financial policy values -- precision loss risk

**Files**:
- `src/pages/AgentDetail.tsx:122-125` -- `parseFloat(editPolicy.per_tx_max).toString()`
- `src/pages/Settings/GlobalPolicy.tsx:63-66` -- same pattern for caps
- `src/pages/Settings/Notifications.tsx:41` -- threshold normalization
- `src/components/shared/CurrencyDisplay.tsx:13` -- `parseFloat(amount)`
- `src/pages/Dashboard.tsx:23` -- `formatBalance` uses `parseFloat`

**Impact**: `parseFloat` uses IEEE 754 doubles, which lose precision beyond ~15-16 significant digits. For typical USDC amounts (2 decimal places, sub-million), this is safe in practice. However, if a user enters a value like `0.1` it becomes `0.1` (representable), but `0.30000000000000004` could appear for computed values. The Rust backend should be the source of truth for financial math, so the real risk is cosmetic display errors.

**Severity**: MEDIUM -- no actual fund loss (backend enforces), but displayed values could diverge from backend values, causing user confusion about limits. In a financial UI, displayed values must match enforced values exactly.

**Recommendation**: For the normalization step before `invoke()`, consider sending the raw string to the backend and letting Rust's decimal parsing handle precision. For display, the `CurrencyDisplay` component correctly uses `toLocaleString` which rounds, so display is acceptable.

---

### MEDIUM-3: Client-side `updated_at` timestamp in GlobalPolicy

**File**: `src/pages/Settings/GlobalPolicy.tsx:67`

```typescript
updated_at: Math.floor(Date.now() / 1000),
```

**Impact**: The frontend sets `updated_at` using the client's local clock. If the user's system clock is incorrect, this timestamp will be wrong. More importantly, the frontend should not be the source of truth for audit timestamps. If the Rust backend trusts this value for ordering or conflict resolution, a manipulated timestamp could cause policy updates to be applied out of order.

**Severity**: MEDIUM -- depends on whether the backend uses this timestamp for anything beyond display. If the backend overwrites it server-side, this is LOW. If not, it is a data integrity issue.

**Recommendation**: Remove the `updated_at` from the frontend payload. Let the Rust backend set this timestamp authoritatively.

---

### LOW-1: No upper bound validation on spending policy values

**Files**:
- `src/pages/AgentDetail.tsx:91-96` -- validates `>= 0` but no upper bound
- `src/pages/Settings/GlobalPolicy.tsx:42-49` -- same

**Impact**: A user could set a per-transaction limit to `999999999999999` or similar. This is not exploitable in the desktop context (the user is the owner), but it defeats the purpose of having spending limits. The backend should enforce sane upper bounds.

**Severity**: LOW -- the user is the one setting their own limits.

---

### LOW-2: No logical consistency validation between cap tiers

**File**: `src/pages/AgentDetail.tsx:103-114`

**Impact**: A user could set `daily_cap = 1000` but `weekly_cap = 100`, which is logically inconsistent (daily cap exceeds weekly cap). The frontend does not warn about this. Whether the backend enforces `daily <= weekly <= monthly` is unknown from this review.

**Severity**: LOW -- user self-harm only, no external attacker vector.

---

### LOW-3: Stale data window during concurrent approval resolution

**File**: `src/pages/Approvals.tsx:100-117`

**Impact**: If two browser windows (or rapid clicks) both attempt to resolve the same approval, the second call will fail at the backend. The error is properly surfaced via `setResolveError`. However, the `processingId` guard only prevents the same button from being double-clicked -- it does not prevent a different approval from being resolved while one is in-flight (line 293: `disabled={processingId !== null}` correctly gates this). This is actually well-handled.

**Severity**: LOW -- the backend is the final arbiter; the UI correctly disables all action buttons while one is processing.

---

### LOW-4: Onboarding wallet address initialized with displayable placeholder

**File**: `src/pages/Onboarding.tsx:21`

```typescript
const [walletAddress, setWalletAddress] = useState("0x...");
```

**Impact**: If `auth_status` fails to return an address (lines 42-43, 65-66 catch silently), the `FundStep` component receives `"0x..."` as a real address. A user could copy this placeholder thinking it is their wallet address and send funds to it.

**Severity**: LOW -- `"0x..."` is not a valid Ethereum address, so no funds would actually be lost (the transaction would fail). But it is a poor UX signal. Initialize to `""` or `null` and conditionally render.

---

### LOW-5: Clipboard API failure silently ignored

**Files**:
- `src/components/shared/MonoAddress.tsx:26-28`
- `src/pages/Fund.tsx:38-40`

**Impact**: If `navigator.clipboard.writeText` fails, the user gets no feedback. The button stays in the "Copy" state, but the user might assume it worked. In Tauri's webview context, clipboard access is generally available, so this is unlikely to trigger.

**Severity**: LOW -- Tauri webview generally supports clipboard; failure is edge-case only.

---

## What Was Done Well

1. **No XSS vectors**: All dynamic content is rendered via JSX text interpolation. No `dangerouslySetInnerHTML`, no `eval()`, no `innerHTML`. React's automatic escaping handles everything correctly.

2. **Race condition mitigation**: `AgentDetail`, `Approvals`, and `Transactions` all use `requestRef` counters to prevent stale data from overwriting fresh data. This is a solid pattern.

3. **Confirmation dialogs for destructive actions**: Suspend agent, kill switch activation, approval resolution, and invitation code revocation all require a two-step confirmation.

4. **Input validation before invoke**: Spending policy fields are validated for NaN and negative values before being sent to the backend. The validation clears per-field on edit.

5. **State reset on route parameter change**: `AgentDetail` resets all state when `id` changes (line 30-39), preventing stale agent data from flashing.

6. **No sensitive data in URLs**: Wallet addresses and agent IDs use path parameters (not query strings). No credentials or tokens are passed through the URL.

7. **Proper loading states**: All pages show loading indicators and handle the null/empty data case gracefully.

---

## Non-Issues (Correctly Not Flagged)

- **No CSRF risk**: This is a Tauri app, not a web server. There are no cookies or session tokens to steal.
- **No SSR injection**: No server-side rendering. All rendering is client-side in a sandboxed webview.
- **No open redirect**: All `Link` and `NavLink` components use hardcoded relative paths.
- **`JSON.parse` in Approvals payload**: The parsed data is only used for display via JSX (not `dangerouslySetInnerHTML`), so malformed JSON payloads cannot cause XSS.
- **Route parameter `id` from `useParams`**: Passed directly to `invoke()` which forwards to the Rust backend. The backend must validate this, but the frontend correctly does not try to use it in any dangerous way (no URL construction, no DOM insertion as HTML).
