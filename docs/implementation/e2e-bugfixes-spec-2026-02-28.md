# E2E Bugfix Spec — 2026-02-28

## Scope
Fix 3 issues found during E2E testing. Single wave — all files are independent.

**Repomix include**: `src/components/ui/OtpInput.tsx,src/pages/InstallSkill.tsx,src/components/layout/BottomNav.tsx,src/App.tsx,src/pages/Stats.tsx`

## Issue 1: OTP Inputs Not Rendering (CRITICAL)

**File**: `src/components/ui/OtpInput.tsx`
**Line**: 14
**Bug**: `''.padEnd(6, '')` pads with empty string — does nothing. Returns `''`. Then `''.split('')` returns `[]` (empty array in modern JS). Result: 0 inputs rendered.
**Fix**: Use `Array.from({ length }, (_, i) => value[i] || '')` instead of the padEnd approach.

## Issue 2: Install Skill Expand/Collapse Not Working

**File**: `src/pages/InstallSkill.tsx`
**Lines**: 51-66
**Bug**: Code logic looks correct (useState + onClick + conditional render). Likely a CSS/event issue — the button may not be receiving clicks due to styling. Need to investigate and fix.
**Fix**: Ensure the button element is properly interactive. May need to add `p-0` or explicit padding reset, check for stacking context issues.

## Issue 3: Stats Tab & FAB Button Routes

**Files**: `src/components/layout/BottomNav.tsx`, `src/App.tsx`, new `src/pages/Stats.tsx`
**Bug**: Stats tab has `path: '/home'` (wrong). FAB hardcoded to `/agents` (should add funds).
**Fix**:
- Create a Stats placeholder page
- Add `/stats` route to App.tsx
- Fix BottomNav: Stats path → `/stats`, FAB → `/add-funds`
