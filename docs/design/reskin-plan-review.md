# Reskin Plan — External Design Review

**Reviewer:** Gemini 3.1 Pro (Preview)
**Date:** 2026-03-01
**Context:** 390x640px fixed-window Tauri desktop neobank, React + Tailwind CSS v4
**Inputs reviewed:** Design system, CSS tokens, Privacy.com dashboard reference
**Note:** The reskin plan file (`dynamic-gliding-harbor.md`) was not found at the expected path; review is based on the design system, tokens, and Privacy.com reference.

---

### 1. CRITICAL ISSUES (390x640px Layout)

- **Sidebar Navigation:** A 240px sidebar consumes 61% of your 390px width. Even a 72px collapsed sidebar leaves only 318px for content, forcing severe cramping.
  - **Fix:** Abandon sidebar. Use a fixed bottom tab bar (`h-16`, 4-5 icons, text `text-[10px]`) or a top hamburger/drawer.
- **Grid Systems:** Plan specifies `3-col` and `5-col` grids (e.g., `<div className="grid grid-cols-5 gap-6">`). This will violently overflow 390px.
  - **Fix:** Use `flex-col` or `grid-cols-1` with `gap-4` for all lists and cards.
- **Detail Page Layout:** `grid-template-columns: 426px 1fr` is specified. 426px is wider than the entire window.
  - **Fix:** Stack the layout vertically. Card visual on top, transaction list below.
- **Scrollbars:** Windows/Linux default scrollbars are ~16px wide, which will break 390px pixel-perfect math.
  - **Fix:** Add `::-webkit-scrollbar { width: 4px; display: none; }` to hide them or make them minimal overlays.
- **Window Drag Region:** Tauri fixed-windows require a draggable area if frameless.
  - **Fix:** Allocate top `h-12` as a drag zone with Tailwind class `[data-tauri-drag-region]`.

### 2. COLOR REVIEW

- **Contrast & Base:** `#0F1117` (page) and `#1A1D27` (container) are excellent. `#F1F1F4` for primary text is perfect (WCAG AAA).
- **Mobile Density Warning:** Stacking multiple `#1A1D27` container cards on a 390px screen can feel bulky.
  - **Fix:** For continuous lists (e.g., Transactions), use the page background (`#0F1117`) and separate rows with `border-b border-[#2D3142]` rather than placing the whole list inside a `#1A1D27` card.
- **Warning Token:** `--warning-main: #F59E0B`. Good, but against `#1A1D27` it has a contrast ratio of ~2.9:1.
  - **Fix:** Shift to `#FBBF24` (`amber-400`) for text/icons to hit >4.5:1 WCAG AA.

### 3. TYPOGRAPHY SCALE REVIEW

- **`mono-large` (48px):** Too large. "$12,345.67" at 48px monospace is ~280px wide. With paddings, it will text-wrap or overflow.
  - **Fix:** Reduce to `text-4xl` (`36px`) or `text-3xl` (`30px`) for the main balance.
- **`display-1` (36px):** Takes up too much horizontal space for a mobile hero greeting.
  - **Fix:** Reduce to `text-2xl font-bold` (`24px`).
- **`display-2` (28px) & `heading-1` (24px):** Shift down one step.
  - **Fix:** Page titles = `text-xl font-semibold` (`20px`). Card titles = `text-base font-semibold` (`16px`).
- **`body` (14px) & `body-small` (13px):** Keep these. They are perfect for 390px readability.

### 4. SPACING CONCERNS

- **Page Gutters:** 40px padding on left/right (`px-10`) leaves only 310px for content.
  - **Fix:** Change to `px-4` (16px) or `px-5` (20px). This provides 350px-358px of usable width.
- **Section Gaps:** 24px (`gap-6`) and 32px (`gap-8`) eat up too much of the 640px vertical height.
  - **Fix:** Use `gap-4` (16px) between related cards, and `gap-6` (24px) between major page sections.
- **Hero Padding:** 40px (`p-10`) is a massive waste of mobile space.
  - **Fix:** Reduce to `p-5` (20px) or `p-6` (24px).

### 5. COMPONENT SIZING

- **Card Padding:** 24px (`p-6`) makes the inner content area too narrow (390 - 32 gutter - 48 padding = 310px).
  - **Fix:** Change default card padding to `p-4` (16px).
- **Button Heights:** 40px default is slightly small for touch/click on a mobile-proportioned interface.
  - **Fix:** Make default buttons `h-11` (44px) or `h-12` (48px) for better hit areas.
- **Toast Notifications:** "max-width 360px" + 16px right margin = 376px. If aligned to the right, it leaves 14px on the left (asymmetrical).
  - **Fix:** Change to `w-[calc(100vw-32px)]` centered, with `left-4 right-4`.
- **OTP Input:** 6 inputs of 48px + 8px gaps = 328px total. This fits perfectly within 390px (31px margin each side). Keep this.

### 6. MISSING PIECES

- **Truncation:** At 390px, agent names, merchant names, and wallet addresses *will* wrap unpredictably.
  - **Fix:** Enforce `truncate` (whitespace-nowrap overflow-hidden text-ellipsis) on all list item titles.
- **Modals / Drawers:** Standard centered modals look awkward on 390px mobile-style views.
  - **Fix:** Replace centered modals with Bottom Sheets (`fixed bottom-0 w-full rounded-t-2xl slide-up-animation`) for actions like "Kill Switch" or "Add Agent".
- **Sticky Headers:** With only 640px height, scrolling down a list of 20 transactions will hide the balance.
  - **Fix:** Implement a collapsing header pattern or make the Top Nav + Page Title sticky (`sticky top-0 z-20 bg-page-background/90 backdrop-blur-md`).
