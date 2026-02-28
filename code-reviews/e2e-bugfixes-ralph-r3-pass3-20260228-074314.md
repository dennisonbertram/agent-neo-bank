## HIGH

### 1) Auth gating can permanently misroute authenticated users on refresh/deep-link
**File:** `src/App.tsx` (+ interacts with `src/stores/authStore.ts`)  
**Problem:** `ProtectedRoute` immediately redirects to `/onboarding` when `isAuthenticated` is initially `false`, but `checkAuthStatus()` is async and runs after mount. If a user reloads on a protected route (e.g. `/home`), they can be redirected to onboarding before auth status resolves, and there’s no automatic redirect back once `isAuthenticated` flips to `true` (they remain on `/onboarding` route).

**Why it matters:** Incorrect state transition: “unknown auth” is treated as “unauthenticated”, breaking navigation on refresh/deep links.

**Fix idea:** Add an `authChecked`/`isAuthInitialized` boolean to the store and gate redirects until it’s true.
```ts
// authStore.ts
interface AuthState {
  isAuthenticated: boolean
  authChecked: boolean
  ...
}

export const useAuthStore = create<AuthState>((set) => ({
  isAuthenticated: false,
  authChecked: false,
  ...
  checkAuthStatus: async () => {
    try {
      const result = await tauriApi.auth.status()
      if (result.authenticated && result.email) {
        set({ isAuthenticated: true, email: result.email, flowId: null, authChecked: true })
      } else {
        // keep browser-safe behavior as-is, but ensure authChecked flips in Tauri
        if (typeof window !== 'undefined' && !(window as any).__TAURI_INTERNALS__) return
        set({ isAuthenticated: false, email: null, flowId: null, authChecked: true })
      }
    } catch {
      if (typeof window !== 'undefined' && !(window as any).__TAURI_INTERNALS__) return
      set({ isAuthenticated: false, email: null, flowId: null, authChecked: true })
    }
  },
}))
```
```tsx
// App.tsx
function ProtectedRoute({ children }: { children: React.ReactNode }) {
  const { isAuthenticated, authChecked } = useAuthStore()
  if (!authChecked) return null // or splash/loading
  if (!isAuthenticated) return <Navigate to="/onboarding" replace />
  return <>{children}</>
}
```

---

## MEDIUM

### 2) `flowId` can remain stale after successful authentication
**File:** `src/stores/authStore.ts`  
**Problem:** When setting authenticated state (`setAuthenticated`) or when `checkAuthStatus` marks the user authenticated, `flowId` is not cleared. If `flowId` represents an in-progress auth/OTP flow, keeping it when `isAuthenticated: true` can leave the store in an inconsistent state and potentially confuse later flows/pages that check `flowId`.

**Fix idea:** Clear `flowId` whenever authentication becomes true:
```ts
setAuthenticated: (email) => set({ isAuthenticated: true, email, flowId: null })

// and in checkAuthStatus authenticated branch:
set({ isAuthenticated: true, email: result.email, flowId: null })
```

---

## LOW

### 3) Accessibility: FAB button lacks an accessible name
**File:** `src/components/layout/BottomNav.tsx`  
**Problem:** The center “+” button has no text label and no `aria-label`, so screen readers may announce it as an unlabeled button.

**Fix idea:**
```tsx
<button aria-label="Add funds" ...>
```

### 4) Accessibility: Expand/collapse control missing `aria-expanded`
**File:** `src/pages/InstallSkill.tsx`  
**Problem:** Expand button doesn’t expose expanded state via ARIA.

**Fix idea:**
```tsx
<button aria-expanded={expanded} aria-controls="changes-panel" ... />
{expanded && <div id="changes-panel" ...>}
```

---

CRITICAL: 0 HIGH: 1 MEDIUM: 1 APPROVED: NO
