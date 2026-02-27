# Agent Neo Bank -- Design Brief

> **Version:** 1.0
> **Date:** 2026-02-27
> **Status:** Draft
> **Related:** [Architecture Plan](../architecture/architecture-plan.md) | [Onboarding Flow](../architecture/user-onboarding-flow.md)

---

## Table of Contents

1. [Product Context](#1-product-context)
2. [Design Philosophy](#2-design-philosophy)
3. [Design System Foundation](#3-design-system-foundation)
4. [Typography](#4-typography)
5. [Color Palette](#5-color-palette)
6. [Spacing & Grid System](#6-spacing--grid-system)
7. [Component Specifications](#7-component-specifications)
8. [Micro-Interactions & Motion](#8-micro-interactions--motion)
9. [Layout Structure](#9-layout-structure)
10. [Screen Specifications](#10-screen-specifications)
11. [Empty & Loading States](#11-empty--loading-states)
12. [Dark Mode](#12-dark-mode)
13. [Accessibility](#13-accessibility)
14. [Responsive Behavior](#14-responsive-behavior)
15. [Icon System](#15-icon-system)
16. [Implementation Notes](#16-implementation-notes)

---

## 1. Product Context

### What Is Agent Neo Bank?

Agent Neo Bank is a Tauri v2 desktop application that lets users give their AI agents autonomous spending power through Coinbase Agent Wallets. Users set up a wallet, define budgets, and let AI agents pay for services -- with human-controlled guardrails.

### Target User

Non-technical to semi-technical users who work with AI agents (Claude Code, custom scripts). They want to delegate spending to agents without losing visibility or control. They expect a consumer-grade financial app, not a developer tool.

### Tech Stack (Frontend)

- **Framework:** React + TypeScript
- **Build:** Vite
- **Styling:** Tailwind CSS v4
- **Components:** shadcn/ui
- **State:** Zustand
- **Shell:** Tauri v2 (desktop app, not browser)

### Core Screens

| Screen | Purpose | Phase |
|---|---|---|
| Onboarding (3 steps) | Welcome, email input, OTP verification | 1a |
| Dashboard | Balance card, agent summary, recent transactions | 1 |
| Agent List | Grid of all agents with status and spending | 1 |
| Agent Detail | Per-agent spending limits, allowlist, activity | 1 |
| Transactions | Full transaction history table with filters | 1 |
| Approvals | Pending agent spend requests requiring user action | 1 |
| Settings | Global policy, notifications, invitation codes, network | 2 |
| Fund | Buy crypto / deposit address (Coinbase Onramp) | 3 |
| Spending Breakdown | Charts and analytics for spending patterns | 3 |

---

## 2. Design Philosophy

### Guiding Principles

1. **Clarity over cleverness.** Every screen should be instantly understandable. If a user has to think about what something means, the design has failed.
2. **Consumer app, not dev tool.** This is a banking app that happens to serve AI agents. It should feel like Mercury, Revolut, or Apple Wallet -- not a Swagger UI or admin panel.
3. **Bright with restraint.** The palette is clean and luminous, not garish. Color is used purposefully: to indicate status, draw attention, or create hierarchy. Most of the UI is neutral.
4. **Trust through structure.** Financial apps must feel secure. Consistent spacing, aligned elements, predictable layouts, and careful typography all communicate reliability.
5. **Progressive disclosure.** Show the essential information first. Details are available on demand (expandable sections, detail pages, tooltips) but never forced.

### Aesthetic References

| Reference | What to Borrow |
|---|---|
| **Apple.com** | Extreme whitespace, typography hierarchy, minimal but powerful layouts |
| **OpenAI** | Rounded corners, clean card-based layouts, subtle gradients, modern warmth |
| **Mercury** | Financial dashboard patterns, transaction tables, balance displays |
| **Revolut** | Card-based balance display, spending breakdowns, agent/account management |
| **Monzo** | Colorful but controlled accents, clean mobile-first patterns |
| **Linear** | Sidebar navigation, keyboard-first feel, snappy transitions |

---

## 3. Design System Foundation

### Border Radius Scale

| Token | Value | Usage |
|---|---|---|
| `radius-sm` | `6px` | Small badges, tags, toggle tracks |
| `radius-md` | `8px` | Buttons, inputs, small interactive elements |
| `radius-lg` | `12px` | Cards, dropdowns, modals, larger containers |
| `radius-xl` | `16px` | Hero cards (balance card), prominent containers |
| `radius-full` | `9999px` | Avatars, status dots, pill badges |

### Shadow Scale

Shadows use multi-layered definitions for realistic depth. All shadows use a slight warm tint.

| Token | Value | Usage |
|---|---|---|
| `shadow-xs` | `0 1px 2px rgba(0, 0, 0, 0.04)` | Subtle lift for flat elements on hover |
| `shadow-sm` | `0 1px 3px rgba(0, 0, 0, 0.06), 0 1px 2px rgba(0, 0, 0, 0.04)` | Default card shadow |
| `shadow-md` | `0 4px 6px rgba(0, 0, 0, 0.05), 0 2px 4px rgba(0, 0, 0, 0.04)` | Elevated cards, dropdowns |
| `shadow-lg` | `0 10px 15px rgba(0, 0, 0, 0.06), 0 4px 6px rgba(0, 0, 0, 0.04)` | Modals, floating panels |
| `shadow-xl` | `0 20px 25px rgba(0, 0, 0, 0.08), 0 8px 10px rgba(0, 0, 0, 0.04)` | Popovers, context menus |
| `shadow-card-hover` | `0 8px 20px rgba(0, 0, 0, 0.08), 0 2px 6px rgba(0, 0, 0, 0.04)` | Card hover state |

### Opacity Scale

| Token | Value | Usage |
|---|---|---|
| `opacity-disabled` | `0.5` | Disabled buttons, inputs |
| `opacity-placeholder` | `0.4` | Placeholder text |
| `opacity-overlay` | `0.6` | Modal backdrop overlays |
| `opacity-glass` | `0.7` | Glass-morphism panels |

### Z-Index Scale

| Token | Value | Usage |
|---|---|---|
| `z-base` | `0` | Normal flow |
| `z-dropdown` | `10` | Dropdowns, select menus |
| `z-sticky` | `20` | Sticky headers, sidebar |
| `z-modal-backdrop` | `30` | Modal overlay backdrop |
| `z-modal` | `40` | Modal content |
| `z-toast` | `50` | Toast notifications |
| `z-tooltip` | `60` | Tooltips |

---

## 4. Typography

### Font Stack

```css
--font-sans: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
--font-mono: 'JetBrains Mono', 'SF Mono', 'Fira Code', monospace;
```

Use Inter as the primary typeface. Load weights 400 (regular), 500 (medium), 600 (semibold), and 700 (bold). Monospace is used for wallet addresses, transaction hashes, and code-like values.

### Type Scale

| Token | Size | Weight | Line Height | Letter Spacing | Usage |
|---|---|---|---|---|---|
| `display-lg` | `48px` | 700 | 1.1 | `-0.02em` | Hero balance number on dashboard |
| `display-md` | `36px` | 700 | 1.15 | `-0.02em` | Page-level balance displays |
| `display-sm` | `30px` | 700 | 1.2 | `-0.01em` | Section-level large numbers |
| `heading-xl` | `24px` | 600 | 1.3 | `-0.01em` | Page titles ("Dashboard", "Agents") |
| `heading-lg` | `20px` | 600 | 1.35 | `-0.005em` | Card titles, section headers |
| `heading-md` | `16px` | 600 | 1.4 | `0` | Sub-section headers, card names |
| `heading-sm` | `14px` | 600 | 1.4 | `0.01em` | Small section labels, table headers |
| `body-lg` | `16px` | 400 | 1.6 | `0` | Primary body text |
| `body-md` | `14px` | 400 | 1.5 | `0` | Default body text, table cells |
| `body-sm` | `12px` | 400 | 1.5 | `0.01em` | Helper text, timestamps, captions |
| `label` | `13px` | 500 | 1.4 | `0.02em` | Form labels, badge text |
| `mono-md` | `14px` | 400 | 1.5 | `0` | Wallet addresses, tx hashes (mono font) |
| `mono-sm` | `12px` | 400 | 1.5 | `0` | Small code values, status codes (mono font) |

### Hierarchy Rules

- **One display-size number per screen.** The primary balance is the focal point -- nothing else should compete.
- **Two levels of heading per card maximum.** A card title and optional subtitle. Deeper hierarchy belongs on a detail page.
- **Body text defaults to `body-md` (14px).** Use `body-lg` only for standalone paragraphs or onboarding copy.
- **All monetary amounts use tabular (monospace) numerals** for alignment in tables and lists.

---

## 5. Color Palette

### Light Mode

#### Core Colors

| Token | Hex | Usage |
|---|---|---|
| `--background` | `#FAFAF9` | Page background (warm off-white) |
| `--surface` | `#FFFFFF` | Cards, panels, elevated surfaces |
| `--surface-raised` | `#FFFFFF` | Elevated cards with shadow |
| `--border` | `#E8E5E0` | Default borders (warm gray) |
| `--border-subtle` | `#F0EDE8` | Subtle dividers, inner card borders |
| `--ring` | `#6366F1` | Focus rings on interactive elements |

#### Text Colors

| Token | Hex | Usage |
|---|---|---|
| `--text-primary` | `#1A1A1A` | Primary text, headings, important values |
| `--text-secondary` | `#6B7280` | Secondary text, descriptions, timestamps |
| `--text-tertiary` | `#9CA3AF` | Placeholder text, disabled text |
| `--text-inverse` | `#FFFFFF` | Text on dark/colored backgrounds |

#### Primary Gradient (Brand / Balance Card)

| Token | Hex | Usage |
|---|---|---|
| `--primary-50` | `#EEF2FF` | Subtle primary backgrounds |
| `--primary-100` | `#E0E7FF` | Light primary fills |
| `--primary-200` | `#C7D2FE` | Hover states on primary backgrounds |
| `--primary-500` | `#6366F1` | Interactive elements, links |
| `--primary-600` | `#4F46E5` | Primary buttons, active states |
| `--primary-700` | `#4338CA` | Primary button hover |
| `--primary-900` | `#312E81` | Dark primary text |
| `--gradient-primary` | `linear-gradient(135deg, #4F46E5 0%, #7C3AED 50%, #6366F1 100%)` | Balance card, hero elements |
| `--gradient-primary-hover` | `linear-gradient(135deg, #4338CA 0%, #6D28D9 50%, #4F46E5 100%)` | Balance card hover state |

#### Semantic Colors

| Token | Hex | Usage |
|---|---|---|
| `--success-50` | `#ECFDF5` | Success background |
| `--success-100` | `#D1FAE5` | Success light fill |
| `--success-500` | `#10B981` | Success text, icons, positive amounts |
| `--success-600` | `#059669` | Success buttons, active indicators |
| `--success-700` | `#047857` | Success button hover |
| `--warning-50` | `#FFFBEB` | Warning background |
| `--warning-100` | `#FEF3C7` | Warning light fill |
| `--warning-500` | `#F59E0B` | Warning text, icons, pending states |
| `--warning-600` | `#D97706` | Warning emphasis |
| `--danger-50` | `#FEF2F2` | Danger background |
| `--danger-100` | `#FEE2E2` | Danger light fill |
| `--danger-500` | `#EF4444` | Danger text, icons, negative amounts |
| `--danger-600` | `#DC2626` | Danger buttons (deny, suspend) |
| `--danger-700` | `#B91C1C` | Danger button hover |

#### Neutral Gray Scale

| Token | Hex | Usage |
|---|---|---|
| `--gray-50` | `#F9FAFB` | Subtle backgrounds, table row hover |
| `--gray-100` | `#F3F4F6` | Input backgrounds, skeleton base |
| `--gray-200` | `#E5E7EB` | Borders, dividers |
| `--gray-300` | `#D1D5DB` | Disabled borders |
| `--gray-400` | `#9CA3AF` | Placeholder text |
| `--gray-500` | `#6B7280` | Secondary text |
| `--gray-600` | `#4B5563` | Body text emphasis |
| `--gray-700` | `#374151` | Strong text |
| `--gray-800` | `#1F2937` | Headings in dark contexts |
| `--gray-900` | `#111827` | Maximum contrast text |

### Dark Mode

| Token | Hex | Usage |
|---|---|---|
| `--background` | `#0F1117` | Page background (deep navy) |
| `--surface` | `#1A1D27` | Cards, panels |
| `--surface-raised` | `#232736` | Elevated cards |
| `--border` | `#2D3142` | Default borders |
| `--border-subtle` | `#232736` | Subtle dividers |
| `--text-primary` | `#F1F1F4` | Primary text |
| `--text-secondary` | `#9CA3AF` | Secondary text |
| `--text-tertiary` | `#6B7280` | Tertiary text |
| `--gradient-primary` | `linear-gradient(135deg, #6366F1 0%, #8B5CF6 50%, #7C3AED 100%)` | Balance card (slightly brighter in dark) |

All semantic colors (success, warning, danger) remain the same hex values in dark mode. The 50/100 tints are replaced with `{color}` at 10-15% opacity on the dark surface.

---

## 6. Spacing & Grid System

### Base Unit

All spacing derives from a **4px base grid**. Every margin, padding, gap, and dimension should be a multiple of 4.

### Spacing Scale

| Token | Value | Usage |
|---|---|---|
| `space-0` | `0px` | Reset |
| `space-0.5` | `2px` | Micro gaps (between inline status dot and text) |
| `space-1` | `4px` | Tightest spacing (icon padding, badge internal) |
| `space-1.5` | `6px` | Small internal padding |
| `space-2` | `8px` | Small gaps between related elements |
| `space-3` | `12px` | Default gap between form elements |
| `space-4` | `16px` | Standard padding for cards, inputs |
| `space-5` | `20px` | Medium section spacing |
| `space-6` | `24px` | Card internal padding (default) |
| `space-8` | `32px` | Section gaps within a page |
| `space-10` | `40px` | Large section dividers |
| `space-12` | `48px` | Page-level top/bottom padding |
| `space-16` | `64px` | Major layout spacing |

### Layout Grid

| Property | Value |
|---|---|
| Sidebar width (expanded) | `240px` |
| Sidebar width (collapsed) | `72px` |
| Main content max width | `1200px` |
| Main content padding | `32px` (top, left, right), `48px` (bottom) |
| Card gap (in grids) | `16px` |
| Page header margin-bottom | `24px` |
| Section gap | `32px` |

### Card Grid

Agent cards and dashboard sections use a responsive grid:

| Breakpoint | Columns | Gap |
|---|---|---|
| `< 768px` | 1 | `16px` |
| `768px - 1024px` | 2 | `16px` |
| `> 1024px` | 3 | `16px` |

---

## 7. Component Specifications

### 7.1 Balance Card (Credit Card Style)

The balance card is the hero element of the dashboard. It should look and feel like a premium physical credit card.

```
+-------------------------------------------------------+
|                                                         |
|   0x1a2B...9cDe  [copy icon]                           |
|                                                         |
|                                                         |
|   $20.00                                               |
|   USDC                                                  |
|                                                         |
|   0.10 ETH  ·  0.10 WETH                              |
|                                                         |
|           +-------------------+                         |
|           |   Fund Wallet     |                         |
|           +-------------------+                         |
|                                                         |
+-------------------------------------------------------+
```

| Property | Value |
|---|---|
| Width | `100%` (max `480px`) |
| Height | Auto (min `220px`) |
| Border radius | `16px` (`radius-xl`) |
| Background | `--gradient-primary` |
| Padding | `28px 32px` |
| Shadow | `shadow-lg` |
| Text color | `--text-inverse` (#FFFFFF) |
| Balance font | `display-lg` (48px, bold) |
| Asset label | `body-sm` (12px), `opacity: 0.8` |
| Secondary balances | `body-md` (14px), `opacity: 0.7` |
| Wallet address | `mono-sm` (12px), truncated (first 6 + last 4 chars), `opacity: 0.8` |
| Fund button | White background, primary text, `radius-md`, medium weight |
| Card shine effect | CSS pseudo-element, diagonal white gradient at 5% opacity, animated on hover |

**Hover behavior:** Subtle 3D parallax tilt (max 3 degrees rotation) following cursor position. Shadow deepens slightly. Shine effect shifts with tilt.

### 7.2 Buttons

All buttons use shadcn/ui `Button` as the base.

| Variant | Background | Text | Border | Hover | Usage |
|---|---|---|---|---|---|
| Primary | `--primary-600` | `--text-inverse` | none | `--primary-700` | Main CTAs ("Get Started", "Fund Wallet") |
| Secondary | `--surface` | `--text-primary` | `--border` | `--gray-50` background | Secondary actions ("Cancel", "View All") |
| Ghost | transparent | `--text-secondary` | none | `--gray-50` background | Tertiary actions, navigation ("Back") |
| Danger | `--danger-600` | `--text-inverse` | none | `--danger-700` | Destructive actions ("Deny", "Suspend") |
| Success | `--success-600` | `--text-inverse` | none | `--success-700` | Positive actions ("Approve") |
| Link | transparent | `--primary-500` | none | underline | Inline text links ("Resend code") |

**Shared button properties:**

| Property | Value |
|---|---|
| Height (default) | `40px` |
| Height (sm) | `32px` |
| Height (lg) | `48px` |
| Padding | `0 16px` (default), `0 12px` (sm), `0 24px` (lg) |
| Border radius | `8px` (`radius-md`) |
| Font | `label` (13px, medium, 0.02em tracking) |
| Transition | `all 150ms ease` |
| Active state | `scale(0.98)` transform |
| Disabled | `opacity: 0.5`, `cursor: not-allowed` |
| Focus | `2px` ring using `--ring` color, `2px` offset |

### 7.3 Inputs

| Property | Value |
|---|---|
| Height | `40px` (default), `48px` (lg for onboarding) |
| Border radius | `8px` (`radius-md`) |
| Border | `1px solid --border` |
| Background | `--surface` |
| Padding | `0 12px` |
| Font | `body-md` (14px) |
| Placeholder color | `--text-tertiary` |
| Focus border | `--primary-500` with `--ring` shadow |
| Error border | `--danger-500` |
| Error text | `body-sm`, `--danger-500`, below input with `4px` gap |
| Label | `label` style, `--text-secondary`, `4px` gap above input |

### 7.4 OTP Input

Six individual digit inputs in a row, used for email verification.

| Property | Value |
|---|---|
| Individual input size | `48px x 56px` |
| Border radius | `12px` (`radius-lg`) |
| Gap between inputs | `8px` |
| Font | `display-sm` (30px, bold), centered |
| Border | `2px solid --border` |
| Focused border | `2px solid --primary-500` |
| Filled state | `--primary-50` background |
| Auto-advance | On digit entry, focus moves to next input |
| Backspace | Clears current, moves focus to previous |
| Paste | Distributes pasted 6-digit string across all inputs |

### 7.5 Cards

The primary container element throughout the app.

| Property | Value |
|---|---|
| Background | `--surface` |
| Border | `1px solid --border-subtle` |
| Border radius | `12px` (`radius-lg`) |
| Padding | `24px` (default) |
| Shadow | `shadow-sm` |
| Hover shadow (if interactive) | `shadow-card-hover` |
| Header padding | `24px 24px 0 24px` |
| Body padding | `0 24px 24px 24px` |
| Header-body gap | `16px` |

### 7.6 Agent Card

Used in the Agent List page and Dashboard agent summary.

```
+------------------------------------------+
|                                            |
|  [icon]  Agent Name           [status dot] |
|          Purpose description line          |
|                                            |
|  Today's spend                             |
|  ████████░░░░░░░░░░  $12 / $50             |
|                                            |
+------------------------------------------+
```

| Property | Value |
|---|---|
| Width | Fill grid column |
| Padding | `20px` |
| Icon | `32px` circle with agent's color/avatar |
| Name | `heading-md` (16px, semibold) |
| Purpose | `body-sm` (12px), `--text-secondary`, single line, truncated |
| Status dot | `8px` circle, positioned top-right of icon |
| Status colors | Green: `--success-500`, Yellow: `--warning-500`, Red: `--danger-500` |
| Progress bar height | `6px`, `radius-full` |
| Progress bar track | `--gray-100` |
| Progress bar fill | `--primary-500` (under 75%), `--warning-500` (75-90%), `--danger-500` (90%+) |
| Spend text | `body-sm`, right side shows `$spent / $limit` |
| Hover | Translate Y -2px, shadow `shadow-card-hover`, 200ms ease |
| Click | Navigates to Agent Detail page |

### 7.7 Transaction Row

Used in the recent transactions list and full transactions table.

```
[icon]  Agent Name  ·  Description               +$5.00 USDC    2m ago
```

| Property | Value |
|---|---|
| Row height | `56px` |
| Padding | `12px 16px` |
| Icon | `36px` circle with type-specific icon and background |
| Icon colors | Send: `--primary-50` bg + `--primary-500` icon, Receive: `--success-50` bg + `--success-500` icon, Earn: `--warning-50` bg + `--warning-500` icon |
| Agent name | `body-md`, `--text-primary`, medium weight |
| Description | `body-sm`, `--text-secondary` |
| Amount (positive) | `body-md`, `--success-500`, medium weight |
| Amount (negative) | `body-md`, `--danger-500`, medium weight |
| Timestamp | `body-sm`, `--text-tertiary`, right-aligned |
| Divider | `1px solid --border-subtle` between rows |
| Hover | `--gray-50` background |

### 7.8 Approval Card

Used on the Approvals page for pending spend requests.

```
+-------------------------------------------------------------+
|                                                               |
|  [agent icon]  Agent Name               2 minutes ago        |
|                                                               |
|  Requesting $15.00 USDC                                      |
|  To: 0x1a2B...9cDe                                           |
|  Reason: "API subscription payment for Anthropic"            |
|                                                               |
|  [v Expand details]                                          |
|                                                               |
|       +-------------+    +-------------+                     |
|       |   Approve    |    |    Deny     |                    |
|       +-------------+    +-------------+                     |
|                                                               |
+-------------------------------------------------------------+
```

| Property | Value |
|---|---|
| Padding | `24px` |
| Agent icon | `40px` circle |
| Amount | `heading-lg` (20px, semibold) |
| Recipient address | `mono-md`, `--text-secondary` |
| Reason | `body-md`, `--text-primary`, italic or quoted |
| Timestamp | `body-sm`, `--text-tertiary`, top-right |
| Approve button | `success` variant, medium size |
| Deny button | `danger` variant (outline), medium size |
| Button gap | `12px` |
| Expandable details | `body-sm`, shows full tx metadata, agent context |
| Border-left accent | `3px solid --warning-500` (pending indicator) |

### 7.9 Status Badge

| Status | Background | Text | Dot |
|---|---|---|---|
| Active | `--success-50` | `--success-700` | `--success-500` |
| Pending | `--warning-50` | `--warning-700` | `--warning-500` (pulsing) |
| Suspended | `--danger-50` | `--danger-700` | `--danger-500` |
| Expired | `--gray-100` | `--gray-500` | `--gray-400` |

**Badge properties:** `radius-full`, `padding: 2px 10px 2px 8px`, `body-sm` font, `6px` dot with `4px` gap before text.

### 7.10 Toast Notifications

| Property | Value |
|---|---|
| Position | Top-right, `16px` from edges |
| Width | `360px` max |
| Border radius | `12px` |
| Shadow | `shadow-lg` |
| Padding | `16px` |
| Enter animation | Slide in from right + fade in, 300ms ease-out |
| Exit animation | Slide out to right + fade out, 200ms ease-in |
| Auto-dismiss | 5 seconds (info), 8 seconds (warning/error) |
| Close button | `X` icon, top-right corner |
| Types | Info (neutral), Success (green left border), Warning (amber left border), Error (red left border) |

### 7.11 Data Table

Used for the full Transactions page.

| Property | Value |
|---|---|
| Header row | `--gray-50` background, `heading-sm` text, sticky |
| Header border | `2px solid --border` bottom |
| Row height | `52px` |
| Row padding | `12px 16px` |
| Row divider | `1px solid --border-subtle` |
| Row hover | `--gray-50` background |
| Sorted column header | `--primary-500` text + sort icon |
| Pagination | Bottom of table, `16px` padding, page size selector + page navigation |
| Stripe (optional) | Alternate rows with `--gray-50`, disabled by default |
| Column alignment | Text left, numbers right, status center |

### 7.12 Progress Bar

| Property | Value |
|---|---|
| Track height | `6px` (compact) or `8px` (standard) |
| Track background | `--gray-100` |
| Track radius | `radius-full` |
| Fill radius | `radius-full` |
| Fill colors | Normal: `--primary-500`, Warning: `--warning-500`, Danger: `--danger-500` |
| Label position | Right-aligned inline or above the bar |
| Animation | Width transition 500ms ease-out on value change |

### 7.13 Sidebar Navigation

| Property | Value |
|---|---|
| Width (expanded) | `240px` |
| Width (collapsed) | `72px` |
| Background | `--surface` |
| Border right | `1px solid --border-subtle` |
| Logo area | `64px` height, centered, `16px` bottom margin |
| Nav item height | `40px` |
| Nav item padding | `8px 12px` |
| Nav item radius | `8px` |
| Nav item gap | `4px` |
| Icon size | `20px` |
| Icon-text gap | `12px` |
| Active item | `--primary-50` background, `--primary-600` text + icon |
| Hover item | `--gray-50` background |
| Inactive text | `--text-secondary` |
| Section divider | `1px solid --border-subtle` with `12px` vertical margin |
| Bottom area | User email (truncated), wallet indicator, settings link |
| Collapse trigger | Toggle icon at bottom of sidebar or keyboard shortcut |
| Transition | Width 200ms ease |

---

## 8. Micro-Interactions & Motion

### Timing Tokens

| Token | Duration | Easing | Usage |
|---|---|---|---|
| `duration-fast` | `100ms` | `ease` | Button press (scale), toggle switch |
| `duration-normal` | `200ms` | `ease` | Hover states, color changes, focus rings |
| `duration-slow` | `300ms` | `ease-out` | Card lift, page content fade-in |
| `duration-slower` | `500ms` | `ease-out` | Progress bar fill, number count-up |
| `duration-page` | `250ms` | `ease-in-out` | Page transitions |

### Interaction Catalog

| Element | Trigger | Animation |
|---|---|---|
| **Balance card** | Hover | Subtle 3D parallax tilt (max 3deg). Shadow deepens. Diagonal shine shifts. |
| **Balance number** | Data load | Count-up from 0 to actual value over 500ms with easing. |
| **Agent card** | Hover | Translate Y -2px, shadow increases to `shadow-card-hover`. |
| **Agent card** | Click | Scale to 0.98 (100ms), then navigate. |
| **Any button** | Hover | Background color shift (200ms ease). |
| **Any button** | Press | Scale 0.98 (100ms). |
| **Primary button** | Click (success) | Brief color flash to lighter tint (100ms), then return. |
| **Approve button** | Click | Flash `--success-100` background (150ms), then card fades out. |
| **Deny button** | Click | Flash `--danger-100` background (150ms), then card fades out. |
| **Status badge (pending)** | Idle | Dot pulses with opacity (1.0 to 0.4, 2s loop, ease-in-out). |
| **Skeleton loader** | Loading | Shimmer animation: gradient sweep left-to-right, 1.5s loop. |
| **Page transition** | Navigate | Content fades out (100ms) + slight slide left, new content fades in (200ms) + slight slide from right. |
| **Toast** | Appear | Slide in from right (300ms ease-out) + fade in. |
| **Toast** | Dismiss | Slide out right (200ms ease-in) + fade out. |
| **Modal** | Open | Backdrop fades in (200ms). Content scales from 0.95 to 1.0 + fades in (250ms ease-out). |
| **Modal** | Close | Content scales to 0.95 + fades out (150ms). Backdrop fades out (200ms). |
| **Progress bar** | Value change | Width animates (500ms ease-out). Color transitions if threshold crossed. |
| **Sidebar** | Collapse | Width animates (200ms ease), text fades out, icons remain centered. |
| **Copy button** | Click | Icon swaps to checkmark, tooltip shows "Copied!", reverts after 2s. |
| **Number values** | Update | Cross-fade old/new value (200ms). |

### Motion Principles

1. **Physics-based feel.** Ease-out for elements entering (decelerating into place). Ease-in for elements leaving (accelerating away).
2. **No motion sickness.** Maximum transform distance is 8px translate / 3 degrees rotation. No bouncing or overshooting.
3. **Respect prefers-reduced-motion.** Disable all transforms and reduce durations to 0ms when the OS setting is enabled.
4. **Purposeful only.** Every animation must communicate state change. Purely decorative animation is not used.

---

## 9. Layout Structure

### App Shell

```
+-----------------------------------------------------------+
| [Sidebar]  |  [Main Content Area]                          |
|            |                                                |
| Logo       |  Page Header (title + actions)                |
|            |  -------------------------------------------- |
| Dashboard  |                                                |
| Agents     |  Page Content                                  |
| Txns       |  (scrollable)                                  |
| Approvals  |                                                |
| Fund       |                                                |
| Settings   |                                                |
|            |                                                |
| --------   |                                                |
| user@email |                                                |
| 0x1a2B...  |                                                |
+-----------------------------------------------------------+
```

| Property | Value |
|---|---|
| Shell layout | CSS Grid: `[sidebar] [main]` |
| Sidebar | Fixed height `100vh`, `overflow-y: auto` |
| Main content | `overflow-y: auto`, scroll container |
| Page header | Sticky at top of main scroll area, `--surface` background with `shadow-xs` on scroll |
| Header height | `64px` |
| Header padding | `0 32px` |
| Content padding | `32px 32px 48px 32px` |
| Max content width | `1200px`, centered |

### Navigation Items

| Icon | Label | Route |
|---|---|---|
| `LayoutDashboard` | Dashboard | `/` |
| `Bot` | Agents | `/agents` |
| `ArrowLeftRight` | Transactions | `/transactions` |
| `ShieldCheck` | Approvals | `/approvals` |
| `Wallet` | Fund | `/fund` |
| `Settings` | Settings | `/settings` |

Icons sourced from Lucide (the icon set bundled with shadcn/ui).

---

## 10. Screen Specifications

### 10.1 Onboarding -- Welcome Screen

**State:** ONBOARDING (first app launch)

**Layout:** Full-screen centered, no sidebar. Content within a centered card (max-width `440px`).

**Elements (top to bottom):**
1. App logo / icon (`48px`)
2. Spacer (`24px`)
3. Headline: "Give your AI agents spending power" -- `heading-xl`, `--text-primary`, centered
4. Spacer (`12px`)
5. Subtext: "Set up a wallet, define budgets, and let your AI agents pay for services autonomously -- with guardrails you control." -- `body-lg`, `--text-secondary`, centered
6. Spacer (`8px`)
7. Setup time indicator: "Set up in 2 minutes" -- `body-sm`, `--text-tertiary`, centered
8. Spacer (`32px`)
9. CTA button: "Get Started" -- primary variant, large size, full width within card
10. Spacer (`24px`)
11. Footer: "Powered by Coinbase Agent Wallet" -- `body-sm`, `--text-tertiary`, centered, with subtle Coinbase icon

**Background:** Subtle gradient mesh or soft radial gradient using `--primary-50` and `--background`.

### 10.2 Onboarding -- Email Input

**State:** WALLET_LINKING

**Layout:** Full-screen centered card (max-width `440px`).

**Elements:**
1. Back button (ghost, top-left of card) -- navigates to Welcome
2. Header: "Connect your wallet" -- `heading-xl`, centered
3. Subtext: "Enter your email to set up your Agent Wallet. We'll send you a verification code." -- `body-md`, `--text-secondary`, centered
4. Spacer (`24px`)
5. Email input -- large size (`48px` height), with label "Email address"
6. Spacer (`12px`)
7. Display name input -- standard size, with label "Display name (optional)"
8. Spacer (`24px`)
9. Submit button: "Send Verification Code" -- primary, large, full width
10. Spacer (`16px`)
11. Legal text: "By continuing, you agree to the Coinbase Agent Wallet terms." -- `body-sm`, `--text-tertiary`
12. Spacer (`16px`)
13. Info collapsible: "What happens next?" with numbered steps -- uses shadcn `Collapsible` or `Accordion`

**Loading state:** Button shows spinner + "Sending..." while CLI processes `awal auth login`.

### 10.3 Onboarding -- OTP Verification

**State:** OTP_VERIFICATION

**Layout:** Full-screen centered card (max-width `440px`).

**Elements:**
1. Back button (ghost, top-left)
2. Header: "Check your email" -- `heading-xl`, centered
3. Subtext showing the submitted email address -- `body-md`, `--text-secondary`
4. Spacer (`32px`)
5. OTP input (6 digits) -- centered, per spec in Section 7.4
6. Spacer (`16px`)
7. Resend row: "Didn't receive it?" + "Resend code" link (with 60s cooldown timer)
8. Spacer (`24px`)
9. Verify button: "Verify" -- primary, large, full width (also auto-submits on 6th digit)
10. Spacer (`12px`)
11. Timer text: "Code expires in 4:32" -- `body-sm`, `--text-tertiary`, countdown from 5:00

**Error state:** Inputs shake horizontally (3 cycles, 4px amplitude, 300ms total), border turns `--danger-500`, error message appears below: "Invalid code. Please try again."

**Success transition:** Inputs flash `--success-100` background, brief checkmark icon overlay, then smooth transition to dashboard.

### 10.4 Dashboard

**State:** ACTIVE

**Layout:** App shell with sidebar. Main content has the following sections stacked vertically.

**Section 1 -- Page Header:**
- "Welcome, {displayName}!" or "Dashboard" -- `heading-xl`
- Optional subtitle with date

**Section 2 -- Balance Card:**
- Full spec in Section 7.1
- Positioned prominently, full width up to `480px`

**Section 3 -- Quick Actions:**
- Horizontal row of action chips
- Items: "Send", "Fund", "Invite Agent", "Settings"
- Each chip: icon (20px) + label, `radius-full`, `--surface` background, `--border` border, `12px 16px` padding
- Hover: `--gray-50` background
- Gap: `8px`

**Section 4 -- Agent Summary:**
- Section header: "Your Agents" + "View all" link (right-aligned)
- Horizontal scrollable row or 3-column grid of Agent Cards (spec 7.6)
- If 0 agents: Empty state card with illustration, "Your agents will appear here. Generate an invitation code to get started.", CTA button "Generate Invitation Code"
- "Add Agent" card: dashed border, `+` icon centered, "Add Agent" label, `--text-tertiary`, hover fills `--gray-50`

**Section 5 -- Recent Transactions:**
- Section header: "Recent Transactions" + "View all" link
- List of 5 most recent transactions (Transaction Row spec 7.7)
- If 0 transactions: Empty state with "No transactions yet. Transactions will appear when your agents start spending."

### 10.5 Agent List Page

**Layout:** App shell. Page header + content.

**Page Header:**
- Title: "Agents" -- `heading-xl`
- Right side: "Generate Invitation Code" button (primary) + search input (240px wide)

**Filter Tabs (below header):**
- Tabs: "All", "Active", "Pending", "Suspended"
- Uses shadcn `Tabs` component
- Active tab: `--primary-500` underline, `--text-primary` text
- Inactive tab: `--text-secondary` text, hover `--text-primary`
- Count badge on each tab (e.g., "Active (3)")

**Content:**
- Grid of Agent Cards (spec 7.6)
- 3 columns on desktop, 2 on medium, 1 on narrow
- If filtered view is empty: contextual empty state ("No suspended agents")

### 10.6 Agent Detail Page

**Layout:** App shell. Breadcrumb navigation at top: "Agents > Agent Name".

**Header Section:**
- Agent icon (48px) + Name (`heading-xl`) + Status badge
- Subtitle: Purpose description + "Created {date}" -- `body-md`, `--text-secondary`
- Actions: "Suspend Agent" (danger outline button) + "Rotate Token" (secondary button)

**Card 1 -- Spending Limits:**
- Card header: "Spending Limits" + "Edit" toggle
- Four limit rows, each with:
  - Label (per-transaction / daily / weekly / monthly)
  - Progress bar (spent / limit)
  - Numeric display: "$X.XX / $Y.YY"
  - Edit mode: input field replaces display
- Auto-approve threshold: "Auto-approve transactions under $X.XX" with input
- Save button (appears in edit mode)

**Card 2 -- Allowed Recipients:**
- Card header: "Allowed Recipients"
- List of addresses/service names with `X` remove button each
- Add input + "Add" button at bottom
- If empty: "All recipients allowed" message with info icon

**Card 3 -- Activity Feed:**
- Card header: "Activity" + filter dropdown (All / Sends / Receives)
- Timeline-style list: vertical line with dots at each event
- Each entry: type icon, amount, recipient (truncated address), status badge, timestamp
- Pagination: "Load more" button at bottom

### 10.7 Transactions Page

**Layout:** App shell. Full-width data table.

**Page Header:**
- Title: "Transactions"
- Right side: "Export CSV" button (secondary)

**Filter Bar:**
- Row of filter controls, `16px` gap
- Date range picker (shadcn DateRangePicker or two date inputs)
- Agent dropdown (multi-select)
- Type toggles: "All", "Send", "Receive", "Earn" -- pill toggle group
- Status toggles: "All", "Completed", "Pending", "Failed"
- Search input (right-aligned, `240px`)

**Table:**
- Columns: Date, Agent, Type, Amount, Recipient/Service, Status, Description
- Full spec in Section 7.11
- Sortable columns: Date (default desc), Amount, Agent
- Pagination: bottom-right, "Showing 1-25 of 142", page size selector (25/50/100), prev/next

### 10.8 Approvals Page

**Layout:** App shell. List of approval cards.

**Page Header:**
- Title: "Approvals"
- Subtitle: "X pending" count -- `body-md`, `--text-secondary`

**Content:**
- Vertical stack of Approval Cards (spec 7.8), `16px` gap
- Cards ordered by time (newest first)
- Empty state: Illustration (checkmark shield), "All caught up!", "No pending approvals. Your agents are operating within their limits.", with subtle confetti or celebration icon

### 10.9 Settings Page

**Layout:** App shell. Vertical stack of settings cards.

**Page Header:**
- Title: "Settings"

**Card 1 -- Global Spending Policy:**
- Card header: "Global Spending Limits"
- Three rows: Daily cap, Weekly cap, Monthly cap
- Each row: label, input field, current utilization progress bar
- Minimum reserve balance input: "Always keep at least $X.XX in wallet"
- Kill switch: large toggle with red color, label "Emergency: Disable all agent spending"
- Kill switch confirm: modal with "Are you sure? This will immediately block all agent transactions." + "Disable" / "Cancel" buttons

**Card 2 -- Notification Preferences:**
- Card header: "Notifications"
- Toggle rows, each with: label, description, switch
- Events: "Transaction completed", "Approval needed", "Budget threshold (80%)", "Agent suspended", "Incoming deposit"

**Card 3 -- Invitation Codes:**
- Card header: "Invitation Codes" + "Generate Code" button (primary, small)
- Table: Code (mono), Status (badge), Created date, Linked agent, Revoke button
- Status values: Active (green badge), Used (gray badge), Expired (gray badge), Revoked (red badge)

**Card 4 -- Network:**
- Card header: "Network"
- Toggle: "Testnet (Sepolia)" / "Mainnet (Base)"
- Warning: switching to mainnet shows safety checklist modal
- Current network indicator with colored dot

**Card 5 -- Wallet Info:**
- Card header: "Wallet"
- Address with copy button (full address displayed)
- Connected email
- Session status: "Active" / "Expired" with re-auth link

### 10.10 Fund Page (Phase 3 -- Wireframe)

**Layout:** App shell. Tab layout.

**Tabs:**
- "Buy Crypto" | "Deposit"

**Buy Tab:**
- Coinbase Onramp widget embed area (placeholder box with Coinbase branding)
- Note: "Purchase crypto directly with card or bank transfer"

**Deposit Tab:**
- "Your Wallet Address" header
- Full address display with copy button
- QR code (large, centered, `200px`)
- Note: "Send USDC, ETH, or WETH on Base (mainnet) or Sepolia (testnet)"
- Network indicator badge

### 10.11 Spending Breakdown (Phase 3 -- Wireframe)

**Layout:** App shell. Charts and tables.

**Header:**
- Title: "Spending Breakdown"
- Time range selector: pill group "7d", "30d", "90d"

**Row 1 (charts, 2 columns):**
- Bar chart: spending by agent (horizontal bars, agent name labels)
- Pie/donut chart: spending by category

**Row 2 (full width):**
- Line chart: daily spending trend over selected period

**Row 3 (full width):**
- "Top Services" table: Service name, total spent, transaction count, last used

**Chart styling:**
- Use primary gradient colors for chart fills
- Axis labels: `body-sm`, `--text-secondary`
- Grid lines: `--border-subtle`, 1px dashed
- Tooltips: `shadow-md`, `radius-md`, `body-sm`

---

## 11. Empty & Loading States

### Skeleton Loading

Every data-dependent section uses skeleton loaders instead of spinners. Skeletons mirror the exact layout of the content they replace.

| Property | Value |
|---|---|
| Base color | `--gray-100` |
| Shimmer color | `--gray-200` |
| Animation | Linear gradient sweep, left to right, 1.5s loop |
| Border radius | Matches the element being replaced |
| Reduce motion | Static `--gray-100` fill (no animation) |

**Skeleton shapes per screen:**

- **Dashboard balance card:** Full card-shaped skeleton with gradient background placeholder
- **Agent cards:** Rectangle skeletons for icon (circle), name (60% width line), purpose (80% width line), progress bar
- **Transaction rows:** Circle skeleton (icon) + two line skeletons (name, description) + right-aligned line (amount)
- **Table:** Header row + 5 placeholder rows with column-width-matched rectangles

### Empty States

Each empty state follows a consistent pattern:

1. **Illustration or icon** (64px, `--text-tertiary` color, or a subtle SVG illustration)
2. **Title** -- `heading-md`, `--text-primary`
3. **Description** -- `body-md`, `--text-secondary`, max-width `360px`, centered
4. **CTA button** (optional) -- primary or secondary, contextual action

| Screen | Title | Description | CTA |
|---|---|---|---|
| Agent Summary (dashboard) | "No agents yet" | "Generate an invitation code to let an AI agent connect to your wallet." | "Generate Invitation Code" |
| Recent Transactions | "No transactions yet" | "Transactions will appear here when your agents start spending." | none |
| Agent List | "No agents" | "Invite your first AI agent to get started." | "Generate Invitation Code" |
| Transactions (filtered) | "No matching transactions" | "Try adjusting your filters." | "Clear filters" |
| Approvals | "All caught up!" | "No pending approvals. Your agents are operating within their limits." | none |
| Allowlist | "All recipients allowed" | "Add specific addresses to restrict where this agent can send funds." | "Add address" |

---

## 12. Dark Mode

### Implementation

Dark mode is toggled via system preference (`prefers-color-scheme`) with an optional manual override in Settings. The toggle adds a `dark` class to the root HTML element. All color tokens swap values as defined in Section 5.

### Key Differences from Light Mode

| Element | Light | Dark |
|---|---|---|
| Page background | `#FAFAF9` | `#0F1117` |
| Card background | `#FFFFFF` | `#1A1D27` |
| Card borders | `#E8E5E0` | `#2D3142` |
| Shadows | Visible, warm-tinted | Reduced opacity, cooler tint |
| Balance card gradient | Indigo to violet | Slightly brighter indigo to violet |
| Primary button | `#4F46E5` | `#6366F1` (slightly brighter for contrast) |
| Semantic backgrounds (50/100) | Solid light tints | 10-15% opacity of the main color |
| Skeleton shimmer | `#F3F4F6` to `#E5E7EB` | `#232736` to `#2D3142` |
| Charts | Same data colors | Same data colors, darker grid lines |

### Transition

Dark/light mode transitions use a `150ms ease` on `background-color` and `color` properties via a global CSS transition. Card borders and shadows also transition smoothly.

---

## 13. Accessibility

### Requirements

- **WCAG 2.1 AA** compliance minimum for all interactive elements.
- All text meets minimum contrast ratios: 4.5:1 for normal text, 3:1 for large text (18px+).
- Focus indicators are always visible: `2px` ring using `--ring` color with `2px` offset.
- All interactive elements are keyboard-accessible with visible focus states.
- Screen reader labels for all icon-only buttons (via `aria-label`).
- Status information is not conveyed by color alone (always paired with text or icon).
- `prefers-reduced-motion: reduce` disables all transform animations and reduces transition durations to 0.
- OTP inputs support clipboard paste for the full 6-digit code.
- All form inputs have associated labels (visible or via `aria-label`).
- Error messages are announced via `aria-live="polite"` regions.
- Toast notifications use `role="status"` with `aria-live="polite"`.

### Focus Order

Tab order follows visual layout: sidebar nav (top to bottom), then main content (top to bottom, left to right within grids).

---

## 14. Responsive Behavior

While Agent Neo Bank is a Tauri desktop app, the window can be resized. The design should gracefully handle different window sizes.

| Breakpoint | Width | Layout Adjustments |
|---|---|---|
| Compact | `< 768px` | Sidebar collapses to icon-only (`72px`). Grids become single column. Balance card goes full width. Table switches to card-based list view. |
| Standard | `768px - 1200px` | Sidebar expanded. Grids use 2 columns. Content centered with padding. |
| Wide | `> 1200px` | Full layout. 3-column grids. Content max-width `1200px` centered. |

### Minimum Window Size

`800px x 600px` (set via Tauri window config). Below this, content scrolls rather than collapses further.

---

## 15. Icon System

Use **Lucide React** icons exclusively (bundled with shadcn/ui). All icons render at `20px` by default with `1.5px` stroke width.

### Icon Map

| Context | Icon Name | Notes |
|---|---|---|
| Dashboard nav | `LayoutDashboard` | |
| Agents nav | `Bot` | |
| Transactions nav | `ArrowLeftRight` | |
| Approvals nav | `ShieldCheck` | |
| Fund nav | `Wallet` | |
| Settings nav | `Settings` | |
| Send transaction | `ArrowUpRight` | |
| Receive transaction | `ArrowDownLeft` | |
| Earn transaction | `Sparkles` | |
| Copy address | `Copy` -> `Check` on click | |
| Add / Create | `Plus` | |
| Remove / Delete | `X` | |
| Edit | `Pencil` | |
| Search | `Search` | |
| Filter | `Filter` | |
| Export | `Download` | |
| Back navigation | `ChevronLeft` | |
| Expand/collapse | `ChevronDown` / `ChevronUp` | |
| External link | `ExternalLink` | |
| Info | `Info` | |
| Warning | `AlertTriangle` | |
| Error | `AlertCircle` | |
| Success | `CheckCircle` | |
| Notification bell | `Bell` | |
| Kill switch | `Power` | In red when active |
| QR code | `QrCode` | |

---

## 16. Implementation Notes

### Tailwind v4 + shadcn/ui

All design tokens should be defined as CSS custom properties in the Tailwind v4 theme configuration. shadcn/ui components provide the structural foundation; this design brief defines the visual layer on top.

```css
/* Example token setup in globals.css */
@layer base {
  :root {
    --background: 40 20% 98%;
    --foreground: 0 0% 10%;
    --card: 0 0% 100%;
    --primary: 239 84% 67%;
    --primary-foreground: 0 0% 100%;
    --success: 160 84% 39%;
    --warning: 38 92% 50%;
    --danger: 0 84% 60%;
    --radius: 0.75rem;
    /* ... full token list ... */
  }

  .dark {
    --background: 228 25% 7%;
    --foreground: 240 7% 95%;
    --card: 228 17% 13%;
    /* ... */
  }
}
```

### Component Priority

Build components in this order to maximize reuse:

1. **Foundation:** Color tokens, typography classes, spacing utilities
2. **Primitives:** Button, Input, Badge, Card (shadcn base + custom styling)
3. **Composites:** BalanceCard, AgentCard, TransactionRow, ApprovalCard, OTPInput
4. **Layout:** Sidebar, Shell, PageHeader
5. **Pages:** Onboarding flow, Dashboard, then remaining pages

### Animation Implementation

Use CSS transitions for hover/focus states. Use Framer Motion (or CSS `@keyframes`) for:
- Page transitions (AnimatePresence)
- Balance count-up animation
- Skeleton shimmer
- Toast enter/exit
- Modal open/close
- Parallax tilt on balance card

### Asset Checklist

| Asset | Format | Notes |
|---|---|---|
| App logo | SVG | Used in sidebar + onboarding |
| Empty state illustrations | SVG | One per empty state context (5-6 total) |
| Onboarding hero graphic | SVG or gradient CSS | Abstract gradient mesh or geometric illustration |
| Favicon / app icon | PNG (multiple sizes) | Tauri requires multiple icon sizes |

---

## Appendix: Quick Reference Card

### At a Glance

| Property | Value |
|---|---|
| Primary font | Inter |
| Mono font | JetBrains Mono |
| Base spacing unit | 4px |
| Card radius | 12px |
| Button radius | 8px |
| Primary color | #4F46E5 (indigo-600) |
| Success color | #10B981 (emerald-500) |
| Warning color | #F59E0B (amber-500) |
| Danger color | #EF4444 (red-500) |
| Background (light) | #FAFAF9 |
| Background (dark) | #0F1117 |
| Card shadow | 0 1px 3px rgba(0,0,0,0.06), 0 1px 2px rgba(0,0,0,0.04) |
| Default transition | 200ms ease |
| Sidebar width | 240px (expanded) / 72px (collapsed) |
| Content max-width | 1200px |
| Balance card gradient | linear-gradient(135deg, #4F46E5, #7C3AED, #6366F1) |
