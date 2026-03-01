# Tauri v2 Custom In-App Notification Overlays

**Date**: 2026-02-28
**Status**: Research Complete
**Use Case**: Agent spending notifications -- small overlay cards that appear, animate in, auto-dismiss, and link to transaction details.

---

## Table of Contents

1. [Architecture Options](#1-architecture-options)
2. [Creating Non-Focusable Overlay Windows](#2-creating-non-focusable-overlay-windows)
3. [Animation Approaches](#3-animation-approaches)
4. [Event Flow](#4-event-flow)
5. [Queuing and Stacking](#5-queuing-and-stacking)
6. [Reference Implementations](#6-reference-implementations)
7. [Recommendation](#7-recommendation)

---

## 1. Architecture Options

### Option A: Separate Tauri Window Per Toast

Create a new `WebviewWindow` for each notification, destroy it on dismiss.

**Pros:**
- True OS-level isolation -- each toast is independent
- Can position each toast at different screen coordinates (vertical stacking is natural)
- If one toast crashes, others are unaffected
- Simple lifecycle: create window, show, auto-close after timeout

**Cons:**
- Window creation has measurable overhead (~50-150ms per window on macOS)
- Each window is a separate webview process with its own memory footprint
- Bursty events (5 agent charges in 1 second) will spawn 5 webviews -- expensive
- Managing z-order and position of many independent windows is complex
- On some Linux window managers, new windows may briefly steal focus despite `focusable: false`

**Verdict:** Not recommended for bursty notification scenarios. Too much overhead per notification.

### Option B: Single Persistent Toast Window (Show/Hide + Update Content)

Create one `WebviewWindow` at app startup with `visible: false`. When a notification fires, update its content via events and show it. Manage a queue of notifications internally.

**Pros:**
- Zero window creation cost at notification time -- the window already exists
- Single webview manages its own notification queue, animations, stacking
- Memory-efficient: one webview regardless of notification volume
- Full control over layout (stack multiple cards vertically within one window)
- The window can be tall (e.g., 400px) but only render visible cards, keeping transparent areas click-through

**Cons:**
- Requires `set_ignore_cursor_events` toggling for transparent areas (or accept that the full window rect blocks clicks)
- Window resizing needed as notifications stack/dismiss
- Slightly more complex state management in the toast webview
- Single point of failure (if the toast webview crashes, all notifications are lost)

**Verdict:** Best balance of performance and control. Recommended approach.

### Option C: Overlay Within the Main App Window

Render toast notifications as React components inside the main app window (like a standard `react-hot-toast` or `sonner` library).

**Pros:**
- Simplest implementation -- just a React component with CSS animations
- No multi-window complexity
- Shares the main app's state, routing, and stores directly
- Many battle-tested React toast libraries available (sonner, react-hot-toast, react-toastify)
- Click-to-navigate is trivial (just a React Router `navigate()` call)

**Cons:**
- Only visible when the main app window is visible and in the foreground
- Cannot show notifications when the app is minimized or behind other windows
- Competes for z-index with other UI elements in the main window
- Not a true "system-level" overlay

**Verdict:** Good enough if notifications only matter when the app is in focus. Does not satisfy "show overlay near top of screen regardless of app state."

---

## 2. Creating Non-Focusable Overlay Windows in Tauri v2

### Window Configuration (Rust -- WebviewWindowBuilder)

The following `WebviewWindowBuilder` methods are verified to exist in the Tauri v2 API ([docs.rs/tauri](https://docs.rs/tauri/latest/tauri/webview/struct.WebviewWindowBuilder.html)):

```rust
use tauri::WebviewWindowBuilder;
use tauri::WebviewUrl;

#[tauri::command]
async fn create_toast_window(app: tauri::AppHandle) -> Result<(), String> {
    let toast_window = WebviewWindowBuilder::new(
        &app,
        "toast",                                    // unique label
        WebviewUrl::App("toast.html".into()),       // dedicated toast page
    )
    .title("Notifications")
    .inner_size(380.0, 500.0)                       // width x max height
    .position(0.0, 0.0)                             // will reposition after build
    .always_on_top(true)                            // float above all windows
    .focusable(false)                               // do NOT steal keyboard focus
    .decorations(false)                             // no title bar, no borders
    .transparent(true)                              // transparent background
    .skip_taskbar(true)                             // don't show in dock/taskbar
    .visible(false)                                 // start hidden, show on first notification
    .shadow(false)                                  // no OS window shadow
    .build()
    .map_err(|e| e.to_string())?;

    Ok(())
}
```

### Key Properties Explained

| Property | Value | Purpose |
|----------|-------|---------|
| `always_on_top(true)` | `bool` | Ensures toast floats above all other windows including the main app |
| `focusable(false)` | `bool` | Window cannot receive keyboard focus. **Critical** -- without this, showing the toast steals focus from whatever the user is typing. Note: on macOS, if a window is *already* focused, calling `set_focusable(false)` cannot unfocus it ([Tauri docs](https://v2.tauri.app/reference/javascript/api/namespacewindow/)) |
| `decorations(false)` | `bool` | Removes OS title bar and window chrome |
| `transparent(true)` | `bool` | Allows the webview to render with alpha transparency. The HTML/CSS must also have `background: transparent` on both `html` and `body` |
| `skip_taskbar(true)` | `bool` | Hides the window from the OS taskbar/dock so it appears as a pure overlay |
| `shadow(false)` | `bool` | Removes the OS-level drop shadow that would appear around the transparent window rect |
| `visible(false)` | `bool` | Start hidden; show programmatically when the first notification arrives |

### Transparency: The Three-Layer Problem

Tauri windows have three layers that all need transparent backgrounds ([Window Customization docs](https://v2.tauri.app/learn/window-customization/)):

1. **Window layer** -- controlled by `transparent(true)` in the builder
2. **Webview layer** -- controlled by the Tauri runtime (transparent when window is transparent)
3. **HTML/CSS layer** -- you **must** set `background: transparent` on `html` and `body`

```css
/* toast.css */
html, body {
  background: transparent !important;
  margin: 0;
  padding: 0;
  overflow: hidden;
}
```

**macOS caveat:** There is a known issue where transparent windows may lose transparency after building a DMG ([Issue #13415](https://github.com/tauri-apps/tauri/issues/13415)). Test with release builds early.

### Positioning at Top of Screen

**Option 1: Positioner Plugin**

The official `@tauri-apps/plugin-positioner` provides predefined positions ([Positioner docs](https://v2.tauri.app/plugin/positioner/)):

```rust
use tauri_plugin_positioner::{WindowExt, Position};

// Move to top-right of screen
let win = app.get_webview_window("toast").unwrap();
win.as_ref().window().move_window(Position::TopRight)?;
```

Available positions: `TopLeft` (0), `TopRight` (1), `BottomLeft` (2), `BottomRight` (3), `TopCenter` (4), `BottomCenter` (5), `LeftCenter` (6), `RightCenter` (7), `Center` (8).

**Option 2: Manual Positioning**

Use `set_position` with a `LogicalPosition` for precise control:

```rust
use tauri::LogicalPosition;

let win = app.get_webview_window("toast").unwrap();
// Get the monitor to calculate position
if let Some(monitor) = win.current_monitor()? {
    let screen_size = monitor.size();
    let scale = monitor.scale_factor();
    let logical_width = screen_size.width as f64 / scale;

    // Position at top-right with 16px padding
    let x = logical_width - 380.0 - 16.0;
    let y = 48.0; // below macOS menu bar
    win.set_position(LogicalPosition::new(x, y))?;
}
```

**Note:** Use `LogicalPosition` not `PhysicalPosition`. There is a known bug where `PhysicalPosition` can misplace windows on multi-monitor setups ([Issue #11718](https://github.com/tauri-apps/tauri/issues/11718)).

### Making Non-Interactive Areas Click-Through

The `set_ignore_cursor_events` method on `Window` controls whether mouse events pass through to windows behind ([Tauri Window API](https://docs.rs/tauri/latest/tauri/window/struct.Window.html)):

```rust
// Rust side
let win = app.get_webview_window("toast").unwrap();
win.set_ignore_cursor_events(true)?;  // clicks pass through
```

```typescript
// JavaScript side
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
const win = getCurrentWebviewWindow();
await win.setIgnoreCursorEvents(true);
```

**The problem:** This is all-or-nothing per window. When `true`, the *entire* window ignores mouse events -- you cannot click the toast card itself.

**Workaround -- dynamic toggling with mousemove:**

```typescript
// In the toast webview's frontend code
const win = getCurrentWebviewWindow();

document.addEventListener('mousemove', (e) => {
  const target = e.target as HTMLElement;
  const isOverCard = target.closest('.toast-card') !== null;
  win.setIgnoreCursorEvents(!isOverCard);
});
```

This approach makes transparent areas click-through while keeping toast cards interactive. However, it has a known issue on Windows 10 where `setIgnoreCursorEvents` may not work correctly ([Issue #11461](https://github.com/tauri-apps/tauri/issues/11461)).

**Alternative -- `tauri-plugin-polygon`:** A community plugin that lets you define polygon regions as the mouse-responsive area ([crates.io](https://crates.io/crates/tauri-plugin-polygon)). Only works for fullscreen transparent windows.

---

## 3. Animation Approaches

### Approach 1: CSS Animations in the Webview (Recommended)

Since each toast is a React/HTML element inside the webview, standard CSS animations work perfectly:

```css
/* Slide in from right */
@keyframes slideIn {
  from {
    transform: translateX(100%);
    opacity: 0;
  }
  to {
    transform: translateX(0);
    opacity: 1;
  }
}

@keyframes slideOut {
  from {
    transform: translateX(0);
    opacity: 1;
  }
  to {
    transform: translateX(100%);
    opacity: 0;
  }
}

.toast-card {
  animation: slideIn 300ms cubic-bezier(0.16, 1, 0.3, 1) forwards;
}

.toast-card.dismissing {
  animation: slideOut 200ms cubic-bezier(0.16, 1, 0.3, 1) forwards;
}
```

**Pros:**
- Smooth 60fps animations handled by the GPU
- Full control over easing, timing, and effects
- Works identically on macOS, Windows, and Linux
- Can combine transforms (slide + fade + scale)
- Easy to implement with React transition libraries (framer-motion, react-spring)

**Cons:**
- Animation is within the webview bounds -- the window itself does not move
- Requires the window to be pre-sized large enough to contain the animation path

### Approach 2: Programmatic Window Position Animation

Animate the window's OS-level position over time using `set_position`:

```rust
use std::time::Duration;
use tokio::time::sleep;

async fn animate_toast_in(app: &tauri::AppHandle) {
    let win = app.get_webview_window("toast").unwrap();
    let target_y = 48.0;
    let start_y = -100.0; // start off-screen
    let steps = 20;

    for i in 0..=steps {
        let progress = i as f64 / steps as f64;
        // ease-out cubic
        let eased = 1.0 - (1.0 - progress).powi(3);
        let y = start_y + (target_y - start_y) * eased;
        win.set_position(LogicalPosition::new(1000.0, y)).unwrap();
        sleep(Duration::from_millis(16)).await; // ~60fps
    }
}
```

**Pros:**
- The window physically moves on screen -- true OS-level animation
- Works for windows without transparency

**Cons:**
- Janky on some platforms -- `set_position` is not designed for 60fps animation
- Blocks a Rust async task for the animation duration
- Multi-monitor position bugs can cause windows to jump to wrong monitors
- Not smooth compared to GPU-accelerated CSS animations
- Different behavior across macOS, Windows, Linux

### Approach 3: Hybrid (Recommended)

- Use a **fixed-position window** (created once, positioned at top-right)
- Animate **content within the webview** using CSS/JS
- Only use `set_position` for initial placement and when the window needs to resize

**This is the recommended approach.** CSS animations within the webview are smoother, more reliable, and cross-platform consistent.

---

## 4. Event Flow

### Architecture: Rust Backend to Toast Window

```
Agent Charge Detected (Rust Service / MCP handler)
  │
  ├── app.emit_to("toast", "agent-charge", payload)
  │         │
  │         └── Toast webview listens: listen("agent-charge", handler)
  │                   │
  │                   └── Adds to notification queue, triggers CSS animation
  │
  └── (optional) app.emit("agent-charge", payload)  // global, for main app too
```

### Step-by-Step Flow

**1. Rust backend detects an agent charge:**

```rust
use tauri::{AppHandle, Emitter};
use serde::Serialize;

#[derive(Clone, Serialize)]
struct AgentChargePayload {
    agent_name: String,
    amount: String,      // e.g., "0.50 USDC"
    description: String, // e.g., "API call to OpenAI"
    tx_hash: String,
    timestamp: u64,
}

fn notify_agent_charge(app: &AppHandle, charge: AgentChargePayload) {
    // Ensure toast window is visible
    if let Some(win) = app.get_webview_window("toast") {
        let _ = win.show();
    }

    // Emit to toast window specifically
    app.emit_to("toast", "agent-charge", &charge).unwrap();

    // Also emit globally so the main app can update its transaction list
    app.emit("agent-charge-global", &charge).unwrap();
}
```

**2. Toast webview listens for the event:**

```typescript
// toast/App.tsx
import { listen } from '@tauri-apps/api/event';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';

interface AgentChargePayload {
  agent_name: string;
  amount: string;
  description: string;
  tx_hash: string;
  timestamp: number;
}

// Use getCurrentWebviewWindow().listen() for webview-specific events
const appWindow = getCurrentWebviewWindow();

appWindow.listen<AgentChargePayload>('agent-charge', (event) => {
  addNotification(event.payload);
});
```

**Important:** Use `getCurrentWebviewWindow().listen()` (not the global `listen()`) when receiving events sent via `emit_to`. Global `listen()` will NOT receive webview-specific events. To catch all events regardless of targeting, use `listen_any()` ([Tauri Events docs](https://v2.tauri.app/develop/calling-frontend/)).

**3. Clicking a toast navigates the main app:**

```typescript
// In toast webview -- emit event to main window
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';

function handleToastClick(txHash: string) {
  // Option A: Emit to the main window
  const mainWindow = WebviewWindow.getByLabel('main');
  mainWindow?.emit('navigate-to-transaction', { txHash });

  // Option B: Use app-level emit (Rust side)
  // invoke('navigate_main_window', { txHash });
}
```

```typescript
// In main app window -- listen for navigation requests
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';

const mainWindow = getCurrentWebviewWindow();
mainWindow.listen('navigate-to-transaction', (event) => {
  const { txHash } = event.payload as { txHash: string };
  navigate(`/transactions/${txHash}`);
  // Bring main window to front
  mainWindow.setFocus();
});
```

### Multi-Window Event Targeting Summary

| Method | Scope | Use Case |
|--------|-------|----------|
| `app.emit("event", payload)` | All global listeners | Broadcast to all windows |
| `app.emit_to("label", "event", payload)` | Specific webview by label | Target toast window specifically |
| `app.emit_filter("event", payload, \|target\| ...)` | Filtered set of webviews | Target multiple specific windows |
| `webviewWindow.emit("event", payload)` | JS: emit from one window | Window-to-window communication |

---

## 5. Queuing and Stacking

### Recommended Strategy: Vertical Stack with Queue

Show up to N visible toasts stacked vertically. Additional notifications queue behind. Each toast auto-dismisses after a timeout.

```typescript
// toast/notificationStore.ts
import { create } from 'zustand';

interface Notification {
  id: string;
  agent_name: string;
  amount: string;
  description: string;
  tx_hash: string;
  timestamp: number;
  state: 'entering' | 'visible' | 'exiting';
}

interface NotificationStore {
  notifications: Notification[];
  queue: Notification[];
  maxVisible: number;

  addNotification: (payload: Omit<Notification, 'id' | 'state'>) => void;
  dismissNotification: (id: string) => void;
  promoteFromQueue: () => void;
}

const AUTO_DISMISS_MS = 5000;
const MAX_VISIBLE = 3;

export const useNotificationStore = create<NotificationStore>((set, get) => ({
  notifications: [],
  queue: [],
  maxVisible: MAX_VISIBLE,

  addNotification: (payload) => {
    const id = `${Date.now()}-${Math.random().toString(36).slice(2)}`;
    const notification: Notification = { ...payload, id, state: 'entering' };

    const { notifications } = get();

    if (notifications.length >= MAX_VISIBLE) {
      // Queue it -- will show when a visible toast dismisses
      set((s) => ({ queue: [...s.queue, notification] }));
    } else {
      set((s) => ({ notifications: [...s.notifications, notification] }));

      // Mark as visible after enter animation
      setTimeout(() => {
        set((s) => ({
          notifications: s.notifications.map((n) =>
            n.id === id ? { ...n, state: 'visible' } : n
          ),
        }));
      }, 300);

      // Auto-dismiss
      setTimeout(() => {
        get().dismissNotification(id);
      }, AUTO_DISMISS_MS);
    }
  },

  dismissNotification: (id) => {
    // Start exit animation
    set((s) => ({
      notifications: s.notifications.map((n) =>
        n.id === id ? { ...n, state: 'exiting' } : n
      ),
    }));

    // Remove after exit animation, then promote from queue
    setTimeout(() => {
      set((s) => ({
        notifications: s.notifications.filter((n) => n.id !== id),
      }));
      get().promoteFromQueue();
    }, 200);
  },

  promoteFromQueue: () => {
    const { queue } = get();
    if (queue.length > 0) {
      const [next, ...rest] = queue;
      set({ queue: rest });
      get().addNotification(next);
    }
  },
}));
```

### Window Resizing Based on Active Notifications

The toast window should resize its height to match the number of visible notifications to minimize the click-through area:

```typescript
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { LogicalSize } from '@tauri-apps/api/dpi';

const CARD_HEIGHT = 80;     // px per notification card
const CARD_GAP = 8;         // px between cards
const WINDOW_PADDING = 16;  // px top/bottom padding
const WINDOW_WIDTH = 380;

function resizeToastWindow(notificationCount: number) {
  if (notificationCount === 0) {
    getCurrentWebviewWindow().hide();
    return;
  }

  const height = (CARD_HEIGHT * notificationCount)
    + (CARD_GAP * (notificationCount - 1))
    + (WINDOW_PADDING * 2);

  getCurrentWebviewWindow().setSize(new LogicalSize(WINDOW_WIDTH, height));
  getCurrentWebviewWindow().show();
}
```

### Alternative Strategies

| Strategy | Behavior | Best For |
|----------|----------|----------|
| **Vertical stack** (recommended) | Show N cards stacked, queue the rest | Most notification UIs (Slack, macOS) |
| **Replace** | New notification replaces the current one | Single high-priority alerts |
| **Counter badge** | Show latest + "+N more" badge | Very bursty events (100s/minute) |
| **Collapse by agent** | Group notifications per agent, show count | Multi-agent scenarios |

For agent spending notifications, **vertical stack with max 3 visible** is the right choice. Spending events are important enough to see individually but not so frequent they need collapsing.

---

## 6. Reference Implementations

### Official Tauri Resources

- **Tauri Discussion #7951** -- "Overlay notification?" Asks about non-focus-stealing overlays. Suggests `decorations: false, ignoreCursorEvents: true, focus: false` as the approach ([Discussion #7951](https://github.com/tauri-apps/tauri/discussions/7951)).
- **Tauri Discussion #4810** -- "How to make an Overlay" -- covers transparent overlay windows ([Discussion #4810](https://github.com/tauri-apps/tauri/discussions/4810)).
- **Tauri Discussion #6604** -- "Notification System for Tauri" -- discusses custom vs OS notifications ([Discussion #6604](https://github.com/tauri-apps/tauri/discussions/6604)).

### Community Plugins

- **`tauri-plugin-polygon`** -- Defines polygon regions for mouse-responsive areas in transparent windows. Useful for click-through on non-card areas ([crates.io](https://crates.io/crates/tauri-plugin-polygon)).
- **`tauri-plugin-positioner`** -- Official plugin for positioning windows at predefined screen locations like `TopRight` ([Positioner docs](https://v2.tauri.app/plugin/positioner/)).

### No Dedicated "Tauri Toast" Library Exists

As of February 2026, there is no dedicated Tauri plugin or library specifically for in-app toast/overlay notifications. The pattern must be built from primitives:
- `WebviewWindowBuilder` with the right options
- CSS animations in the toast webview
- Tauri event system for Rust-to-toast communication

### How Other Desktop Apps Handle This

| App | Approach |
|-----|----------|
| **Slack (Electron)** | In-app toast within main window (Option C) |
| **Discord (Electron)** | In-app overlay within main window |
| **VS Code** | In-app notification panel (bottom-right corner of main window) |
| **1Password** | Separate small window, always-on-top, no focus steal |
| **macOS native apps** | `NSWindow` with `level: .floating` and `canBecomeKey: false` |

The Tauri approach (Option B) most closely mirrors the 1Password / macOS native pattern.

---

## 7. Recommendation

### Recommended Architecture: Option B + C Hybrid

Use **Option B** (single persistent toast window) for when the app is in the background, combined with **Option C** (in-app React toast) for when the main window is in the foreground.

However, if simplicity is the priority, **start with Option B alone** -- it works in both foreground and background scenarios.

### Implementation Plan

#### Phase 1: Foundation
1. Create `toast.html` -- a minimal React app with its own entry point
2. Configure the toast window in Rust with all the properties from Section 2
3. Use the positioner plugin to place it at `TopRight`
4. Implement the notification Zustand store (Section 5)
5. Wire up `emit_to("toast", ...)` from the Rust backend

#### Phase 2: Polish
1. Add CSS slide-in/slide-out animations
2. Implement `setIgnoreCursorEvents` toggling for click-through
3. Add dynamic window resizing based on notification count
4. Implement click-to-navigate (emit to main window)
5. Add auto-dismiss with configurable timeout

#### Phase 3: Edge Cases
1. Handle app startup -- create toast window early but keep hidden
2. Handle main app quit -- clean up toast window
3. Test multi-monitor positioning
4. Test with rapid-fire notifications (10+ in 1 second)
5. Test transparency on macOS release builds (DMG)

### Required Tauri Plugins

```toml
# Cargo.toml
[dependencies]
tauri-plugin-positioner = "2"
```

```json
// package.json
{
  "dependencies": {
    "@tauri-apps/plugin-positioner": "^2.0.0"
  }
}
```

### Required Permissions

```json
// capabilities/default.json
{
  "permissions": [
    "positioner:default",
    "core:window:allow-show",
    "core:window:allow-hide",
    "core:window:allow-set-size",
    "core:window:allow-set-position",
    "core:window:allow-set-focus",
    "core:window:allow-set-ignore-cursor-events",
    "core:event:default"
  ]
}
```

---

## Sources

- [Tauri v2 WebviewWindowBuilder API](https://docs.rs/tauri/latest/tauri/webview/struct.WebviewWindowBuilder.html)
- [Tauri v2 Window struct (set_ignore_cursor_events)](https://docs.rs/tauri/latest/tauri/window/struct.Window.html)
- [Tauri v2 WindowConfig reference](https://docs.rs/tauri-utils/latest/tauri_utils/config/struct.WindowConfig.html)
- [Tauri v2 Window Customization guide](https://v2.tauri.app/learn/window-customization/)
- [Tauri v2 Calling the Frontend from Rust](https://v2.tauri.app/develop/calling-frontend/)
- [Tauri v2 Positioner Plugin](https://v2.tauri.app/plugin/positioner/)
- [Tauri v2 Event API reference](https://v2.tauri.app/reference/javascript/api/namespaceevent/)
- [Tauri v2 Configuration reference](https://v2.tauri.app/reference/config/)
- [Tauri Discussion #7951 -- Overlay notification](https://github.com/tauri-apps/tauri/discussions/7951)
- [Tauri Discussion #4810 -- How to make an Overlay](https://github.com/tauri-apps/tauri/discussions/4810)
- [Tauri Discussion #6604 -- Notification System](https://github.com/tauri-apps/tauri/discussions/6604)
- [Tauri Issue #13070 -- Transparent Window Click-Through](https://github.com/tauri-apps/tauri/issues/13070)
- [Tauri Issue #11461 -- setIgnoreCursorEvents bug](https://github.com/tauri-apps/tauri/issues/11461)
- [Tauri Issue #13415 -- macOS transparency lost after DMG build](https://github.com/tauri-apps/tauri/issues/13415)
- [Tauri Issue #11718 -- PhysicalPosition bug on Windows](https://github.com/tauri-apps/tauri/issues/11718)
- [tauri-plugin-polygon (click-through regions)](https://crates.io/crates/tauri-plugin-polygon)
- [Emitter trait (emit_to, emit_filter)](https://docs.rs/tauri/latest/tauri/trait.Emitter.html)
