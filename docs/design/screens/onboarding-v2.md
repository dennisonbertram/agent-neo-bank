# Onboarding Flow — Screen Specs (v2)

Source: `docs/design/redesign-v2.md` lines 1–875.
This is a single HTML file with six `div.screen` panels navigated via `navTo()`.

For design tokens (colors, spacing, typography, shadows) see `docs/design/system/tokens.md`.

---

## App Shell

```
Window size:  390 x 844 px
Container:    .app-container — full width/height, white bg, overflow hidden, flex column
```

The prototype wraps the app in a phone-shaped border (`box-shadow: 0 0 0 10px #000`, `border-radius: 40px`) — **exclude this in Tauri**. The Tauri window IS the phone.

**Global persistent elements** (shown/hidden per screen by `navTo()`):

- `#main-header` — shown on `screen-home` only
- `#bottom-nav` — shown on `screen-home`, `screen-agent-detail`, `screen-add-funds`

---

## Navigation Structure

```
screen-onboarding
    → [Get set up] → screen-setup

screen-setup
    → [← Back] → screen-onboarding
    → [Send Code] → screen-verify

screen-verify
    → [Verify] → screen-home  (calls completeSetup())

screen-home
    → [agent row tap] → screen-agent-detail
    → [FAB +] → screen-add-funds

screen-agent-detail
    → [← Back] → screen-home

screen-add-funds
    → [Close] → screen-home
```

Screens with bottom nav visible: `screen-home`, `screen-agent-detail`, `screen-add-funds`
Screens with main header visible: `screen-home` only

---

## Screen 1: `screen-onboarding` — Welcome

**Purpose:** First launch welcome slide. No header, no bottom nav.

**Layout:** Full-height flex column, centered content with bottom CTA.

### Content Structure

```
.screen#screen-onboarding
  .onboarding-slide.animate-in           [flex-col, justify-center, pb-80px]
    [App Logo Icon]                       60x60px green rounded square
    h1.text-display.mb-sm               "A wallet built\nfor agents."
    p.text-body                          "Securely manage local AI agents with precise
                                          spending limits and full transparency."
                                          font-size: 18px, mb: 40px
    .indicator-row                       [3 indicators, first active]
      .indicator.active                  24x6px pill, #111111
      .indicator                         6x6px circle, #C7C7CC
      .indicator                         6x6px circle, #C7C7CC

  [CTA wrapper]                           position: absolute; bottom: 50px; left: 40px; right: 40px
    button.btn.btn-primary               "Get set up" → navTo('screen-setup')
```

### App Logo Icon

```
Container: 60x60px, background: #8FB5AA, border-radius: 20px
           margin-bottom: 32px
           display: flex, align-items: center, justify-content: center
Icon:      32x32px SVG hexagon with inner circle
           stroke: black, stroke-width: 2, fill: none
SVG paths:
  <path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8
           a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/>
  <circle cx="12" cy="12" r="4"/>
```

### Onboarding Slide Class

```css
.onboarding-slide {
  height: 100%;
  display: flex;
  flex-direction: column;
  justify-content: center;
  padding-bottom: 80px;
}
```

---

## Screen 2: `screen-setup` — Connect Coinbase / Email Setup

**Purpose:** Collect user email for OTP login. Also shows local skill file status.

**No header, no bottom nav.**

### Content Structure

```
.screen#screen-setup
  div.mt-xl.animate-in
    button.btn.btn-outline.btn-sm.mb-sm  "← Back" → navTo('screen-onboarding')
    h2.text-title.mb-sm                  "Connect Coinbase"
    p.text-body.mb-sm                    "Enter your email to receive a
                                          one-time login code."

    .input-group.mt-lg                   [bg-secondary, radius-md, padding 16px]
      label.input-label                  "Email Address"
      input[type=email].input-field      placeholder="jim@example.com"
                                         value="jim@example.com" (prefilled demo)

    button.btn.btn-primary.mt-md        "Send Code" → navTo('screen-verify')

    div.mt-xl.pt-xl                      [border-top: 1px solid --surface-hover]
      h3.text-subtitle.mb-sm            "Local Skill Setup"
      div                               [bg: --bg-secondary, radius: 16px, padding: 16px]
        .flex-row.justify-between        agents.md row
          span.text-mono               "agents.md"
          span                          "UPDATED" — color: --accent-green, 12px, weight 600
        .flex-row.justify-between.mt-sm  claude.md row
          span.text-mono               "claude.md"
          span                          "UPDATED" — color: --accent-green, 12px, weight 600
```

---

## Screen 3: `screen-verify` — OTP Verification

**Purpose:** 6-digit OTP entry. Shows partially filled state (4 digits entered, 2 masked).

**No header, no bottom nav.**

### Content Structure

```
.screen#screen-verify
  div.mt-xl.animate-in
    h2.text-title.mb-sm                 "Check your email"
    p.text-body.mb-sm                   "We sent a 6-digit code to jim@example.com"

    .flex-row.gap-sm.mt-lg.justify-center   [6 OTP digit boxes]
      [digit box] "4"                   48x56px, bg: --bg-secondary, radius: 12px
      [digit box] "2"                   (same)
      [digit box] "9"                   (same)
      [digit box] "•"                   (masked, same styling)
      [digit box] "•"                   (masked)
      [digit box] "•"                   (masked)

    button.btn.btn-primary.mt-xl       "Verify" → completeSetup() → navTo('screen-home')
```

### OTP Digit Box Spec

```
width:        48px
height:       56px
background:   var(--bg-secondary)   [#F8F9FA]
border-radius: 12px
display:      flex, align-items: center, justify-content: center
font-size:    24px
font-weight:  600
```

---

## Screen 4: `screen-home` — Main Dashboard

**Purpose:** Primary dashboard. Shows total balance, wallet address, agent list, recent transactions.

**Has header and bottom nav.**

### Header (`#main-header`)

```
position: absolute; top: 0; left: 0; right: 0;
padding: 50px 24px 16px 24px          [50px top = status bar clearance]
z-index: 10
background: linear-gradient(to bottom, white 80%, rgba(255,255,255,0))

Left side: .profile-pill
  .avatar                             40x40px circle, bg: --bg-secondary
                                       icon: person SVG in --accent-terracotta (#D9A58B)
  .flex-col
    span.text-caption (11px)          "Wallet • Base"
    span.text-subtitle                "Good morning, Jim"

Right side: button.icon-btn           [bell/notifications icon]
  (40x40px circle, border: 1px solid --surface-hover)
```

### Content Structure

```
.screen#screen-home                   padding-top: 80px (clears absolute header)
  div.animate-in

    [Balance section]
      div                             text-align: center, margin-bottom: 32px
        h1.text-display               "81,450"
          span                        "$" — font-size: 24px, color: --text-secondary,
                                           vertical-align: top
        p.text-caption.mt-sm          "Total Balance (USDC + ETH)"

        .flex-row.justify-center.gap-sm.mt-lg
          span                        wallet address pill:
                                       bg: --bg-secondary, padding: 6px 12px,
                                       border-radius: 100px, font-family: monospace,
                                       font-size: 12px, color: --text-secondary
                                       text: "0x71C...9A2"
          [copy icon SVG]             16x16px, color: --text-secondary

    [Segment control]
      .segment-control
        .segment-opt.active           "Overview"
        .segment-opt                  "Agents"

    [Active Agents section]
      h3.text-title.mb-md             "Active Agents"

      [Agent Row 1 — Research]
        .dna-list-item → navTo('screen-agent-detail')
          .dna-pill-container.pill-green   width: 65%
            .dna-icon-box             search icon SVG (18x18, stroke-width: 2.5)
            span.dna-label            "Research"
          .flex-col (align-items: flex-end)
            span.dna-value            "$25.00"
            span.dna-subvalue         "Daily Limit"

      [Agent Row 2 — Deploy Bot]
        .dna-list-item
          .dna-pill-container.pill-yellow  width: 50%
            .dna-icon-box             code/brackets icon SVG
            span.dna-label            "Deploy Bot"
          .flex-col (align-items: flex-end)
            span.dna-value            "$100.00"
            span.dna-subvalue         "Daily Limit"

      [Agent Row 3 — Treasury]
        .dna-list-item
          .dna-pill-container.pill-terracotta  width: 40%
            .dna-icon-box             dollar/finance icon SVG
            span.dna-label            "Treasury"
          .flex-col (align-items: flex-end)
            span.dna-value            "Paused"
            span.dna-subvalue         "Check Settings"

    [Recent Transactions section]
      .flex-row.justify-between.mt-lg.mb-md
        h3.text-title                 "Recent"
        span.text-caption             "See All" (cursor: pointer)

      [Transaction Row 1]
        .flex-row.justify-between
          padding: 12px 0, border-bottom: 1px solid --surface-hover
          .flex-row.gap-md
            [icon container]          40x40px, bg: --bg-secondary, radius: 12px
                                       icon: file/document SVG, stroke: --text-secondary
            .flex-col
              span.text-subtitle (15px)  "API Usage"
              span.text-caption (11px)   "Research Runner • 2m ago"
          span.text-subtitle           "- $6.50"

      [Transaction Row 2]
        .flex-row.justify-between
          padding: 12px 0            [no border — last item]
          .flex-row.gap-md
            [icon container]          40x40px, bg: --bg-secondary, radius: 12px
                                       icon: hexagon/package SVG, stroke: --text-secondary
            .flex-col
              span.text-subtitle (15px)  "Contract Deploy"
              span.text-caption (11px)   "Deploy Bot • 1h ago"
          span.text-subtitle           "- 0.03 ETH"
```

### Agent Pill Colors

| Agent | Pill class | Background color |
|---|---|---|
| Research | `.pill-green` | `#8FB5AA` |
| Deploy Bot | `.pill-yellow` | `#F2D48C` |
| Treasury | `.pill-terracotta` | `#D9A58B` |

The pill width percentage controls a visual "fill" illusion — wider pill = more budget consumed.

---

## Screen 5: `screen-agent-detail` — Agent Detail

**Purpose:** Drilldown for individual agent. Shows budget usage, toggle controls, logs link.

**No header (hidden by navTo). Has bottom nav.**

### Content Structure

```
.screen#screen-agent-detail
  div.animate-in
    button.btn.btn-outline.btn-sm.mb-md   "← Back" → navTo('screen-home')

    .flex-row.justify-between.align-center.mb-lg
      h1.text-title (font-size: 28px)     "Research Runner"
      span.status-badge.status-active     "Active"
                                           bg: --accent-green-dim, color: #4A6E65

    [Daily Budget Card]
      div                                 bg: --bg-secondary, border-radius: 24px,
                                           padding: 24px, margin-bottom: 24px
        span.text-caption                 "Daily Budget"
        .flex-row.justify-between.align-end.mt-xs.mb-md
          span.text-display (32px)        "$6.50"
          span.text-body (mb: 6px)        "of $25.00"
        [Progress Bar]
          track:  height: 8px, width: 100%, bg: rgba(0,0,0,0.05), radius: 4px
          fill:   height: 100%, width: 26%, bg: --accent-green, radius: 4px

    [Controls section]
      h3.text-subtitle.mb-md             "Controls"

      [Toggle Row 1 — Pause Agent]
        .flex-row.justify-between.items-center
          padding: 16px 0, border-bottom: 1px solid --surface-hover
          .flex-col
            span.text-body (weight 500, color: --text-primary)  "Pause Agent"
            span.text-caption (text-transform: none)            "Temporarily disable spending"
          [Toggle — OFF state]
            track: 50x30px, bg: --bg-secondary, radius: 30px
            thumb: 26x26px white circle, top: 2px, left: 2px (LEFT = OFF)

      [Toggle Row 2 — Require Approval]
        .flex-row.justify-between.items-center
          padding: 16px 0, border-bottom: 1px solid --surface-hover
          .flex-col
            span.text-body (weight 500, color: --text-primary)  "Require Approval"
            span.text-caption (text-transform: none)            "For txs over $5.00"
          [Toggle — ON state]
            track: 50x30px, bg: --black, radius: 30px
            thumb: 26x26px white circle, top: 2px, right: 2px (RIGHT = ON)

      button.btn.btn-outline.mt-xl       "View Metadata Logs"
```

### Status Badge Spec

```css
.status-badge {
  padding: 4px 8px;
  border-radius: 6px;
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
}
.status-active   { background: rgba(143,181,170,0.15); color: #4A6E65; }
.status-pending  { background: rgba(242,212,140,0.15); color: #8F7843; }
.status-paused   { background: rgba(217,165,139,0.15); color: #8F6652; }
```

---

## Screen 6: `screen-add-funds` — Deposit Funds

**Purpose:** Show QR code and wallet address for depositing USDC/ETH. CTA for card buy (disabled).

**No header. Has bottom nav. padding-top: 80px.**

### Content Structure

```
.screen#screen-add-funds (padding-top: 80px)
  div.animate-in.text-center
    h2.text-title.mb-lg               "Deposit Funds"

    [QR Code Card]
      div                             bg: white, padding: 20px, border-radius: 20px,
                                       display: inline-block,
                                       box-shadow: --shadow-subtle, mb: 24px
        div                           200x200px, bg: --bg-secondary, radius: 12px
                                       display: flex, align-items: center, justify-content: center
                                       position: relative
          [QR placeholder SVG]        40x40px grid icon, stroke: --text-tertiary
          div                         position: absolute, inset: 16px,
                                       border: 2px dashed --text-tertiary, opacity: 0.3

    [Warning Pill]
      div.pill-yellow-subtle           display: inline-flex, align-items: center, gap: 6px,
                                        padding: 8px 16px, border-radius: 8px, mb: 24px
        [alert triangle SVG]           14x14px
        span                           "Send only USDC or ETH on Base"
                                        font-size: 13px, font-weight: 500

    [Wallet Address Row]
      .input-group.flex-row.justify-between   text-align: left
        .flex-col
          span.input-label             "Wallet Address"
          span.text-mono (14px)        "0x71C9...9A2"
        [copy icon SVG]               20x20px, cursor: pointer

    button.btn.btn-secondary.mt-lg    "Buy with Card (Coming Soon)"
                                       disabled, opacity: 0.5

    button.btn.btn-outline.mt-sm      "Close" → navTo('screen-home')
```

---

## Persistent: Bottom Navigation (`#bottom-nav`)

Shown on: `screen-home`, `screen-agent-detail`, `screen-add-funds`

```
.bottom-nav
  position: absolute, bottom: 0, left: 0, right: 0
  height: 84px
  background: rgba(255,255,255,0.95)
  backdrop-filter: blur(10px)
  border-top: 1px solid rgba(0,0,0,0.05)
  display: flex, justify-content: space-around, align-items: center
  padding-bottom: 20px         [iOS safe area]
  z-index: 100

  Items (5 total, center = FAB):

  1. .nav-item.active → navTo('screen-home')
       [home SVG icon]         24x24px, fill: currentColor
       span.nav-label          "Home"

  2. .nav-item (no action in prototype)
       [calendar SVG icon]     24x24px, fill: none, stroke: currentColor
       span.nav-label          "History"

  3. .nav-fab → navTo('screen-add-funds')
       [+ SVG icon]            24x24px, stroke: white, stroke-width: 2.5
       56x56px black circle, margin-top: -28px (floats above bar)
       box-shadow: 0 8px 24px rgba(0,0,0,0.15)

  4. .nav-item
       [analytics/chart icon]  24x24px
       span.nav-label          "Analytics"

  5. .nav-item
       [more/ellipsis icon]    24x24px
       span.nav-label          "More"
```

---

## Persistent: Main Header (`#main-header`)

Shown only on: `screen-home`

```
position: absolute; top: 0; left: 0; right: 0; z-index: 10
padding: 50px 24px 16px 24px
background: linear-gradient(to bottom, white 80%, rgba(255,255,255,0))

Left: .profile-pill
  .avatar (40x40px circle)
  .flex-col
    span (11px, uppercase, --text-secondary)   "Wallet • Base"
    span.text-subtitle                          "Good morning, Jim"

Right: button.icon-btn (40x40px circle)
  [bell/notification SVG]  → toggleNotifs()
```

---

## Navigation Logic (JavaScript)

```javascript
function navTo(screenId) {
  // Hide all screens
  document.querySelectorAll('.screen').forEach(s => {
    s.classList.remove('active');
    s.style.display = 'none';
  });

  // Show target screen
  const target = document.getElementById(screenId);
  target.style.display = 'block';
  setTimeout(() => target.classList.add('active'), 10); // triggers opacity transition

  // Show/hide bottom nav
  const nav = document.getElementById('bottom-nav');
  const header = document.getElementById('main-header');
  const showNav = ['screen-home', 'screen-agent-detail', 'screen-add-funds'];
  nav.style.display = showNav.includes(screenId) ? 'flex' : 'none';
  header.style.display = showNav.includes(screenId) ? 'flex' : 'none';

  // Hide header on detail/add-funds screens (back button is inline instead)
  if (screenId === 'screen-add-funds' || screenId === 'screen-agent-detail') {
    header.style.display = 'none';
  }
}

function completeSetup() { navTo('screen-home'); }
```

In the React/Tauri implementation replace `navTo()` with router navigation (e.g. React Router or a simple screen state machine).

---

## Component Inventory Summary

| Component | Used in screens |
|---|---|
| Primary button (`.btn-primary`, black fill) | onboarding, setup, verify, add-funds |
| Outline button (`.btn-outline`) | setup (back), agent-detail (back, logs), add-funds (close) |
| Secondary button (`.btn-secondary`, gray fill) | add-funds (disabled) |
| Small outline button (`.btn-sm`) | setup (back), agent-detail (back) |
| Input group | setup (email), add-funds (wallet address row) |
| OTP digit boxes | verify |
| Segment control | home |
| DNA list item (agent pill row) | home (×3) |
| Transaction row | home (×2) |
| Status badge | agent-detail |
| Budget card + progress bar | agent-detail |
| Toggle switch | agent-detail (×2) |
| QR code placeholder | add-funds |
| Warning pill (yellow-subtle) | add-funds |
| Bottom nav (with FAB) | home, agent-detail, add-funds |
| Main header | home |
| Onboarding slide + indicators | onboarding |
| App logo icon | onboarding |
| Wallet address pill + copy icon | home, add-funds |
| Profile avatar | header |
