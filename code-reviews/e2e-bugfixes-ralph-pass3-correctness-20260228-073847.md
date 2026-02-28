## CRITICAL

None found.

---

## HIGH

### 1) Auth check happens *after* routing decisions → authenticated users can be redirected to onboarding incorrectly
**File:** `src/App.tsx`  
**What happens:** On first render `isAuthenticated` is `false`, so any visit to a protected route (e.g. `/home`) immediately returns `<Navigate to="/onboarding" />`. Only **after** that render does `useEffect()` run `checkAuthStatus()`. If the user actually *is* authenticated, the app will still have already navigated them to onboarding, and nothing here navigates them back.

**Why it’s high severity:** It can break normal app entry and deep linking for real authenticated users (ends up on onboarding even though auth is valid).

**Fix options (typical):**
- Add an auth bootstrap flag in the store, e.g. `authChecked` / `isCheckingAuth`.
  - `checkAuthStatus()` sets `authChecked=true` when done.
  - `ProtectedRoute` renders a loading/skeleton/blank screen until `authChecked` is true, and only then decides to redirect or render children.
- Alternatively, perform auth initialization before rendering routes (e.g. in a top-level bootstrap boundary).

---

## MEDIUM

### 1) Store invariant drift: `flowId` is not cleared when auth is cleared by `checkAuthStatus`
**File:** `src/stores/authStore.ts`  
**What happens:** In `checkAuthStatus`, when unauthenticated (or on error in Tauri), you do:
```ts
set({ isAuthenticated: false, email: null })
```
…but **do not clear `flowId`**. `logout()` clears it, but auth status checks do not.

**Why it matters:** If code elsewhere assumes `!isAuthenticated => flowId === null` (common for auth flows), that invariant can be violated and lead to confusing behavior (stale flow continuation, wrong OTP verification context, etc.).

**Suggested fix:** When setting unauthenticated state in `checkAuthStatus`, also set `flowId: null`.

---

### 2) `OtpInput` ignores multi-character paste entirely (common OTP UX)
**File:** `src/components/ui/OtpInput.tsx`  
**What happens:** `handleInput` rejects any `char` that isn’t exactly `''` or a single digit:
```ts
if (!/^\d?$/.test(char)) return
```
So pasting `123456` into the first box does nothing.

**Why it matters:** Not a correctness crash, but it’s a significant expected behavior for OTP inputs and can look “broken” to users.

**Suggested fix:** Handle paste events (or detect `char.length > 1`) and distribute digits across inputs, update value, and call `onComplete` if filled.

---

## LOW

### 1) `OtpInput` backspace behavior clears previous digit when current is empty (may be surprising)
**File:** `src/components/ui/OtpInput.tsx`  
On backspace in an empty field, you both focus previous **and clear the previous digit**:
```ts
newDigits[index - 1] = ''
onChange(...)
```
Some OTP components only move focus and let the next backspace clear. Not necessarily wrong, but can feel unexpected.

---

## Summary
CRITICAL: 0  
HIGH: 1  
MEDIUM: 2  
APPROVED: NO
