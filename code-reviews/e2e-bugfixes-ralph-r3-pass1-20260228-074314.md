## CRITICAL

None found in the files shown.

## HIGH

### 1) Renderer-only authz gating is trivially bypassable (`ProtectedRoute` trusts client state)
**Where:** `src/App.tsx` (`ProtectedRoute`), `src/stores/authStore.ts` (`setAuthenticated`)

**Why it matters:** Access control is implemented purely in the React renderer via `isAuthenticated`. Any attacker who can execute JS in the renderer context (XSS, malicious dependency, devtools in production builds, or any renderer compromise) can flip the Zustand auth state and unlock all “protected” routes (`/add-funds`, `/agents/*`, `/transactions/*`, etc.) without possessing a real authenticated session.

**Practical bypass:**
- If the attacker can reach store setters (directly or via state mutation tooling), they can set:
  - `isAuthenticated=true`
  - `email="victim@domain"`
- Then navigate directly to protected routes.

**Impact:** Authorization bypass at the UI layer; if any privileged actions are gated only by route/UI checks (and not re-verified in Tauri commands / backend), this becomes a full privilege escalation.

**Fix / mitigation:**
- Treat route guards as UX only. Enforce authorization on every privileged operation (e.g., in Tauri command handlers / backend API), using an unforgeable session/token verified server-side or in the Rust layer.
- In production, consider disabling devtools / hardening the renderer (still not sufficient without backend enforcement).

## MEDIUM

### 2) `flowId` is not cleared on successful authentication (stale auth-flow state)
**Where:** `src/stores/authStore.ts` (`checkAuthStatus` success branch)

**Why it matters:** On successful auth (`result.authenticated && result.email`), the store sets `isAuthenticated` and `email` but leaves `flowId` untouched. If `flowId` is used as part of OTP / verification, a stale `flowId` can cause:
- replay/confusion in subsequent verification attempts,
- unintended coupling between sessions/users (especially if multiple logins happen in one runtime).

**Fix:**
- Clear `flowId` when auth becomes true, e.g. `set({ isAuthenticated: true, email: result.email, flowId: null })`.

## LOW

### 3) OTP inputs lack explicit autofill/clipboard hardening hints
**Where:** `src/components/ui/OtpInput.tsx`

**Why it matters:** Not a direct exploit, but OTP fields commonly should specify attributes to reduce accidental persistence/leakage and improve correct handling:
- `autoComplete="one-time-code"` (or in some threat models `autocomplete="off"`),
- `name`/`aria-label` per digit,
- optional `pattern="\d*"`.

**Fix:** Add appropriate `autoComplete` / labeling attributes based on desired UX/security posture.

---

CRITICAL: 0 HIGH: 1 MEDIUM: 1 APPROVED: NO
