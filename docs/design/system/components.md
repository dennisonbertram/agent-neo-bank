# Design System: Reusable Component Patterns (v2)

Extracted from `docs/design/redesign-v2.md` (lines 876–1792).
These patterns appear across multiple screens and form the core component library.

---

## Design Tokens

All screens share a common set of CSS custom properties.

### Colors

```css
/* Backgrounds */
--bg-primary:           #FFFFFF;
--bg-secondary:         #F8F9FA;
--surface-hover:        #F2F2F7;

/* Text */
--text-primary:         #111111;
--text-secondary:       #8E8E93;
--text-tertiary:        #C7C7CC;

/* Accent palette */
--accent-green:         #8FB5AA;
--accent-yellow:        #F2D48C;
--accent-terracotta:    #D9A58B;
--accent-blue:          #BCCCDC;

/* Accent dims (15% opacity backgrounds) */
--accent-green-dim:     rgba(143, 181, 170, 0.15);
--accent-yellow-dim:    rgba(242, 212, 140, 0.15);
--accent-terracotta-dim: rgba(217, 165, 139, 0.15);

/* Monotones */
--black:                #000000;
--white:                #FFFFFF;
```

**Semantic color usage:**
- Positive/incoming amounts: `#4A6E65` (dark green)
- Negative/outgoing amounts: `#111111` (text-primary, neutral)
- Link/action color: `#0052FF` (Base blue — used for "View All", network badge)
- Active status text: `#4A6E65`
- Pending status text: `#8F7843`
- Paused status text: `#8F6652`

### Spacing

```css
--space-sm:   8px;
--space-md:   16px;
--space-lg:   24px;
--space-xl:   32px;
```

### Border Radius

```css
--radius-sm:   12px;
--radius-md:   20px;
--radius-lg:   32px;
--radius-pill: 999px;
```

### Typography

```css
--font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
/* Monospace */
font-family: "SF Mono", "Menlo", monospace;
```

---

## App Container (Mobile Frame)

All screens use the same outer container simulating an iPhone 14 (390×844):

```css
.app-container {
  width: 390px;
  height: 844px;
  background-color: #FFFFFF;
  position: relative;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  box-shadow: 0 0 0 10px #000, 0 20px 50px rgba(0,0,0,0.2);
  border-radius: 40px;
}
```

The double box-shadow creates a 10px-wide black bezel around the device frame.

---

## Bottom Navigation Bar

Used on: Dashboard, Agents List.

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
  padding-bottom: 20px;   /* home indicator clearance */
  z-index: 100;           /* dashboard only; agents list omits z-index */
}
```

### Nav Tab Item

```css
.nav-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  color: #8E8E93;   /* text-secondary — inactive state */
  width: 60px;
}

.nav-item.active {
  color: #111111;   /* text-primary — active state */
}

.nav-label {
  font-size: 10px;
  font-weight: 600;
}
```

Nav icon sizes by screen:
- Dashboard: `width: 22px; height: 22px;`
- Agents List: `width: 24px; height: 24px;`

Both: `fill: none; stroke: currentColor; stroke-width: 2;`

### Tab Definitions

**Dashboard tabs** (HOME active):

| Position | Label    | Icon Type           |
|----------|----------|---------------------|
| 1        | HOME     | House               |
| 2        | AGENTS   | Person/group        |
| 3        | LIMITS   | Activity/pulse line |
| 4        | SETTINGS | Gear/cog            |

**Agents List tabs** (AGENTS active, includes FAB):

| Position | Label   | Type             |
|----------|---------|------------------|
| 1        | Home    | House            |
| 2        | Agents  | Stacked layers   |
| 3        | —       | FAB (center)     |
| 4        | Stats   | Activity pulse   |
| 5        | Wallet  | Card/wallet      |

---

## FAB Button (Floating Action Button)

Used on: Agents List (center of bottom nav).

```css
.nav-fab {
  width: 56px;
  height: 56px;
  border-radius: 50%;
  background-color: #000000;
  color: #FFFFFF;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-top: -28px;   /* lifts 50% of height above nav bar */
  box-shadow: 0 8px 24px rgba(0,0,0,0.15);
}
```

- 56×56px black circle
- White plus (+) SVG: `stroke: white; stroke-width: 3; size: 24x24`
- Negative top margin (`-28px`) makes it protrude upward from the nav bar
- Shadow: `0 8px 24px rgba(0,0,0,0.15)`
- Placed as a direct child of `.bottom-nav` between the second and third tab items

---

## Agent Card

Used on: Agents List.

```css
.agent-card {
  background: #F8F9FA;   /* --bg-secondary */
  border-radius: 20px;   /* --radius-md */
  padding: 20px;
  margin-bottom: 16px;
  display: flex;
  flex-direction: column;
  gap: 16px;
  transition: transform 0.2s ease;
  cursor: pointer;
}

.agent-card:active { transform: scale(0.98); }
```

**Agent Card Header Row:**
```css
.agent-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
}

.agent-info {
  display: flex;
  align-items: center;
  gap: 12px;
}
```

**Agent Icon:**
```css
.agent-icon {
  width: 44px;
  height: 44px;
  border-radius: 14px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #000000;
}
```

Icon backgrounds use solid accent colors (not dim variants):
- Green: `#8FB5AA`
- Yellow: `#F2D48C`
- Blue: `#BCCCDC`
- Terracotta: `#D9A58B`

---

## Status Pill / Badge

Used on: Agents List (status), Onboarding 2 (file change tags).

### Agent Status Pills

```css
.status-pill {
  padding: 4px 10px;
  border-radius: 8px;
  font-size: 11px;
  font-weight: 700;
  text-transform: uppercase;
}

.status-active  { background: rgba(143,181,170,0.15);  color: #4A6E65; }
.status-pending { background: rgba(242,212,140,0.15);  color: #8F7843; }
.status-paused  { background: rgba(217,165,139,0.15);  color: #8F6652; }
```

### File Change Status Tag (Onboarding)

```css
.status-tag {
  font-size: 10px;
  font-weight: 700;
  padding: 4px 8px;
  border-radius: 4px;
  background: rgba(143, 181, 170, 0.15);   /* accent-green-dim */
  color: #4A6E65;
  text-transform: uppercase;
}
```

### Inline Code/Type Tags (Dashboard Activity)

Applied using `.text-mono` class inline:
```css
.text-mono {
  font-family: "SF Mono", "Menlo", monospace;
  font-size: 12px;
  background: #F8F9FA;    /* --bg-secondary */
  padding: 4px 8px;
  border-radius: 6px;
}
```

Used for activity type labels: "API FEE", "GAS", "SWAP". Also smaller inline: `font-size: 10px; padding: 2px 6px;`

---

## Spending Progress Bar

Used on: Agent cards.

```css
.spending-bar-container {
  height: 6px;
  width: 100%;
  background: rgba(0,0,0,0.05);
  border-radius: 3px;
  overflow: hidden;
}

.spending-bar-fill {
  height: 100%;
  border-radius: 3px;
  /* background color set per agent accent */
}

.spending-labels {
  display: flex;
  justify-content: space-between;
  font-size: 13px;
  font-weight: 500;
}
```

Label patterns:
- Active: `"$X.XX spent"` (`color: #111111`) / `"$Y.YY limit"` (`color: #8E8E93`)
- Paused: `"Spending disabled"` (`color: #8E8E93`) / limit shown in secondary
- Pending: Label row replaced entirely with italic status text

Bar opacity:
- Active: full opacity
- Paused: `opacity: 0.3`

---

## Balance Card

Used on: Dashboard.

```css
.balance-card {
  background: #000000;
  color: #FFFFFF;
  border-radius: 32px;    /* --radius-lg */
  padding: 32px 24px;
  margin-bottom: 24px;
  position: relative;
  overflow: hidden;
}

/* Decorative green glow overlay */
.balance-card::after {
  content: "";
  position: absolute;
  top: -50%;
  right: -20%;
  width: 200px;
  height: 200px;
  background: radial-gradient(circle, rgba(143, 181, 170, 0.2) 0%, transparent 70%);
  pointer-events: none;
}
```

---

## Action Buttons

### Full-Width CTA Buttons (Onboarding)

```css
.btn {
  height: 56px;
  border-radius: 999px;    /* pill */
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 600;
  font-size: 16px;
  cursor: pointer;
  width: 100%;
  border: none;
  transition: transform 0.1s;
}

.btn:active { transform: scale(0.98); }

.btn-primary {
  background-color: #000000;
  color: #FFFFFF;
}

.btn-outline {
  background: transparent;
  border: 1px solid #C7C7CC;   /* --text-tertiary */
  color: #111111;
}
```

### Quick-Action Buttons (Dashboard)

```css
.btn-action {
  flex: 1;
  height: 52px;
  border-radius: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  font-weight: 600;
  font-size: 15px;
  border: none;
  cursor: pointer;
  background: #F8F9FA;    /* --bg-secondary */
  color: #111111;
}
```

---

## Segment Control

Used on: Agents List.

```css
.segment-control {
  background-color: #F8F9FA;
  border-radius: 999px;
  padding: 4px;
  display: flex;
  margin-bottom: 24px;
}

.segment-opt {
  flex: 1;
  text-align: center;
  padding: 8px 0;
  font-size: 14px;
  font-weight: 600;
  color: #8E8E93;
  border-radius: 999px;
}

.segment-opt.active {
  background-color: #FFFFFF;
  color: #111111;
  box-shadow: 0 2px 8px rgba(0,0,0,0.06);
}
```

Pattern: outer pill-shaped track in bg-secondary, inner active option is white with a soft shadow. All options take equal width (`flex: 1`).

---

## Expandable Card Section

Used on: Onboarding 2 skill card.

```css
.skill-card {
  background: #F8F9FA;
  border-radius: 20px;
  padding: 20px;
  margin-top: 32px;
}

.expand-panel {
  border-top: 1px solid rgba(0,0,0,0.05);
  margin-top: 16px;
  padding-top: 16px;
}
```

Toggle pattern: header row with label + chevron SVG; chevron rotates `0deg` (open) / `-90deg` (closed) via CSS transform. Content toggled with JS `display: block / none`.

---

## Agent Avatar (Dashboard Activity)

```css
.agent-avatar {
  width: 44px;
  height: 44px;
  border-radius: 14px;
  display: flex;
  align-items: center;
  justify-content: center;
}
```

Same dimensions and border-radius as the `.agent-icon` on the agent card. Background uses dim accent colors (15% opacity) in the activity feed vs solid accent colors in the agent list.

---

## Success State Circle

Used on: Onboarding 2 success state.

```css
.success-check {
  width: 64px;
  height: 64px;
  background: #8FB5AA;    /* --accent-green */
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  margin: 0 auto 24px auto;
}
```

Contains a white checkmark: `stroke: white; stroke-width: 3; size: 32x32`.

---

## Screen Entry Animation

Used on screens with `.animate-in` class.

```css
@keyframes slideUp {
  from { opacity: 0; transform: translateY(20px); }
  to   { opacity: 1; transform: translateY(0); }
}

.animate-in {
  animation: slideUp 0.5s ease forwards;
}
```

---

## Navigation Structure

```
App
├── Onboarding 2 (no bottom nav)
│   ├── install-state
│   └── success-state
├── Dashboard (HOME active)
│   └── Bottom Nav: HOME | AGENTS | LIMITS | SETTINGS
└── Agents List (AGENTS active)
    └── Bottom Nav: Home | Agents | [FAB] | Stats | Wallet
```

Key navigation differences between screens:
- Dashboard nav: 4 tabs, labels uppercase, no FAB, icon size 22px
- Agents List nav: 4 tabs + FAB (5 items), labels title-case, icon size 24px
- Agents tab icon differs: group/people (dashboard) vs stacked-layers (agents list)
