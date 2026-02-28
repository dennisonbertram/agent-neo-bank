## CRITICAL

### 1) Client-side-only “auth” is a bypassable UI flag (route protection is not a security boundary)
**Files:** `src/App.tsx`, `src/stores/authStore.ts`

**What’s happening**
- `ProtectedRoute` gates pages purely on `useAuthStore().isAuthenticated`.
- `isAuthenticated` is a mutable boolean in the renderer process (JS). Any JS execution in the renderer (devtools, XSS elsewhere, malicious dependency, compromised webview content) can flip it and immediately unlock all “protected” routes.

**Exploit sketch**
- From any JS execution context in the renderer:
  - set the Zustand state to authenticated (or call `setAuthenticated()` if reachable)
  - navigate to `/add-funds`, `/agents/:id`, `/settings`, etc.
- If those pages trigger privileged Tauri commands (wallet actions, transfers, secrets), you’ve effectively turned a UI flag into an authorization mechanism.

**Why this is critical**
- In Tauri/electron-style apps, the renderer must be treated as untrusted. If backend commands rely on “UI says you’re authed”, this becomes a direct authorization bypass.

**Fix**
- Treat `ProtectedRoute` as UX only; enforce authorization in the privileged backend layer (Tauri commands) on every sensitive operation.
- Use a real session/token model where the backend validates a token/handle that the renderer cannot forge (or at minimum, backend maintains the session and rejects unauthenticated commands regardless of UI route).

---

## HIGH

### 2) Auth “fail open” in non‑Tauri contexts (state is preserved on auth failure)
**File:** `src/stores/authStore.ts`

**What’s happening**
- In both the “not authenticated” path and the `catch`, the code does this:

```ts
if (typeof window !== 'undefined' && !(window as ...).__TAURI_INTERNALS__) return
```

Meaning: **if it thinks it’s “browser/non‑Tauri”, it returns without clearing auth state**.

**Exploitation / unexpected behavior**
- Any situation that results in `isAuthenticated` being `true` (devtools tampering, future persistence, a bug elsewhere, hot-reload state carryover, etc.) will remain true even if `tauriApi.auth.status()` fails or returns unauthenticated—*as long as the environment is (or is made to look) non‑Tauri*.
- This also creates a brittle trust dependency on `window.__TAURI_INTERNALS__` for correctness.

**Fix**
- Never “keep current auth state” on failed status checks in production builds.
- Gate this visual-testing behavior behind an explicit dev flag (e.g. `import.meta.env.DEV`) and default to clearing auth on any error/unauth result.

---

### 3) `window.__TAURI_INTERNALS__` is a spoofable switch that can freeze an authenticated UI state
**File:** `src/stores/authStore.ts`

**What’s happening**
- The decision to clear auth on failure depends on a writable global (`window.__TAURI_INTERNALS__`).

**Exploit sketch**
- If an attacker gains JS execution in the renderer, they can do:
  - `delete window.__TAURI_INTERNALS__` (or set it falsy)
  - trigger `checkAuthStatus()`
- Now the function may “return early” in paths where it would otherwise clear auth, preserving whatever auth state the attacker wants.

**Fix**
- Don’t use a writable global as an environment/auth correctness control.
- Prefer a compile-time check (`import.meta.env`) or a hardened runtime capability check that can’t be trivially toggled from page JS.

---

## MEDIUM

### 4) Logout is purely local; no backend/session invalidation shown
**File:** `src/stores/authStore.ts`

**What’s happening**
- `logout()` only clears Zustand state:
```ts
logout: () => set({ isAuthenticated: false, email: null, flowId: null }),
```
- If a backend session exists, it may remain valid.

**Impact**
- Users can believe they logged out while privileged backend operations remain authorized (depending on backend design).
- In shared-device scenarios this can be meaningful.

**Fix**
- Add a backend logout/invalidate call (e.g., `await tauriApi.auth.logout()`), then clear local state, and handle failure modes safely.

---

### 5) No “auth-check in progress” state → redirect race/UX confusion with security side-effects
**File:** `src/App.tsx`

**What’s happening**
- On first load, `isAuthenticated` starts `false`, so protected routes immediately redirect to `/onboarding` before `checkAuthStatus()` resolves.

**Why it can matter**
- If onboarding screens can trigger side effects (installation steps, wallet connection prompts, etc.), a legitimate authenticated user can be forced through flows unintentionally.
- This is also a common source of “confused deputy” behavior when side-effectful setup routes are reachable due to transient auth state.

**Fix**
- Add `authChecked/authLoading` state; render a loading/splash until `checkAuthStatus()` completes at least once.

---

## LOW

### 6) OTP component can be coerced into inconsistent state / unexpected completion
**File:** `src/components/ui/OtpInput.tsx`

**What’s happening**
- `digits` is derived from `value` prop, and handlers close over `digits`. With rapid input or parent-side transformations, you can end up applying edits to a stale `digits` snapshot.
- `onComplete(newValue)` triggers when `newValue.length === length`, but `newValue` is only whitespace-stripped (`replace(/\s/g,'')`) and not otherwise normalized.

**Impact**
- Mostly correctness/robustness; could become security-relevant if `onComplete` triggers a sensitive action (submitting OTP) and the parent can inject non-digit characters into `value`.

**Fix**
- Build `newDigits` from the latest `value` inside the handler (or use functional updates upstream).
- Normalize OTP to digits only before calling `onComplete`.

---

### 7) Navigation “Cancel” uses `navigate(-1)` (history-dependent)
**File:** `src/pages/InstallSkill.tsx`

**Impact**
- In embedded contexts / deep links, “Cancel” may navigate to an unexpected prior location. Typically low risk in SPA routing, but it can cause surprising transitions if history was manipulated.

**Fix**
- Prefer explicit safe routes for cancellation (e.g., `navigate('/onboarding')` or a known previous step).

---

## Summary
CRITICAL: 1  
HIGH: 2  
MEDIUM: 2  
LOW: 2  
APPROVED: NO
