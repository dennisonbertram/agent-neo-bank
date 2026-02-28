## CRITICAL

### 1) `OtpInput` can corrupt the OTP by shifting digits (data integrity bug)
**File:** `src/components/ui/OtpInput.tsx`

`newValue` is built via:

```ts
const newValue = newDigits.join('').replace(/\s/g, '')
```

Because `join('')` collapses empty slots, the component cannot represent “a digit at index 3 while index 1 is empty”. If a user clicks a later box and types, the value becomes a shorter string and on the next render digits “shift left” (e.g., typing into box 3 makes it appear in box 2 after rerender). This can cause the wrong OTP to be submitted.

**Repro:** click 4th input first, type `7` → component emits `"7"`; rerender puts `7` into the 1st box, not the 4th.

**Fix options (pick one):**
- Enforce sequential entry: on focus/click of index `i`, redirect focus to `firstEmptyIndex` (and prevent editing out-of-order).
- Or store OTP internally as an array (source of truth), and only emit a string when complete; don’t try to round-trip positional state through a plain string.
- Or use a placeholder representation (e.g. value stored as fixed-length with sentinel chars) so positions don’t collapse—then strip sentinels only when calling `onComplete`.

---

## HIGH

### 2) Auth routing can strand authenticated users on onboarding + breaks deep links
**File:** `src/App.tsx`

`ProtectedRoute` immediately redirects when `isAuthenticated` is initially `false`:

```tsx
if (!isAuthenticated) return <Navigate to="/onboarding" replace />
```

But `checkAuthStatus()` runs async after mount. Users who are actually authenticated will still get redirected to `/onboarding` before the status check completes, and nothing in this file navigates them back to where they intended to go.

**Impact:**
- Deep linking to `/home`, `/agents/:id`, etc. can bounce to onboarding even when already authed.
- Returning users may see onboarding unexpectedly.

**Suggested fix:**
Introduce an auth “unknown/loading” state (e.g. `authStatus: 'unknown'|'authed'|'unauthed'`), render a splash/blank while `unknown`, and only redirect once status is resolved.

---

### 3) Default route always sends users to onboarding even when authenticated
**File:** `src/App.tsx`

```tsx
<Route path="/" element={<Navigate to="/onboarding" replace />} />
```

If a user opens the app at `/`, they will always land on onboarding, even if authenticated.

**Suggested fix:**
Replace with an auth-aware redirect component:
- if authed → `/home`
- else → `/onboarding`

---

## MEDIUM

### 4) `flowId` can become stale when auth becomes unauthenticated
**File:** `src/stores/authStore.ts`

On unauth/error, you reset `isAuthenticated` + `email`, but **not** `flowId`:

```ts
set({ isAuthenticated: false, email: null })
```

If other parts of the auth flow rely on `flowId`, this can cause mismatched OTP verification attempts or confusing UI state.

**Fix:** also clear `flowId` on unauth/error in `checkAuthStatus`.

---

### 5) Non‑Tauri “visual testing” logic can preserve stale auth and unintentionally bypass protection
**File:** `src/stores/authStore.ts`

In browser (non-Tauri), `checkAuthStatus` returns early without changing state:

```ts
if (typeof window !== 'undefined' && !(window as ...).__TAURI_INTERNALS__) return
```

If some UI path sets `isAuthenticated = true` in a browser session, later checks won’t reset it, potentially allowing access to `ProtectedRoute` pages in a web build/dev environment unintentionally.

**Fix:** gate this behavior behind an explicit dev flag (e.g. `import.meta.env.DEV && ALLOW_BROWSER_AUTH_MOCK`) and otherwise set unauthenticated deterministically on web.

---

### 6) Backspace behavior is inconsistent with user expectations
**File:** `src/components/ui/OtpInput.tsx`

You only move focus back when the current box is empty:

```ts
if (e.key === 'Backspace' && !digits[index] && index > 0) { ... }
```

If a digit exists and user hits backspace, it clears but focus stays; many OTP UIs move to previous after clearing, and/or handle “backspace on filled” in `onKeyDown` to avoid a laggy feeling.

---

### 7) No paste / autofill handling (common OTP flow)
**File:** `src/components/ui/OtpInput.tsx`

There’s no `onPaste` handler to distribute digits across inputs. This is a major usability miss for OTP flows (users frequently paste codes).

**Fix:** implement `onPaste` on the container or first input, parse digits, fill forward, and call `onComplete` when filled.

---

## LOW

### 8) `setExpanded(!expanded)` can use a functional update to avoid stale closures
**File:** `src/pages/InstallSkill.tsx`

```tsx
onClick={() => setExpanded(!expanded)}
```

Prefer:

```tsx
setExpanded((v) => !v)
```

Minor, but more robust under rapid clicks / concurrent rendering.

---

### 9) Bottom nav safe-area / gesture area overlap risk
**File:** `src/components/layout/BottomNav.tsx`

Hardcoded bottom padding (`pb-5`) + absolute positioning may overlap iOS home indicator / Android gesture bar.

**Fix:** incorporate `env(safe-area-inset-bottom)` (Tailwind arbitrary value or CSS var) into padding.

---

### 10) React 18 StrictMode double-invokes `useEffect` in dev → duplicate `checkAuthStatus` calls
**File:** `src/App.tsx`

Not a production bug, but can cause confusing double calls/flicker in dev. Consider idempotent status checks or a “didRun” guard if this is noisy (especially if Tauri bridge logs/errors on duplicate calls).

---

## Summary
CRITICAL: 1  
HIGH: 2  
MEDIUM: 4  
APPROVED: NO
