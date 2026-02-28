## HIGH

### 1) `src/App.tsx` — Root (`/`) and catch-all (`*`) always redirect to onboarding, even when authenticated
**Impact:** Authenticated users landing on `/` (common SPA entrypoint) or any unknown route will be forced into the onboarding flow instead of the main app (e.g. `/home`). This is independent of the known async-auth timing issue: even with `isAuthenticated === true`, `/` still goes to `/onboarding`.

**Where:**
```tsx
<Route path="/" element={<Navigate to="/onboarding" replace />} />
<Route path="*" element={<Navigate to="/onboarding" replace />} />
```

**Fix idea:** Make these redirects conditional on auth state (e.g., redirect to `/home` when authenticated), or introduce an `IndexRoute` component that decides based on `isAuthenticated`.

---

## MEDIUM

### 2) `src/stores/authStore.ts` — `checkAuthStatus` clears `email` but can leave a stale `flowId`
**Impact:** If `flowId` is used for OTP/verification flows, a stale `flowId` can persist after the store is transitioned to unauthenticated via `checkAuthStatus`, potentially causing confusing behavior or accidental reuse of an old flow.

**Where:**
```ts
set({ isAuthenticated: false, email: null })
```
(similarly in `catch`)

**Fix idea:** Clear `flowId` in the unauthenticated transitions:
```ts
set({ isAuthenticated: false, email: null, flowId: null })
```

---

### 3) `src/components/ui/OtpInput.tsx` — Potential digit loss on fast multi-field entry due to closure over `digits`
**Impact / edge case:** `handleInput` builds `newDigits` from `digits` captured at render time. If a user enters a digit, focus advances, and the next input event fires before the parent has re-rendered with the updated `value`, the second change can be computed from stale `digits` and overwrite/lose earlier digits.

**Where:**
```ts
const newDigits = [...digits]
newDigits[index] = char
onChange(newDigits.join(''))
```

**Fix idea:** Derive the working array from the latest `value` inside the handler (not from the render-closure `digits`), or maintain internal state and reconcile to `value`. (With a controlled prop, you can reconstruct digits from `value` at call time.)

---

## LOW

### 4) `src/components/ui/OtpInput.tsx` — No support for pasting a full OTP
**Impact:** Pasting “123456” into a field won’t work because `handleInput` only accepts `^\d?$` and `maxLength={1}`; this is a common UX expectation for OTP components.

**Fix idea:** Add `onPaste` handler to distribute digits across inputs starting at the focused index.

---

### 5) `src/components/layout/BottomNav.tsx` — `tabs` includes a `fab` entry with unused `path: ''`
**Impact:** Minor maintainability issue: the `fab` tab is special-cased anyway, so the `path` field is misleading/dead data.

**Fix idea:** Remove `path` from the `fab` object or model FAB separately from tab navigation items.

---

CRITICAL: 0 HIGH: 1 MEDIUM: 2 APPROVED: NO
