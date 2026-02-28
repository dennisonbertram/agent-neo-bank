# App Layout System — v2 Design Spec

**Source**: `docs/design/redesign-v2.md` — extracted from all three screens (lines 1793–2666)

---

## Overview

The app uses a fixed-size mobile phone frame rendered inside a desktop browser window. The design simulates a physical iPhone (iPhone 14 Pro / standard size) with a visible device bezel. All screens share the same outer container dimensions and box-shadow treatment.

---

## Device Frame

### Outer Canvas

```css
body {
  margin: 0;
  padding: 0;
  font-family: var(--font-family);
  background-color: #F2F2F2;         /* Light gray canvas behind the device */
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 100vh;
  color: var(--text-primary);
}
```

The canvas is centered both horizontally and vertically within the viewport. This is a prototype/preview context — in a real Tauri app the container would fill the window.

### App Container (`.app-container`)

```css
.app-container {
  width: 390px;
  height: 844px;
  background-color: #FFFFFF;
  position: relative;
  display: flex;
  flex-direction: column;
  box-shadow: 0 0 0 10px #000, 0 20px 50px rgba(0,0,0,0.2);
  border-radius: 40px;
}
```

- **Width**: 390px — matches iPhone 14 logical resolution width
- **Height**: 844px — matches iPhone 14 logical resolution height
- **Border radius**: 40px — approximates iPhone rounded corners
- **Box shadow**: Two-layer shadow:
  - Inner layer: `0 0 0 10px #000` — 10px solid black ring simulating the device bezel
  - Outer layer: `0 20px 50px rgba(0,0,0,0.2)` — soft drop shadow for depth

The `overflow` property varies by screen:
- Agent detail (`.app-container`): `overflow-y: auto` — scroll is on the container itself
- Transaction detail and Settings: `overflow: hidden` — scroll is delegated to the inner `.screen` div

---

## Screen Scroll Container (`.screen`)

Used in the transaction detail and settings screens:

```css
.screen {
  flex: 1;
  overflow-y: auto;
  display: block;
}
```

Padding varies by screen:

| Screen | Top | Right | Bottom | Left |
|---|---|---|---|---|
| Transaction Detail | 60px | 24px | 40px | 24px |
| Settings | 60px | 24px | 100px | 24px |

The agent detail screen uses `.content` instead:
```css
.content {
  padding: 0 24px 100px;
}
```
With the header at `padding: 60px 24px 16px` (the 60px top is on the header, not the content).

---

## Safe Areas

### Top (Status Bar)

All screens reserve 60px at the top for the iOS status bar / notch area. This is applied via:
- Agent detail: header padding `padding: 60px 24px 16px` (top padding is on the sticky header)
- Transaction detail and Settings: screen padding `padding: 60px 24px ...`

The 60px value accounts for the iOS status bar (~44px) plus a small visual gap.

### Bottom (Home Indicator)

Screens with a bottom navigation bar use 100px bottom padding on the scroll container (`padding-bottom: 100px`), which is slightly larger than the nav bar height (84px) to ensure content is not clipped behind it.

Screens without a bottom nav (detail views) use 40px bottom padding — enough breathing room but no nav bar compensation needed.

---

## Sticky / Fixed Header Pattern

The agent detail screen uses a sticky header that stays visible while content scrolls beneath it:

```css
.header-nav {
  padding: 60px 24px 16px;
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

The frosted glass effect (`backdrop-filter: blur(10px)`) with 90% opacity white background lets content scroll behind the header visually while remaining readable.

---

## Bottom Navigation Bar (`.bottom-nav`)

Present only on top-level screens (settings is the only example in this range). Detail/drill-down screens omit it.

```css
.bottom-nav {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  height: 84px;
  background: rgba(255,255,255,0.95);
  backdrop-filter: blur(10px);
  border-top: 1px solid rgba(0,0,0,0.05);
  display: flex;
  justify-content: space-around;
  align-items: center;
  padding-bottom: 20px;
  z-index: 100;
}
```

- **Height**: 84px total (64px visible nav area + 20px home indicator safe area)
- **Background**: 95% opacity white with 10px blur — slightly more opaque than the header
- **Border**: 1px top border at 5% opacity black (very subtle)
- **Z-index**: 100 (highest in the stack — always above content)
- **Positioning**: `position: absolute` pinned to container bottom (not `fixed` — stays within the device frame)

### Nav Tabs

Five tabs, evenly spaced with `justify-content: space-around`:

```css
.nav-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  color: var(--text-secondary);   /* #8E8E93 — inactive */
  width: 60px;
}
.nav-item.active { color: var(--text-primary); }   /* #111111 */

.nav-icon { width: 24px; height: 24px; fill: currentColor; }
.nav-label { font-size: 10px; font-weight: 500; }
```

The active tab changes icon and label color from `#8E8E93` to `#111111`. No additional active indicator (no dot, underline, or background).

### Floating Action Button (Center Tab)

The center "Add" tab uses an elevated circular button:

```css
width: 56px;
height: 56px;
border-radius: 50%;
background: #000000;
margin-top: -40px;           /* Raises FAB 40px above nav baseline */
box-shadow: 0 4px 12px rgba(0,0,0,0.2);
```

The FAB extends 40px above the nav bar, visually protruding from the navigation rail.

---

## Content Padding System

Standard horizontal content padding: **24px** on left and right. This is consistent across all screens.

Vertical spacing uses the token system:

| Token | Value |
|---|---|
| `--space-xs` | 4px |
| `--space-sm` | 8px |
| `--space-md` | 16px |
| `--space-lg` | 24px |
| `--space-xl` | 32px |

---

## Screen Entry Animation

All screens use the same `fadeIn` keyframe:

```css
@keyframes fadeIn {
  from { opacity: 0; transform: translateY(10px); }
  to   { opacity: 1; transform: translateY(0); }
}
```

Applied via different class names depending on screen:
- Agent detail: `.animate { animation: fadeIn 0.4s ease forwards; }` — applied to `<main>`
- Transaction detail: `.animate-in { animation: fadeIn 0.4s ease forwards; }` — applied to `.screen`
- Settings: No animation class used

Duration: `0.4s ease`, `forwards` fill mode (stays at final state).

---

## Screen Type Classification

| Screen Type | Has Back Button | Has Bottom Nav | Header Style | Scroll Container |
|---|---|---|---|---|
| Top-level (Settings) | Text button "← Home" | Yes | Inline in scroll area | `.screen` (flex:1) |
| Detail drill-down (Agent, Transaction) | Icon button or text "Details" | No | Sticky `.header-nav` or inline | `.content` or `.screen` |

---

## Global CSS Reset

```css
* {
  box-sizing: border-box;
  -webkit-tap-highlight-color: transparent;
}
```

`box-sizing: border-box` means all padding and borders are included within declared widths/heights. `-webkit-tap-highlight-color: transparent` suppresses the iOS default blue tap flash on interactive elements.

---

## Font Stack

```css
--font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
```

System font stack that resolves to SF Pro on Apple devices, Segoe UI on Windows, Roboto on Android. This ensures the UI reads as native on each platform.

Monospace override (transaction detail screen only):
```css
.text-mono {
  font-family: "SF Mono", "Menlo", monospace;
}
```

Used for technical identifiers (transaction request IDs).

---

## Color System Summary

### Core Palette

| Variable | Hex / Value | Usage |
|---|---|---|
| `--bg-primary` | `#FFFFFF` | App container background, cards |
| `--bg-secondary` | `#F8F9FA` | Card fills, input backgrounds |
| `--surface-hover` | `#F2F2F7` | Dividers, hover states |
| `--text-primary` | `#111111` | Body text, headings |
| `--text-secondary` | `#8E8E93` | Labels, captions, inactive nav |
| `--text-tertiary` | `#C7C7CC` | Placeholder text, chevrons |
| `--black` | `#000000` | Toggle active bg, FAB, primary buttons |
| `--white` | `#FFFFFF` | Toggle knobs, button text |

### Accent Palette

| Variable | Hex | Usage |
|---|---|---|
| `--accent-green` | `#8FB5AA` | Progress fill, agent avatar bg, active indicators |
| `--accent-green-dim` | `rgba(143,181,170,0.15)` | Tag backgrounds, badge fills |
| `--accent-yellow` | `#F2D48C` | Available (not used in visible HTML of these screens) |
| `--accent-yellow-dim` | `rgba(242,212,140,0.15)` | Available |
| `--accent-terracotta` | `#D9A58B` | Profile avatar background |
| `--accent-terracotta-dim` | `rgba(217,165,139,0.15)` | Available |
| `--accent-blue` | `#BCCCDC` | Available (not used in visible HTML) |

### Semantic Colors (Not in CSS vars)

| Color | Hex | Usage |
|---|---|---|
| Danger | `#E5484D` | `.danger-text` — Reset connection action |
| Active badge text | `#4A6E65` | Status badge and tag text on green-dim background |
| Canvas background | `#F2F2F2` | Body background (device preview frame) |
