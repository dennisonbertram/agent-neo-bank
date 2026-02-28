# Agents List Screen Spec (v2)

Source lines: 1465–1792 of `docs/design/redesign-v2.md`
HTML title: `Agent Wallet - Agents List`

---

## Overview

The Agents List screen shows all of the user's active, pending, and paused agents. Each agent is displayed as a card with a status pill, spending bar, and contextual sub-information. A segmented control filters the list. The bottom nav has a FAB (floating action button) in its center for adding a new agent.

---

## Design Tokens (CSS Variables)

```css
:root {
  --bg-primary:              #FFFFFF;
  --bg-secondary:            #F8F9FA;
  --surface-hover:           #F2F2F7;
  --text-primary:            #111111;
  --text-secondary:          #8E8E93;
  --text-tertiary:           #C7C7CC;
  --accent-green:            #8FB5AA;
  --accent-yellow:           #F2D48C;
  --accent-terracotta:       #D9A58B;
  --accent-blue:             #BCCCDC;
  --accent-green-dim:        rgba(143, 181, 170, 0.15);
  --accent-yellow-dim:       rgba(242, 212, 140, 0.15);
  --accent-terracotta-dim:   rgba(217, 165, 139, 0.15);
  --black:                   #000000;
  --white:                   #FFFFFF;
  --radius-sm:               12px;
  --radius-md:               20px;
  --radius-lg:               32px;
  --radius-pill:             999px;
  --space-md:                16px;
  --space-lg:                24px;
  --space-xl:                32px;
  --font-family:             -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
}
```

Note: This screen adds `--accent-yellow-dim` and `--accent-terracotta-dim` as explicit variables (vs inline rgba in the dashboard).

---

## App Container

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

---

## Typography

| Class            | Size  | Weight | Color         | Notes                          |
|------------------|-------|--------|---------------|--------------------------------|
| `.text-title`    | 24px  | 700    | inherited     | `letter-spacing: -0.5px`      |
| `.text-caption`  | 13px  | 500    | `#8E8E93`     | `text-transform: uppercase; letter-spacing: 0.5px;` |
| `.text-subtitle` | 16px  | 600    | inherited     | Agent card name                |
| `.text-body`     | 14px  | —      | `#8E8E93`     | Agent card description         |

Note: Typography scale differs slightly from dashboard (title is 24px/700 here vs 20px/600 there).

---

## Layout Structure (Top to Bottom)

1. **Header** — Screen title + settings icon
2. **Screen / Main Content** — Segment control + agent cards list
3. **Bottom Navigation** — 4 tabs with center FAB

---

## 1. Header

```css
.header {
  padding: 60px 24px 16px 24px;
  display: flex;
  justify-content: space-between;
  align-items: center;
}
```

```html
<header class="header">
  <div class="text-title">My Agents</div>
  <div style="
    width: 40px;
    height: 40px;
    border-radius: 50%;
    background: #F8F9FA;   /* --bg-secondary */
    border: 1px solid #F2F2F7;   /* --surface-hover */
    display: flex;
    align-items: center;
    justify-content: center;
  ">
    <!-- Settings/gear SVG: 20x20 -->
  </div>
</header>
```

- Title: "My Agents" — 24px, weight 700, letter-spacing -0.5px
- Right action: Settings gear icon button — 40×40px circle, `bg: #F8F9FA`, `border: 1px solid #F2F2F7`
- Top padding 60px provides status bar clearance

---

## 2. Main Content Area

```css
.screen {
  flex: 1;
  overflow-y: auto;
  padding: 0 24px 100px 24px;
  /* sides: 24px, bottom: 100px (clears bottom nav) */
}
```

### Segment Control

```css
.segment-control {
  background-color: #F8F9FA;   /* --bg-secondary */
  border-radius: 999px;        /* --radius-pill */
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
  color: #8E8E93;             /* --text-secondary, inactive */
  border-radius: 999px;
}

.segment-opt.active {
  background-color: #FFFFFF;
  color: #111111;             /* --text-primary */
  box-shadow: 0 2px 8px rgba(0,0,0,0.06);
}
```

Three options:
1. "Active" — inactive
2. "All Agents" — **active** (default selected state)
3. "Archived" — inactive

---

## 3. Agent Cards

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

### Card Layout

Each card contains (vertical, 16px gap):
1. **Agent Header row** — icon+name+description on left, status pill on right
2. **Spending section** — spending labels + progress bar (or status message for pending)

### Agent Icon

- 44×44px, border-radius 14px
- Background is the agent's accent color (solid, not dim)
- Icon: 20×20 SVG, stroke currentColor (black), stroke-width 2.5

### Status Pills

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

| Status  | Background             | Text Color | Label   |
|---------|------------------------|------------|---------|
| Active  | green-dim              | `#4A6E65`  | ACTIVE  |
| Pending | yellow-dim             | `#8F7843`  | PENDING |
| Paused  | terracotta-dim         | `#8F6652`  | PAUSED  |

### Spending Bar

```css
.spending-labels {
  display: flex;
  justify-content: space-between;
  font-size: 13px;
  font-weight: 500;
}

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
  /* background color matches agent accent */
}
```

Labels layout:
- Left: "X spent" — `color: #111111`
- Right: "Y limit" — `color: #8E8E93`
- Bar container: `margin-top: 8px;`

---

## Agent Card Data

### Card 1: Research Runner
- Icon bg: `#8FB5AA` (accent-green solid)
- Icon: Magnifier/search SVG
- Name: "Research Runner"
- Description: "Daily search & summary"
- Status: **Active** (`status-active`)
- Spending: `$6.50 spent` / `$25.00 limit`
- Bar fill width: `26%`
- Bar color: `#8FB5AA` (accent-green)

### Card 2: Deploy Bot
- Icon bg: `#F2D48C` (accent-yellow solid)
- Icon: Code brackets `<> </>` SVG
- Name: "Deploy Bot"
- Description: "Smart contract updates"
- Status: **Active** (`status-active`)
- Spending: `$82.10 spent` / `$100.00 limit`
- Bar fill width: `82%`
- Bar color: `#F2D48C` (accent-yellow)

### Card 3: Data Buyer
- Icon bg: `#BCCCDC` (accent-blue solid)
- Icon: Table/grid SVG (rect with lines)
- Name: "Data Buyer"
- Description: "Market intelligence feeds"
- Status: **Pending** (`status-pending`)
- Spending section replaced by text: *"Awaiting signature from primary wallet..."*
  - Style: `font-style: italic; font-size: 13px; color: #8E8E93;`

### Card 4: Treasury Watcher
- Icon bg: `#D9A58B` (accent-terracotta solid)
- Icon: Dollar/currency path SVG
- Name: "Treasury Watcher"
- Description: "Auto-rebalance & alerts"
- Status: **Paused** (`status-paused`)
- Spending: `Spending disabled` / `$500.00 limit`
  - Left label: `color: #8E8E93` (secondary, not primary)
- Bar container: `opacity: 0.3;`
- Bar fill width: `0%`
- Bar color: `#D9A58B` (accent-terracotta)

---

## 4. Bottom Navigation (with FAB)

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
}

.nav-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  color: #8E8E93;
  width: 60px;
}

.nav-item.active { color: #111111; }

.nav-icon {
  width: 24px;
  height: 24px;
  fill: none;
  stroke: currentColor;
  stroke-width: 2;
}

.nav-label {
  font-size: 10px;
  font-weight: 600;
}
```

Note: nav-icon is 24×24 here (vs 22×22 in the dashboard).

### FAB Button

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
  margin-top: -28px;   /* lifts FAB above nav bar */
  box-shadow: 0 8px 24px rgba(0,0,0,0.15);
}
```

- Size: 56×56px circle
- Black background (`#000000`)
- White plus (+) icon: `stroke: white; stroke-width: 3; size: 24x24`
- `margin-top: -28px` — half the FAB height, lifts it out of the nav bar
- Shadow: `0 8px 24px rgba(0,0,0,0.15)`

### Tabs (Agents List screen — AGENTS is active)

| Tab     | Icon                | Label   | State  |
|---------|---------------------|---------|--------|
| Home    | House SVG           | Home    | —      |
| Agents  | 3D layers SVG       | Agents  | Active |
| (FAB)   | Plus (+) icon       | —       | center |
| Stats   | Activity/pulse SVG  | Stats   | —      |
| Wallet  | Card/wallet SVG     | Wallet  | —      |

Note: Tab labels here use title-case ("Home", "Agents") rather than the uppercase used in the dashboard nav ("HOME", "AGENTS"). There are 5 visual items in the nav (4 tabs + 1 FAB = 5 flex children).

The Agents tab icon on this screen differs from the dashboard: it uses a stacked layers SVG (`M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5`) rather than the people/group icon.

---

## Component Summary

| Component          | Class                    | Key Styles                                                        |
|--------------------|--------------------------|-------------------------------------------------------------------|
| Header             | `.header`                | 60px top padding, space-between                                   |
| Segment control    | `.segment-control`       | bg-secondary, pill radius, 4px padding                            |
| Segment option     | `.segment-opt`           | flex 1, 14px/600, inactive: text-secondary                       |
| Segment (active)   | `.segment-opt.active`    | white bg, text-primary, subtle shadow                             |
| Agent card         | `.agent-card`            | bg-secondary, radius-md (20px), 20px padding, 16px col gap       |
| Agent icon         | `.agent-icon`            | 44×44px, radius 14px, solid accent bg                            |
| Status pill        | `.status-pill`           | 4/10px padding, radius 8px, 11px/700, uppercase                  |
| Status: active     | `.status-active`         | green-dim bg / `#4A6E65` text                                    |
| Status: pending    | `.status-pending`        | yellow-dim bg / `#8F7843` text                                   |
| Status: paused     | `.status-paused`         | terracotta-dim bg / `#8F6652` text                               |
| Spending bar track | `.spending-bar-container`| 6px h, rgba(0,0,0,0.05) bg, radius 3px                          |
| Spending bar fill  | `.spending-bar-fill`     | 100% h, radius 3px, color = agent accent                         |
| Bottom nav         | `.bottom-nav`            | 84px h, frosted glass, absolute bottom                            |
| Nav tab            | `.nav-item`              | 60px w, column flex, text-secondary (inactive)                   |
| NAV FAB            | `.nav-fab`               | 56px circle, black, -28px margin-top, shadow                     |
