# Design System Reskin Plan

## Context
The app already has a dark theme with Privacy.com-inspired tokens (`--bg-primary: #1b1c26`, `--brand-main: #4949f1`, etc.). However, it lacks light theme support, has some hardcoded colors (Button `bg-black`, avatar `bg-[var(--accent-terracotta)]`), and needs typography/spacing adjustments for the 390px viewport per Gemini 3.1 Pro's design review (`docs/design/reskin-plan-review.md`).

This plan adds light theme + theme switching, applies the design system's semantic token naming, fixes remaining hardcoded values, and applies Gemini's viewport-specific sizing feedback.

**There is no Stats page** — the placeholder will be removed and the BottomNav simplified.

## Scope
- Add `[data-theme="light"]` color block to tokens.css (dark is already default via `:root`)
- Add three-way theme toggle (Dark / Light / System) in Settings
- Create Zustand theme store + FOUC prevention
- Rename tokens to semantic names from design-system.md
- Fix hardcoded colors in components (Button `bg-black`, etc.)
- Apply Gemini viewport adjustments (typography, spacing, button height)
- Remove Stats page and Stats tab from BottomNav
- Get Gemini 3.1 Pro design review of finished result

## Gemini Review Adjustments

| Issue | Current Value | Adjusted Value | Reason |
|-------|--------------|----------------|--------|
| Warning color | `#df9e33` | `#FBBF24` (amber-400) | WCAG AA contrast on dark bg |
| Balance font | 40px inline | 30px (`text-3xl`) | Overflows 390px with padding at larger sizes |
| `.text-display` | 42px | 30px mono bold | Scale for viewport |
| `.text-title` | 22px | 20px (`text-xl`) | Scale for viewport |
| Button height | 56px primary | 48px (`h-12`) | Gemini said 44-48px; keep consistent |
| Text overflow | none | `truncate` on list items | Names wrap at 390px |
| Sticky headers | none | `sticky top-0 z-20 backdrop-blur-md` | Keep context while scrolling |
| Scrollbar | hidden | Keep hidden (already `scrollbar-width: none`) | Already handled |

## Implementation Order

### Step 1: Theme Store
**New file: `src/stores/themeStore.ts`**
- Zustand store with `persist` middleware (localStorage key: `theme-preference`)
- State: `preference` ('dark' | 'light' | 'system'), `resolved` ('dark' | 'light')
- Default preference: `'dark'`
- `setPreference(p)`: saves preference, resolves system via `matchMedia('prefers-color-scheme: dark')`, sets `data-theme` attribute on `document.documentElement`
- `initialize()`: called on app mount, reads persisted preference, registers `matchMedia` change listener for system theme tracking

### Step 2: FOUC Prevention
**Edit: `index.html`**
- Add inline `<script>` before React bundle that reads `theme-preference` from localStorage and sets `data-theme` on `<html>` immediately
- Prevents flash of wrong theme on reload

### Step 3: Add Light Theme to Tokens
**Edit: `src/styles/tokens.css`**
- Keep existing `:root` block as dark theme (it's already correct)
- Rename tokens to semantic names throughout:

| Current Name | New Semantic Name |
|-------------|-------------------|
| `--bg-primary` | `--page-background` |
| `--bg-secondary` | `--container-background` |
| `--bg-elevated` | `--surface-raised` |
| `--surface-hover` | `--surface-hover` (keep) |
| `--border-subtle` | `--border-default` |
| `--border-strong` | `--border-strong` (keep) |
| `--text-primary` | `--text-primary` (keep) |
| `--text-secondary` | `--text-subtle` |
| `--text-tertiary` | `--text-muted` |
| `--brand-main` | `--brand-bright` |
| `--brand-container` | `--brand-container` (keep) |
| `--brand-on-container` | `--brand-on-container` (keep) |
| `--color-danger` | `--danger-main` |
| `--color-positive` | `--success-main` |
| `--color-link` | `--brand-bright` (merge) |
| `--accent-green` | `--success-main` (alias) |
| `--accent-yellow` | `--warning-main` |
| `--accent-terracotta` | `--danger-main` (alias) |
| `--accent-blue` | `--info-main` |

- Add `[data-theme="light"]` block with light variants:
  - `--page-background: #F8F8FA`
  - `--container-background: #FFFFFF`
  - `--surface-raised: #F0F0F5`
  - `--text-primary: #1A1D27`
  - `--text-subtle: #4A4A5A`
  - `--text-muted: #8A8A9A`
  - `--border-default: #E2E2EA`
  - `--border-strong: #C8C8D4`
  - `--brand-bright: #4949f1` (same in both themes)
  - Status containers: lighter pastel variants
- Fix warning color: `--warning-main: #FBBF24` (was `#df9e33`)
- Add new tokens: `--surface-sunken`, `--ring`, `--gradient-primary`, `--text-inverse`

### Step 4: Tailwind Theme Bridge
**Edit: `src/index.css`**
- Update `@theme` block to use new semantic token names
- Map: `--color-page-bg: var(--page-background)`, `--color-container-bg: var(--container-background)`, etc.
- Remove old accent-* mappings, replace with semantic equivalents

### Step 5: Global Styles
**Edit: `src/styles/globals.css`**
- `#root` background: `var(--page-background)` (already uses `--bg-primary`, just rename)
- Typography scale adjustments (Gemini feedback):
  - `.text-display`: 42px → **30px**, add `font-family: var(--font-mono)`
  - `.text-title`: 22px → **20px**
  - `.text-subtitle`, `.text-body`, `.text-caption`, `.text-mono`: keep as-is
- Status classes: already use variant tokens, just rename if tokens renamed

### Step 6: Wire Up Theme Init
**Edit: `src/App.tsx`**
- Import `useThemeStore`
- Call `useThemeStore.getState().initialize()` in top-level `useEffect`

### Step 7: Remove Stats Page
**Delete: `src/pages/Stats.tsx`**
**Edit: `src/App.tsx`** — Remove `/stats` route
**Edit: `src/components/layout/BottomNav.tsx`** — Remove Stats tab from `tabs` array. Update `isRootScreen` check to only `['/home', '/agents']`

### Step 8: UI Components

| Component | File | Changes |
|-----------|------|---------|
| **Button.tsx** | `src/components/ui/Button.tsx` | Primary: `bg-black` → `bg-[var(--brand-bright)] text-white`. Height: 56px → 48px (`h-12`). Outline border: `var(--border-default)` |
| **StatusPill.tsx** | `src/components/ui/StatusPill.tsx` | Rename variant token refs if token names changed |
| **ProgressBar.tsx** | `src/components/ui/ProgressBar.tsx` | Track: `var(--surface-raised)` |
| **SegmentControl.tsx** | `src/components/ui/SegmentControl.tsx` | Container: `var(--surface-raised)`. Active bg: `var(--container-background)` |
| **Toggle.tsx** | `src/components/ui/Toggle.tsx` | Checked: `var(--brand-bright)` |
| **InputGroup.tsx** | `src/components/ui/InputGroup.tsx` | Bg: `var(--surface-raised)` |
| **OtpInput.tsx** | `src/components/ui/OtpInput.tsx` | Bg: `var(--surface-raised)`. Focus ring: `var(--brand-bright)` |
| **SuccessCheck.tsx** | `src/components/ui/SuccessCheck.tsx` | Color: `var(--success-main)` |

### Step 9: Layout Components

| Component | File | Changes |
|-----------|------|---------|
| **TopBar.tsx** | `src/components/layout/TopBar.tsx` | Avatar bg: `var(--brand-bright)` instead of hardcoded |
| **BottomNav.tsx** | `src/components/layout/BottomNav.tsx` | Remove Stats tab. Rename token refs |
| **ScreenHeader.tsx** | `src/components/layout/ScreenHeader.tsx` | Add `sticky top-0 z-20 backdrop-blur-md`. Bg: `var(--page-background)/90` |

### Step 10: Domain Components

| Component | File | Changes |
|-----------|------|---------|
| **AgentCard.tsx** | `src/components/agent/AgentCard.tsx` | Add `border border-[var(--border-default)]`. Rename token refs |
| **AgentAvatar.tsx** | `src/components/agent/AgentAvatar.tsx` | Rename token refs |
| **AgentPillRow.tsx** | `src/components/agent/AgentPillRow.tsx` | Add `truncate` on label. Rename token refs |
| **TransactionItem.tsx** | `src/components/transaction/TransactionItem.tsx` | Add `truncate` on merchant name. Rename token refs |
| **MetaCard.tsx** | `src/components/transaction/MetaCard.tsx` | Rename token refs |

### Step 11: Pages

| Page | File | Key Changes |
|------|------|-------------|
| **Settings.tsx** | `src/pages/Settings.tsx` | Add "Appearance" section with Dark/Light/System three-way toggle using SegmentControl. Avatar: `var(--brand-bright)` (was `--accent-terracotta`). Rename token refs |
| **Home.tsx** | `src/pages/Home.tsx` | Balance font: already 40px, reduce to 30px. Remove hardcoded accent color maps → use semantic tokens. Rename token refs |
| **AgentsList.tsx** | `src/pages/AgentsList.tsx` | Rename token refs |
| **AgentDetail.tsx** | `src/pages/AgentDetail.tsx` | Rename token refs, fix any hardcoded colors |
| **AddFunds.tsx** | `src/pages/AddFunds.tsx` | QR stays white bg. Rename token refs |
| **TransactionDetail.tsx** | `src/pages/TransactionDetail.tsx` | Rename token refs |
| **AllTransactions.tsx** | `src/pages/AllTransactions.tsx` | Rename token refs |
| **Onboarding.tsx** | `src/pages/Onboarding.tsx` | Rename token refs |
| **InstallSkill.tsx** | `src/pages/InstallSkill.tsx` | Rename token refs |
| **ConnectCoinbase.tsx** | `src/pages/ConnectCoinbase.tsx` | Rename token refs |
| **VerifyOtp.tsx** | `src/pages/VerifyOtp.tsx` | Rename token refs |

### Step 12: Gemini 3.1 Pro Design Review
After all changes are implemented:
- Run the app and take screenshots in both dark and light themes
- Flatten relevant source with repomix and pipe to Gemini 3.1 Pro
- Review framing: "senior fintech UI/UX designer reviewing a 390x640px neobank for AI agents"
- Fix any issues found before declaring complete

## Verification
1. Launch app — dark theme by default, all pages dark bg with light text
2. Toggle to Light in Settings — light backgrounds, dark text, brand colors stay consistent
3. Toggle to System — follows OS preference
4. Reload — theme persists, no flash of wrong theme (FOUC script works)
5. Walk all pages in both themes: Onboarding → Home → Agents → Agent Detail → Add Funds → Transaction Detail → Settings
6. Verify Stats tab is gone from BottomNav and /stats route removed
7. Check button states (hover/active/disabled), toggles, status pills, progress bars
8. Check text truncation on long agent/merchant names
9. Run `npm run dev` for visual check, `npm test` for regression

## Files Modified (28 total)
- **New**: `src/stores/themeStore.ts`
- **Delete**: `src/pages/Stats.tsx`
- **Rewrite**: `src/styles/tokens.css`
- **Edit**: `index.html`, `src/index.css`, `src/styles/globals.css`, `src/App.tsx`
- **Edit**: 8 UI components (Button, StatusPill, ProgressBar, SegmentControl, Toggle, InputGroup, OtpInput, SuccessCheck)
- **Edit**: 3 layout components (TopBar, BottomNav, ScreenHeader)
- **Edit**: 5 domain components (AgentCard, AgentAvatar, AgentPillRow, TransactionItem, MetaCard)
- **Edit**: 11 pages (Home, Settings, AgentsList, AgentDetail, AddFunds, TransactionDetail, AllTransactions, Onboarding, InstallSkill, ConnectCoinbase, VerifyOtp)
