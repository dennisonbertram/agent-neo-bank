# Main Wallet / Dashboard Screen Spec (v2)

Source lines: 1121–1464 of `docs/design/redesign-v2.md`
HTML title: `Agent Wallet`

---

## Overview

The home/dashboard screen is the primary screen of the app. It displays the user's wallet identity, a large balance card, two quick-action buttons, an activity feed of recent agent transactions, and a bottom navigation bar.

---

## Design Tokens (CSS Variables)

```css
:root {
  --bg-primary:           #FFFFFF;
  --bg-secondary:         #F8F9FA;
  --surface-hover:        #F2F2F7;
  --text-primary:         #111111;
  --text-secondary:       #8E8E93;
  --text-tertiary:        #C7C7CC;
  --accent-green:         #8FB5AA;
  --accent-yellow:        #F2D48C;
  --accent-terracotta:    #D9A58B;
  --accent-blue:          #BCCCDC;
  --accent-green-dim:     rgba(143, 181, 170, 0.15);
  --black:                #000000;
  --white:                #FFFFFF;
  --radius-sm:            12px;
  --radius-md:            20px;
  --radius-lg:            32px;
  --radius-pill:          999px;
  --space-sm:             8px;
  --space-md:             16px;
  --space-lg:             24px;
  --space-xl:             32px;
  --font-family:          -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
}
```

Note: The dashboard includes two additional accent colors not present in Onboarding 2:
- `--accent-yellow: #F2D48C`
- `--accent-terracotta: #D9A58B`
- `--accent-blue: #BCCCDC`

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

## Screen Container

```css
.screen {
  flex: 1;
  overflow-y: auto;
  padding: 60px 24px 100px 24px;
  /* top: 60px (status bar clearance) */
  /* sides: 24px */
  /* bottom: 100px (clears the 84px bottom nav) */
  display: block;
}
```

---

## Typography

| Class           | Size  | Weight | Color              | Letter Spacing | Notes                          |
|-----------------|-------|--------|--------------------|----------------|--------------------------------|
| `.text-display` | 42px  | 600    | inherited          | -1px           | `line-height: 1.1`            |
| `.text-title`   | 20px  | 600    | inherited          | -0.5px         |                                |
| `.text-body`    | 15px  | —      | `#8E8E93`          | —              |                                |
| `.text-caption` | 12px  | 500    | `#8E8E93`          | 0.5px          | `text-transform: uppercase`   |
| `.text-mono`    | 12px  | —      | inherited          | —              | `"SF Mono", "Menlo", monospace`; `background: #F8F9FA; padding: 4px 8px; border-radius: 6px;` |

---

## Layout Structure (Top to Bottom)

1. **Top Bar** — User identity + notification bell
2. **Balance Card** — Black card with wallet balance, network badge, address, holdings
3. **Action Row** — "Add Funds" and "Agents" quick-action buttons
4. **Activity Section** — Section header + list of transaction items
5. **Bottom Navigation** — Fixed 4-tab nav bar

---

## 1. Top Bar

```html
<div style="
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
">
  <!-- Left: avatar + wallet name -->
  <div style="display: flex; align-items: center; gap: 12px;">
    <div style="
      width: 36px;
      height: 36px;
      background: #D9A58B;   /* --accent-terracotta */
      border-radius: 50%;
    "></div>
    <span class="text-title" style="font-size: 17px;">Jim's Wallet</span>
  </div>

  <!-- Right: notification bell -->
  <div style="
    width: 40px;
    height: 40px;
    border-radius: 50%;
    border: 1px solid #F2F2F7;   /* --surface-hover */
    display: flex;
    align-items: center;
    justify-content: center;
  ">
    <!-- Bell SVG: 20x20 -->
  </div>
</div>
```

**Avatar:**
- 36×36px circle
- Background: `#D9A58B` (terracotta accent — user color)

**Wallet Name:**
- Font size: 17px (overrides `.text-title` 20px)
- Font weight: 600

**Notification Button:**
- 40×40px circle
- Border: `1px solid #F2F2F7`
- Icon: Bell SVG, 20×20, stroke currentColor, stroke-width 2

---

## 2. Balance Card

```css
.balance-card {
  background: #000000;
  color: #FFFFFF;
  border-radius: 32px;     /* --radius-lg */
  padding: 32px 24px;
  margin-bottom: 24px;     /* --space-lg */
  position: relative;
  overflow: hidden;
}

/* Decorative radial glow — top-right overlay */
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

### Balance Card Internal Layout

**Top row** (`display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 24px;`):

Left side:
- Caption label: "Base Network Balance" — `color: rgba(255,255,255,0.6); font-size: 11px;`
- Balance amount: `$81,450.00` — `font-size: 40px; font-weight: 600; margin: 4px 0;`

Right side — Network badge:
```html
<div style="
  background: rgba(255,255,255,0.1);
  padding: 4px 8px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  gap: 6px;
">
  <div style="width: 8px; height: 8px; background: #0052FF; border-radius: 50%;"></div>
  <span style="font-size: 11px; font-weight: 600; letter-spacing: 0.5px;">BASE</span>
</div>
```
- Network dot: 8×8px circle, `#0052FF` (Base blue)
- Label: "BASE", 11px, weight 600, letter-spacing 0.5px

**Bottom row** (`display: flex; justify-content: space-between; align-items: center;`):

Left: Wallet address
- `font-family: monospace; font-size: 13px; color: rgba(255,255,255,0.5);`
- Value: `0x71C9...9A2`

Right: Token holdings (two items, `gap: 12px;`):
- `12.4 ETH` — `font-size: 12px; font-weight: 500;`
- `45,200 USDC` — `font-size: 12px; font-weight: 500;`

---

## 3. Action Row

```css
.action-row {
  display: flex;
  gap: 12px;
  margin-bottom: 32px;
}

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
  color: #111111;         /* --text-primary */
}
```

Two equal-width buttons:

**"Add Funds":**
- Icon: Plus (+) SVG, 20×20, stroke-width 2.5
- Label: "Add Funds"

**"Agents":**
- Icon: People/group SVG, 20×20, stroke-width 2.5
- Label: "Agents"

---

## 4. Activity Section

### Section Header

```html
<div style="
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
">
  <h3 class="text-title">Activity</h3>
  <span style="font-size: 13px; font-weight: 600; color: #0052FF;">View All</span>
</div>
```

- "Activity" title: class `.text-title` (20px, weight 600)
- "View All" link: 13px, weight 600, `#0052FF` (Base blue)

### Transaction Item

```css
.tx-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 0;
  border-bottom: 1px solid #F2F2F7;   /* --surface-hover */
}

.agent-avatar {
  width: 44px;
  height: 44px;
  border-radius: 14px;
  display: flex;
  align-items: center;
  justify-content: center;
}
```

Left side of each item:
- `display: flex; gap: 14px; align-items: center;`
- Agent avatar (44×44, radius 14px, colored bg)
- Name + tag row + description

### Activity Feed Data

**Transaction 1: Research Agent**
- Avatar bg: `rgba(143, 181, 170, 0.15)` (green-dim)
- Icon: Search magnifier SVG, stroke `#8FB5AA` (accent-green)
- Name: "Research Agent" — `font-weight: 600; font-size: 15px;`
- Tag: "API FEE" — `.text-mono` style: `font-size: 10px; padding: 2px 6px; background: #F8F9FA; border-radius: 6px;`
- Description: "LLM data synthesis for market analysis" — `font-size: 13px; color: #8E8E93;`
- Amount: `-$12.40` — `font-weight: 600; font-size: 15px; color: #111111;`

**Transaction 2: Deploy Bot**
- Avatar bg: `rgba(242, 212, 140, 0.15)` (yellow-dim)
- Icon: Code brackets SVG `<> </>`; stroke `#F2D48C` (accent-yellow)
- Name: "Deploy Bot"
- Tag: "GAS"
- Description: "Contract deployment on Base Mainnet"
- Amount: `-0.004 ETH` — `color: #111111;`

**Transaction 3: Treasury**
- Avatar bg: `rgba(188, 204, 220, 0.15)` (blue-dim, using `#BCCCDC` at 15% opacity)
- Icon: Dollar sign / currency path SVG; stroke `#BCCCDC` (accent-blue)
- Name: "Treasury"
- Tag: "SWAP"
- Description: "USDC to ETH automated rebalance"
- Amount: `+2.1 ETH` — `color: #4A6E65;` (green, positive)

**Amount color convention:**
- Negative / outgoing: `color: #111111` (text-primary, neutral)
- Positive / incoming: `color: #4A6E65` (dark green)

---

## 5. Bottom Navigation

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
  z-index: 100;
}

.nav-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  color: #8E8E93;   /* --text-secondary (inactive) */
  width: 60px;
}

.nav-item.active {
  color: #111111;   /* --text-primary */
}

.nav-icon {
  width: 22px;
  height: 22px;
  fill: none;
  stroke: currentColor;
  stroke-width: 2;
}

.nav-label {
  font-size: 10px;
  font-weight: 600;
}
```

### Tabs (Dashboard screen — HOME is active)

| Tab      | Icon                  | Label      | State  |
|----------|-----------------------|------------|--------|
| Home     | House SVG             | HOME       | Active |
| Agents   | Person/group SVG      | AGENTS     | —      |
| Limits   | Activity/pulse SVG    | LIMITS     | —      |
| Settings | Gear/cog SVG          | SETTINGS   | —      |

Labels are uppercase (per the HTML content, not CSS `text-transform`).

No FAB button on the dashboard bottom nav (FAB appears only in the Agents List screen).

---

## Component Summary

| Component         | Class             | Key Styles                                                   |
|-------------------|-------------------|--------------------------------------------------------------|
| Balance card      | `.balance-card`   | black bg, radius-lg (32px), 32px/24px padding, glow pseudo  |
| Network badge     | inline            | white/10% bg, radius 8px, 4/8px padding                     |
| Action button     | `.btn-action`     | bg-secondary, radius 16px, 52px h, flex 1                   |
| Agent avatar      | `.agent-avatar`   | 44×44px, border-radius 14px                                  |
| Transaction item  | `.tx-item`        | padding 16px 0, border-bottom surface-hover                  |
| Inline tag/badge  | `.text-mono`      | bg-secondary, radius 6px, 4/8px padding, 12px mono font     |
| Bottom nav bar    | `.bottom-nav`     | 84px h, frosted glass, absolute bottom                       |
| Nav tab           | `.nav-item`       | 60px w, column flex, inactive: text-secondary               |
| Nav tab (active)  | `.nav-item.active`| text-primary (#111111)                                       |
