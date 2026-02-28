## CRITICAL

None found in the newly shown files.

---

## HIGH

### 1) `checkAuthStatus` fails open in non‑Tauri contexts (and can be forced to fail open in Tauri)
**File:** `src/stores/authStore.ts`

**What changed / issue:**  
On “unauthenticated” results or on exceptions, the code intentionally **returns early without clearing auth state** when it believes it is running in a browser:

```ts
if (typeof window !== 'undefined' && !(window as ...).__TAURI_INTERNALS__) return
set({ isAuthenticated: false, email: null })
```

This is a security regression if:
- This UI is ever deployed in a web context (even temporarily), or
- The Tauri detection is wrong/missing in some runtime, or
- Anything in the renderer can tamper with `window.__TAURI_INTERNALS__` (e.g., a compromised renderer, devtools in a debug build, or any XSS). Deleting/overwriting that property makes the app treat a real Tauri session as “browser”, causing **stale `isAuthenticated: true` to persist even when `tauriApi.auth.status()` fails or returns unauthenticated**.

**Impact:**  
Client-side route protection (`ProtectedRoute`) can be bypassed by keeping `isAuthenticated` stuck on `true` during auth check failures or “logged out” states. If any privileged Tauri commands/pages assume the front-end gate is sufficient, this becomes an app-level authorization bypass.

**Exploit sketch (Tauri renderer, assuming attacker JS execution):**
1. Ensure user is authenticated once (state becomes `true`).
2. `delete (window as any).__TAURI_INTERNALS__` (or set it falsy).
3. Cause `tauriApi.auth.status()` to throw/fail (or return unauthenticated).
4. `checkAuthStatus()` returns early and does not clear `isAuthenticated`.
5. Navigate to protected routes; UI gating remains open.

**Fix recommendations:**
- In production, always **fail closed**: clear auth on `unauthenticated` or `catch`.
- If you need “visual testing” behavior, gate it behind an explicit build flag (e.g. `import.meta.env.DEV`) rather than runtime heuristics.
- Avoid using a writable global (`__TAURI_INTERNALS__`) as the security-relevant environment detector; prefer a robust Tauri API presence check or compile-time target.

---

## MEDIUM

### 1) `flowId` not cleared when auth becomes unauthenticated via `checkAuthStatus`
**File:** `src/stores/authStore.ts`

**Issue:**  
On unauthenticated status/error, the store clears `isAuthenticated` and `email` but leaves `flowId` untouched (it is only cleared on `logout()`).

**Impact:**  
Stale `flowId` can persist across auth invalidation and may be reused unintentionally in subsequent auth/OTP flows (risk depends on how `flowId` is used elsewhere). This is a common source of “confused deputy” behavior in multi-step auth flows.

**Fix:**  
When setting unauthenticated state in `checkAuthStatus`, also `set({ flowId: null })` unless there is a strong reason not to.

---

## LOW

No notable new injection vectors or bypasses in the UI-only components (`BottomNav`, `OtpInput`, `InstallSkill`, `Stats`, `cn`) as shown.

---

CRITICAL: 0 HIGH: 1 MEDIUM: 1 APPROVED: NO
