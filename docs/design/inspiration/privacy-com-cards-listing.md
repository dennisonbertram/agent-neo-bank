# Privacy.com Cards Listing Page — Design Reference
Source: https://app.privacy.com/cards?cardStates=OPEN&cardStates=PAUSED
Extracted: 2026-02-28

## Overview
Grid-based listing of all virtual cards. Shows card visuals in a 5-column responsive grid with filtering, sorting, and pagination. Each card tile shows the card visual, merchant name, last 4 digits, status badges, and spend limit summary.

## Page Layout
- Same top navigation as other pages
- No sidebar — full-width content area
- Action bar at top: "New Card" button (left) + Filters/Sort (right)
- Card grid below
- Pagination at bottom

## Action Bar

### New Card Button (left)
- Brand color: rgb(73, 73, 241) / #4949f1
- Text: "New Card +" with plus icon
- Pill shape: border-radius 64px
- Padding: 13px 24px
- Height: 56px
- Font: 14px, weight 500, white

### Filter/Sort Area (right)
Arranged as a horizontal bar with several controls:

#### State Filter Pill
- Container: `content-box box-container radius-64` (pill-shaped)
- Background: rgb(27, 28, 38) / #1b1c26
- Border-radius: 64px
- Contains: count text ("40 cards") + toggle buttons (Open | Paused | Closed)
- Toggle buttons are inline pill buttons
- Active state filter buttons appear highlighted/selected
- Gap: 8px between items

#### Filter by Tags
- Dropdown button: "Filter by tags" with chevron
- Popover/dropdown on click

#### Sort Dropdown
- Default: "Last Used" with chevron
- Sorts: Last Used, alphabetical, etc.

#### Sort Direction Toggle
- Icon button to toggle ascending/descending

#### Reset Filters
- "Reset Filters" text button with chevron

## Card Grid

### Grid Container
- Class: `cards-list`
- Display: grid
- Grid template: 5 equal columns (~286px each)
- Gap: 24px
- Full width: 1528px
- 20 cards per page

### Card Tile
- Class: `content-box box-container radius-24 card-tile`
- Background: rgb(27, 28, 38) / #1b1c26
- Border-radius: **24px** (larger than typical 16px)
- Padding: 16px 16px 8px (slightly less padding at bottom)
- Border: 0px solid rgb(76, 76, 93) (border defined but 0 width — likely appears on hover)
- Width: ~286px

### Card Visual (inside tile)
- Class: `privacy-card preview-card` (or `privacy-card merchant-locked preview-card`)
- Border-radius: 16px
- Size: ~254 x 163px (credit card proportions)
- Border: 1px solid rgba(0, 0, 0, 0.1)
- Overflow: hidden

#### Card Visual Elements
- **Merchant logo**: Top-left corner (e.g., Twilio, AWS, Apple logo)
- **Lock icon**: If merchant-locked, shows padlock icon
- **Status badge**: Top-right — "Unused" green dot badge
- **PAUSED banner**: Blue/brand-colored horizontal banner across card bottom area
- **Card name**: Bottom-left text (e.g., "Twilio")
- **Last 4 digits**: Bottom-right (e.g., "2013")
- **Background**: Gradient patterns unique to each card — purples, blues, reds, brand colors with Privacy "P" watermark pattern

#### Card States Visual Treatment
- **Active**: Normal card appearance
- **Paused**: Blue "PAUSED" banner overlaid across the card
- **Unused**: Green dot + "Unused" badge in top-right
- **Merchant-locked**: Lock icon visible

### Spend Limit Text (below card)
- Position: Below card visual, within the tile
- Layout: flex, space-between
- Examples: "$0 / $40 monthly", "$107.79 / $200 yearly", "$100 per transaction", "(no spend limit)"
- Color: rgb(240, 240, 245) / --text-primary
- Font size: 16px
- Also contains a "Close" action (likely a small X or button, for quick card close)

## Pagination
- Position: Bottom-right of page
- Text: "Showing 1 - 20 of 40 cards"
- Page numbers: 1, 2 with prev/next arrows (< >)
- Active page number highlighted
- Pill/button style for page numbers
- Muted text for the "Showing X - Y of Z" label

## Computed Styles Summary

| Element | Background | Border Radius | Padding | Size |
|---------|-----------|---------------|---------|------|
| Action bar | transparent | 0 | 0 | full width, 56px height |
| New Card btn | #4949f1 | 64px | 13px 24px | 136x56px |
| State filter pill | #1b1c26 | 64px | 8px 16px | auto |
| Card tile | #1b1c26 | 24px | 16px 16px 8px | ~286px wide |
| Card visual | transparent | 16px | 0 | ~254x163px |
| Pagination | — | pill | — | — |

## HTML Structure (Simplified)

```html
<main class="app-main">
  <div>
    <section class="page-content flex-column gap-24">

      <!-- ACTION BAR -->
      <header class="action-bar flex justify-content-between">
        <h2 class="visually-hidden">All Cards</h2>

        <!-- New Card Button -->
        <button class="btn btn-variant-brand radius-64 btn-new-card">
          New Card <svg><!-- plus --></svg>
        </button>

        <!-- Filters & Sort -->
        <div class="sort-filter-actions flex">

          <!-- State Filter Pill -->
          <div class="content-box radius-64 state-filter">
            <span class="count-text">40 cards</span>
            <div class="toggle-btn active">Open</div>
            <div class="toggle-btn active">Paused</div>
            <div class="toggle-btn">Closed</div>
          </div>

          <!-- Tag Filter -->
          <button class="btn popover-trigger">
            Filter by tags <svg><!-- chevron --></svg>
          </button>

          <!-- Sort Dropdown -->
          <button class="btn popover-trigger">
            Last Used <svg><!-- chevron --></svg>
          </button>

          <!-- Sort Direction -->
          <button class="btn btn-action">
            <svg><!-- sort icon --></svg>
          </button>

          <!-- Reset -->
          <button class="btn popover-trigger">
            Reset Filters <svg><!-- chevron --></svg>
          </button>
        </div>
      </header>

      <!-- CARD GRID -->
      <div class="cards-list" style="display: grid; grid-template-columns: repeat(5, 1fr); gap: 24px;">

        <!-- CARD TILE (repeated 20x) -->
        <div class="content-box box-container radius-24 card-tile">
          <!-- Card Visual Link -->
          <a class="card-link" href="/cards/{id}">
            <div class="privacy-card locked preview-card">
              <!-- Hover overlay -->
              <div class="overlay">
                <button>Pause</button>
              </div>
              <!-- Badge -->
              <div class="badge">Unused</div>
              <!-- Card content -->
              <div class="card-bg"><!-- gradient + P watermark --></div>
              <div class="card-logo"><!-- merchant logo --></div>
              <div class="lock-icon"><!-- padlock --></div>
              <div class="card-footer">
                <span class="card-name">Twilio</span>
                <span class="card-last4">2013</span>
              </div>
              <!-- Paused banner (if paused) -->
              <div class="status-banner">PAUSED</div>
            </div>
          </a>

          <!-- Spend Limit -->
          <div class="flex justify-content-between">
            <span>$0 / $40 monthly</span>
            <button>Close</button>
          </div>
        </div>

        <!-- ... more card tiles ... -->
      </div>

      <!-- PAGINATION -->
      <div class="pagination">
        <span>Showing 1 - 20 of 40 cards</span>
        <button>&lt;</button>
        <button class="active">1</button>
        <button>2</button>
        <button>&gt;</button>
      </div>

    </section>
  </div>
</main>
```

## Design Patterns

1. **5-column responsive grid**: Cards in fixed 5-column layout with 24px gaps
2. **Card tile container**: Larger border-radius (24px) for tiles vs 16px for card visual inside
3. **Hover overlay**: Cards reveal action buttons on hover (Pause, etc.)
4. **State filter pill**: Inline segmented control within a pill-shaped container
5. **Status visualization**: Visual banners on cards (PAUSED) + dot badges (Unused)
6. **Spend limit inline**: Quick-glance spend info below each card
7. **Pagination**: Simple numbered pagination for large card collections
8. **Consistent action bar**: "New Card" CTA always accessible at top
9. **Card proportions**: Standard credit card aspect ratio (~1.586:1)
10. **Gradient card backgrounds**: Each card has unique gradient + P watermark pattern for visual distinction
