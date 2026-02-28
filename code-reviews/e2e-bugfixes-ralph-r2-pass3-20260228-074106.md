## CRITICAL
None.

## HIGH

### 1) `flowId` can remain set while unauthenticated (state invariant violation)
**File:** `src/stores/authStore.ts`  
**What changed / issue:** In `checkAuthStatus`, when the user is not authenticated (or on error in Tauri), the store sets `{ isAuthenticated: false, email: null }` but **does not clear `flowId`**. That allows an invalid state: `isAuthenticated === false` while `flowId` still points to a previous OTP/login flow.

**Why it matters:** Any screens relying on `flowId` (OTP verify, etc.) can accidentally proceed using a stale flow id after a failed auth check/logout, causing incorrect verification attempts or confusing UI behavior.

**Fix:** Clear `flowId` whenever auth is reset, e.g.
```ts
set({ isAuthenticated: false, email: null, flowId: null })
```
Also consider clearing `flowId` in `setAuthenticated` (once login completes) to avoid carrying temporary login state forward.

## MEDIUM

### 2) Root and catch-all routes always redirect to onboarding (ignores authenticated users)
**File:** `src/App.tsx`  
**Issue:** Both:
- `path="/" -> /onboarding`
- `path="*" -> /onboarding`

redirect unconditionally, even when `isAuthenticated` is true. That can route authenticated users to onboarding on app start (if they land on `/`) or on unknown routes, instead of taking them to the main app.

**Fix:** Make the default redirect conditional on auth state (without addressing the known async auth-race separately), e.g. choose between `/home` and `/onboarding` based on current store state.

### 3) React namespace types used without explicit type import (config-dependent build risk)
**Files:**
- `src/components/ui/OtpInput.tsx` (`React.KeyboardEvent`)
- `src/App.tsx` (`React.ReactNode`)

**Issue:** These rely on the global `React` namespace being available for types. In some TS configs (depending on `jsx`, `types`, and module settings), this can fail typechecking.

**Fix:** Import the types explicitly:
```ts
import type { KeyboardEvent, ReactNode } from 'react'
```
and update annotations accordingly.

## LOW

### 4) OTP paste UX likely broken (single-character regex rejects multi-char input)
**File:** `src/components/ui/OtpInput.tsx`  
**Issue:** `handleInput` rejects any `char` that isn’t `^\d?$`, so pasting an entire OTP into one cell (common behavior) is ignored rather than distributed across inputs.

**Fix:** Detect multi-character input and spread across digits (or at least accept the first char).

---

CRITICAL: 0 HIGH: 1 MEDIUM: 2 APPROVED: NO
