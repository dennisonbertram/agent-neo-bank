## CRITICAL

1) **Build will not compile: many imports point to files not present in this packed codebase**
- **File:** `src/App.tsx`
- **Missing imports (per provided directory structure):**
  - `./stores/authStore`
  - `./pages/Onboarding`, `ConnectCoinbase`, `VerifyOtp`, `Home`, `AddFunds`, `AgentsList`, `AgentDetail`, `TransactionDetail`, `Settings`
- **Impact:** TypeScript/webpack will fail at build time.
- **Fix:** Ensure these files exist (and are included in the packed representation), or remove/replace the imports and routes.

2) **Build will not compile: `cn` utility missing**
- **Files:**  
  - `src/components/layout/BottomNav.tsx` (`../../lib/cn`)  
  - `src/components/ui/OtpInput.tsx` (`../../lib/cn`)  
  - `src/pages/InstallSkill.tsx` (`../lib/cn`)
- **Impact:** Immediate module resolution failure.
- **Fix:** Add `src/lib/cn.ts` (or correct the import paths) and ensure it’s exported correctly.

3) **Build will not compile: missing UI components**
- **File:** `src/pages/InstallSkill.tsx`
- **Missing imports:** `../components/ui/Button`, `../components/ui/SuccessCheck`
- **Impact:** Module resolution failure.
- **Fix:** Add these components (or update imports to correct locations).

4) **Build will not compile: missing store**
- **File:** `src/App.tsx`
- **Missing import:** `./stores/authStore`
- **Impact:** Module resolution failure + auth gating cannot work.
- **Fix:** Add/restore the auth store (and ensure it exports `useAuthStore` with `isAuthenticated` and `checkAuthStatus`).


## HIGH

1) **Auth gating race condition / returning users may be redirected incorrectly**
- **File:** `src/App.tsx` (`ProtectedRoute` + `useEffect(checkAuthStatus)`)
- **Problem:** `checkAuthStatus()` runs *after* the first render. If the user loads a protected URL directly (e.g. `/home`), `isAuthenticated` may initially be `false`, causing an immediate redirect to `/onboarding` before the auth check completes. Depending on onboarding behavior, the user can get stuck or experience a redirect “bounce”.
- **Impact:** Broken navigation for authenticated users; unreliable deep-linking.
- **Fix:** Introduce an auth “loading/initialized” state in the store and gate routing on it, e.g.:
  - `isAuthInitialized` (or `authStatus: 'unknown'|'authed'|'unauthed'`)
  - Render a loading screen until initialized
  - Only redirect once status is known


## MEDIUM

1) **Root and catch-all routes always send users to onboarding (even if authenticated)**
- **File:** `src/App.tsx`
- **Problem:** `"/"` and `"*"` always `<Navigate to="/onboarding" />`.
- **Impact:** Authenticated users visiting the base URL won’t land in the app (unless onboarding itself re-routes). Also breaks typical “unknown route → home” behavior.
- **Fix:** Make redirects conditional:
  - If authed → `/home`
  - Else → `/onboarding`

2) **`BottomNav` can desync from actual route**
- **File:** `src/components/layout/BottomNav.tsx`
- **Problem:** Active state is driven by `activeTab` prop, not by `location.pathname`.
- **Impact:** UI can show the wrong active tab if a parent forgets to pass the right value or for nested routes (e.g. `/agents/:id`).
- **Fix:** Derive active tab from `useLocation()` (or compute it in a shared layout wrapper).

3) **Accessibility: icon buttons missing ARIA labeling**
- **File:** `src/components/layout/BottomNav.tsx`
- **Problem:** The FAB is icon-only with an empty label, and tab buttons rely on visual text but could still benefit from `aria-current` for the active tab.
- **Impact:** Poor screen reader usability.
- **Fix:** Add:
  - `aria-label="Add funds"` to FAB
  - `aria-current={isActive ? 'page' : undefined}` to active tab
  - Consider `<nav aria-label="Primary">`

4) **OTP input lacks expected UX features (paste / one-time-code autofill)**
- **File:** `src/components/ui/OtpInput.tsx`
- **Problems:**
  - No `onPaste` handling to distribute pasted digits across inputs
  - Missing `autoComplete="one-time-code"` (commonly used for SMS OTP autofill)
- **Impact:** Friction on mobile; users can’t paste codes easily.
- **Fix:** Add `onPaste` to first (or all) inputs to spread digits; add `autoComplete="one-time-code"` and consider `type="tel"`.

5) **`onComplete` can fire repeatedly**
- **File:** `src/components/ui/OtpInput.tsx`
- **Problem:** Any edit that results in `newValue.length === length` triggers `onComplete` again.
- **Impact:** Duplicate verification requests unless parent code is defensive.
- **Fix:** Track prior “completed” value, debounce, or only call `onComplete` when transitioning from incomplete → complete.

6) **Pinned bottom UI ignores safe-area insets**
- **Files:** `src/components/layout/BottomNav.tsx`, `src/pages/InstallSkill.tsx`, `src/pages/Stats.tsx`
- **Problem:** Absolute bottom positioning doesn’t account for `env(safe-area-inset-bottom)` on iOS and can clash with system UI / home indicator.
- **Impact:** Overlapped or hard-to-tap controls on some devices.
- **Fix:** Add padding using safe-area env variables (or a global layout wrapper that handles it).


## LOW

1) **State update uses non-functional pattern**
- **File:** `src/pages/InstallSkill.tsx`
- **Code:** `onClick={() => setExpanded(!expanded)}`
- **Impact:** Can be wrong in edge cases with rapid state batching.
- **Fix:** `setExpanded((v) => !v)`

2) **Type usage depends on global `React` namespace**
- **Files:** `src/components/ui/OtpInput.tsx` (`React.KeyboardEvent`), `src/App.tsx` (`React.ReactNode`)
- **Impact:** May fail under stricter TS/lint configs.
- **Fix:** Import types explicitly: `import type { KeyboardEvent, ReactNode } from 'react'`.

3) **Minor cleanup: unnecessary whitespace replacement**
- **File:** `src/components/ui/OtpInput.tsx`
- **Code:** `.replace(/\s/g, '')` after joining digits (digits are never whitespace)
- **Impact:** None; just noise.
- **Fix:** Remove unless you plan to support space-separated input/paste.


CRITICAL: 4  
HIGH: 1  
MEDIUM: 6  
APPROVED: NO
