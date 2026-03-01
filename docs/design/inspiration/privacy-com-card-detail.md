# Privacy.com Card Detail Page — Design Reference
Source: https://app.privacy.com/cards/{card-id}
Extracted: 2026-02-28

## Overview
Individual card management page showing card details, settings, and transaction history. Two-column grid layout with a sidebar for card info and a main area for transactions.

## Page Layout
- **Grid layout**: `display: grid; grid-template-columns: 426px 1fr; gap: 24px;`
- Left column: Card details sidebar (426px fixed)
- Right column: Transaction history (flexible)
- Same header/nav as dashboard

## Breadcrumb Navigation
- Pill-shaped breadcrumb: `Cards > Apple Developer`
- Background: rgb(27, 28, 38) / #1b1c26
- Border-radius: 100px (pill)
- Padding: 0 24px
- Font: Graphik, 14px, weight 500
- Active link: --text-primary color
- Current page: --text-subtle color
- Gap between items: 8px
- Chevron separator SVG between items

## Action Bar (Top Right)
Four circular icon buttons in a row:
1. **Star** (Favorite) — "Add Card to Favorites"
2. **Download** — export/share
3. **Pause** (||) — pause card
4. **Delete** (trash, red) — close card

Button styles:
- Shape: circle (`border-radius: 50%`)
- Size: 56x54px
- Background: rgb(27, 28, 38) / #1b1c26
- Border: 1px solid #1b1c26 (same as bg, so invisible)
- Color: rgb(240, 240, 245) / --text-primary
- Font: 14px, weight 500
- Delete button has red/danger icon color

## Left Sidebar — Card Details

### Card Visual
- Container: `content-box box-container radius-16`
- Background: rgb(27, 28, 38) / #1b1c26
- Border-radius: 16px
- Padding: 24px
- No visible border (0px solid)

#### Virtual Card Display
- Class: `privacy-card locked full-card`
- Dark card design with Privacy "P" watermark pattern
- Merchant logo (Apple icon) top-left
- "MERCHANT-LOCKED" badge with lock icon, top-right
- Mastercard logo bottom-right
- Card number area: "Show Details" button + `•••• 7547`
- Card background: dark gradient with geometric pattern

#### Card Name
- Editable field: "Apple Developer" with pencil/edit icon
- Large text, appears to be an inline-editable input
- Font size appears ~20px, bold

#### Tags
- "+ Add Tags" button
- Full-width, bordered button style
- Centered text with plus icon

#### Spend Limit Section
- Label: "Spend Limit" (small, muted text)
- Amount: "$107.79 of $200 yearly"
- Large bold amount + "of" + limit + period
- Three-dot menu (...) on the right for editing
- Separated by horizontal dividers (1px border lines)

#### Funding Source
- Bank icon (Chase blue logo)
- "TOTAL CHECKING" label (uppercase, small)
- Status badge: "Connection Expired" (warning/yellow)
- Bank info: "Chase ... 2355"

#### Merchant Lock
- Hamburger/list icon on left
- "Merchant-Locked" label
- "How It Works" link (brand color)
- Locked merchant badge: "APPLE.COM/US" in a pill/badge

### Section Dividers
- Thin horizontal lines between each section within the card
- Color: --foreground-border (#4c4c5d)

## Right Column — Transactions

### Header
- Title: "Transactions" (h2, large bold)
- Filter buttons on right: "Approved" | "Declined" (toggle/segmented control)
- Filter buttons have border, rounded, pill-like appearance

### Transaction List

#### Month Grouping
- "This Month" section header (small, muted text)

#### Transaction Rows
Each row contains:
- Date/time (left): "Feb 11, 10:24am" — muted/subtle color
- Merchant name: "Apple.Com/Us" — primary text, bold
- Status badge (right area): SETTLED, AUTHORIZED
- Amount: "$8.79", "$99", "$0"
- Chevron (>) icon for navigation

#### Status Badges
- **SETTLED**: Default style (neutral border, muted text)
- **AUTHORIZED**: Similar neutral style
- Badge: small pill with border, uppercase text

### Transaction Row Styles
- Full width, padding for comfortable click targets
- Hover state likely highlights
- Chevron (>) on far right for drill-down

## Color Palette (same dark theme as dashboard)
```css
--page-background: #323242;
--container-background: #1b1c26;
--foreground-border: #4c4c5d;
--text-primary: #f0f0f5;
--text-subtle: #d1d1df;
--text-muted: #828299;
--variant-brand-main: #4949f1;
--variant-danger-main: #ed5a5a;
--variant-warning-main: #df9e33;
```

## Key Computed Styles Summary

| Element | Background | Text Color | Border Radius | Padding | Font Size |
|---------|-----------|------------|---------------|---------|-----------|
| Card container | #1b1c26 | #f0f0f5 | 16px | 24px | 16px |
| Action buttons | #1b1c26 | #f0f0f5 | 50% (circle) | 20px 24px | 14px |
| Breadcrumb pill | #1b1c26 | #f0f0f5 | 100px | 0 24px | 14px |
| Transaction area | transparent | #f0f0f5 | 16px | - | 16px |
| Badge | varies | varies | pill | 4px 12px | 12-14px |

## Design Patterns

1. **Two-column grid**: Fixed sidebar + flexible main content
2. **Card-as-container**: Everything in the sidebar is one big card (content-box)
3. **Inline editing**: Card name is directly editable
4. **Section dividers**: Thin horizontal lines within the card container
5. **Circular action buttons**: Icon-only actions in the header
6. **Pill breadcrumb**: Navigation context in a rounded pill
7. **Segmented filter**: Approved/Declined toggle for transactions
8. **Month grouping**: Transactions grouped by time period
9. **Status badges**: Color-coded transaction status indicators
10. **Consistent spacing**: 24px gap between grid columns, 24px padding in containers

## HTML Structure (Simplified)

```html
<main class="app-main">
  <div class="page-content flex-column gap-24">

    <!-- ACTION BAR -->
    <header>
      <div class="action-bar-content flex-wrap">
        <!-- Breadcrumb -->
        <div class="pill breadcrumb">
          <a>Cards</a> <svg/> <span>Apple Developer</span>
        </div>
        <!-- Action Buttons -->
        <button class="btn btn-action radius-circle btn-control"><!-- star --></button>
        <button class="btn btn-action radius-circle btn-control"><!-- download --></button>
        <button class="btn btn-action radius-circle btn-control"><!-- pause --></button>
        <button class="btn btn-action radius-circle btn-control"><!-- delete --></button>
      </div>
    </header>

    <!-- MAIN CONTENT GRID -->
    <div class="card-content" style="display: grid; grid-template-columns: 426px 1fr; gap: 24px;">

      <!-- LEFT: CARD DETAILS SIDEBAR -->
      <section class="details flex-column gap-24">
        <h1 class="visually-hidden">Apple Developer Card Details</h1>
        <div class="content-box box-container radius-16 card-details" style="padding: 24px;">

          <!-- Card Visual -->
          <div class="privacy-card locked full-card">
            <div class="card-logo"><!-- Apple icon --></div>
            <div class="badge">MERCHANT-LOCKED</div>
            <div class="card-footer">
              <button>Show Details</button>
              <span>•••• 7547</span>
              <div class="mastercard-logo"/>
            </div>
          </div>

          <!-- Card Name (editable) -->
          <div class="card-name">
            <input value="Apple Developer"/>
            <button><!-- pencil icon --></button>
          </div>

          <!-- Tags -->
          <button class="add-tags">+ Add Tags</button>

          <hr/>

          <!-- Spend Limit -->
          <div class="spend-limit">
            <span class="label">Spend Limit</span>
            <span class="amount">$107.79 of $200 yearly</span>
            <button>...</button>
          </div>

          <hr/>

          <!-- Funding Source -->
          <div class="funding-source">
            <div class="bank-icon"><!-- Chase logo --></div>
            <div>
              <div class="bank-name">TOTAL CHECKING</div>
              <span class="badge warning">Connection Expired</span>
              <span>Chase ... 2355</span>
            </div>
          </div>

          <hr/>

          <!-- Merchant Lock -->
          <div class="merchant-lock">
            <span>Merchant-Locked</span>
            <a>How It Works</a>
            <span class="badge">APPLE.COM/US</span>
          </div>

        </div>
      </section>

      <!-- RIGHT: TRANSACTIONS -->
      <div class="card-transactions radius-16">
        <div class="content-box box-container">
          <header>
            <h2>Transactions</h2>
            <div class="filters">
              <button class="active">Approved</button>
              <button>Declined</button>
            </div>
          </header>

          <div class="month-group">
            <h4>This Month</h4>

            <div class="transaction-row">
              <span class="date">Feb 11, 10:24am</span>
              <span class="merchant">Apple.Com/Us</span>
              <span class="badge">SETTLED</span>
              <span class="amount">$8.79</span>
              <svg><!-- chevron --></svg>
            </div>
            <!-- More rows... -->
          </div>
        </div>
      </div>

    </div>
  </div>
</main>
```
