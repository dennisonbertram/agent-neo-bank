# App Shell Implementation Report

> **Date:** 2026-02-27
> **Task:** #7 - App shell with routing, layout, and dark theme

## Summary

Implemented the full app shell with routing, layout components, dark theme, onboarding wizard, dashboard with empty states, shared components, hooks, and stores. All components use named exports. TDD approach followed -- 23 tests written first, all passing.

## Changes Made

### Configuration
- **vite.config.ts**: Added vitest config (globals, jsdom, setupFiles)
- **index.html**: Added `class="dark"` to `<html>` element for dark theme
- **src/test/setup.ts**: Added `@testing-library/jest-dom/vitest` import for DOM matchers
- **src/test/render.tsx**: New test helper for rendering with router context

### Layout Components
- **src/components/layout/Shell.tsx**: Flex layout with Sidebar + Header + Outlet
- **src/components/layout/Sidebar.tsx**: Nav links with lucide-react icons (LayoutDashboard, Bot, ArrowUpDown, Settings), active route highlighting
- **src/components/layout/Header.tsx**: Title + balance display using useBalance hook, loading state

### Pages
- **src/App.tsx**: Full routing setup with Shell wrapper for authenticated routes, standalone Onboarding
- **src/main.tsx**: Updated to use named exports, imports index.css
- **src/pages/Dashboard.tsx**: Balance card, "Your Agents" empty state, "Recent Transactions" empty state
- **src/pages/Onboarding.tsx**: 4-step wizard (Welcome -> Email -> OTP -> Fund)
- **src/pages/Agents.tsx**: Placeholder
- **src/pages/Transactions.tsx**: Placeholder
- **src/pages/Settings.tsx**: Placeholder

### Onboarding Steps
- **WelcomeStep.tsx**: Welcome text + "Get Started" button
- **EmailStep.tsx**: Email input with regex validation, error display
- **OtpStep.tsx**: 6-digit input with validation, digits-only filtering
- **FundStep.tsx**: Wallet address display + copy button

### Shared Components
- **CurrencyDisplay.tsx**: Formats amounts as `$1,247.83`
- **StatusBadge.tsx**: Color-coded badge (pending=yellow, active=green, etc.)
- **EmptyState.tsx**: Centered text + optional lucide icon

### Hooks
- **useBalance.ts**: Calls `invoke("get_balance")`, returns `{ balance, isLoading, error, refetch }`
- **useInvoke.ts**: Generic Tauri invoke hook with loading/error state
- **useTauriEvent.ts**: Listens to Tauri events with cleanup

### Stores (already existed, unchanged)
- **authStore.ts**: `{ isAuthenticated, email, setAuthenticated, clearAuth }`
- **settingsStore.ts**: `{ mockMode, network, setMockMode, setNetwork }`

## Test Results

```
 Test Files  9 passed (9)
      Tests  23 passed (23)
```

### Test Files
| File | Tests | Status |
|------|-------|--------|
| Shell.test.tsx | 3 | Pass |
| Sidebar.test.tsx | 2 | Pass |
| Header.test.tsx | 3 | Pass |
| Dashboard.test.tsx | 3 | Pass |
| WelcomeStep.test.tsx | 3 | Pass |
| EmailStep.test.tsx | 3 | Pass |
| OtpStep.test.tsx | 2 | Pass |
| FundStep.test.tsx | 2 | Pass |
| Onboarding.test.tsx | 2 | Pass |

## Build Verification
- `tsc -b`: Clean (no errors)
- `vite build`: Succeeds (269.5 kB JS, 33 kB CSS gzipped)
- No Rust files modified
