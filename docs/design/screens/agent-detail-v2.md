# Agent Detail Screen — v2 Design Spec

**Source**: `docs/design/redesign-v2.md` lines 1793–2090
**Screen title**: `Tally Wallet - Research Runner`
**Screen ID / label in source**: `individual agent screen`

---

## Purpose

The individual agent screen gives the user full control over a single AI agent. It shows the agent's current daily spending progress, exposes granular spending controls (limits and approval thresholds), and lists recent transaction history scoped to that agent. A primary action button saves any control changes.

---

## CSS Custom Properties (Design Tokens)

```css
:root {
  --bg-primary:             #FFFFFF;
  --bg-secondary:           #F8F9FA;
  --surface-hover:          #F2F2F7;
  --text-primary:           #111111;
  --text-secondary:         #8E8E93;
  --text-tertiary:          #C7C7CC;
  --accent-green:           #8FB5AA;
  --accent-green-dim:       rgba(143, 181, 170, 0.15);
  --accent-terracotta-dim:  rgba(217, 165, 139, 0.15);
  --black:                  #000000;
  --white:                  #FFFFFF;

  --radius-sm:   12px;
  --radius-md:   20px;
  --radius-lg:   32px;
  --radius-pill: 999px;

  --space-xs: 4px;
  --space-sm: 8px;
  --space-md: 16px;
  --space-lg: 24px;
  --space-xl: 32px;

  --font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
}
```

---

## App Container

```css
.app-container {
  width: 390px;
  height: 844px;
  background-color: var(--bg-primary);   /* #FFFFFF */
  position: relative;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  box-shadow: 0 0 0 10px #000, 0 20px 50px rgba(0,0,0,0.2);
  border-radius: 40px;
}
```

The body is centered on a `#F2F2F2` background using `display:flex; justify-content:center; align-items:center; min-height:100vh`.

---

## Typography Scale

| Class | Font Size | Font Weight | Color | Notes |
|---|---|---|---|---|
| `.text-display` | 34px | 700 | `--text-primary` | `letter-spacing: -1px`, `line-height: 1.1` |
| `.text-title` | 20px | 600 | `--text-primary` | — |
| `.text-subtitle` | 15px | 600 | `--text-primary` | — |
| `.text-body` | 14px | 400 | `--text-secondary` | `line-height: 1.4` |
| `.text-caption` | 11px | 600 | `--text-secondary` | `text-transform: uppercase`, `letter-spacing: 0.5px` |

---

## Layout Structure

```
app-container (390 x 844px, border-radius: 40px)
├── header.header-nav (sticky)
│   ├── button.back-btn (← chevron)
│   └── span.status-badge.status-active ("Running")
└── main.content.animate
    ├── Agent identity block
    ├── div.card — Daily Spend card
    ├── div — Spending Controls section
    │   ├── .limit-row — Daily Limit + stepper
    │   ├── .limit-row — Per Transaction + stepper
    │   └── .limit-row — Approval Threshold + toggle
    ├── div — Agent History section
    │   ├── .tx-item (Arxiv API Call)
    │   ├── .tx-item (Cross-Chain Query)
    │   └── .tx-item (Metadata Storage)
    └── button — "Save Changes" (primary CTA)
```

---

## Component Specifications

### Header Nav (`.header-nav`)

```css
.header-nav {
  padding: 60px 24px 16px;   /* 60px top accounts for iOS status bar safe area */
  display: flex;
  align-items: center;
  justify-content: space-between;
  position: sticky;
  top: 0;
  background: rgba(255,255,255,0.9);
  backdrop-filter: blur(10px);
  z-index: 10;
}
```

Contains:
- **Back button** (`.back-btn`): 40x40px circle, `border-radius: 50%`, `background: var(--bg-secondary)` (#F8F9FA). Contains a left-arrow SVG (20x20, stroke #000, stroke-width 2.5). Path: `M19 12H5M12 19l-7-7 7-7`.
- **Status badge** (`.status-badge.status-active`): Shows "Running". `padding: 6px 12px`, `border-radius: 8px`, `font-size: 12px`, `font-weight: 700`, `text-transform: uppercase`. Active state: `background: rgba(143,181,170,0.15)`, `color: #4A6E65`.

### Content Area (`.content`)

```css
.content {
  padding: 0 24px 100px;  /* 100px bottom provides space above bottom nav */
}
```

Includes entry animation class `.animate`:

```css
@keyframes fadeIn {
  from { opacity: 0; transform: translateY(10px); }
  to   { opacity: 1; transform: translateY(0); }
}
.animate { animation: fadeIn 0.4s ease forwards; }
```

### Agent Identity Block

Rendered inside `<div style="margin-top: 12px;">`:

- **Label**: `.text-caption` — "Local Agent"
- **Name**: `<h1 class="text-display">` — "Research Runner" (34px, 700 weight)
- **Description**: `<p class="text-body" style="margin-top: 8px;">` — "Scanning academic journals and cross-referencing market data on Base."

### Daily Spend Card (`.card`)

```css
.card {
  background: var(--bg-secondary);   /* #F8F9FA */
  border-radius: 24px;
  padding: 20px;
  margin-top: 24px;
}
```

Internal layout:
- Top row: `display: flex; justify-content: space-between; align-items: flex-end`
  - **Left**: label `.text-caption` "Daily Spend" + amount display `font-size: 28px; font-weight: 700` showing `$6.50 / $25.00` where the limit portion is `font-size: 14px; color: var(--text-secondary); font-weight: 500`
  - **Right**: Pause toggle (`.toggle-container`, id `pause-toggle`) — inactive state (agent is running)

#### Progress Bar

```css
.progress-bar {
  height: 6px;
  background: rgba(0,0,0,0.05);
  border-radius: 3px;
  margin-top: 12px;
  overflow: hidden;
}
.progress-fill {
  height: 100%;
  background: var(--accent-green);   /* #8FB5AA */
  width: 26%;
}
```

Below progress bar: two captions in a flex row:
- Left: `.text-caption` with `color: var(--accent-green)` — "26% Used"
- Right: `.text-caption` — "Reset in 14h"

### Toggle Switch Component

Two variants share common structure:

**Agent Detail (smaller) variant:**
```css
.toggle-container {
  width: 44px;
  height: 24px;
  background: #E9E9EB;
  border-radius: 12px;
  position: relative;
  cursor: pointer;
  transition: background 0.2s;
}
.toggle-container.active { background: var(--black); }

.toggle-knob {
  width: 20px;
  height: 20px;
  background: white;
  border-radius: 50%;
  position: absolute;
  top: 2px;
  left: 2px;
  transition: transform 0.2s;
  box-shadow: 0 1px 3px rgba(0,0,0,0.1);
}
.toggle-container.active .toggle-knob { transform: translateX(20px); }
```

- Off state: `background: #E9E9EB`, knob at `left: 2px`
- On state (class `active`): `background: #000000`, knob translated `+20px`

### Spending Controls Section

Container: `<div style="margin-top: 32px;">` with heading `.text-title` "Spending Controls"

#### Limit Row (`.limit-row`)

```css
.limit-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 0;
  border-bottom: 1px solid rgba(0,0,0,0.05);
}
.limit-row:last-child { border-bottom: none; }
```

Each row has:
- **Left**: label (`.text-subtitle`) + sub-label (`.text-body` at 12px)
- **Right**: stepper control OR toggle

#### Stepper Component (`.stepper`)

```css
.stepper {
  display: flex;
  align-items: center;
  gap: 12px;
  background: var(--white);
  padding: 4px;
  border-radius: 12px;
  border: 1px solid rgba(0,0,0,0.05);
}
```

Stepper button (`.step-btn`):
```css
.step-btn {
  width: 28px;
  height: 28px;
  border-radius: 8px;
  background: var(--bg-secondary);   /* #F8F9FA */
  border: none;
  font-size: 18px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
}
```

Value display (`.step-value`):
```css
.step-value {
  font-weight: 700;
  font-size: 14px;
  min-width: 50px;
  text-align: center;
}
```

**Control rows:**

| Row Label | Sub-label | Control | Value |
|---|---|---|---|
| Daily Limit | "Max spend per 24h" | Stepper | $25.00 |
| Per Transaction | "Auto-approve cap" | Stepper | $5.00 |
| Approval Threshold | "Prompt for any tx > $5.00" | Toggle (active/on) | — |

### Agent History Section

Container: `<div style="margin-top: 40px;">` with flex header row containing `.text-title` "Agent History" and a "Filter" link (`.text-caption`, `color: var(--black)`, `border-bottom: 1px solid var(--black)`, `cursor: pointer`).

#### Transaction Item (`.tx-item`)

```css
.tx-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 0;
  border-bottom: 1px solid var(--surface-hover);   /* #F2F2F7 */
}
```

Last item uses inline `border-bottom: none`.

Each item:
- **Left**: activity name (`.text-subtitle` at `font-weight: 500`) + timestamp/status (`.text-caption` at `font-size: 10px`)
- **Right**: amount (`.text-subtitle` at `font-weight: 700`)

**Sample data:**

| Activity | Timestamp | Status | Amount |
|---|---|---|---|
| Arxiv API Call | Today, 2:45 PM | Success | -$1.20 |
| Cross-Chain Query | Today, 11:20 AM | Success | -$3.80 |
| Metadata Storage | Yesterday, 9:15 PM | Success | -$1.50 |

### Primary CTA Button — "Save Changes"

```css
/* Inline styles on <button> */
width: 100%;
height: 56px;
background: var(--black);   /* #000000 */
color: white;
border-radius: 28px;        /* effectively pill for 56px height */
border: none;
font-weight: 600;
font-size: 16px;
margin-top: 32px;
```

---

## Interaction Notes

- The **pause toggle** in the Daily Spend card uses `onclick="this.classList.toggle('active')"` — toggling pauses/resumes the agent.
- The **Approval Threshold toggle** starts in `active` (on) state — meaning the agent will prompt for approval on transactions above $5.00.
- The **stepper buttons** show `−` and `+` glyphs; values are shown as currency strings.
- No bottom navigation bar is present on this screen (it is a detail/drill-down view).

---

## Screen Entry Animation

```css
@keyframes fadeIn {
  from { opacity: 0; transform: translateY(10px); }
  to   { opacity: 1; transform: translateY(0); }
}
.animate { animation: fadeIn 0.4s ease forwards; }
```

Applied to `<main class="content animate">`.
