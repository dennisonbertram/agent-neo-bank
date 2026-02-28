# Settings Screen — v2 Design Spec

**Source**: `docs/design/redesign-v2.md` lines 2383–2666
**Screen title**: `Tally Wallet - Settings`
**Screen ID / label in source**: `app settings`

---

## Purpose

The settings screen manages user-level preferences and account controls. It provides:
- User profile display (avatar + name + email)
- Notification preference toggles (5 notification types)
- Account and security actions (reset wallet connection, export history)
- App version and network information
- Bottom navigation bar (this screen is a top-level tab, not a detail view)

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
  --accent-yellow:          #F2D48C;
  --accent-terracotta:      #D9A58B;
  --accent-blue:            #BCCCDC;
  --accent-green-dim:       rgba(143, 181, 170, 0.15);
  --accent-yellow-dim:      rgba(242, 212, 140, 0.15);
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

This screen includes the full extended palette including `--accent-yellow-dim` and `--accent-terracotta-dim`, which are not present in the agent detail screen.

---

## App Container

```css
.app-container {
  width: 390px;
  height: 844px;
  background-color: var(--bg-primary);   /* #FFFFFF */
  position: relative;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  box-shadow: 0 0 0 10px #000, 0 20px 50px rgba(0,0,0,0.2);
  border-radius: 40px;
}
```

---

## Typography Scale

| Class | Font Size | Font Weight | Color | Notes |
|---|---|---|---|---|
| `.text-title` | 22px | 600 | `--text-primary` | `letter-spacing: -0.5px` |
| `.text-subtitle` | 17px | 600 | `--text-primary` | `letter-spacing: -0.3px` |
| `.text-body` | 15px | 400 | `--text-secondary` | — |
| `.text-caption` | 13px | 500 | `--text-secondary` | `text-transform: uppercase`, `letter-spacing: 0.5px` |

Note: `.text-caption` is 13px here vs 12px in the transaction detail screen and 11px in the agent detail screen. The settings screen uses the largest caption size of the three.

---

## Layout Structure

```
app-container (390 x 844px, overflow: hidden)
├── div.screen (flex:1, overflow-y: auto, with bottom nav padding)
│   ├── button.btn.btn-outline.btn-sm — "← Home" (back nav)
│   ├── div.profile-header — User profile block
│   ├── div.settings-group — "Notifications" section
│   │   ├── span.text-caption.settings-label — "NOTIFICATIONS"
│   │   ├── .settings-row — Agent Requests (toggle: ON)
│   │   ├── .settings-row — Transaction Completed (toggle: ON)
│   │   ├── .settings-row — Approval Required (toggle: ON)
│   │   ├── .settings-row — Daily Limit Reached (toggle: OFF)
│   │   └── .settings-row — Low Balance (toggle: ON)
│   ├── div.settings-group — "Account & Security" section
│   │   ├── span.text-caption.settings-label — "ACCOUNT & SECURITY"
│   │   ├── .settings-row — Reset Coinbase Connection (danger text + chevron)
│   │   └── .settings-row — Export Wallet History (chevron)
│   └── p — Version string "v1.2.4 (Base Mainnet)"
└── div.bottom-nav — 5-tab bottom navigation bar
```

---

## Component Specifications

### Screen Container (`.screen`)

```css
.screen {
  flex: 1;
  overflow-y: auto;
  padding: 60px var(--space-lg) 100px var(--space-lg);
  /* = 60px top, 24px sides, 100px bottom (clears bottom nav) */
  display: block;
}
```

No entry animation on this screen (unlike the detail screens).

### Back Navigation — Home Button

```css
/* Inline styles on button element */
.btn {
  height: 56px;
  border-radius: var(--radius-pill);
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 600;
  font-size: 16px;
  cursor: pointer;
  width: 100%;
  border: none;
  transition: opacity 0.2s;
}
.btn-outline {
  background: transparent;
  border: 1px solid var(--text-tertiary);   /* #C7C7CC */
  color: var(--text-primary);
}
.btn-sm {
  height: 36px;
  padding: 0 16px;
  font-size: 13px;
  width: auto;
  margin-bottom: 16px;
}
```

Displayed as "← Home" — text-only back link (no icon SVG), rendered as a small outlined pill button.

### Profile Header (`.profile-header`)

```css
.profile-header {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 24px 0;
  border-bottom: 1px solid var(--surface-hover);   /* #F2F2F7 */
  margin-bottom: 24px;
}
```

**Profile Avatar (`.profile-avatar`):**
```css
.profile-avatar {
  width: 64px;
  height: 64px;
  background-color: var(--accent-terracotta);   /* #D9A58B */
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 24px;
  font-weight: 600;
  color: white;
}
```
Displays user initials — "JS" (for "Jim Smith"). The terracotta color is used as the avatar background.

**Profile info block (no class, sibling of avatar):**
- Name: `.text-subtitle` — "Jim Smith" (`margin-bottom: 2px`)
- Email: `.text-body` at `font-size: 14px` — "jim@example.com"

### Settings Group (`.settings-group`)

```css
.settings-group { margin-bottom: 32px; }
.settings-label { margin-bottom: 12px; display: block; }
```

Section label: `<span class="text-caption settings-label">` — uppercase text acting as a group header.

### Settings Row (`.settings-row`)

```css
.settings-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 0;
  border-bottom: 1px solid var(--surface-hover);   /* #F2F2F7 */
}
```

Left side of each row:
- Primary label: `.text-subtitle` at `font-size: 15px` (overrides the 17px default)
- Description: `.text-body` at `font-size: 13px` (overrides the 15px default)

Right side: toggle switch OR chevron icon.

### Toggle Switch Component (Settings variant)

This is a larger toggle than the agent detail screen variant:

```css
.toggle-container {
  width: 50px;
  height: 30px;
  background: var(--bg-secondary);   /* #F8F9FA (off state) */
  border-radius: 30px;
  position: relative;
  cursor: pointer;
  transition: background 0.2s;
}
.toggle-container.on { background: var(--black); }   /* #000000 (on state) */

.toggle-handle {
  width: 26px;
  height: 26px;
  background: white;
  border-radius: 50%;
  position: absolute;
  top: 2px;
  left: 2px;
  transition: transform 0.2s;
  box-shadow: 0 2px 4px rgba(0,0,0,0.1);
}
.toggle-container.on .toggle-handle { transform: translateX(20px); }
```

Comparison with agent detail toggle:
- Agent detail: 44x24px track, 20x20px knob, off bg `#E9E9EB`, active class `active`
- Settings: 50x30px track, 26x26px knob, off bg `#F8F9FA`, active class `on`

#### Notification Toggle States

| Setting | Default State |
|---|---|
| Agent Requests | ON (class `on`) |
| Transaction Completed | ON (class `on`) |
| Approval Required | ON (class `on`) |
| Daily Limit Reached | OFF (no class) |
| Low Balance | ON (class `on`) |

### Chevron Navigation Rows

For settings rows that navigate to sub-screens (no toggle), the right side shows a right-pointing chevron SVG:

```svg
<svg width="20" height="20" viewBox="0 0 24 24" fill="none"
     stroke="var(--text-tertiary)" stroke-width="2">
  <path d="M9 18l6-6-6-6"/>
</svg>
```

Stroke color: `var(--text-tertiary)` = `#C7C7CC` (light gray).

### Danger Text Style

```css
.danger-text {
  color: #E5484D;
  font-weight: 600;
}
```

Used on "Reset Coinbase Connection" label. This is the only red/danger color in the design system.

### Account & Security Section Content

| Row | Label style | Sub-label | Right element |
|---|---|---|---|
| Reset Coinbase Connection | `.danger-text` (`color: #E5484D`) | "Disconnect and re-authenticate your wallet" | Chevron |
| Export Wallet History | default | "Download CSV of all agent activity" | Chevron |

The Reset Coinbase row also has `style="cursor: pointer;"` applied to the row itself, suggesting the entire row is tappable.

### Version String

```html
<p style="text-align: center; font-size: 12px; color: var(--text-tertiary); margin-top: 20px;">
  v1.2.4 (Base Mainnet)
</p>
```

Centered, 12px, `color: #C7C7CC` (tertiary). Includes both version number and active network name.

---

## Bottom Navigation Bar (`.bottom-nav`)

This is the only screen of the three that shows the persistent bottom navigation.

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
  padding-bottom: 20px;   /* home indicator safe area */
  z-index: 100;
}
```

### Nav Item (`.nav-item`)

```css
.nav-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  color: var(--text-secondary);   /* #8E8E93 — inactive */
  width: 60px;
}
.nav-item.active { color: var(--text-primary); }   /* #111111 — active */
```

Icon: `.nav-icon { width: 24px; height: 24px; fill: currentColor; }`
Label: `.nav-label { font-size: 10px; font-weight: 500; }`

### Nav Tab Definitions

| Tab | Icon SVG | Label | Active on this screen |
|---|---|---|---|
| Home | House/home path — filled | "Home" | No |
| History | Calendar rect with header | "History" | No |
| Add | Floating action button (see below) | "Add" | No |
| Stats | Activity/pulse line | "Stats" | No |
| Settings | Gear/cog (settings icon) | "Settings" | YES (`.nav-item.active`) |

### Floating Action Button (Add Tab)

The center "Add" tab uses a raised circular FAB instead of a standard flat icon:

```css
/* Inline styles on the FAB container div */
width: 56px;
height: 56px;
border-radius: 50%;
background: var(--black);          /* #000000 */
margin-top: -40px;                 /* lifts the FAB above the nav bar */
display: flex;
align-items: center;
justify-content: center;
box-shadow: 0 4px 12px rgba(0,0,0,0.2);
```

Contains a plus/cross SVG (24x24, `stroke: white`, `stroke-width: 2.5`) with two perpendicular lines.

The FAB floats 40px above the nav bar baseline, creating the characteristic raised center button pattern.

---

## Interaction Notes

- Toggle switches use class `on` for the on state (vs `active` in the agent detail screen — inconsistency noted)
- All notification toggles default to ON except "Daily Limit Reached"
- The Reset Coinbase row uses both `cursor: pointer` on the row and `.danger-text` on the label, signaling a destructive action
- The bottom nav `Settings` tab is the only `.active` item, confirming this is the settings top-level view
- The `← Home` button at the top is redundant with the nav bar's Home tab, suggesting this may have been a prototype navigation artifact
