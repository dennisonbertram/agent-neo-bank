# Design Tokens — Tally Agentic Wallet

Extracted from `docs/design/redesign-v2.md` (onboarding prototype CSS).
These are the canonical CSS custom properties that define the entire visual language.

---

## App Shell Dimensions

The app container mimics a mobile device at a fixed size:

| Property | Value |
|---|---|
| Width | `390px` (max-width) |
| Height | `844px` (max-height) |
| Container border-radius | `40px` (design artifact — exclude in Tauri) |
| Body background | `#F2F2F2` (outer chrome — exclude in Tauri) |

In Tauri the window itself is the phone shell. Set the window to `390 x 844` and do not apply `border-radius` or `box-shadow` to `.app-container`.

---

## Color Tokens

### Backgrounds

| Token | Value | Usage |
|---|---|---|
| `--bg-primary` | `#FFFFFF` | Main screen background |
| `--bg-secondary` | `#F8F9FA` | Cards, input fields, list item backgrounds |
| `--surface-hover` | `#F2F2F7` | Dividers, hover states, toggle tracks |

### Text

| Token | Value | Usage |
|---|---|---|
| `--text-primary` | `#111111` | Headings, primary labels, active nav |
| `--text-secondary` | `#8E8E93` | Body copy, captions, inactive nav |
| `--text-tertiary` | `#C7C7CC` | Placeholder text, inactive indicators |

### Accents (Pastel Palette)

| Token | Value | Hex |
|---|---|---|
| `--accent-green` | `#8FB5AA` | Sage green — primary accent, Research agent pill |
| `--accent-yellow` | `#F2D48C` | Warm yellow — secondary accent, Deploy Bot pill |
| `--accent-terracotta` | `#D9A58B` | Terracotta — tertiary accent, Treasury pill |
| `--accent-blue` | `#BCCCDC` | Muted blue — quaternary accent |

### Accent Dim (15% opacity fills for badges/tags)

| Token | Value |
|---|---|
| `--accent-green-dim` | `rgba(143, 181, 170, 0.15)` |
| `--accent-yellow-dim` | `rgba(242, 212, 140, 0.15)` |
| `--accent-terracotta-dim` | `rgba(217, 165, 139, 0.15)` |
| `--accent-blue-dim` | `rgba(188, 204, 220, 0.15)` |

### Neutrals

| Token | Value |
|---|---|
| `--black` | `#000000` |
| `--white` | `#FFFFFF` |

### Status Badge Text Colors (paired with dim backgrounds)

| State | Background Token | Text Color |
|---|---|---|
| Active | `--accent-green-dim` | `#4A6E65` |
| Pending | `--accent-yellow-dim` | `#8F7843` |
| Paused / Error | `--accent-terracotta-dim` | `#8F6652` |

### Subtle Pill Text Colors (for `.pill-*-subtle` classes)

| Class | Background | Text |
|---|---|---|
| `.pill-green-subtle` | `--accent-green-dim` | `#4A6E65` |
| `.pill-yellow-subtle` | `--accent-yellow-dim` | `#8F7843` |
| `.pill-terracotta-subtle` | `--accent-terracotta-dim` | `#8F6652` |

---

## Border Radius

| Token | Value | Usage |
|---|---|---|
| `--radius-sm` | `12px` | OTP digit boxes, transaction icon containers, small cards |
| `--radius-md` | `20px` | Input groups, QR code container, deposit cards |
| `--radius-lg` | `32px` | Budget card on agent detail screen |
| `--radius-pill` | `999px` | Buttons, segment controls, agent pills, status badges |

Additional radii used inline (not tokenized):
- `16px` — local skill list card
- `24px` — agent detail budget card
- `20px` — add funds QR wrapper
- `6px` — status badge (`.status-badge`)
- `4px` — progress bar fill
- `8px` — warning pill on add-funds

---

## Spacing

| Token | Value |
|---|---|
| `--space-xs` | `4px` |
| `--space-sm` | `8px` |
| `--space-md` | `16px` |
| `--space-lg` | `24px` |
| `--space-xl` | `32px` |

### Screen Padding (`.screen`)
- Left/right: `var(--space-lg)` = `24px`
- Top: `var(--space-md)` = `16px`
- Bottom: `100px` (clears bottom nav)

### Screen-specific overrides
- `screen-add-funds`: `padding-top: 80px` (no persistent header)
- `screen-agent-detail`: header hidden, back button inline

---

## Shadows

| Token | Value | Usage |
|---|---|---|
| `--shadow-subtle` | `0 4px 24px rgba(0,0,0,0.04)` | Cards, QR code wrapper |
| `--shadow-float` | `0 8px 32px rgba(0,0,0,0.08)` | Floating elements, modals |

Additional shadows used inline:
- Segment active tab: `0 2px 8px rgba(0,0,0,0.08)`
- FAB button: `0 8px 24px rgba(0,0,0,0.15)`
- Toggle thumb: `0 2px 4px rgba(0,0,0,0.1)`

---

## Typography

### Font Family

```
--font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
```

Monospace (used for addresses, file names):
```
"SF Mono", "Menlo", monospace
```

### Type Scale

| Class | Size | Weight | Letter-Spacing | Line-Height | Color |
|---|---|---|---|---|---|
| `.text-display` | `42px` | `600` | `-1px` | `1.1` | `--text-primary` |
| `.text-title` | `22px` | `600` | `-0.5px` | — | `--text-primary` |
| `.text-subtitle` | `17px` | `500` | `-0.3px` | — | `--text-primary` |
| `.text-body` | `15px` | `400` | — | `1.5` | `--text-secondary` |
| `.text-caption` | `13px` | `500` | `+0.5px` | — | `--text-secondary` (uppercase) |
| `.text-mono` | `13px` | — | — | — | inherited |

### Inline overrides seen in screens

| Usage | Size | Weight | Color |
|---|---|---|---|
| Balance display (home) | `42px` | `600` | `--text-primary` |
| Balance currency suffix | `24px` | — | `--text-secondary` |
| Agent detail budget | `32px` | `600` | `--text-primary` |
| OTP digit boxes | `24px` | `600` | `--text-primary` |
| Wallet address pill | `12px` (monospace) | — | `--text-secondary` |
| Local skill file label | `13px` (monospace) | — | inherited |
| Local skill status | `12px` | `600` | `--accent-green` |
| Transaction label | `15px` | `500` | `--text-primary` |
| Transaction sub-label | `11px` | `500` | `--text-secondary` |
| Agent detail title | `28px` | `600` | `--text-primary` |

---

## Animation

```css
@keyframes fadeIn {
  from { opacity: 0; transform: translateY(10px); }
  to   { opacity: 1; transform: translateY(0); }
}

.animate-in {
  animation: fadeIn 0.4s ease forwards;
}
```

Screen transition: opacity fade triggered by JavaScript — `display: block` then class `.active` added after `10ms` delay (`transition: opacity 0.2s ease`).

---

## Global Resets

```css
* {
  box-sizing: border-box;
  -webkit-tap-highlight-color: transparent;
}

h1, h2, h3, p { margin: 0; }
```

---

## Utility Classes

### Flexbox

| Class | Style |
|---|---|
| `.flex-row` | `display: flex; align-items: center;` |
| `.flex-col` | `display: flex; flex-direction: column;` |
| `.justify-between` | `justify-content: space-between;` |
| `.justify-center` | `justify-content: center;` |
| `.gap-sm` | `gap: 8px` |
| `.gap-md` | `gap: 16px` |
| `.gap-lg` | `gap: 24px` |

### Spacing

| Class | Style |
|---|---|
| `.mt-md` | `margin-top: 16px` |
| `.mt-lg` | `margin-top: 24px` |
| `.mt-xl` | `margin-top: 32px` |
| `.mb-sm` | `margin-bottom: 8px` |
| `.mb-md` | `margin-bottom: 16px` |
| `.mb-lg` | `margin-bottom: 24px` |
| `.pt-xl` | `padding-top: 32px` |
| `.text-center` | `text-align: center` |

---

## Component Tokens Summary

### Button Heights
- `.btn` (standard): `56px` tall, `border-radius: 999px`, `font-size: 16px`, `font-weight: 600`, `width: 100%`
- `.btn-sm`: `36px` tall, `padding: 0 16px`, `font-size: 13px`, `width: auto`

### Agent Pill (DNA list item)
- Pill container height: `48px`, `border-radius: 999px`, `padding-right: 16px`, `min-width: 140px`
- Icon box: `32px x 32px`, `margin-left: 8px`
- Label: `font-size: 14px`, `font-weight: 600`, `margin-left: 8px`
- Value: `font-size: 15px`, `font-weight: 600`
- Sub-value: `font-size: 12px`, color `--text-secondary`

### Bottom Nav
- Height: `84px`
- Padding-bottom: `20px` (safe area compensation)
- Background: `rgba(255,255,255,0.95)` with `backdrop-filter: blur(10px)`
- Border-top: `1px solid rgba(0,0,0,0.05)`
- Nav item width: `60px`
- Nav icon: `24px x 24px`
- Nav label: `10px`, `font-weight: 500`
- FAB: `56px` circle, `background: #000000`, `margin-top: -28px` (floats above bar), `box-shadow: 0 8px 24px rgba(0,0,0,0.15)`

### Input Group
- Background: `--bg-secondary`
- Border-radius: `--radius-md` (20px)
- Padding: `16px`
- Label: `12px`, color `--text-secondary`, `margin-bottom: 8px`
- Field: `16px`, `background: transparent`, `border: none`, `outline: none`

### OTP Digit Box
- Size: `48px x 56px`
- Background: `--bg-secondary`
- Border-radius: `12px`
- Font-size: `24px`, font-weight: `600`

### Toggle Switch
- Track: `50px x 30px`, `border-radius: 30px`
  - Off: `background: --bg-secondary`
  - On: `background: --black`
- Thumb: `26px x 26px` circle, `background: white`, `box-shadow: 0 2px 4px rgba(0,0,0,0.1)`
  - Off position: `top: 2px; left: 2px`
  - On position: `top: 2px; right: 2px`

### Status Badge
- Padding: `4px 8px`
- Border-radius: `6px`
- Font-size: `11px`, font-weight: `600`, `text-transform: uppercase`

### Progress Bar
- Track: `height: 8px`, `border-radius: 4px`, `background: rgba(0,0,0,0.05)`
- Fill: `background: --accent-green`, `border-radius: 4px`

### Segment Control
- Container: `background: --bg-secondary`, `border-radius: 999px`, `padding: 4px`
- Option: `flex: 1`, `padding: 8px 0`, `font-size: 14px`, `font-weight: 500`, `color: --text-secondary`
- Active option: `background: white`, `color: --text-primary`, `box-shadow: 0 2px 8px rgba(0,0,0,0.08)`

### Icon Button (header)
- Size: `40px x 40px`, `border-radius: 50%`
- Border: `1px solid --surface-hover`
- Background: `transparent`

### Avatar
- Size: `40px x 40px`, `border-radius: 50%`
- Background: `--bg-secondary` with inline SVG fill `#D9A58B` (terracotta person icon)
- Background-size: `60%`, centered

### Onboarding Indicator
- Inactive: `6px x 6px` circle, `background: --text-tertiary`
- Active: `24px x 6px` pill (width expands), `background: --text-primary`, `border-radius: 4px`
- Gap between indicators: `6px`

### App Logo Icon (onboarding)
- Container: `60px x 60px`, `background: --accent-green`, `border-radius: 20px`
- Icon: `32px x 32px` SVG (hexagon with circle), `stroke: black`, `stroke-width: 2`

### Transaction Row
- Padding: `12px 0`, `border-bottom: 1px solid --surface-hover` (except last)
- Icon container: `40px x 40px`, `background: --bg-secondary`, `border-radius: 12px`
- Label: `15px`, `font-weight: 500`
- Sublabel: `11px`, `font-weight: 500`, `color: --text-secondary`
- Amount: `17px`, `font-weight: 500`
