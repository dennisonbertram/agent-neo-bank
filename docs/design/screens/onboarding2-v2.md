# Onboarding Step 2: Install Skill Screen Spec (v2)

Source lines: 876–1120 of `docs/design/redesign-v2.md`
HTML title: `Agent Wallet - Install Skill`

---

## Overview

This screen handles the "Install Skill" onboarding step. It has two discrete states rendered inside the same `.app-container` — only one is visible at a time. Toggling between states is done via JavaScript (`showSuccess()`).

- **State 1: `#install-state`** — The skill confirmation screen. Shows what files will be changed, with a collapsible details panel and two action buttons.
- **State 2: `#success-state`** — The post-install success confirmation. Shows a green checkmark, a success message, and a Continue button.

---

## Design Tokens (CSS Variables)

```css
:root {
  --bg-primary:         #FFFFFF;
  --bg-secondary:       #F8F9FA;
  --surface-hover:      #F2F2F7;
  --text-primary:       #111111;
  --text-secondary:     #8E8E93;
  --text-tertiary:      #C7C7CC;
  --accent-green:       #8FB5AA;
  --accent-green-dim:   rgba(143, 181, 170, 0.15);
  --black:              #000000;
  --white:              #FFFFFF;
  --radius-sm:          12px;
  --radius-md:          20px;
  --radius-lg:          32px;
  --radius-pill:        999px;
  --space-md:           16px;
  --space-lg:           24px;
  --space-xl:           32px;
  --font-family:        -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
}
```

---

## App Container

```css
.app-container {
  width: 390px;
  height: 844px;
  background-color: #FFFFFF;
  position: relative;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  box-shadow: 0 0 0 10px #000, 0 20px 50px rgba(0,0,0,0.2);
  border-radius: 40px;
}
```

This simulates an iPhone frame (390×844). The double box-shadow creates a black device bezel effect.

---

## Screen Container

```css
.screen {
  flex: 1;
  padding: 60px 24px 40px 24px;  /* top: 60px, sides: 24px (--space-lg), bottom: 40px */
  display: flex;
  flex-direction: column;
}
```

Entry animation on both states:
```css
.animate-in {
  animation: slideUp 0.5s ease forwards;
}

@keyframes slideUp {
  from { opacity: 0; transform: translateY(20px); }
  to   { opacity: 1; transform: translateY(0); }
}
```

---

## Typography

| Class         | Size  | Weight | Color                  | Notes                              |
|---------------|-------|--------|------------------------|------------------------------------|
| `.text-title` | 28px  | 600    | `--text-primary`       | `letter-spacing: -0.5px`, `line-height: 1.2` |
| `.text-body`  | 16px  | 400    | `--text-secondary`     | `line-height: 1.5`                |
| `.text-mono`  | 13px  | —      | inherited              | `"SF Mono", "Menlo", monospace`   |

---

## State 1: Install State (`#install-state`)

### Layout (top to bottom)

1. **Header Nav** — Icon badge
2. **Title** — "Install Research Skill"
3. **Body text** — Skill description
4. **Skill Card** — Expandable "What changes?" panel
5. **Action buttons** — pushed to the bottom via `margin-top: auto`

---

### Header Nav Icon

```html
<div class="header-nav" style="margin-bottom: 24px;">
  <div style="
    width: 48px;
    height: 48px;
    background: rgba(143, 181, 170, 0.15);  /* --accent-green-dim */
    border-radius: 14px;
    display: flex;
    align-items: center;
    justify-content: center;
  ">
    <!-- SVG: 3D box / package icon -->
    <!-- stroke: #4A6E65, stroke-width: 2, size: 24x24 -->
  </div>
</div>
```

- Size: 48×48px
- Background: `rgba(143, 181, 170, 0.15)` (green-tinted dim)
- Border-radius: 14px
- Icon: package/cube SVG, stroke `#4A6E65`, 24×24

---

### Title and Description

```html
<h1 class="text-title">Install Research Skill</h1>
<p class="text-body" style="margin-top: 12px;">
  Enable your agent to browse the web and summarize technical documentation locally.
</p>
```

- Title: 28px, weight 600, `letter-spacing: -0.5px`
- Body: 16px, weight 400, `color: #8E8E93`, `margin-top: 12px`

---

### Skill Card (Expandable)

```css
.skill-card {
  background: #F8F9FA;     /* --bg-secondary */
  border-radius: 20px;     /* --radius-md */
  padding: 20px;
  margin-top: 32px;
}
```

**Header row** (toggle trigger):
- `display: flex; justify-content: space-between; align-items: center; cursor: pointer;`
- Left: "What changes?" — `font-weight: 600; font-size: 15px;`
- Right: Chevron SVG (20×20), rotates -90deg when collapsed via JS

**Expand Panel** (`.expand-panel`):
```css
.expand-panel {
  border-top: 1px solid rgba(0,0,0,0.05);
  margin-top: 16px;
  padding-top: 16px;
}
```

**File Change Items** (`.file-change`):
```css
.file-change {
  background: #FFFFFF;
  border-radius: 12px;
  padding: 12px;
  margin-bottom: 8px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  border: 1px solid rgba(0,0,0,0.03);
}
```

Two file change items:
1. `claude.md` — tag: "Config update"
2. `agents.md` — tag: "Permissions"

File names use `.text-mono` (monospace, 13px).

**Status Tag** (`.status-tag`):
```css
.status-tag {
  font-size: 10px;
  font-weight: 700;
  padding: 4px 8px;
  border-radius: 4px;
  background: rgba(143, 181, 170, 0.15);  /* --accent-green-dim */
  color: #4A6E65;
  text-transform: uppercase;
}
```

**Footer note inside expand panel:**
- `font-size: 12px; color: #8E8E93; margin-top: 12px; line-height: 1.4;`
- Text: "This skill will update your local agent configuration to allow read-access to the research directory."

---

### Action Buttons

Both pushed to bottom with `margin-top: auto` on their wrapper div.

**Primary Button** (`.btn.btn-primary`):
```css
.btn {
  height: 56px;
  border-radius: 999px;    /* --radius-pill */
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 600;
  font-size: 16px;
  cursor: pointer;
  width: 100%;
  border: none;
  transition: transform 0.1s;
}
.btn-primary {
  background-color: #000000;
  color: #FFFFFF;
}
.btn:active { transform: scale(0.98); }
```
- Label: "Confirm Installation"

**Cancel Button** (`.btn.btn-outline`):
```css
.btn-outline {
  background: transparent;
  border: 1px solid #C7C7CC;   /* --text-tertiary */
  color: #111111;
}
```
- Override inline style: `border: none` (so effectively borderless in this context)
- `margin-top: 12px`
- Label: "Cancel"

---

## State 2: Success State (`#success-state`)

Default: `display: none`. Shown on `showSuccess()` call (triggered by Confirm Installation button), toggled to `display: flex`.

```css
#success-state {
  display: none;
  text-align: center;
  justify-content: center;
  height: 100%;
}
```

Layout: vertically centered content block, Continue button pushed to bottom.

### Success Check Icon

```css
.success-check {
  width: 64px;
  height: 64px;
  background: #8FB5AA;    /* --accent-green */
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  margin: 0 auto 24px auto;
}
```

- Contains white checkmark SVG: `stroke: white; stroke-width: 3; size: 32x32`
- Checkmark path: `polyline points="20 6 9 17 4 12"`

### Title and Description

- Title: "Skill installed" — class `.text-title` (28px, weight 600)
- Body: "Research Runner is now active and ready to process requests." — class `.text-body` (16px, `color: #8E8E93`), `margin-top: 12px`

### Continue Button

- Same `.btn.btn-primary` style as above
- Label: "Continue"
- Wrapper: `margin-top: auto; width: 100%;`

---

## Interaction Logic (JavaScript)

```js
// Chevron expand/collapse
let expanded = true;
function toggleExpand() {
  const content = document.getElementById('expand-content');
  const chevron = document.getElementById('chevron');
  expanded = !expanded;
  content.style.display = expanded ? 'block' : 'none';
  chevron.style.transform = expanded ? 'rotate(0deg)' : 'rotate(-90deg)';
}

// State transition: install → success
function showSuccess() {
  document.getElementById('install-state').style.display = 'none';
  document.getElementById('success-state').style.display = 'flex';
}
```

---

## Navigation

No bottom nav on this screen. This is a modal/flow screen with only forward/cancel actions.

---

## Component Summary

| Component       | Class             | Notes                                             |
|-----------------|-------------------|---------------------------------------------------|
| Screen wrapper  | `.screen`         | flex column, 60px top padding, 40px bottom        |
| Icon badge      | inline styles     | 48×48, radius 14px, green-dim bg                 |
| Skill card      | `.skill-card`     | bg-secondary, radius-md (20px), 20px padding     |
| Expand panel    | `.expand-panel`   | border-top divider, 16px gap                     |
| File change row | `.file-change`    | white bg, radius 12px, space-between layout      |
| Status tag      | `.status-tag`     | green-dim bg, #4A6E65 text, uppercase, 10px/700  |
| Success circle  | `.success-check`  | 64px circle, accent-green (#8FB5AA) bg           |
| Primary button  | `.btn.btn-primary`| black bg, white text, 56px h, pill radius        |
| Cancel button   | `.btn.btn-outline`| transparent, border tertiary color               |
