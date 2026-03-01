# Privacy.com Dashboard — Design Reference
Source: https://app.privacy.com/home
Extracted: 2026-02-28

## 1. Overview
Privacy.com is a virtual card/financial dashboard app. Dark mode by default. Clean, minimal financial UI with card management, spend tracking, and transaction history.

## 2. Layout Structure
- Full-width top navigation bar (sticky)
- Main content: max-width ~124.25rem, centered with 40px gutters
- Hero banner (full width, gradient bg)
- Two-column layout below hero:
  - Left: Wallet (card carousel) + Recent Transactions
  - Right: Spend Snapshot + Cashback/Credits
- Cards in content sections have 16px border-radius

## 3. Navigation
- Logo (PRIVACY wordmark, bordered) on left
- Search button
- Nav links: Home, Cards, Transactions, Help (centered)
- Theme switcher (Auto/Light/Dark toggle) on right
- User profile dropdown on far right
- Nav background: rgb(27, 28, 38) / #1b1c26

## 4. Color System (Dark Theme)
```css
--color-scheme: dark;
--page-background: #323242;
--container-background: #1b1c26;
--foreground-border: #4c4c5d;
--foreground-border-strong: #66667a;
--text-primary: #f0f0f5;
--text-subtle: #d1d1df;
--text-muted: #828299;

/* Brand */
--variant-brand-main: #4949f1;
--variant-brand-on-main: #fff;
--variant-brand-container: #302e6e;
--variant-brand-on-container: #b5b5f9;

/* Neutral */
--variant-neutral-main: #4c4c5d;
--variant-neutral-on-main: #f0f0f5;
--variant-neutral-container: #66667a;
--variant-neutral-on-container: #fff;

/* Strong */
--variant-strong-main: #f0f0f5;
--variant-strong-on-main: #1b1c26;
--variant-strong-container: #828299;
--variant-strong-on-container: #fff;

/* Success */
--variant-success-main: #55b938;
--variant-success-on-main: #fff;
--variant-success-container: #254d1e;
--variant-success-on-container: #a2ec8e;

/* Danger */
--variant-danger-main: #ed5a5a;
--variant-danger-on-main: #fff;
--variant-danger-container: #5e2121;
--variant-danger-on-container: #f7abab;

/* Warning */
--variant-warning-main: #df9e33;
--variant-warning-on-main: #fff;
--variant-warning-container: #5e4212;
--variant-warning-on-container: #f1d394;

/* Info */
--variant-info-main: #2c98d6;
--variant-info-on-main: #fff;
--variant-info-container: #133f58;
--variant-info-on-container: #9ed5f4;
```

## 5. Typography
- Font: **Graphik** (proprietary) — closest alternatives: Inter, -apple-system
- Base size: 16px, line-height: 1.3 (20.8px)
- Font weights: 400 (regular), 600 (semibold/strong)
- Monospace: "Courier New", Courier, monospace
- H1 (greeting): large bold text
- H2 (subheader): lighter weight subtitle
- H3 (section headers): semibold "weight-strong" class

## 6. Spacing System
Uses a size-based variable system:
- --size-1 through --size-64 (pixel-based spacing tokens)
- Common gaps: 8px, 16px, 24px, 40px, 64px
- Section padding: 40px (p-40)
- Card padding: 16px (p-16) to 24px (p-24)
- Grid: 4px base grid

## 7. Component Inventory

### Hero Banner
- Full-width dark container with padding-40
- Contains: plan badge, greeting, subtitle, "New Card +" CTA button
- "New Card" button: brand color (#4949f1), pill shape (radius-64), white text
- Plan badge: small pill with brand-container bg

### Wallet Card Carousel
- Horizontal scrolling cards with overlapping layout
- Cards have merchant logos/icons, lock icons, status badges (PAUSED, Unused)
- Card backgrounds: gradients (purple/blue, brand colors)
- Labels below each card

### Spend Snapshot
- Three metric cards side by side:
  - Daily Spend Limit: $0.00 of $5,000
  - Monthly Spend Limit: $152.12 of $20,000
  - Monthly Card Limit: 0 of 36
- Each has: label, large dollar amount, "of" max, progress bar, description text
- Progress bars: thin, colored based on percentage

### Recent Transactions
- List layout with rows
- Each row: merchant icon (rounded square), date, merchant name, status badge, amount, chevron
- Status badges: SETTLING (yellow/muted), AUTHORIZED (neutral), SETTLED (default)
- Hover state likely highlights row

### Cashback and Credits Section
- Available Credit: $0.00 with toggle for auto-apply
- Upgrade promotion card with illustration and "Upgrade Now" button (green/brand)
- "Refer a Friend" link button

### Section Headers
- Pattern: Title (left) + Action button/link (right)
- e.g., "Recent Transactions" + "View All Transactions"
- e.g., "Wallet" with tab bar: Favorites | Recently Used + "View All Cards"

### Theme Switcher
- Pill-shaped toggle with 3 options: Auto, Light (sun), Dark (moon)
- Active state highlighted

## 8. Shadows and Effects
```css
--shadow-small: 0 2px 8px 1px color-mix(in srgb, #000 8%, transparent);
--shadow-medium: 0 4px 12px 1px color-mix(in srgb, #000 8%, transparent);
--shadow-large: 0 8px 16px 1px color-mix(in srgb, #000 8%, transparent);
```

## 9. Borders
```css
--border-1: 1px solid #4c4c5d;
```
- Content boxes: 1px border with --foreground-border color
- Border radius: 16px for cards/sections, 64px for pill buttons

## 10. Transitions
```css
--transition-ease-in-out-100: .1s ease-in-out;
--transition-ease-in-out-300: .3s ease-in-out;
--transition-bezier-200: .2s cubic-bezier(.55,0,.1,1);
--transition-bezier-500: .5s cubic-bezier(.55,0,.1,1);
```

## 11. Key CSS Classes Observed
- Layout: `.flex`, `.flex-column`, `.gap-8/16/24/40/64`, `.p-16/24/40`, `.radius-16/64`
- Typography: `.weight-strong`, `.text-large`, `.text-muted`
- Components: `.content-box`, `.box-container`, `.base-badge`, `.btn`, `.btn-content`
- Variants: `.btn-variant-brand`, `.btn-variant-neutral`, `.on-variant-brand-main`
- Sizing: `.w-100`, `.grow-1`, `.shrink-0`

## 12. Design Patterns Worth Noting
- **Consistent section pattern**: header (title + action) -> content
- **Card-based layout**: Everything in bordered, rounded containers
- **Status badge system**: Color-coded badges for transaction states
- **Progressive disclosure**: Tabs in wallet section (Favorites / Recently Used)
- **Dark-first design**: All colors optimized for dark backgrounds
- **Subtle gradient use**: Hero banner, card backgrounds
- **Micro-interactions**: fade-in animations on hero content
