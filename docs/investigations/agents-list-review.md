# AgentsList Page — Gemini 3.1 Pro Design Review

**Reviewer**: Gemini 3.1 Pro Preview
**Date**: 2026-03-01
**Target**: `src/pages/AgentsList.tsx`, `src/components/agent/AgentCard.tsx`, `src/components/layout/TopBar.tsx`

---

## 1. TopBar vs. ScreenHeader

Pushed screens require explicit backward navigation and a clear title rather than duplicating the root wallet header, along with a primary action to add new agents.

```tsx
// AgentsList.tsx
import { ChevronLeft, Plus } from 'lucide-react'
import { useNavigate } from 'react-router-dom'

// Inside AgentsList component:
const navigate = useNavigate()

// Replace <TopBar /> with:
<div className="flex items-center justify-between px-6 pt-6 pb-2">
  <div className="flex items-center gap-2">
    <button
      onClick={() => navigate(-1)}
      className="w-8 h-8 -ml-2 flex items-center justify-center rounded-full hover:bg-[var(--surface-hover)] text-[var(--text-secondary)] transition-colors"
    >
      <ChevronLeft size={24} />
    </button>
    <h1 className="text-[18px] font-semibold text-[var(--text-primary)]">Agents</h1>
  </div>
  <button
    onClick={() => navigate('/agents/new')}
    className="w-[36px] h-[36px] rounded-full bg-[var(--brand-container)] border border-[var(--brand-main)]/30 flex items-center justify-center text-[var(--brand-on-container)] hover:bg-[var(--brand-main)] hover:text-[var(--text-primary)] transition-colors"
  >
    <Plus size={20} />
  </button>
</div>
```

## 2. SegmentControl vs Filter Pattern

The strings "Active / All Agents / Archived" crowd a 390px screen width; shortening to "All / Active / Archived" makes the existing SegmentControl fit perfectly without truncation.

```tsx
// AgentsList.tsx
// Update initial state:
const [segment, setSegment] = useState('All')

// Update the SegmentControl implementation:
<SegmentControl
  options={['All', 'Active', 'Archived']}
  value={segment}
  onChange={setSegment}
  className="mb-6 mt-0"
/>
```

## 3. AgentCard Dark Theme Enhancements

Relying solely on a flat background color lacks depth in dark mode; adding subtle borders, switching to the elevated background, and introducing distinct hover states improves the tactile feel.

```tsx
// AgentCard.tsx
// Replace the button className with:
className={cn(
  'bg-[var(--bg-elevated)] rounded-[16px] p-5 flex flex-col gap-4 w-full text-left border border-[var(--border-subtle)] shadow-[var(--shadow-subtle)] cursor-pointer transition-all duration-200 hover:bg-[var(--surface-hover)] hover:border-[var(--border-strong)] active:scale-[0.98]',
  className
)}
```

## 4. Missing Premium Fintech Elements

Premium neobanks anchor list views with a high-level financial aggregate summary to provide immediate context before the user scrolls through individual items.

```tsx
// AgentsList.tsx
// Insert this directly above the SegmentControl to provide aggregate financial context:
<div className="mb-5 mt-2">
  <p className="text-[13px] font-medium text-[var(--text-secondary)] mb-1">Total Daily Spend</p>
  <div className="flex items-baseline gap-1.5">
    <span className="text-[28px] font-semibold text-[var(--text-primary)] tracking-tight">
      ${filteredAgents.reduce((sum, a) => sum + parseFloat(a.budget?.daily_spent || '0'), 0).toFixed(2)}
    </span>
    <span className="text-[14px] font-medium text-[var(--text-tertiary)]">
      / ${filteredAgents.reduce((sum, a) => sum + parseFloat(a.budget?.daily_cap || '0'), 0).toFixed(2)} cap
    </span>
  </div>
</div>
```
