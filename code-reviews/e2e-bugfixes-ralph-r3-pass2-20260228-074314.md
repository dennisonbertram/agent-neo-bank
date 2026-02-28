## MEDIUM

### 1) Zustand subscriptions are overly broad → unnecessary rerenders across the app
**Files:** `src/App.tsx`, `src/stores/authStore.ts` (consumers), `ProtectedRoute`, `DefaultRedirect`  
**What:** Multiple components call `useAuthStore()` without a selector and then destructure (`const { isAuthenticated } = useAuthStore()`, etc.). In Zustand this subscribes the component to *all* store changes, so unrelated updates (e.g., `flowId`) can rerender routing/layout unnecessarily.  
**Why it matters:** Can cause extra rerenders, route tree re-evaluation, and harder-to-debug UI behavior as the app grows.  
**Fix:** Use selectors:
```ts
const isAuthenticated = useAuthStore(s => s.isAuthenticated)
const checkAuthStatus = useAuthStore(s => s.checkAuthStatus)
```

### 2) `flowId` can remain stale after becoming authenticated
**File:** `src/stores/authStore.ts`  
**What:** When `checkAuthStatus()` sets `{ isAuthenticated: true, email: result.email }`, it does not clear `flowId`. If a user started an OTP flow and then becomes authenticated, `flowId` may remain set.  
**Why it matters:** Stale flow state can create subtle navigation/logic bugs (e.g., UI thinking an OTP flow is in progress when it shouldn’t be).  
**Fix:** Clear `flowId` when auth becomes true:
```ts
set({ isAuthenticated: true, email: result.email, flowId: null })
```

---

## LOW

### 1) FAB button lacks an accessible label
**File:** `src/components/layout/BottomNav.tsx`  
**What:** The “fab” button is icon-only and has no `aria-label`.  
**Why it matters:** Screen readers will announce it poorly.  
**Fix:** Add `aria-label="Add funds"` (or similar).

### 2) OTP input could be improved for platform OTP autofill + accessibility
**File:** `src/components/ui/OtpInput.tsx`  
**What:** Missing common attributes that improve OTP UX:
- `autoComplete="one-time-code"` (often on the first input)
- `aria-label` per digit (or a grouped label)
**Why it matters:** Better mobile OTP autofill and screen reader behavior.  
**Fix:** Add appropriate `autoComplete` and labeling.

### 3) OTP completion callback can fire repeatedly after “complete” state is reached
**File:** `src/components/ui/OtpInput.tsx`  
**What:** `onComplete(trimmed)` runs any time `trimmed.length === length` after an input change—so edits while complete can trigger again.  
**Why it matters:** Can double-submit verification requests unless callers debounce/guard.  
**Fix:** Track prior completion state (e.g., ref) and only fire on transition to complete.

### 4) Minor state toggle pattern is less robust than functional update
**File:** `src/pages/InstallSkill.tsx`  
**What:** `setExpanded(!expanded)` can be fragile if the handler is ever used in async/batched contexts.  
**Fix:** `setExpanded(v => !v)`.

---

CRITICAL: 0 HIGH: 0 MEDIUM: 2 APPROVED: YES
