# Screen: OTP Verify (v2)

## Overview

Second step of the Coinbase connection flow. The user enters a 6-digit one-time passcode sent to their email. The screen features an animated OTP input grid with distinct visual states for empty, filled, and active (cursor) boxes, plus a countdown resend footer.

---

## Layout

| Property | Value |
|---|---|
| Padding | `60px 24px` (top: `60px`, left/right: `24px`) |
| Direction | Vertical, single column |

---

## Components

### Back Button

| Property | Value |
|---|---|
| Size | `40x40px` |
| Shape | Circle |
| Border | `1px solid --surface-hover` |
| Background | Transparent |
| Icon | Left arrow SVG |
| Margin bottom | `32px` |

---

### Title

| Property | Value |
|---|---|
| Text | "Verify it's you" |
| Font size | `28px` |
| Font weight | `700` |
| Letter spacing | `-0.5px` |
| Margin bottom | `8px` |

---

### Subtitle

| Property | Value |
|---|---|
| Text | "Enter the 6-digit code sent to [email]" |
| Email segment | Bold, `color: --text-primary` |
| Remainder | Default body weight and color |

The email address within the subtitle is rendered as a `<strong>` or `<span>` with `font-weight: 700` and `color: --text-primary` to distinguish it from the surrounding instructional text.

---

### OTP Input Grid

| Property | Value |
|---|---|
| Display | `grid` |
| Columns | `repeat(6, 1fr)` |
| Gap | `8px` |
| Margin | `40px 0` (top and bottom) |

#### OTP Box — Default (empty)

| Property | Value |
|---|---|
| Aspect ratio | `1 / 1.2` |
| Background | `--bg-secondary` |
| Border radius | `12px` |
| Font size | `24px` |
| Font weight | `600` |
| Border | `2px solid transparent` |
| Text align | Centered |

#### OTP Box — Filled State (`.filled`)

| Property | Value |
|---|---|
| Border color | `--black` |
| Background | `white` |
| Box shadow | `0 2px 8px rgba(0, 0, 0, 0.04)` |

#### OTP Box — Active / Cursor State (`.active`)

| Property | Value |
|---|---|
| Border color | `--accent-green` (`#8FB5AA`) |
| Blinking cursor | Rendered as a pseudo-element or child element |

##### Blinking Cursor

| Property | Value |
|---|---|
| Width | `2px` |
| Height | `24px` |
| Background | `--black` |
| Animation | `blink 1s infinite` |
| Keyframes | `opacity: 1` → `opacity: 0` → `opacity: 1` |

```css
@keyframes blink {
  0%   { opacity: 1; }
  50%  { opacity: 0; }
  100% { opacity: 1; }
}
```

---

### CTA Button

Identical to the Connect Coinbase screen CTA.

| Property | Value |
|---|---|
| Text | "Verify code" |
| Width | Full-width |
| Height | `56px` |
| Border radius | `999px` |
| Background | Black (`--black`) |
| Text color | White |
| Font weight | `600` |

---

### Footer — Resend Row

Displayed below the CTA button, horizontally centered.

#### "Resend code" Label

| Property | Value |
|---|---|
| Text | "Resend code" |
| Font weight | `600` |
| Font size | `14px` |

#### Countdown Timer

| Property | Value |
|---|---|
| Text | "in 0:42" (counts down) |
| Color | `--text-secondary` |
| Font size | `14px` |
| Margin left | `4px` |

The "Resend code" label and countdown timer sit inline. When the timer reaches `0:00`, the timer text is hidden and the "Resend code" label becomes tappable.

---

## States

### OTP Box States Summary

| State | Class | Border | Background | Shadow | Cursor |
|---|---|---|---|---|---|
| Empty | (default) | `2px solid transparent` | `--bg-secondary` | None | None |
| Filled | `.filled` | `2px solid --black` | `white` | `0 2px 8px rgba(0,0,0,0.04)` | None |
| Active | `.active` | `2px solid #8FB5AA` | `--bg-secondary` | None | Blinking `2x24px` black bar |

---

## Notes

- The OTP grid is driven by a hidden single `<input>` field (or individual inputs per cell). The visual boxes are rendered separately and receive class updates (`.filled`, `.active`) based on the value and cursor position.
- The blinking cursor inside the `.active` box mimics a native text cursor. It is implemented as an absolutely positioned child element or `::after` pseudo-element centered vertically within the box.
- The accent green (`#8FB5AA`) used for the active border is the same `--accent-green` token used elsewhere in the design system.
- The back button on this screen uses a bordered circle style (differs from the plain icon on the Connect Coinbase screen — maintain consistency within each screen but note the intentional variation).
- Top padding is `60px` (vs `40px` on the Connect screen) to give more breathing room below the status bar on this secondary screen.
- The footer resend row should not be tappable while the countdown is active; apply `pointer-events: none` or `opacity: 0.5` to the "Resend code" label during countdown.
