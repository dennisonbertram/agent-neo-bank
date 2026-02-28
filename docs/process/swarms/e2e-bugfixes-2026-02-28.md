# Swarm: E2E Bugfixes

**Started**: 2026-02-28
**Team**: otp-fixer, expand-fixer, routes-fixer
**Scope**: `src/components/ui/OtpInput.tsx,src/pages/InstallSkill.tsx,src/components/layout/BottomNav.tsx,src/App.tsx,src/pages/Stats.tsx,src/stores/authStore.ts`
**Spec**: `docs/implementation/e2e-bugfixes-spec-2026-02-28.md`

---

## Wave 1 — Fix 3 E2E Issues

**Teammates**:
- otp-fixer: Fixed OTP input rendering (0 inputs in DOM)
- expand-fixer: Fixed Install Skill expand/collapse button
- routes-fixer: Created Stats page, fixed BottomNav routes

**Files changed**:
- `src/components/ui/OtpInput.tsx` — Fixed `padEnd('')` bug, added sequential entry enforcement, `onFocus` handler
- `src/pages/InstallSkill.tsx` — Added `p-0 appearance-none outline-none text-left` to button, `data-testid`
- `src/components/layout/BottomNav.tsx` — Stats path `/home` → `/stats`, FAB `/agents` → `/add-funds`
- `src/App.tsx` — Added Stats route, `DefaultRedirect` component for auth-aware root redirect
- `src/pages/Stats.tsx` — New placeholder page
- `src/stores/authStore.ts` — Clear `flowId` on auth failure, reverted test bypass

**Codex tasks**: None (fixes too small for delegation)

## Review Round 1 (Initial)
4 CRITICAL (all false positives from scoped review), 1 HIGH (pre-existing auth race condition)
**Action**: Added context to review prompts

## Review Round 2 (After OTP digit-shift fix)
- Pass 1 (Adversarial): 0 CRITICAL, 1 HIGH (pre-existing checkAuthStatus fail-open)
- Pass 2 (Skeptical): 0 CRITICAL, 1 HIGH (pre-existing root redirect)
- Pass 3 (Correctness): 0 CRITICAL, 1 HIGH (pre-existing flowId not cleared)
**Action**: Fixed root redirect (DefaultRedirect component) and flowId cleanup

## Ralph Loop (Round 3)
- **Pass 1 (Adversarial)**: 0 CRITICAL, 1 HIGH (inherent SPA client-side auth — not fixable at frontend)
- **Pass 2 (Skeptical)**: APPROVED (0 CRITICAL, 0 HIGH)
- **Pass 3 (Correctness)**: 0 CRITICAL, 1 HIGH (pre-existing auth loading state — tracked separately)
**Result**: 0 CRITICALs, 0 new HIGHs. Remaining HIGHs are pre-existing architectural concerns.

## Final Status
- [x] 0 CRITICAL issues
- [x] 0 new HIGH issues (remaining are pre-existing auth architecture)
- [x] TypeScript check passes
- [x] Vite build passes (1.25s)
- [ ] Committed and pushed
