# Transaction Detail Screen — v2 Design Spec

**Source**: `docs/design/redesign-v2.md` lines 2091–2382
**Screen title**: `Tally Wallet - Transaction Transparency`
**Screen ID / label in source**: `individual transaction`

---

## Purpose

The transaction detail screen provides full transparency into a single agent-initiated transaction. It shows the total amount paid, the agent that initiated it, structured metadata about the request, an itemized cost breakdown, free-text agent notes, and a link to view the transaction on a blockchain explorer. The design emphasizes trust and auditability.

---

## CSS Custom Properties (Design Tokens)

```css
:root {
  --bg-primary:      #FFFFFF;
  --bg-secondary:    #F8F9FA;
  --surface-hover:   #F2F2F7;
  --text-primary:    #111111;
  --text-secondary:  #8E8E93;
  --text-tertiary:   #C7C7CC;
  --accent-green:    #8FB5AA;
  --accent-yellow:   #F2D48C;
  --accent-terracotta: #D9A58B;
  --accent-blue:     #BCCCDC;
  --accent-green-dim: rgba(143, 181, 170, 0.15);
  --black:           #000000;
  --white:           #FFFFFF;

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

Note: This screen adds `--accent-yellow`, `--accent-terracotta`, and `--accent-blue` tokens not present in the agent detail screen, suggesting an expanded palette for categorization / tag coloring.

---

## App Container

```css
.app-container {
  width: 390px;
  height: 844px;
  background-color: var(--bg-primary);   /* #FFFFFF */
  position: relative;
  overflow: hidden;                       /* Note: hidden, not auto — scroll is on .screen */
  display: flex;
  flex-direction: column;
  box-shadow: 0 0 0 10px #000, 0 20px 50px rgba(0,0,0,0.2);
  border-radius: 40px;
}
```

Body centered on `#F2F2F2` background identical to other screens.

---

## Typography Scale

| Class | Font Size | Font Weight | Color | Notes |
|---|---|---|---|---|
| `.text-display` | 42px | 600 | `--text-primary` | `letter-spacing: -1px`, `line-height: 1.1` |
| `.text-title` | 22px | 600 | `--text-primary` | `letter-spacing: -0.5px` |
| `.text-subtitle` | 17px | 600 | `--text-primary` | `letter-spacing: -0.3px` |
| `.text-body` | 15px | 400 | `--text-secondary` | `line-height: 1.5` |
| `.text-caption` | 12px | 500 | `--text-secondary` | `text-transform: uppercase`, `letter-spacing: 0.5px` |
| `.text-mono` | 13px | 400 | inherited | `font-family: "SF Mono", "Menlo", monospace` |

Note: `.text-display` here is 42px vs 34px on the agent detail screen — the transaction amount gets more visual prominence.

---

## Layout Structure

```
app-container (390 x 844px, border-radius: 40px, overflow: hidden)
└── div.screen.animate-in (flex:1, overflow-y: auto)
    ├── button.nav-back (← Details)
    ├── div — Amount hero block (centered)
    ├── div — Agent identity row
    ├── h3.text-caption — "Agent Metadata"
    ├── div.meta-card — Metadata table
    ├── h3.text-caption — "Cost Breakdown"
    ├── div.meta-card — Cost breakdown table
    ├── h3.text-caption — "Notes"
    ├── div.meta-card — Agent notes (free text)
    └── div — CTA: "View on Explorer" button
```

No bottom navigation bar is present on this screen (detail view).

---

## Screen / Scroll Container (`.screen`)

```css
.screen {
  flex: 1;
  overflow-y: auto;
  padding: 60px var(--space-lg) 40px var(--space-lg);
  /* = 60px 24px 40px 24px */
  display: block;
}
```

Entry animation (`.animate-in`):
```css
@keyframes fadeIn {
  from { opacity: 0; transform: translateY(10px); }
  to   { opacity: 1; transform: translateY(0); }
}
.animate-in { animation: fadeIn 0.4s ease forwards; }
```

---

## Component Specifications

### Back Navigation (`.nav-back`)

```css
.nav-back {
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--text-primary);
  font-weight: 600;
  font-size: 15px;
  margin-bottom: var(--space-lg);   /* 24px */
  cursor: pointer;
  border: none;
  background: none;
  padding: 0;
}
```

Contains a left-chevron SVG (20x20, `stroke: currentColor`, `stroke-width: 2.5`, path `M15 18l-6-6 6-6`) followed by the text "Details". This is a text+icon back button, not a circular button like the agent detail screen.

### Amount Hero Block

Centered div (`text-align: center; margin-bottom: var(--space-xl)` = 32px):

- **Label**: `.text-caption` — "Transaction Amount"
- **Amount**: `<h1 class="text-display" style="margin-top: var(--space-xs);">` (margin-top 4px)
  - Main numeral: `-6.50` at 42px, font-weight 600
  - Currency suffix: ` USDC` at `font-size: 24px; color: var(--text-secondary); vertical-align: top`
- **Timestamp**: `<p class="text-body" style="font-size: 14px;">` — "February 24, 2025 • 10:42 AM"

### Agent Identity Row

`display: flex; align-items: center; gap: 12px; margin-bottom: var(--space-md)` (16px)

**Agent icon container:**
```css
width: 48px;
height: 48px;
background: var(--accent-green);   /* #8FB5AA */
border-radius: 14px;
display: flex;
align-items: center;
justify-content: center;
```
Contains a magnifying glass SVG (24x24, `stroke: black`, `stroke-width: 2`) — representing a research/search agent.

**Agent info:**
- Name: `.text-subtitle` — "Research Runner" (`margin: 0`)
- Badge: `.tag` — "Verified Agent"

#### Tag Component (`.tag`)

```css
.tag {
  background: var(--accent-green-dim);   /* rgba(143, 181, 170, 0.15) */
  color: #4A6E65;
  padding: 4px 10px;
  border-radius: 6px;
  font-size: 11px;
  font-weight: 700;
}
```

### Section Headings

Section labels use `.text-caption` with `style="margin-bottom: var(--space-sm);"` (8px).
Three sections: "Agent Metadata", "Cost Breakdown", "Notes".

### Meta Card (`.meta-card`)

```css
.meta-card {
  background: var(--bg-secondary);   /* #F8F9FA */
  border-radius: var(--radius-md);   /* 20px */
  padding: var(--space-md);          /* 16px */
  margin-bottom: var(--space-md);    /* 16px */
}
```

#### Meta Row (`.meta-row`)

```css
.meta-row {
  display: flex;
  justify-content: space-between;
  padding: 10px 0;
  border-bottom: 1px solid rgba(0,0,0,0.03);
}
.meta-row:last-child { border-bottom: none; }
```

Label and value styles:

```css
.meta-label {
  color: var(--text-secondary);   /* #8E8E93 */
  font-size: 14px;
}

.meta-value {
  color: var(--text-primary);     /* #111111 */
  font-weight: 500;
  font-size: 14px;
  text-align: right;
}
```

**Agent Metadata card content:**

| Label | Value | Notes |
|---|---|---|
| Category | Market Analysis | plain `.meta-value` |
| Purpose | Token sentiment scraping | plain `.meta-value` |
| Request ID | REQ_0921_AFB2 | `.meta-value.text-mono` (monospace) |

**Cost Breakdown card content:**

| Label | Value |
|---|---|
| API Compute (Claude 3.5) | $4.20 |
| Vector DB Ingress | $1.80 |
| Base Network Fee | $0.50 |

Total implied: $6.50 (matches amount hero).

**Notes card:**

The notes card uses an overridden padding: `style="padding: 16px;"` (same as default but explicitly set).

Content: `<p class="text-body" style="color: var(--text-primary); font-size: 14px;">` — overrides secondary color to primary for legibility.

Sample note text: "Agent successfully retrieved 15 data points regarding 'Base L2 Scaling' and indexed them for the daily brief."

### CTA — "View on Explorer" Button

Container: `<div style="margin-top: var(--space-xl);">` (32px)

Button uses `.btn.btn-outline`:

```css
.btn {
  height: 56px;
  border-radius: var(--radius-pill);   /* 999px */
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 600;
  font-size: 16px;
  cursor: pointer;
  width: 100%;
  border: none;
  text-decoration: none;
  transition: transform 0.1s;
}

.btn-outline {
  background: transparent;
  border: 1px solid var(--text-tertiary);   /* #C7C7CC */
  color: var(--text-primary);               /* #111111 */
}
```

The button renders as `<a href="#">` (link styled as button). Contains an external-link SVG (18x18, `stroke: currentColor`, `stroke-width: 2`, `margin-right: 8px`) followed by "View on Explorer".

#### Small Button Variant (`.btn-sm`)

Also defined but not used in this screen's visible HTML:

```css
.btn-sm {
  height: 36px;
  padding: 0 16px;
  font-size: 13px;
  width: auto;
}
```

---

## Transparency / Trust Design Patterns

1. **Amount is the hero** — 42px display font, centered, above the fold
2. **Currency unit is explicit** — "USDC" shown separately, smaller, in secondary color
3. **Verified Agent badge** — `.tag` component signals agent authenticity
4. **Monospace Request ID** — technical identifier uses `SF Mono` / `Menlo` to signal verifiability
5. **Cost breakdown is itemized** — not just a total; each API/service charge is listed separately
6. **Notes section** — agent-written summary of what it did with the money
7. **Explorer link** — direct path to on-chain verification
