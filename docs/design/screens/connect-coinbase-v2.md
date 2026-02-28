# Screen: Connect Coinbase (v2)

## Overview

Entry screen for connecting a user's Coinbase account. The user provides their email address to receive a secure login link. The screen is structured as a single-column form with a trust badge pinned to the bottom of the viewport.

---

## Layout

| Property | Value |
|---|---|
| Padding | `40px` all sides |
| Direction | Vertical, single column |
| Trust badge position | Pinned to bottom via `margin-top: auto` |

---

## Components

### Back Button

| Property | Value |
|---|---|
| Size | `40x40px` |
| Alignment | Left-aligned |
| Icon | Chevron left SVG |
| Margin bottom | `32px` |

---

### Headline

| Property | Value |
|---|---|
| Text | "Connect your Coinbase account" |
| Font size | `34px` |
| Font weight | `600` |
| Letter spacing | `-1px` |
| Line height | `1.1` |
| Margin bottom | `12px` |

---

### Body Text

| Property | Value |
|---|---|
| Text | "Connect your wallet to authorize agent spending and track automated transactions in real-time." |
| Font size | `17px` |
| Color | `--text-secondary` |
| Line height | `1.5` |

---

### Input Group

| Property | Value |
|---|---|
| Background | `--bg-secondary` |
| Border radius | `20px` |
| Padding | `16px 20px` |
| Margin top | `40px` |
| Border (default) | `1px solid transparent` |
| Border (focus) | `border-color: --text-tertiary` |

#### Input Label

| Property | Value |
|---|---|
| Text | "EMAIL ADDRESS" |
| Font size | `12px` |
| Font weight | `600` |
| Color | `--text-secondary` |
| Text transform | Uppercase |
| Letter spacing | `0.5px` |
| Margin bottom | `4px` |

#### Input Field

| Property | Value |
|---|---|
| Font size | `18px` |
| Placeholder | "name@email.com" |
| Autofocus | `true` |

---

### CTA Button

| Property | Value |
|---|---|
| Text | "Send code" |
| Width | Full-width |
| Height | `56px` |
| Border radius | `999px` |
| Background | Black (`#000` / `--black`) |
| Text color | White |
| Font weight | `600` |
| Margin top | `24px` |

---

### Subtext

| Property | Value |
|---|---|
| Text | "A secure login link will be sent to your inbox." |
| Font size | `13px` |
| Text align | Centered |
| Opacity | `0.8` |
| Margin top | `16px` |

---

### Trust Badge

Pinned to the bottom of the screen.

| Property | Value |
|---|---|
| Position | `margin-top: auto` |
| Padding bottom | `20px` |
| Layout | Horizontal, centered, items aligned center |

#### Coinbase Logo Mark

| Property | Value |
|---|---|
| Size | `20x20px` |
| Background | `#0052FF` |
| Border radius | `4px` |
| Icon inside | White circle SVG |

#### Badge Label

| Property | Value |
|---|---|
| Text | "Secured by Coinbase Cloud" |
| Font size | `13px` |
| Font weight | `500` |
| Color | `--text-secondary` |

---

## States

### Input Group: Default

- Border: `1px solid transparent`
- Background: `--bg-secondary`

### Input Group: Focused

- Border color: `--text-tertiary`
- Transition applies on the border color

---

## Notes

- The input group uses a floating label pattern: the label sits above the actual `<input>` element within the same rounded container rather than as a separate external label.
- Autofocus is set on the email field so the keyboard opens immediately on mobile.
- The trust badge must remain pinned to the bottom of the screen at all viewport heights — use a flex column layout on the screen container with `justify-content: space-between` or `margin-top: auto` on the badge wrapper.
- The CTA pill radius (`999px`) renders as a fully rounded pill at `56px` height regardless of button width.
