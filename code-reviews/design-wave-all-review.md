## CRITICAL

### 1) Onboarding removed backend integration (invoke calls not preserved)
**Files:** `src/pages/Onboarding.tsx`, `src/pages/Onboarding.test.tsx`  
**Problem:** The onboarding flow is currently stubbed:
- `handleEmailSubmit` and `handleOtpSubmit` do **not** call `invoke()` at all (they just advance steps and hardcode an address).
- Test file mocks `@tauri-apps/api/core.invoke`, but the component never calls it, so the test would still pass even if backend auth is totally broken.

This violates the stated requirement: **“all backend invoke() calls should be preserved.”** It also means onboarding cannot work in production.

**Fix:**
- Reintroduce the original `invoke("auth_login", { email })` and `invoke("auth_verify", { email, otp })` calls (or whatever the real commands are) and manage loading/error state for those steps.
- Update tests to assert `invoke` was called with the correct command/args and that failures render an error state.

---

## HIGH

### 2) Approvals can falsely show “All caught up!” on backend failure (dangerous UX)
**File:** `src/pages/Approvals.tsx`  
**Problem:** `loadApprovals()` swallows errors and leaves `approvals` as `[]`. The UI then shows the empty “All caught up!” state when `!isLoading && approvals.length === 0`.  
In a fintech approvals screen, that can cause a user to miss pending approvals if the backend errors.

**Fix:**
- Track an explicit `error` state (e.g. `approvalsError`) and render an error panel with “Retry”.
- Do not treat “error” as “empty”. Differentiate:
  - Loading
  - Error
  - Empty (true zero results)
  - Loaded results

**Test gap:** No test covers the error path (invoke rejecting) and verifying the UI does *not* show the “caught up” empty state.

---

### 3) AgentDetail “Rotate Token” is a dead control (no invoke, no handler)
**File:** `src/pages/AgentDetail.tsx`  
**Problem:** The “Rotate Token” button has no `onClick` and performs no backend call. If this existed previously, the backend integration was removed. Even if it’s new UI, it’s a security-relevant action being presented as available but not working.

**Fix:**
- Add an `onClick` calling the preserved backend command (e.g. `invoke("rotate_agent_token", { agentId: id })`) and show success/error feedback.
- Disable while in-flight; consider confirmation.

**Test gap:** No test asserts that rotate token triggers the expected `invoke()` call.

---

### 4) Transactions page can crash if `tx.description` is null/undefined
**File:** `src/pages/Transactions.tsx`  
**Problem:** Client-side search does:
```ts
tx.description.toLowerCase().includes(query)
```
and rendering does:
```tsx
{tx.description}
```
If backend returns `null`/missing descriptions (common), this can throw at runtime.

**Fix:**
- Guard everywhere:
```ts
const desc = (tx.description ?? "").toLowerCase();
```
- Consider also including recipient/agent name safely.

**Test gap:** No test covers transactions with `description: null` / missing.

---

## MEDIUM

### 5) Accessibility: multiple controls lack accessible names / semantics
**Files:** multiple (`MonoAddress.tsx`, `Transactions.tsx`, `Agents.tsx`, `Approvals.tsx`, `AgentDetail.tsx`, `Fund.tsx`)  
Key issues:
- **Icon-only buttons** without `aria-label`:
  - `MonoAddress` copy button (uses `title`, which is not a reliable accessible name)
  - Allowed-recipient remove “X” button in `AgentDetail`
- **Inputs/selects rely on placeholder only** (Agents search, Transactions search, status filter, agent filter). Placeholders are not labels.
- Tab-like UIs (“Pending/All”, “Buy/Deposit”, Agent status tabs) are implemented as plain buttons without `role="tablist"`, `role="tab"`, `aria-selected`, and optional arrow-key behavior.

**Fix:**
- Add `aria-label` to icon buttons, e.g.:
  - Copy: `aria-label={copied ? "Address copied" : "Copy address"}`
  - Remove recipient: `aria-label="Remove recipient from allowlist"`
- Add visually-hidden `<label className="sr-only">...</label>` for search and filter controls, or `aria-label`.
- If tabs are intended as tabs, implement proper tab semantics; otherwise keep as buttons but ensure clear labeling and focus styles.

---

### 6) Error handling is generally “silent”, often showing misleading empty states
**Files:** `Agents.tsx`, `Approvals.tsx`, `AgentDetail.tsx`, `Dashboard.tsx`, `Transactions.tsx`  
Patterns like `.catch(() => {})` or `catch { /* silently handle */ }` lead to:
- “No agents yet” shown when `list_agents` failed
- “Agent not found” shown when `get_agent` failed
- “All caught up!” shown when `list_approvals` failed (this one is HIGH as noted)

**Fix:** Add an error state + retry affordance on each page. At minimum: show “Couldn’t load…” and a Retry button calling the same `invoke()` function again.

---

### 7) UX consistency: border color usage frequently deviates from stated system border (#E8E5E0)
**Files:** multiple (`Agents.tsx`, `Approvals.tsx`, `Transactions.tsx`, `Dashboard.tsx`, etc.)  
You often use `#F0EDE8` for borders. Your design requirement states **borders = `#E8E5E0`**. If `#F0EDE8` is a deliberate “subtle border”, it should be documented as part of the design system; otherwise this is inconsistent.

**Fix:** Align border usage to a small set of tokens (prefer CSS variables or Tailwind theme tokens) and ensure the UI uses the specified values.

---

### 8) AgentDetail “Save” is async but triggered from non-async toggle without awaiting / error UI
**File:** `src/pages/AgentDetail.tsx`  
`toggleEdit()` calls `handleSaveLimits()` without awaiting and without try/catch in `handleSaveLimits`. Failures will silently keep UI inconsistent.

**Fix:** Make save explicit and async-safe:
- `toggleEdit` should be `async` and `await handleSaveLimits()`
- show saving state / disable Save
- show error message if update fails

---

## LOW

### 9) Fund “Deposit” tab copy button does nothing + hardcoded address
**File:** `src/pages/Fund.tsx`  
Not an `invoke()` regression per se (no backend calls here), but UX is misleading: “Copy” should copy; address should come from wallet state/backend.

**Fix:** Wire to clipboard (with try/catch + feedback) and source the address from the real wallet state.

---

### 10) Minor semantics: table headers missing `scope="col"`
**File:** `src/pages/Transactions.tsx`  
Not required but improves screen reader navigation.

---

## Test Coverage Review (meaningfulness)
Good:
- Many page-level tests validate loading/empty/populated states.
- Several tests assert correct `invoke` usage (Approvals resolve_approval, Transactions pagination/filter).

Gaps:
- **Onboarding tests don’t assert invoke** and wouldn’t catch the CRITICAL regression.
- No tests for **error UI** (invoke rejection) for Agents/Approvals/AgentDetail/Dashboard/Transactions.
- No accessibility tests: accessible names for icon-only buttons, labels for inputs/selects, keyboard/tab behavior.

---

CRITICAL: 1  
HIGH: 4  
APPROVED: NO
