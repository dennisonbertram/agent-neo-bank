# Navigation Redesign -- Gemini 3.1 Pro Review

> **Reviewer:** Gemini 3.1 Pro Preview (senior fintech UI/UX designer)
> **Date:** 2026-02-28
> **Context:** 390x640px fixed window, dark theme, React + Tailwind CSS v4

---

## Current Problems Identified

1. **Web-style breadcrumbs** (`Home / Agents / AgentName`) waste vertical space (56px+) and look like a web pattern, not a native app
2. **Theme mismatch bugs:** `ScreenHeader` uses `bg-white/90 backdrop-blur-[10px]` -- completely wrong for dark theme (`bg #1b1c26`). `BottomNav` uses `bg-white/95` and `border-black/5` -- also wrong.
3. **BottomNav exists but is unused** -- no page renders it
4. **AllTransactions has its own inline back button** instead of using ScreenHeader
5. **Home has no header** -- hidden settings gear on hover is undiscoverable
6. **Inconsistent navigation patterns** across screens

---

## The Navigation Strategy

Replace web-style breadcrumbs with a **Native-Style Header (iOS/Android hybrid)** and reinstate the **Bottom Nav Bar** for root destinations.

- **Root Screens** (`Home`, `AgentsList`, `Stats`) use `TopBar` and `BottomNav`. No back button.
- **Pushed Screens** (`AgentDetail`, `TransactionDetail`, `AllTransactions`, `Settings`, `AddFunds`) use the new `ScreenHeader`. Back button + centered title. BottomNav hidden.

---

## Exact Component Code

### A. New `ScreenHeader.tsx`

Replaces breadcrumb header with a balanced, mobile-native centered title bar. All light-theme classes removed.

```tsx
import { useNavigate } from 'react-router-dom'
import { ChevronLeft } from 'lucide-react'
import { cn } from '../../lib/cn'

interface ScreenHeaderProps {
  title: string
  showBack?: boolean
  rightElement?: React.ReactNode
  onBack?: () => void
  className?: string
}

export function ScreenHeader({
  title,
  showBack = true,
  rightElement,
  onBack,
  className
}: ScreenHeaderProps) {
  const navigate = useNavigate()

  const handleBack = () => {
    if (onBack) { onBack(); return }
    navigate(-1)
  }

  return (
    <header
      className={cn(
        'sticky top-0 z-50 h-[56px] px-6 flex items-center justify-between',
        'bg-[var(--bg-primary)]/80 backdrop-blur-md border-b border-[var(--border-subtle)]/50',
        className
      )}
    >
      {/* Left Action - Fixed Width to maintain center balance */}
      <div className="flex-1 flex items-center justify-start">
        {showBack && (
          <button
            type="button"
            onClick={handleBack}
            className="w-[36px] h-[36px] rounded-full bg-[var(--bg-secondary)] border border-[var(--border-subtle)] flex items-center justify-center cursor-pointer hover:bg-[var(--surface-hover)] transition-colors"
            aria-label="Go back"
          >
            <ChevronLeft size={20} className="text-[var(--text-secondary)]" />
          </button>
        )}
      </div>

      {/* Centered Title */}
      <div className="flex-[2] flex justify-center text-center">
        <h2 className="text-[16px] font-semibold text-[var(--text-primary)] truncate max-w-[200px]">
          {title}
        </h2>
      </div>

      {/* Right Action - Fixed Width */}
      <div className="flex-1 flex items-center justify-end">
        {rightElement}
      </div>
    </header>
  )
}
```

### B. Fixed `BottomNav.tsx`

All `bg-white/95` and `border-black/5` issues fixed. FAB themed to brand color. Auto-hides on pushed screens.

```tsx
import { useNavigate, useLocation } from 'react-router-dom'
import { Home, Layers, Activity, Plus } from 'lucide-react'
import { cn } from '../../lib/cn'

interface BottomNavProps {
  className?: string
}

const tabs = [
  { id: 'home', label: 'Home', icon: Home, path: '/home' },
  { id: 'agents', label: 'Agents', icon: Layers, path: '/agents' },
  { id: 'fab', label: '', icon: Plus, path: '/add-funds' },
  { id: 'stats', label: 'Stats', icon: Activity, path: '/stats' },
]

export function BottomNav({ className }: BottomNavProps) {
  const navigate = useNavigate()
  const location = useLocation()

  // Only render on root screens
  const isRootScreen = ['/home', '/agents', '/stats'].includes(location.pathname)
  if (!isRootScreen) return null

  return (
    <nav
      className={cn(
        'absolute bottom-0 left-0 right-0 h-[84px] z-50 pb-5',
        'bg-[var(--bg-elevated)]/90 backdrop-blur-md border-t border-[var(--border-subtle)] flex justify-around items-center',
        className
      )}
    >
      {tabs.map((tab) => {
        if (tab.id === 'fab') {
          return (
            <button
              key="fab"
              type="button"
              onClick={() => navigate(tab.path)}
              className="w-[52px] h-[52px] rounded-full bg-[var(--brand-main)] text-[var(--text-primary)] flex items-center justify-center -mt-[26px] shadow-[var(--shadow-fab)] border border-[var(--brand-container)] cursor-pointer transition-transform hover:scale-105 active:scale-95"
            >
              <Plus size={24} strokeWidth={3} />
            </button>
          )
        }

        const isActive = location.pathname.startsWith(tab.path)
        const Icon = tab.icon

        return (
          <button
            key={tab.id}
            type="button"
            onClick={() => navigate(tab.path)}
            className={cn(
              'flex flex-col items-center gap-1 w-[64px] border-none bg-transparent cursor-pointer transition-colors',
              isActive ? 'text-[var(--brand-on-container)]' : 'text-[var(--text-tertiary)] hover:text-[var(--text-secondary)]'
            )}
          >
            <Icon size={22} strokeWidth={isActive ? 2.5 : 2} />
            <span className="text-[10px] font-medium">{tab.label}</span>
          </button>
        )
      })}
    </nav>
  )
}
```

### C. Fixed `TopBar.tsx` (Root Screens)

```tsx
import { Settings } from 'lucide-react'
import { useNavigate } from 'react-router-dom'
import { cn } from '../../lib/cn'

export function TopBar({ className }: { className?: string }) {
  const navigate = useNavigate()

  return (
    <div
      className={cn(
        'flex items-center justify-between px-6 pt-6 pb-2',
        className
      )}
    >
      <div className="flex items-center gap-3">
        <div className="w-[36px] h-[36px] rounded-full bg-[var(--brand-container)] flex items-center justify-center text-[var(--brand-on-container)] text-[14px] font-semibold border border-[var(--brand-main)]/30">
          U
        </div>
        <span className="text-[16px] font-semibold text-[var(--text-primary)]">Tally Wallet</span>
      </div>
      <button
        type="button"
        onClick={() => navigate('/settings')}
        className="w-[36px] h-[36px] rounded-full bg-[var(--bg-secondary)] border border-[var(--border-subtle)] flex items-center justify-center cursor-pointer hover:bg-[var(--surface-hover)] transition-colors"
      >
        <Settings size={18} className="text-[var(--text-secondary)]" />
      </button>
    </div>
  )
}
```

---

## Per-Screen Navigation Specs

| Screen | BEFORE | AFTER |
| :--- | :--- | :--- |
| **Home** | No header. Hidden floating settings gear on hover. | **Header:** `TopBar` (Avatar + 'Tally Wallet' + Settings gear). **Footer:** `BottomNav`. Adjust padding for BottomNav overlap. |
| **AgentDetail** | `Home / Agents / [Name]` breadcrumb + StatusPill. White backdrop-blur. | **Header:** `ScreenHeader` title=`{agent.name}`. Left: back arrow. Right: `<StatusPill />`. **Footer:** No BottomNav. |
| **TransactionDetail** | `Home / Transaction` breadcrumb. White backdrop-blur. | **Header:** `ScreenHeader` title="Transaction". Left: back arrow. Right: empty. **Footer:** No BottomNav. |
| **AllTransactions** | Custom inline header with raw text + custom ChevronLeft. | **Header:** `ScreenHeader` title="All Activity". Left: back arrow. Right: empty. **Footer:** No BottomNav. |
| **AgentsList** | `Home / Agents` breadcrumb. Segment control below. | **Header:** `TopBar` (consistent with Home). **Footer:** `BottomNav` (Agents tab active). |
| **Stats** | `Home / Stats` breadcrumb. "Coming soon" text. | **Header:** `TopBar`. **Footer:** `BottomNav` (Stats tab active). |
| **Settings** | `Home / Settings` breadcrumb. | **Header:** `ScreenHeader` title="Settings". Left: back arrow. Right: empty. **Footer:** No BottomNav. |
| **AddFunds** | `Home / Add Funds` breadcrumb. | **Header:** `ScreenHeader` title="Add Funds". Left: back arrow. Right: empty. **Footer:** No BottomNav. |

---

## Push/Pop Navigation Animations

Add to `globals.css`:

```css
@keyframes slideInRight {
  from { transform: translateX(20px); opacity: 0; }
  to { transform: translateX(0); opacity: 1; }
}

@keyframes slideOutRight {
  from { transform: translateX(0); opacity: 1; }
  to { transform: translateX(20px); opacity: 0; }
}

/* Wrapper for pushed pages (AgentDetail, TxDetail, Settings, AddFunds) */
.page-transition-push {
  animation: slideInRight 0.3s cubic-bezier(0.16, 1, 0.3, 1) forwards;
  will-change: transform, opacity;
}
```

---

## Color Bug Fixes

### ScreenHeader (was)
```
bg-white/90 backdrop-blur-[10px]
```
### ScreenHeader (now)
```
bg-[var(--bg-primary)]/80 backdrop-blur-md border-b border-[var(--border-subtle)]/50
```

### BottomNav (was)
```
bg-white/95 backdrop-blur-[10px] border-t border-black/5
```
### BottomNav (now)
```
bg-[var(--bg-elevated)]/90 backdrop-blur-md border-t border-[var(--border-subtle)]
```

### AgentDetail progress bar border (was)
```
bg-black/5
```
### AgentDetail progress bar border (now)
```
bg-[var(--surface-hover)]
```

---

## Implementation Notes

1. **BottomNav placement:** Add `<BottomNav />` inside `App.tsx` directly above `<Routes>` (since it uses `useLocation` to auto-hide, it must be inside Router context).
2. **Typography constraints:** New `ScreenHeader` title has `truncate max-w-[200px]` to prevent long agent names from pushing layout.
3. **Hit targets:** Back button and settings gear are `36x36px` -- tactile but sized for desktop.
4. **API change:** `ScreenHeader` now takes `title: string` instead of `breadcrumbs: BreadcrumbItem[]`. All page call sites must be updated.
