# Tally Agentic Wallet -- Design System

> Distilled from Privacy.com visual inspiration + Tally design brief. Dark-first fintech aesthetic.
> Single source of truth for UI implementation.

---

## 1. Design Principles

- **Dark-first:** Optimized for dark backgrounds (Privacy.com's proven dark fintech aesthetic). Light mode is secondary but fully supported.
- **Trust through structure:** Consistent patterns, predictable layouts, aligned elements. Financial apps must feel secure and reliable.
- **Financial clarity:** Numbers are prominent, status is always visible. Monetary amounts use tabular/monospace numerals for alignment. One display-size number per screen.
- **Minimal chrome:** Let content breathe, reduce visual noise. Card-based layouts with generous whitespace. Progressive disclosure -- show essentials first, details on demand.
- **Consumer app, not dev tool:** This is a banking app that happens to serve AI agents. It should feel like Mercury or Apple Wallet, not an admin panel.

---

## 2. Color Tokens

### 2.1 Base Palette (Dark Theme -- Primary)

The dark theme is the primary theme, synthesizing Privacy.com's dark palette with Tally's brand identity.

| Token | Hex | Usage |
|---|---|---|
| `--page-background` | `#0F1117` | Page background (deep navy-black, Tally brief) |
| `--container-background` | `#1A1D27` | Card and panel backgrounds |
| `--surface-raised` | `#232736` | Elevated surfaces, hover backgrounds, input backgrounds |
| `--surface-sunken` | `#141620` | Inset areas, sidebar background, well areas |
| `--text-primary` | `#F1F1F4` | Primary text, headings, important values |
| `--text-subtle` | `#D1D1DF` | Secondary text, descriptions (Privacy.com) |
| `--text-muted` | `#828299` | Tertiary text, timestamps, placeholders (Privacy.com) |
| `--text-inverse` | `#1A1A1A` | Text on light/brand backgrounds |
| `--border-default` | `#2D3142` | Default borders (Tally brief dark) |
| `--border-strong` | `#4C4C5D` | Strong borders, active states (Privacy.com) |
| `--ring` | `#6366F1` | Focus rings on interactive elements |

### 2.2 Brand Colors

| Token | Hex | Usage |
|---|---|---|
| `--brand-main` | `#4F46E5` | Primary brand color (indigo-600) |
| `--brand-bright` | `#6366F1` | Brighter variant for dark theme buttons, links (indigo-500) |
| `--brand-hover` | `#4338CA` | Primary button hover (indigo-700) |
| `--brand-on-main` | `#FFFFFF` | Text on brand backgrounds |
| `--brand-container` | `#302E6E` | Brand container bg (badges, pills) -- from Privacy.com |
| `--brand-on-container` | `#B5B5F9` | Text on brand container -- from Privacy.com |
| `--brand-50` | `#EEF2FF` | Subtle brand backgrounds (light contexts) |
| `--brand-100` | `#E0E7FF` | Light brand fills |
| `--gradient-primary` | `linear-gradient(135deg, #6366F1 0%, #8B5CF6 50%, #7C3AED 100%)` | Balance card, hero elements (brighter in dark) |
| `--gradient-primary-hover` | `linear-gradient(135deg, #4F46E5 0%, #7C3AED 50%, #6366F1 100%)` | Balance card hover |

### 2.3 Semantic Colors

Each semantic color has four variants following Privacy.com's `main / on-main / container / on-container` pattern:

#### Success (Green)

| Token | Hex | Usage |
|---|---|---|
| `--success-main` | `#10B981` | Success text, icons, positive amounts, active indicators |
| `--success-on-main` | `#FFFFFF` | Text on success backgrounds |
| `--success-container` | `#254D1E` | Success container bg (dark theme, from Privacy.com) |
| `--success-on-container` | `#A2EC8E` | Text on success container (dark theme) |
| `--success-600` | `#059669` | Success buttons, active states |
| `--success-700` | `#047857` | Success button hover |

#### Danger (Red)

| Token | Hex | Usage |
|---|---|---|
| `--danger-main` | `#EF4444` | Danger text, icons, negative amounts, declined |
| `--danger-on-main` | `#FFFFFF` | Text on danger backgrounds |
| `--danger-container` | `#5E2121` | Danger container bg (dark theme) |
| `--danger-on-container` | `#F7ABAB` | Text on danger container (dark theme) |
| `--danger-600` | `#DC2626` | Danger buttons (deny, suspend) |
| `--danger-700` | `#B91C1C` | Danger button hover |

#### Warning (Amber)

| Token | Hex | Usage |
|---|---|---|
| `--warning-main` | `#F59E0B` | Warning text, icons, pending states, settling |
| `--warning-on-main` | `#FFFFFF` | Text on warning backgrounds |
| `--warning-container` | `#5E4212` | Warning container bg (dark theme) |
| `--warning-on-container` | `#F1D394` | Text on warning container (dark theme) |
| `--warning-600` | `#D97706` | Warning emphasis |

#### Info (Blue)

| Token | Hex | Usage |
|---|---|---|
| `--info-main` | `#2C98D6` | Info text, icons, informational callouts |
| `--info-on-main` | `#FFFFFF` | Text on info backgrounds |
| `--info-container` | `#133F58` | Info container bg (dark theme) |
| `--info-on-container` | `#9ED5F4` | Text on info container (dark theme) |

#### Neutral (for badges, secondary elements)

| Token | Hex | Usage |
|---|---|---|
| `--neutral-main` | `#4C4C5D` | Neutral badge bg, settled status |
| `--neutral-on-main` | `#F1F1F4` | Text on neutral backgrounds |
| `--neutral-container` | `#66667A` | Neutral container bg (stronger variant) |
| `--neutral-on-container` | `#FFFFFF` | Text on neutral container |

### 2.4 Light Theme

| Token | Hex | Usage |
|---|---|---|
| `--page-background` | `#FAFAF9` | Page background (warm off-white) |
| `--container-background` | `#FFFFFF` | Card and panel backgrounds |
| `--surface-raised` | `#FFFFFF` | Elevated cards with shadow |
| `--surface-sunken` | `#F9FAFB` | Inset areas, table row hover |
| `--text-primary` | `#1A1A1A` | Primary text, headings |
| `--text-subtle` | `#6B7280` | Secondary text, descriptions |
| `--text-muted` | `#9CA3AF` | Tertiary text, timestamps, placeholders |
| `--text-inverse` | `#FFFFFF` | Text on dark/colored backgrounds |
| `--border-default` | `#E8E5E0` | Default borders (warm gray) |
| `--border-strong` | `#D1D5DB` | Strong borders |
| `--ring` | `#6366F1` | Focus rings |
| `--brand-main` | `#4F46E5` | Primary brand (same in light) |
| `--brand-bright` | `#4F46E5` | Same as main in light mode |
| `--brand-hover` | `#4338CA` | Hover state |
| `--brand-container` | `#EEF2FF` | Brand container bg (light) |
| `--brand-on-container` | `#312E81` | Text on brand container (light) |
| `--gradient-primary` | `linear-gradient(135deg, #4F46E5 0%, #7C3AED 50%, #6366F1 100%)` | Balance card |
| `--success-main` | `#10B981` | Same hex values |
| `--success-container` | `#ECFDF5` | Light success bg |
| `--success-on-container` | `#047857` | Dark success text |
| `--danger-main` | `#EF4444` | Same hex values |
| `--danger-container` | `#FEF2F2` | Light danger bg |
| `--danger-on-container` | `#B91C1C` | Dark danger text |
| `--warning-main` | `#F59E0B` | Same hex values |
| `--warning-container` | `#FFFBEB` | Light warning bg |
| `--warning-on-container` | `#92400E` | Dark warning text |
| `--info-main` | `#2C98D6` | Same hex values |
| `--info-container` | `#EFF6FF` | Light info bg |
| `--info-on-container` | `#1E40AF` | Dark info text |
| `--neutral-main` | `#E5E7EB` | Light neutral bg |
| `--neutral-on-main` | `#374151` | Dark text on neutral |
| `--neutral-container` | `#F3F4F6` | Lighter neutral container |
| `--neutral-on-container` | `#1F2937` | Dark text on neutral container |

---

## 3. Typography

### 3.1 Font Stack

```css
--font-sans: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
--font-mono: 'JetBrains Mono', 'SF Mono', 'Fira Code', 'Courier New', monospace;
```

- **Inter**: Primary typeface (closest free alternative to Privacy.com's Graphik). Load weights 400, 500, 600, 700.
- **JetBrains Mono**: For wallet addresses, amounts, transaction hashes, code values.
- Fallback chain covers macOS, Windows, and Linux system fonts.

### 3.2 Type Scale

| Token | Size | Weight | Line Height | Letter Spacing | Usage |
|---|---|---|---|---|---|
| `display-1` | `36px` | 700 | 1.3 | `-0.02em` | Hero greetings, page hero text (matches Privacy.com h1: 36px) |
| `display-2` | `28px` | 700 | 1.3 | `-0.01em` | Large section headers |
| `heading-1` | `24px` | 600 | 1.3 | `-0.01em` | Page titles ("Dashboard", "Agents") |
| `heading-2` | `20px` | 600 | 1.35 | `-0.005em` | Card titles, section headers (matches Privacy.com subheader: 20px) |
| `heading-3` | `16px` | 600 | 1.4 | `0` | Card headers, sub-section titles |
| `body-large` | `16px` | 400 | 1.6 | `0` | Default body text (matches Privacy.com base: 16px) |
| `body` | `14px` | 400 | 1.5 | `0` | Standard body text, table cells |
| `body-small` | `13px` | 400 | 1.5 | `0.01em` | Helper text, descriptions |
| `label` | `13px` | 500 | 1.4 | `0.02em` | Form labels, badge text |
| `label-small` | `11px` | 600 | 1.3 | `0.05em` | Uppercase status labels ("SETTLED", "PAUSED") |
| `mono-large` | `48px` | 700 | 1.1 | `-0.02em` | Hero balance number (JetBrains Mono) |
| `mono-display` | `24px` | 600 | 1.3 | `0` | Large monetary amounts (JetBrains Mono) |
| `mono` | `14px` | 400 | 1.5 | `0` | Addresses, hashes, code values (JetBrains Mono) |
| `mono-small` | `12px` | 400 | 1.5 | `0` | Small code values, status codes (JetBrains Mono) |

### 3.3 Hierarchy Rules

1. **One `mono-large` number per screen.** The primary balance is the focal point.
2. **Two levels of heading per card maximum.** Card title + optional subtitle.
3. **Body text defaults to `body` (14px).** Use `body-large` only for standalone paragraphs or onboarding copy.
4. **All monetary amounts use `--font-mono`** for tabular alignment in tables and lists.
5. **Uppercase labels** (`label-small`) always use `text-transform: uppercase` and increased letter-spacing.

---

## 4. Spacing & Layout

### 4.1 Spacing Scale

4px base grid. Every margin, padding, gap, and dimension is a multiple of 4.

| Token | Value | Usage |
|---|---|---|
| `space-0.5` | `2px` | Micro gaps (inline dot-to-text) |
| `space-1` | `4px` | Tightest spacing (icon padding, badge internal) |
| `space-1.5` | `6px` | Small internal padding |
| `space-2` | `8px` | Small gaps between related elements, icon padding |
| `space-3` | `12px` | Default gap between form elements, badge text-dot gap |
| `space-4` | `16px` | Standard card padding, input padding, nav item padding |
| `space-5` | `20px` | Medium section spacing, agent card padding |
| `space-6` | `24px` | Card internal padding (default), grid gap, section gap (Privacy.com standard) |
| `space-8` | `32px` | Section gaps within a page, main content padding |
| `space-10` | `40px` | Large section dividers, hero padding (Privacy.com p-40) |
| `space-12` | `48px` | Page-level bottom padding |
| `space-16` | `64px` | Major layout spacing, nav link gap (Privacy.com gap-64) |

### 4.2 Page Layout

| Property | Value |
|---|---|
| Sidebar width (expanded) | `240px` |
| Sidebar width (collapsed) | `72px` |
| Content max-width | `1200px` |
| Content padding | `32px 32px 48px 32px` |
| Page header height | `64px` |
| Page header padding | `0 32px` |
| Page gutters (full-width sections) | `40px` (from Privacy.com) |
| Section gap (between major sections) | `24px` (from Privacy.com `gap-24`) |

### 4.3 Grid System

| Context | Columns | Gap | Notes |
|---|---|---|---|
| Agent cards (wide) | 3 | `16px` | `> 1024px` |
| Agent cards (medium) | 2 | `16px` | `768px - 1024px` |
| Agent cards (narrow) | 1 | `16px` | `< 768px` |
| Cards listing (Privacy-style) | 5 | `24px` | Full-width at ~1528px, responsive down |
| Dashboard layout | 2 flexible columns | `24px` | Wallet + Spend, Transactions + Sidebar |
| Card detail layout | `426px + 1fr` | `24px` | Fixed sidebar + flexible main (Privacy.com pattern) |

---

## 5. Border Radius Scale

| Token | Value | Usage |
|---|---|---|
| `radius-sm` | `4px` | Small inputs, tight elements |
| `radius-md` | `8px` | Buttons, inputs, small cards, nav items |
| `radius-lg` | `12px` | Cards, sections, modals, dropdowns |
| `radius-xl` | `16px` | Content boxes, hero cards, panels (Privacy.com standard for `content-box`) |
| `radius-2xl` | `24px` | Card tiles in grid layouts (Privacy.com `radius-24` for card tiles) |
| `radius-full` | `64px` | Pills, pill buttons, badges, search input (Privacy.com pill pattern) |
| `radius-circle` | `50%` | Icon buttons, avatars, status dots |

---

## 6. Shadows

Dark theme shadows use reduced opacity and cooler tints. Light theme uses warmer tints.

| Token | Value (Dark) | Value (Light) | Usage |
|---|---|---|---|
| `shadow-sm` | `0 2px 8px 1px color-mix(in srgb, #000 8%, transparent)` | `0 1px 3px rgba(0,0,0,0.06), 0 1px 2px rgba(0,0,0,0.04)` | Default card shadow |
| `shadow-md` | `0 4px 12px 1px color-mix(in srgb, #000 8%, transparent)` | `0 4px 6px rgba(0,0,0,0.05), 0 2px 4px rgba(0,0,0,0.04)` | Elevated cards, dropdowns |
| `shadow-lg` | `0 8px 16px 1px color-mix(in srgb, #000 8%, transparent)` | `0 10px 15px rgba(0,0,0,0.06), 0 4px 6px rgba(0,0,0,0.04)` | Modals, floating panels |
| `shadow-xl` | `0 12px 24px 2px color-mix(in srgb, #000 12%, transparent)` | `0 20px 25px rgba(0,0,0,0.08), 0 8px 10px rgba(0,0,0,0.04)` | Popovers, context menus |
| `shadow-card-hover` | `0 8px 20px color-mix(in srgb, #000 12%, transparent)` | `0 8px 20px rgba(0,0,0,0.08), 0 2px 6px rgba(0,0,0,0.04)` | Card hover state |

---

## 7. Borders

| Token | Value |
|---|---|
| `border-default` | `1px solid var(--border-default)` |
| `border-strong` | `1px solid var(--border-strong)` |
| `border-card` | `1px solid var(--border-default)` (on cards in dark), `1px solid var(--border-default)` (light) |
| `border-divider` | `1px solid var(--border-default)` (horizontal separators within cards, like Privacy.com card detail) |

Dark theme border color: `#2D3142` (default) / `#4C4C5D` (strong, from Privacy.com)
Light theme border color: `#E8E5E0` (default) / `#D1D5DB` (strong)

---

## 8. Transitions / Motion

| Token | Duration | Easing | Usage |
|---|---|---|---|
| `transition-fast` | `100ms` | `ease` | Button press (scale), toggle switch, hover color |
| `transition-normal` | `200ms` | `cubic-bezier(.55, 0, .1, 1)` | Hover states, focus rings, color changes (Privacy.com bezier-200) |
| `transition-slow` | `300ms` | `ease-out` | Card lift, toast enter, panel transitions (Privacy.com ease-in-out-300) |
| `transition-entrance` | `500ms` | `cubic-bezier(.55, 0, .1, 1)` | Page transitions, fade-ins, progress bar fills (Privacy.com bezier-500) |
| `transition-page` | `250ms` | `ease-in-out` | Page content transitions |

### Motion Principles

1. **Ease-out for entering** (decelerating into place). **Ease-in for leaving** (accelerating away).
2. **Max transform:** 8px translate / 3 degrees rotation. No bouncing or overshooting.
3. **Respect `prefers-reduced-motion`:** Disable all transforms, reduce durations to 0ms.
4. **Purposeful only:** Every animation communicates state change. No decorative animation.

### Key Interactions

| Element | Trigger | Animation |
|---|---|---|
| Balance card | Hover | Subtle 3D parallax tilt (max 3deg), shadow deepens, diagonal shine shifts |
| Balance number | Data load | Count-up from 0 over 500ms with easing |
| Agent card | Hover | `translateY(-2px)`, shadow to `shadow-card-hover`, 200ms ease |
| Any button | Hover | Background color shift, 200ms ease |
| Any button | Press | `scale(0.98)`, 100ms |
| Status badge (pending) | Idle | Dot pulses opacity 1.0 -> 0.4, 2s loop, ease-in-out |
| Skeleton loader | Loading | Gradient shimmer sweep left-to-right, 1.5s loop |
| Toast | Enter/Exit | Slide in from right + fade (300ms out), slide out + fade (200ms in) |
| Modal | Open/Close | Backdrop fade 200ms; content scale 0.95->1.0 + fade 250ms ease-out |
| Sidebar | Collapse | Width 200ms ease, text fades, icons remain centered |
| Copy button | Click | Icon swaps to checkmark, tooltip "Copied!", reverts after 2s |

---

## 9. Components

### 9.1 Buttons

#### Primary Button
- Background: `var(--brand-bright)` (`#6366F1` dark, `#4F46E5` light)
- Text: `var(--brand-on-main)` (`#FFFFFF`)
- Border: none
- Radius: `radius-md` (`8px`)
- Padding: `10px 24px`
- Height: `40px` (default), `32px` (sm), `48px` (lg)
- Font: `label` (13px, 500 weight, `0.02em` tracking)
- Hover: `var(--brand-hover)` (`#4338CA`)
- Active: `scale(0.98)` transform
- Disabled: `opacity: 0.5`, `cursor: not-allowed`
- Focus: `2px` ring `var(--ring)`, `2px` offset
- **Pages:** Every page (CTAs: "Get Started", "Fund Wallet", "Generate Code")

#### Primary Pill Button
- Same as Primary, but `radius-full` (`64px`)
- Padding: `13px 24px`
- Height: `56px`
- **Pages:** Cards listing ("New Card +"), hero banner CTA (Privacy.com pattern)

#### Secondary / Ghost Button
- Background: `transparent`
- Text: `var(--text-primary)` (secondary) or `var(--text-subtle)` (ghost)
- Border: `1px solid var(--border-default)` (secondary) or none (ghost)
- Radius: `radius-md` (`8px`)
- Hover: `var(--surface-raised)` background
- **Pages:** Secondary actions ("Cancel", "View All", "Back")

#### Icon Button (Circle)
- Shape: `radius-circle` (`50%`)
- Size: `48px` x `48px` (default), `56px` x `56px` (large)
- Background: `var(--container-background)`
- Border: `1px solid var(--container-background)` (invisible border, from Privacy.com)
- Color: `var(--text-primary)`
- Font: `14px`, weight 500
- Hover: `var(--surface-raised)` background
- Danger variant: icon color `var(--danger-main)`
- **Pages:** Card detail action bar (star, download, pause, delete)

#### Link Button
- Background: none
- Color: `var(--brand-bright)`
- Border: none
- Hover: underline
- **Pages:** Inline text links ("Resend code", "How It Works", "View All Transactions")

#### Danger Button
- Background: `var(--danger-600)` (`#DC2626`)
- Text: `#FFFFFF`
- Hover: `var(--danger-700)` (`#B91C1C`)
- **Pages:** Approvals ("Deny"), agent detail ("Suspend Agent")

#### Success Button
- Background: `var(--success-600)` (`#059669`)
- Text: `#FFFFFF`
- Hover: `var(--success-700)` (`#047857`)
- **Pages:** Approvals ("Approve")

### 9.2 Badges / Status Pills

#### Transaction Status Badge
- Shape: `radius-full` (pill)
- Padding: `4px 12px`
- Font: `label-small` (11px, 600 weight, uppercase)
- Variants:

| Status | Background | Text Color |
|---|---|---|
| SETTLED | `var(--neutral-main)` | `var(--neutral-on-main)` |
| AUTHORIZED | `var(--neutral-main)` | `var(--neutral-on-main)` |
| SETTLING | `var(--warning-container)` | `var(--warning-on-container)` |
| PAUSED | `var(--brand-container)` | `var(--brand-on-container)` |
| DECLINED | `var(--danger-container)` | `var(--danger-on-container)` |

- **Pages:** Transaction lists (dashboard, card detail, transactions page)

#### Agent Status Badge
- Shape: `radius-full` (pill)
- Padding: `2px 10px 2px 8px`
- Font: `body-small` (13px)
- Includes `6px` dot with `4px` gap before text

| Status | Background | Text | Dot Color |
|---|---|---|---|
| Active | `var(--success-container)` | `var(--success-on-container)` | `var(--success-main)` |
| Pending | `var(--warning-container)` | `var(--warning-on-container)` | `var(--warning-main)` (pulsing) |
| Suspended | `var(--danger-container)` | `var(--danger-on-container)` | `var(--danger-main)` |
| Expired | `var(--neutral-main)` | `var(--neutral-on-main)` | `var(--text-muted)` |

- **Pages:** Agent list, agent detail, dashboard agent summary

#### Plan / Category Badge
- Background: `var(--brand-container)` (`#302E6E`)
- Text: `var(--brand-on-container)` (`#B5B5F9`)
- Radius: `radius-full` (`64px`)
- Padding: `4px 12px`
- Font: `body` (14px), weight 600
- **Pages:** Hero banner plan indicator, agent tags

#### Dot Badge
- `8px` colored dot + text label
- Dot colors: success green, warning amber, danger red
- Text: `body-small`, `var(--text-subtle)`
- Gap between dot and text: `space-2` (`8px`)
- **Pages:** Card tiles ("Unused" green dot), network status indicator

### 9.3 Cards / Content Boxes

#### Content Box (Standard Card)
- Background: `var(--container-background)`
- Border: `1px solid var(--border-default)`
- Radius: `radius-xl` (`16px`) -- Privacy.com standard
- Padding: `24px`
- Shadow: `shadow-sm`
- Hover (if interactive): `shadow-card-hover`, `translateY(-2px)`, 200ms ease
- **Pages:** Dashboard sections, settings cards, agent detail cards

#### Card Tile (Grid Item)
- Background: `var(--container-background)`
- Radius: `radius-2xl` (`24px`) -- Privacy.com cards listing
- Padding: `16px 16px 8px`
- Border: `0px solid var(--border-strong)` (invisible, appears on hover)
- Width: fills grid column (~286px in 5-col layout)
- Hover: border becomes visible, subtle overlay for actions
- **Pages:** Cards listing grid

#### Hero Banner Card
- Background: `var(--container-background)`
- Radius: `radius-xl` (`16px`)
- Padding: `40px` (Privacy.com p-40)
- Full-width within content area
- Contains: badge, greeting, subtitle, CTA button
- **Pages:** Dashboard hero section

#### Balance Card (Credit Card Style)
- Width: `100%` (max `480px`)
- Height: auto (min `220px`)
- Radius: `radius-xl` (`16px`)
- Background: `var(--gradient-primary)`
- Padding: `28px 32px`
- Shadow: `shadow-lg`
- Text: `var(--text-inverse)` (`#FFFFFF`)
- Balance font: `mono-large` (48px, 700 weight)
- Asset label: `body-small`, `opacity: 0.8`
- Secondary balances: `body`, `opacity: 0.7`
- Wallet address: `mono-small` (12px), truncated (first 6 + last 4), `opacity: 0.8`
- Fund button: white bg, primary text, `radius-md`
- Hover: subtle 3D parallax tilt (max 3deg), shadow deepens, shine effect shifts
- **Pages:** Dashboard

#### Virtual Card Visual
- Radius: `radius-xl` (`16px`)
- Size: ~`254px x 163px` (credit card proportions, ~1.586:1)
- Border: `1px solid rgba(0, 0, 0, 0.1)`
- Gradient bg with watermark pattern, unique per card
- Elements: merchant logo (top-left), lock icon, status badge (top-right), name (bottom-left), last 4 (bottom-right)
- **Pages:** Cards listing tiles, card detail sidebar

### 9.4 Navigation

#### Sidebar Navigation
- Width: `240px` (expanded), `72px` (collapsed)
- Background: `var(--container-background)` (dark: `#1A1D27`, light: `#FFFFFF`)
- Border right: `1px solid var(--border-default)`
- Logo area: `64px` height, centered
- Nav item height: `40px`
- Nav item padding: `8px 12px`
- Nav item radius: `radius-md` (`8px`)
- Nav item gap: `4px`
- Icon size: `20px`
- Icon-text gap: `12px`
- Active item: `var(--brand-50)` bg (dark: 10% opacity brand), `var(--brand-bright)` text + icon
- Hover item: `var(--surface-raised)` background
- Inactive text: `var(--text-subtle)`
- Section divider: `1px solid var(--border-default)`, `12px` vertical margin
- Bottom area: user email (truncated), wallet indicator, settings link
- Collapse transition: width 200ms ease, text fades, icons remain centered
- **Pages:** All pages (app shell)

#### Top Nav Bar (Privacy.com pattern, for reference)
- Background: `var(--container-background)` (`#1B1C26` dark)
- Height: `64px`
- Padding: `0 40px`
- Nav links gap: `64px`
- Active state: underline indicator
- **Note:** Tally uses sidebar nav instead, but this pattern available for alternate layouts

#### Breadcrumb Pill
- Background: `var(--container-background)`
- Radius: `radius-full` (`64px` / `100px`)
- Padding: `0 24px`
- Font: `body` (14px), weight 500
- Active link: `var(--text-primary)`
- Current page: `var(--text-subtle)`
- Separator: chevron SVG, `8px` gap between items
- **Pages:** Card detail, agent detail

### 9.5 Section Pattern

#### Section Header
- Layout: `display: flex; justify-content: space-between; align-items: center`
- Padding bottom: `16px` (Privacy.com pattern)
- Title: `heading-2` (20px, 600) or `heading-3` (16px, 600)
- Action: link button or secondary button (right-aligned)
- **Pages:** Dashboard sections, card detail, all list pages

#### Tab Bar
- Inline text buttons, `gap: 24px`
- Active: `var(--brand-bright)` underline (2px), `var(--text-primary)` text
- Inactive: `var(--text-subtle)`, hover `var(--text-primary)`
- Count badge: `(N)` suffix on each tab
- **Pages:** Agent list filter tabs, wallet section (Favorites / Recently Used)

#### Segmented Control
- Pill container: `var(--container-background)`, `radius-full`
- Toggle buttons inside: `padding: 8px 16px`
- Active: highlighted background (`var(--surface-raised)`)
- Gap: `8px` between items
- **Pages:** Card listing state filter (Open | Paused | Closed), transaction filter (Approved | Declined)

### 9.6 Transaction Row

- Layout: `display: flex; align-items: center`, full-width
- Row height: `56px`
- Padding: `12px 16px`
- Elements (left to right):
  - **Icon**: `36px` circle, type-specific bg + icon color
    - Send: `var(--brand-50)` bg (dark: 10% brand) + `var(--brand-bright)` icon
    - Receive: `var(--success-container)` bg + `var(--success-main)` icon
    - Earn: `var(--warning-container)` bg + `var(--warning-main)` icon
  - **Date**: `body-small`, `var(--text-muted)` (e.g., "Feb 25, 8:45pm")
  - **Agent/Merchant name**: `body`, `var(--text-primary)`, medium weight (500)
  - **Status badge**: (see 9.2)
  - **Amount**: `body` (14px) in `--font-mono`, medium weight
    - Positive: `var(--success-main)`
    - Negative: `var(--danger-main)`
  - **Chevron**: `>` icon, `var(--text-muted)`, 16px
- Divider: `1px solid var(--border-default)` between rows
- Hover: `var(--surface-raised)` background
- **Pages:** Dashboard recent transactions, card detail transactions, transactions page

### 9.7 Spend / Metric Display

- Label: `body-small`, `var(--text-muted)`, above
- Amount: `mono-display` (24px, 600 weight) or `heading-2` (20px, 600) for spent amount
- Limit: `body`, `var(--text-subtle)`, inline after "of"
- Progress bar:
  - Height: `6px` (compact) or `8px` (standard)
  - Track: `var(--surface-raised)` (dark: `#232736`, light: `#F3F4F6`)
  - Track radius: `radius-full`
  - Fill radius: `radius-full`
  - Fill color: `var(--brand-bright)` (< 75%), `var(--warning-main)` (75-90%), `var(--danger-main)` (> 90%)
  - Animation: width transition 500ms ease-out
- Description: `body-small`, `var(--text-muted)`, below progress bar
- **Pages:** Dashboard spend snapshot, card detail spend limit, agent detail spending limits

### 9.8 Form Inputs

- Height: `40px` (default), `48px` (large, for onboarding)
- Background: `var(--surface-raised)` (dark: `#232736`, light: `#FFFFFF`)
- Border: `1px solid var(--border-default)`
- Radius: `radius-md` (`8px`)
- Padding: `10px 16px`
- Font: `body` (14px)
- Placeholder: `var(--text-muted)`
- Focus: `var(--brand-bright)` border + `2px` ring `var(--ring)`, `2px` offset
- Error: `var(--danger-main)` border, error text `body-small` in `var(--danger-main)` below, `4px` gap
- Label: `label` (13px, 500), `var(--text-subtle)`, `4px` gap above input
- **Search input variant:** `radius-full` (pill), search icon + placeholder, bg `var(--surface-raised)` (dark: `#323242`), padding `8px 16px`
- **Pages:** Onboarding email input, settings forms, agent detail edit, search bars

#### OTP Input
- 6 individual digit inputs in a row
- Each: `48px x 56px`, `radius-lg` (`12px`)
- Gap: `8px`
- Font: `30px`, 700 weight, centered (display-sm equivalent)
- Border: `2px solid var(--border-default)`
- Focused: `2px solid var(--brand-bright)`
- Filled: `var(--brand-50)` background (10% brand in dark)
- Error: shake 3 cycles, 4px amplitude, 300ms; border `var(--danger-main)`
- Success: flash `var(--success-container)` bg, checkmark overlay
- **Pages:** Onboarding OTP verification

### 9.9 Pagination

- Layout: flex row, right-aligned
- Info text: "Showing X - Y of Z" in `body-small`, `var(--text-muted)`
- Page buttons: pill-shaped (`radius-full`), `32px x 32px`
- Active page: `var(--brand-container)` bg, `var(--brand-on-container)` text
- Inactive page: transparent, `var(--text-subtle)`, hover `var(--surface-raised)`
- Prev/next arrows: `<` `>` icon buttons, same size
- Gap: `4px` between buttons
- **Pages:** Cards listing, transactions table

### 9.10 Theme Switcher

- Pill container: `var(--container-background)`, `radius-full`
- 3 options: Auto | Light (sun icon) | Dark (moon icon)
- Option padding: `6px 12px`
- Active: `var(--surface-raised)` bg, `var(--text-primary)` text
- Inactive: `var(--text-muted)` text
- Transition: `150ms ease` on bg and color
- **Pages:** Settings, optionally in header/nav

### 9.11 Toast Notifications

- Position: top-right, `16px` from edges
- Width: max `360px`
- Radius: `radius-lg` (`12px`)
- Shadow: `shadow-lg`
- Padding: `16px`
- Background: `var(--container-background)`
- Border: `1px solid var(--border-default)`
- Enter: slide in from right + fade, 300ms ease-out
- Exit: slide out to right + fade, 200ms ease-in
- Auto-dismiss: 5s (info), 8s (warning/error)
- Close button: `X` icon, top-right
- Type indicator: colored left border 3px
  - Info: `var(--info-main)`
  - Success: `var(--success-main)`
  - Warning: `var(--warning-main)`
  - Error: `var(--danger-main)`
- **Pages:** Global (overlays any page)

### 9.12 Approval Card

- Background: `var(--container-background)`
- Radius: `radius-lg` (`12px`)
- Padding: `24px`
- Border-left accent: `3px solid var(--warning-main)` (pending indicator)
- Agent icon: `40px` circle
- Amount: `heading-2` (20px, 600 weight)
- Recipient: `mono` (14px), `var(--text-subtle)`
- Reason: `body`, `var(--text-primary)`, italic
- Timestamp: `body-small`, `var(--text-muted)`, top-right
- Approve: success button variant
- Deny: danger button (outline variant)
- Button gap: `12px`
- Expandable details: `body-small`, shows full tx metadata
- **Pages:** Approvals page

### 9.13 Agent Card

- Background: `var(--container-background)`
- Radius: `radius-lg` (`12px`)
- Padding: `20px`
- Border: `1px solid var(--border-default)`
- Shadow: `shadow-sm`
- Icon: `32px` circle with agent color/avatar
- Status dot: `8px` circle, positioned top-right of icon
- Name: `heading-3` (16px, 600)
- Purpose: `body-small`, `var(--text-subtle)`, single line, truncated
- Progress bar: `6px` height, `radius-full`
  - Track: `var(--surface-raised)`
  - Fill: `var(--brand-bright)` (< 75%), `var(--warning-main)` (75-90%), `var(--danger-main)` (> 90%)
- Spend text: `body-small`, right-aligned `$spent / $limit`
- Hover: `translateY(-2px)`, `shadow-card-hover`, 200ms ease
- Click: `scale(0.98)` (100ms), then navigate
- **Pages:** Dashboard agent summary, agent list

### 9.14 Data Table

- Header row: `var(--surface-raised)` bg, `heading-sm` equivalent (14px, 600), sticky
- Header border: `2px solid var(--border-default)` bottom
- Row height: `52px`
- Row padding: `12px 16px`
- Row divider: `1px solid var(--border-default)`
- Row hover: `var(--surface-raised)` background
- Sorted column: `var(--brand-bright)` text + sort icon
- Column alignment: text left, numbers right, status center
- Pagination: bottom-right, page size selector (25/50/100) + prev/next
- **Pages:** Transactions page, settings invitation codes table

---

## 10. Page Templates

### 10.1 Dashboard

- **Hero banner**: Full-width content box, `padding: 40px`, greeting (`display-1`, 36px/700) + subtitle + CTA pill button
- **Balance card**: Full spec in 9.3, max `480px`, prominent position
- **Quick actions**: Horizontal chip row, each `radius-full`, `12px 16px` padding, icon + label
- **Two-column row 1**: Agent summary grid (3-col agent cards) + "Add Agent" dashed card
- **Two-column row 2**: Recent transactions list (5 rows) + secondary info panel
- **Section headers**: "Your Agents" + "View all" link, "Recent Transactions" + "View All" link
- Gap between sections: `24px`

### 10.2 Cards / Agent Listing

- **Action bar**: Primary pill button ("New Card +") left + filter/sort controls right
- **Filter pill**: `var(--container-background)`, `radius-full`, contains count text + toggle buttons
- **Grid**: 3-column responsive (agents) or 5-column (cards-style), `24px` or `16px` gap
- **Card tiles**: `radius-2xl` (`24px`), `16px` padding
- **Pagination**: bottom-right, "Showing X - Y of Z", numbered buttons

### 10.3 Card / Agent Detail

- **Breadcrumb**: Pill-shaped, top of page
- **Action buttons**: Circular icon buttons row (star, pause, delete)
- **Two-column grid**: `grid-template-columns: 426px 1fr`, `gap: 24px`
- **Left sidebar**: Content box with card visual, editable name, tags, spend limit, funding source, settings
- **Right main**: Transaction list with filter tabs (Approved | Declined), month grouping
- **Section dividers**: `1px solid var(--border-default)` within sidebar card

### 10.4 Approval Queue (Tally-specific)

- **Page header**: "Approvals" + pending count subtitle
- **Vertical stack**: Approval cards (spec 9.12), `16px` gap
- **Newest first** ordering
- **Empty state**: Shield checkmark illustration, "All caught up!", subtitle, no CTA

### 10.5 Settings

- **Vertical stack** of settings cards, `16px` gap
- **Cards**: Global spending policy, notifications (toggle rows), invitation codes (table), network toggle, wallet info
- **Kill switch**: Large red toggle, confirms via modal
- **Invitation codes table**: code (mono), status badge, date, linked agent, revoke button

### 10.6 Onboarding (Full-screen, no sidebar)

- **Centered card**: max-width `440px`, no sidebar
- **Welcome**: Logo + headline (`heading-1`) + subtext + "Get Started" primary lg button, full width
- **Email input**: Back button (ghost) + header + email input (lg, 48px) + submit button + legal text
- **OTP verification**: 6-digit OTP input (centered) + resend link + verify button + countdown timer
- **Background**: Subtle gradient mesh using `var(--brand-50)` and `var(--page-background)`

---

## 11. Iconography

- **Source**: Lucide React icons (bundled with shadcn/ui)
- **Default size**: `20px`
- **Stroke width**: `1.5px`
- **Color**: inherits from parent text color

| Context | Icon | Notes |
|---|---|---|
| Dashboard nav | `LayoutDashboard` | |
| Agents nav | `Bot` | |
| Transactions nav | `ArrowLeftRight` | |
| Approvals nav | `ShieldCheck` | |
| Fund nav | `Wallet` | |
| Settings nav | `Settings` | |
| Send tx | `ArrowUpRight` | |
| Receive tx | `ArrowDownLeft` | |
| Earn tx | `Sparkles` | |
| Copy | `Copy` -> `Check` on click | 2s revert |
| Add/Create | `Plus` | |
| Remove/Delete | `X` or `Trash2` | |
| Edit | `Pencil` | |
| Search | `Search` | |
| Filter | `Filter` | |
| Export | `Download` | |
| Back | `ChevronLeft` | |
| Expand/Collapse | `ChevronDown` / `ChevronUp` | |
| External link | `ExternalLink` | |
| Info | `Info` | |
| Warning | `AlertTriangle` | |
| Error | `AlertCircle` | |
| Success | `CheckCircle` | |
| Notification | `Bell` | |
| Kill switch | `Power` | Red when active |
| QR code | `QrCode` | |
| Lock | `Lock` | Merchant-locked cards |
| Star/Favorite | `Star` | Card detail actions |
| Pause | `Pause` | Card/agent pause |
| Sun (light mode) | `Sun` | Theme switcher |
| Moon (dark mode) | `Moon` | Theme switcher |

---

## 12. CSS Variable Reference

Complete CSS custom property definitions for both themes. Ready to paste into `globals.css`.

```css
/* ==========================================================================
   Tally Agentic Wallet -- Design System CSS Variables
   Dark-first, with light theme override.
   ========================================================================== */

:root {
  /* -- Font Stacks -- */
  --font-sans: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  --font-mono: 'JetBrains Mono', 'SF Mono', 'Fira Code', 'Courier New', monospace;

  /* -- Spacing Scale (4px base grid) -- */
  --space-0: 0px;
  --space-0-5: 2px;
  --space-1: 4px;
  --space-1-5: 6px;
  --space-2: 8px;
  --space-3: 12px;
  --space-4: 16px;
  --space-5: 20px;
  --space-6: 24px;
  --space-8: 32px;
  --space-10: 40px;
  --space-12: 48px;
  --space-16: 64px;

  /* -- Border Radius -- */
  --radius-sm: 4px;
  --radius-md: 8px;
  --radius-lg: 12px;
  --radius-xl: 16px;
  --radius-2xl: 24px;
  --radius-full: 64px;

  /* -- Transitions -- */
  --transition-fast: 100ms ease;
  --transition-normal: 200ms cubic-bezier(.55, 0, .1, 1);
  --transition-slow: 300ms ease-out;
  --transition-entrance: 500ms cubic-bezier(.55, 0, .1, 1);
  --transition-page: 250ms ease-in-out;

  /* -- Layout -- */
  --sidebar-width: 240px;
  --sidebar-width-collapsed: 72px;
  --content-max-width: 1200px;
  --header-height: 64px;
  --page-gutter: 40px;

  /* -- Z-Index -- */
  --z-base: 0;
  --z-dropdown: 10;
  --z-sticky: 20;
  --z-modal-backdrop: 30;
  --z-modal: 40;
  --z-toast: 50;
  --z-tooltip: 60;

  /* -- Opacity -- */
  --opacity-disabled: 0.5;
  --opacity-placeholder: 0.4;
  --opacity-overlay: 0.6;
  --opacity-glass: 0.7;
}

/* ==========================================================================
   DARK THEME (Default)
   ========================================================================== */
:root,
.dark {
  color-scheme: dark;

  /* -- Backgrounds -- */
  --page-background: #0F1117;
  --container-background: #1A1D27;
  --surface-raised: #232736;
  --surface-sunken: #141620;

  /* -- Text -- */
  --text-primary: #F1F1F4;
  --text-subtle: #D1D1DF;
  --text-muted: #828299;
  --text-inverse: #1A1A1A;

  /* -- Borders -- */
  --border-default: #2D3142;
  --border-strong: #4C4C5D;
  --ring: #6366F1;

  /* -- Brand -- */
  --brand-main: #4F46E5;
  --brand-bright: #6366F1;
  --brand-hover: #4338CA;
  --brand-on-main: #FFFFFF;
  --brand-container: #302E6E;
  --brand-on-container: #B5B5F9;
  --brand-50: rgba(99, 102, 241, 0.1);
  --gradient-primary: linear-gradient(135deg, #6366F1 0%, #8B5CF6 50%, #7C3AED 100%);
  --gradient-primary-hover: linear-gradient(135deg, #4F46E5 0%, #7C3AED 50%, #6366F1 100%);

  /* -- Success -- */
  --success-main: #10B981;
  --success-on-main: #FFFFFF;
  --success-container: #254D1E;
  --success-on-container: #A2EC8E;
  --success-600: #059669;
  --success-700: #047857;

  /* -- Danger -- */
  --danger-main: #EF4444;
  --danger-on-main: #FFFFFF;
  --danger-container: #5E2121;
  --danger-on-container: #F7ABAB;
  --danger-600: #DC2626;
  --danger-700: #B91C1C;

  /* -- Warning -- */
  --warning-main: #F59E0B;
  --warning-on-main: #FFFFFF;
  --warning-container: #5E4212;
  --warning-on-container: #F1D394;
  --warning-600: #D97706;

  /* -- Info -- */
  --info-main: #2C98D6;
  --info-on-main: #FFFFFF;
  --info-container: #133F58;
  --info-on-container: #9ED5F4;

  /* -- Neutral -- */
  --neutral-main: #4C4C5D;
  --neutral-on-main: #F1F1F4;
  --neutral-container: #66667A;
  --neutral-on-container: #FFFFFF;

  /* -- Shadows (dark theme: reduced, cool tint) -- */
  --shadow-color: color-mix(in srgb, #000 8%, transparent);
  --shadow-sm: 0 2px 8px 1px var(--shadow-color);
  --shadow-md: 0 4px 12px 1px var(--shadow-color);
  --shadow-lg: 0 8px 16px 1px var(--shadow-color);
  --shadow-xl: 0 12px 24px 2px color-mix(in srgb, #000 12%, transparent);
  --shadow-card-hover: 0 8px 20px color-mix(in srgb, #000 12%, transparent);

  /* -- Skeleton -- */
  --skeleton-base: #232736;
  --skeleton-shimmer: #2D3142;
}

/* ==========================================================================
   LIGHT THEME
   ========================================================================== */
.light {
  color-scheme: light;

  /* -- Backgrounds -- */
  --page-background: #FAFAF9;
  --container-background: #FFFFFF;
  --surface-raised: #FFFFFF;
  --surface-sunken: #F9FAFB;

  /* -- Text -- */
  --text-primary: #1A1A1A;
  --text-subtle: #6B7280;
  --text-muted: #9CA3AF;
  --text-inverse: #FFFFFF;

  /* -- Borders -- */
  --border-default: #E8E5E0;
  --border-strong: #D1D5DB;
  --ring: #6366F1;

  /* -- Brand -- */
  --brand-main: #4F46E5;
  --brand-bright: #4F46E5;
  --brand-hover: #4338CA;
  --brand-on-main: #FFFFFF;
  --brand-container: #EEF2FF;
  --brand-on-container: #312E81;
  --brand-50: #EEF2FF;
  --gradient-primary: linear-gradient(135deg, #4F46E5 0%, #7C3AED 50%, #6366F1 100%);
  --gradient-primary-hover: linear-gradient(135deg, #4338CA 0%, #6D28D9 50%, #4F46E5 100%);

  /* -- Success -- */
  --success-main: #10B981;
  --success-on-main: #FFFFFF;
  --success-container: #ECFDF5;
  --success-on-container: #047857;
  --success-600: #059669;
  --success-700: #047857;

  /* -- Danger -- */
  --danger-main: #EF4444;
  --danger-on-main: #FFFFFF;
  --danger-container: #FEF2F2;
  --danger-on-container: #B91C1C;
  --danger-600: #DC2626;
  --danger-700: #B91C1C;

  /* -- Warning -- */
  --warning-main: #F59E0B;
  --warning-on-main: #FFFFFF;
  --warning-container: #FFFBEB;
  --warning-on-container: #92400E;
  --warning-600: #D97706;

  /* -- Info -- */
  --info-main: #2C98D6;
  --info-on-main: #FFFFFF;
  --info-container: #EFF6FF;
  --info-on-container: #1E40AF;

  /* -- Neutral -- */
  --neutral-main: #E5E7EB;
  --neutral-on-main: #374151;
  --neutral-container: #F3F4F6;
  --neutral-on-container: #1F2937;

  /* -- Shadows (light theme: warm tint, more visible) -- */
  --shadow-color: rgba(0, 0, 0, 0.06);
  --shadow-sm: 0 1px 3px rgba(0,0,0,0.06), 0 1px 2px rgba(0,0,0,0.04);
  --shadow-md: 0 4px 6px rgba(0,0,0,0.05), 0 2px 4px rgba(0,0,0,0.04);
  --shadow-lg: 0 10px 15px rgba(0,0,0,0.06), 0 4px 6px rgba(0,0,0,0.04);
  --shadow-xl: 0 20px 25px rgba(0,0,0,0.08), 0 8px 10px rgba(0,0,0,0.04);
  --shadow-card-hover: 0 8px 20px rgba(0,0,0,0.08), 0 2px 6px rgba(0,0,0,0.04);

  /* -- Skeleton -- */
  --skeleton-base: #F3F4F6;
  --skeleton-shimmer: #E5E7EB;
}

/* ==========================================================================
   BASE TYPOGRAPHY CLASSES
   ========================================================================== */

/*
.display-1    { font: 700 36px/1.3 var(--font-sans); letter-spacing: -0.02em; }
.display-2    { font: 700 28px/1.3 var(--font-sans); letter-spacing: -0.01em; }
.heading-1    { font: 600 24px/1.3 var(--font-sans); letter-spacing: -0.01em; }
.heading-2    { font: 600 20px/1.35 var(--font-sans); letter-spacing: -0.005em; }
.heading-3    { font: 600 16px/1.4 var(--font-sans); letter-spacing: 0; }
.body-large   { font: 400 16px/1.6 var(--font-sans); letter-spacing: 0; }
.body         { font: 400 14px/1.5 var(--font-sans); letter-spacing: 0; }
.body-small   { font: 400 13px/1.5 var(--font-sans); letter-spacing: 0.01em; }
.label        { font: 500 13px/1.4 var(--font-sans); letter-spacing: 0.02em; }
.label-small  { font: 600 11px/1.3 var(--font-sans); letter-spacing: 0.05em; text-transform: uppercase; }
.mono-large   { font: 700 48px/1.1 var(--font-mono); letter-spacing: -0.02em; }
.mono-display { font: 600 24px/1.3 var(--font-mono); letter-spacing: 0; }
.mono         { font: 400 14px/1.5 var(--font-mono); letter-spacing: 0; }
.mono-small   { font: 400 12px/1.5 var(--font-mono); letter-spacing: 0; }
*/
```

---

## 13. Quick Reference Card

| Property | Value |
|---|---|
| Primary font | Inter (400, 500, 600, 700) |
| Mono font | JetBrains Mono (400, 600, 700) |
| Base spacing unit | 4px |
| Card radius | 16px (`radius-xl`) |
| Button radius | 8px (`radius-md`) |
| Pill radius | 64px (`radius-full`) |
| Tile radius | 24px (`radius-2xl`) |
| Primary color | #4F46E5 (indigo-600) |
| Primary bright (dark) | #6366F1 (indigo-500) |
| Success | #10B981 |
| Warning | #F59E0B |
| Danger | #EF4444 |
| Info | #2C98D6 |
| Background (dark) | #0F1117 |
| Background (light) | #FAFAF9 |
| Container (dark) | #1A1D27 |
| Container (light) | #FFFFFF |
| Border (dark) | #2D3142 |
| Border (light) | #E8E5E0 |
| Card shadow (dark) | `0 2px 8px 1px color-mix(in srgb, #000 8%, transparent)` |
| Card shadow (light) | `0 1px 3px rgba(0,0,0,0.06), 0 1px 2px rgba(0,0,0,0.04)` |
| Default transition | 200ms cubic-bezier(.55, 0, .1, 1) |
| Sidebar width | 240px / 72px collapsed |
| Content max-width | 1200px |
| Header height | 64px |
| Balance card gradient (dark) | `linear-gradient(135deg, #6366F1, #8B5CF6, #7C3AED)` |
| Balance card gradient (light) | `linear-gradient(135deg, #4F46E5, #7C3AED, #6366F1)` |
