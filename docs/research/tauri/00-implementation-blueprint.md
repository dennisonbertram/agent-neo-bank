# Tally Agentic Wallet: Multi-Window Implementation Blueprint

**Date**: 2026-02-28
**Tauri Version**: v2.10.2
**Status**: Ready for Implementation
**Source Research**: `01-core-apis.md`, `02-plugins-tray.md`, `03-cross-platform.md`, `04-notification-overlays.md`

---

## A) Tauri Version & Architecture Summary

### Confirmed Versions

| Package | Version |
|---------|---------|
| `tauri` (Rust crate) | **2.10.2** |
| `@tauri-apps/api` (JS) | **2.10.1** |
| `tauri-cli` | **2.10.0** |
| `wry` (WebView rendering) | **0.54.2** |
| `tao` (Windowing) | **0.34.5** |

Tauri v2.0 stable was released October 2, 2024. The v2.10.x line is the latest as of this research.

### Multi-Window Architecture Overview

The app uses **3 windows**, each with a dedicated purpose and separate capability set:

```
┌─────────────────────────────────────────────────────────┐
│  MAIN WINDOW ("main")                                    │
│  Full app: dashboard, agents, transactions, settings     │
│  Hidden by default, shown on demand                      │
│  Standard decorated window (or custom titlebar)          │
└─────────────────────────────────────────────────────────┘

┌──────────────────────────┐
│  WIDGET WINDOW ("widget") │
│  Floating credit card     │
│  Frameless, transparent   │
│  Always-on-top            │
│  Tray-anchored position   │
│  Compact ⇄ expanded mode │
└──────────────────────────┘

┌──────────────────────────┐
│  TOAST WINDOW ("toast")   │
│  Notification overlay     │
│  Non-focusable            │
│  Transparent, always-top  │
│  Positioned top-right     │
│  Hidden until needed      │
└──────────────────────────┘
```

### Best Practices Confirmed from Research

1. **Create windows at startup, toggle visibility** -- avoid per-event window creation overhead (~50-150ms per window on macOS).
2. **Use `emit_to(label, ...)` for targeted events** -- not global `emit()` -- to keep event routing explicit and avoid unintended cross-window side effects.
3. **CSS animations over programmatic window movement** -- GPU-accelerated CSS is smoother and cross-platform consistent; `set_position` was not designed for 60fps animation.
4. **Disable native shadows, use CSS box-shadow** -- `shadow: false` avoids platform inconsistencies (1px white border on Windows, no shadow on transparent macOS windows). CSS `box-shadow` + `border-radius` gives consistent cross-platform rounded corners.
5. **`LogicalPosition`/`LogicalSize` over Physical** -- avoids multi-monitor DPI bugs.
6. **Per-window capabilities (ACL)** -- scope permissions narrowly. The toast window should not have filesystem access. The widget window should not create new windows.
7. **Secrets stay in Rust** -- wallet keys, API tokens, agent tokens never touch the webview. The webview has DevTools in development.

---

## B) Exact APIs & Config -- Per Behavior

### 1. Floating Credit Card Widget Window

**Goal**: Frameless, transparent, draggable, always-on-top window that shows a compact credit card. Skips taskbar/dock. Toggles between compact card view and expanded mini-app.

#### tauri.conf.json Window Config

```jsonc
{
  "app": {
    "macOSPrivateApi": true,  // Required for transparency on macOS
    "windows": [
      {
        "label": "widget",
        "url": "/widget",
        "title": "Tally Widget",
        "width": 360,
        "height": 240,
        "visible": false,
        "decorations": false,
        "transparent": true,
        "shadow": false,
        "alwaysOnTop": true,
        "skipTaskbar": true,
        "resizable": false,
        "focusable": true,
        "focus": false,
        "center": false,
        "acceptFirstMouse": true
      }
    ]
  }
}
```

**Key config notes:**
- `visible: false` -- starts hidden; shown on tray click.
- `decorations: false` -- no native title bar or borders.
- `transparent: true` + `shadow: false` -- enables CSS-controlled rounded corners and box-shadow.
- `macOSPrivateApi: true` -- **required** at the app level for macOS transparency. Prevents App Store submission.
- `acceptFirstMouse: true` -- macOS: first click on the window is a real click, not just a focus click.

#### CSS Requirements

```css
/* widget/globals.css */
html, body {
  background: transparent !important;
  margin: 0;
  padding: 0;
  overflow: hidden;
}

.widget-container {
  border-radius: 12px;
  background: white;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.12);
  overflow: hidden;
}
```

#### Draggable Region (HTML)

```html
<div data-tauri-drag-region class="widget-header">
  <span>Tally Wallet</span>
  <!-- Buttons inside are NOT draggable -- they remain clickable -->
  <button class="expand-btn">⤢</button>
</div>
```

**Required permission**: `core:window:allow-start-dragging`

**Note**: `data-tauri-drag-region` only applies to the element it is on. Children do NOT inherit drag behavior. This is by design so buttons/inputs remain interactive.

#### Compact ⇄ Expanded Mode Switching

Switch between compact card (360x240) and expanded mini-app (360x520) by resizing:

```typescript
// widget/modeSwitch.ts
import { getCurrentWindow } from '@tauri-apps/api/window';
import { LogicalSize } from '@tauri-apps/api/dpi';

const COMPACT = { width: 360, height: 240 };
const EXPANDED = { width: 360, height: 520 };

export async function setWidgetMode(mode: 'compact' | 'expanded') {
  const win = getCurrentWindow();
  const size = mode === 'compact' ? COMPACT : EXPANDED;
  await win.setSize(new LogicalSize(size.width, size.height));
}
```

#### Rust APIs Used

| API | Purpose |
|-----|---------|
| `WebviewWindowBuilder::new(app, "widget", url)` | Create widget window |
| `.decorations(false)` | Frameless |
| `.transparent(true)` | Transparent background |
| `.always_on_top(true)` | Float above other windows |
| `.skip_taskbar(true)` | Hide from taskbar/dock |
| `.shadow(false)` | No native shadow |
| `.visible(false)` | Start hidden |
| `.focused(false)` | Don't steal focus on creation |
| `window.show()` / `window.hide()` | Toggle visibility |
| `window.set_size(LogicalSize)` | Resize for compact/expanded |
| `window.set_position(LogicalPosition)` | Position near tray |

#### JS APIs Used

| API | Purpose |
|-----|---------|
| `getCurrentWindow().show()` / `.hide()` | Toggle visibility |
| `getCurrentWindow().setSize(LogicalSize)` | Mode switching |
| `getCurrentWindow().startDragging()` | Programmatic drag (alternative to `data-tauri-drag-region`) |
| `getCurrentWindow().isVisible()` | Check state before toggle |

---

### 2. Tray / Status Bar Integration

**Goal**: System tray icon with right-click menu. Left-click toggles the widget window positioned near the tray icon. Close/minimize hides to tray instead of quitting.

#### Installation

Enable the `tray-icon` feature in `src-tauri/Cargo.toml`:

```toml
[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-positioner = { version = "2", features = ["tray-icon"] }
```

No additional npm install needed -- tray API is part of `@tauri-apps/api`.

#### Capabilities

```jsonc
// src-tauri/capabilities/default.json
{
  "permissions": [
    "core:tray:default",
    "core:tray:allow-set-icon",
    "core:menu:default",
    "positioner:default"
  ]
}
```

#### Exact Rust Code Flow

```rust
use tauri::{
    Manager,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};
use tauri_plugin_positioner::{Position, WindowExt};

pub fn setup_tray(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Build the right-click menu
    let show_main = MenuItemBuilder::with_id("show_main", "Open Tally Wallet").build(app)?;
    let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
    let menu = MenuBuilder::new(app).items(&[&show_main, &quit]).build()?;

    // 2. Build the tray icon
    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("Tally Wallet")
        .menu(&menu)
        .show_menu_on_left_click(false) // Left-click = toggle widget, right-click = menu
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show_main" => {
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.show();
                    let _ = w.set_focus();
                }
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            // CRITICAL: Forward tray events to positioner plugin
            tauri_plugin_positioner::on_tray_event(tray.app_handle(), &event);

            // Left-click: toggle widget window near tray icon
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(widget) = app.get_webview_window("widget") {
                    // Toggle: if visible, hide; if hidden, position and show
                    if widget.is_visible().unwrap_or(false) {
                        let _ = widget.hide();
                    } else {
                        let _ = widget
                            .as_ref()
                            .window()
                            .move_window(Position::TrayBottomCenter);
                        let _ = widget.show();
                        let _ = widget.set_focus();
                    }
                }
            }
        })
        .build(app)?;

    Ok(())
}
```

**Critical**: You **must** call `tauri_plugin_positioner::on_tray_event()` inside `on_tray_icon_event`. Without this, `TrayBottomCenter` and other tray-relative positions will not know the tray icon's location.

#### Close Interception (Hide to Tray)

Intercept the close event on the main window and widget window to hide instead of quit:

```rust
// In setup, after creating windows:
let main_window = app.get_webview_window("main").unwrap();
main_window.on_window_event(|event| {
    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
        api.prevent_close();
        // window.hide() -- need to capture window reference
    }
});
```

Or from JavaScript:

```typescript
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';

const appWindow = getCurrentWebviewWindow();
appWindow.onCloseRequested(async (event) => {
    event.preventDefault();
    await appWindow.hide();
});
```

#### Position Behavior by Platform

| Platform | Tray Location | Widget Appears |
|----------|---------------|----------------|
| macOS | Top menu bar (right) | Below the tray icon |
| Windows | Bottom taskbar (right, system tray) | Above the tray icon |
| Linux | Varies by DE | Varies; click events may not fire |

#### Auto-Dismiss on Click-Away

When the user clicks outside the widget, hide it:

```rust
// Listen for focus loss
widget_window.on_window_event(|event| {
    if let tauri::WindowEvent::Focused(false) = event {
        // widget_window.hide()
    }
});
```

---

### 3. Custom Notification Popups

**Goal**: When an agent charge occurs, show a toast card near the top-right of the screen. Non-focusable, transparent overlay. Auto-dismisses after 5 seconds. Clicking navigates the main app to the transaction detail.

#### Architecture: Single Persistent Hidden Window (Option B)

One `WebviewWindow` created at startup with `visible: false`. On notification, update content via events and show. Manages its own queue of up to 3 visible notifications.

**Why not a window per toast**: Window creation costs ~50-150ms. Bursty events (5 agent charges in 1 second) would spawn 5 webviews. Too expensive.

**Why not in-app overlay (Option C)**: Only visible when the main window is in the foreground. Notifications should appear even when the app is minimized.

#### tauri.conf.json Window Config

```jsonc
{
  "label": "toast",
  "url": "/toast",
  "title": "Notifications",
  "width": 380,
  "height": 500,
  "visible": false,
  "decorations": false,
  "transparent": true,
  "shadow": false,
  "alwaysOnTop": true,
  "skipTaskbar": true,
  "focusable": false,
  "focus": false,
  "resizable": false
}
```

#### Toast Window CSS

```css
html, body {
  background: transparent !important;
  margin: 0;
  padding: 0;
  overflow: hidden;
}
```

#### Event Flow: Rust to Toast Window to Main Window

```
Agent Charge Detected (Rust Service / MCP handler)
  |
  ├── app.emit_to("toast", "agent-charge", payload)
  |         |
  |         └── Toast webview: getCurrentWebviewWindow().listen("agent-charge", ...)
  |                   |
  |                   └── Adds to notification queue, triggers CSS slide-in animation
  |
  └── app.emit("agent-charge-global", payload)  // For main app transaction list refresh
```

**Rust -- emit the charge event:**

```rust
use tauri::{AppHandle, Emitter};
use serde::Serialize;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AgentChargePayload {
    agent_name: String,
    amount: String,
    description: String,
    tx_hash: String,
    timestamp: u64,
}

fn notify_agent_charge(app: &AppHandle, charge: AgentChargePayload) {
    if let Some(win) = app.get_webview_window("toast") {
        let _ = win.show();
    }
    app.emit_to("toast", "agent-charge", &charge).unwrap();
    app.emit("agent-charge-global", &charge).unwrap();
}
```

**Toast webview -- listen and queue:**

```typescript
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';

const appWindow = getCurrentWebviewWindow();

// IMPORTANT: Use webview-specific listen, NOT global listen()
// Global listen() will NOT receive events sent via emit_to()
appWindow.listen<AgentChargePayload>('agent-charge', (event) => {
    notificationStore.addNotification(event.payload);
});
```

**Toast click -- navigate main window:**

```typescript
import { emitTo } from '@tauri-apps/api/event';

function handleToastClick(txHash: string) {
    emitTo('main', 'navigate-to-transaction', { txHash });
}
```

**Main window -- listen for navigation:**

```typescript
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';

const mainWindow = getCurrentWebviewWindow();
mainWindow.listen('navigate-to-transaction', (event) => {
    const { txHash } = event.payload as { txHash: string };
    navigate(`/transactions/${txHash}`);
    mainWindow.show();
    mainWindow.setFocus();
});
```

#### Animation (CSS, GPU-accelerated)

```css
@keyframes slideIn {
  from { transform: translateX(100%); opacity: 0; }
  to { transform: translateX(0); opacity: 1; }
}

@keyframes slideOut {
  from { transform: translateX(0); opacity: 1; }
  to { transform: translateX(100%); opacity: 0; }
}

.toast-card {
  animation: slideIn 300ms cubic-bezier(0.16, 1, 0.3, 1) forwards;
}
.toast-card.dismissing {
  animation: slideOut 200ms cubic-bezier(0.16, 1, 0.3, 1) forwards;
}
```

#### Queuing/Stacking Strategy

- **Max 3 visible** cards stacked vertically.
- Additional notifications queue behind; promoted when a visible card dismisses.
- Auto-dismiss after 5 seconds.
- Dynamically resize toast window height to match visible card count (reduces click-blocking area).
- When notification count drops to 0, hide the toast window.

#### Click-Through for Transparent Areas

Toggle `setIgnoreCursorEvents` dynamically based on whether the cursor is over a card:

```typescript
document.addEventListener('mousemove', (e) => {
    const target = e.target as HTMLElement;
    const isOverCard = target.closest('.toast-card') !== null;
    getCurrentWebviewWindow().setIgnoreCursorEvents(!isOverCard);
});
```

---

### 4. Eventing & Window Coordination

#### Rust Event Emission Methods

| Method | Scope | Example Use |
|--------|-------|-------------|
| `app.emit("name", payload)` | All global listeners in all windows | Balance update broadcasts |
| `app.emit_to("label", "name", payload)` | Specific webview by label | Agent charge to toast window |
| `app.emit_filter("name", payload, \|target\| ...)` | Filtered set of webviews | Notify widget + toast but not main |

#### Multi-Window State Sharing

State is **not** shared across windows via JavaScript. Each window has its own webview and its own Zustand stores. Coordination happens through:

1. **Tauri events** -- Rust emits to specific windows, or windows emit to each other via `emitTo`.
2. **Tauri commands** -- All windows call the same Rust backend. The Rust `AppState` (services, DB) is the single source of truth.
3. **No shared JS memory** -- each webview is a separate process.

Pattern for cross-window updates:
```
Widget window calls Rust command (e.g., approve_transaction)
  → Rust service processes it
  → Rust emits event to main window ("refresh-transactions")
  → Rust emits event to toast window ("transaction-approved")
  → Each window's listener updates its own local store
```

#### Security: What Stays in Rust vs Webview

| Data | Location | Reason |
|------|----------|--------|
| Wallet private keys | Rust only | Never exposed to webview |
| Agent tokens | Rust only | Sensitive credentials |
| API keys | Rust only | Backend-only access |
| Balance amounts | Webview (via events) | Display data, OK to expose |
| Transaction history | Webview (via commands) | Display data |
| Agent names/policies | Webview (via commands) | Display data |

#### Capabilities/ACL Per Window

Each window gets its own capability file with minimal permissions:

```jsonc
// src-tauri/capabilities/main-window.json
{
  "identifier": "main-window-cap",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "core:window:default",
    "core:window:allow-start-dragging",
    "core:window:allow-show",
    "core:window:allow-hide",
    "core:window:allow-set-focus",
    "core:window:allow-close",
    "core:event:default",
    "core:tray:default",
    "core:menu:default",
    "positioner:default",
    "autostart:allow-enable",
    "autostart:allow-disable",
    "autostart:allow-is-enabled"
  ]
}
```

```jsonc
// src-tauri/capabilities/widget-window.json
{
  "identifier": "widget-window-cap",
  "windows": ["widget"],
  "permissions": [
    "core:default",
    "core:window:allow-start-dragging",
    "core:window:allow-show",
    "core:window:allow-hide",
    "core:window:allow-set-size",
    "core:window:allow-set-focus",
    "core:event:default"
  ]
}
```

```jsonc
// src-tauri/capabilities/toast-window.json
{
  "identifier": "toast-window-cap",
  "windows": ["toast"],
  "permissions": [
    "core:default",
    "core:window:allow-show",
    "core:window:allow-hide",
    "core:window:allow-set-size",
    "core:window:allow-set-ignore-cursor-events",
    "core:event:default"
  ]
}
```

---

## C) Plugin Map

| Plugin | Cargo Crate | npm Package | Purpose | Required? |
|--------|-------------|-------------|---------|-----------|
| **System Tray** | Built into `tauri` (feature: `tray-icon`) | Built into `@tauri-apps/api` | Tray icon, menu, click handling | **Required** |
| **Positioner** | `tauri-plugin-positioner` (feature: `tray-icon`) | `@tauri-apps/plugin-positioner` | Position widget near tray, toast at top-right | **Required** |
| **Autostart** | `tauri-plugin-autostart` | `@tauri-apps/plugin-autostart` | Launch at system login | Optional (Phase 4) |
| **Window State** | `tauri-plugin-window-state` | `@tauri-apps/plugin-window-state` | Persist/restore main window position | Optional (Phase 4) |
| **Notification** | `tauri-plugin-notification` | `@tauri-apps/plugin-notification` | Native OS notifications (fallback) | Optional |
| **tauri-nspanel** | `tauri-nspanel` (community, v2 branch) | N/A | macOS NSPanel for non-focus-stealing widget | Optional (Phase 3 polish) |
| **tauri-plugin-polygon** | `tauri-plugin-polygon` (community) | N/A | Polygon-based click-through regions | Optional |

### Cargo.toml (MVP)

```toml
[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-positioner = { version = "2", features = ["tray-icon"] }
```

### Cargo.toml (Full)

```toml
[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-positioner = { version = "2", features = ["tray-icon"] }
tauri-plugin-autostart = "2"
tauri-plugin-window-state = "2"
# tauri-plugin-notification = "2"  # Only if native OS notification fallback is needed
```

---

## D) Cross-Platform Risk Matrix

### Per-Behavior Matrix

| Behavior | macOS | Windows | Linux |
|----------|-------|---------|-------|
| **Transparent window** | Works (needs `macOSPrivateApi: true`) | Works (alpha quirks on Win8+) | Works (needs compositor) |
| **Transparency in DMG build** | **RISK** -- may render opaque ([#13415]) | N/A | N/A |
| **Frameless (no decorations)** | Works | Works | Works (WM may add decorations) |
| **Always-on-top** | Works (not over fullscreen apps) | Works (not over exclusive fullscreen) | **Unreliable** (WM-dependent) |
| **Always-on-top over fullscreen** | **Broken** -- needs NSPanel workaround | Broken for exclusive fullscreen | Broken |
| **`focusable: false`** | **Broken** ([#14102]) -- still steals focus | **Broken** ([#7519]) -- `focus: false` ignored | Varies by WM |
| **`skipTaskbar`** | Works for taskbar, NOT Cmd+Tab | **Unreliable** ([#10422]) | Works on X11, **broken on Wayland** ([#9829]) |
| **Hide from Cmd/Alt+Tab** | `ActivationPolicy::Accessory` (hides Dock too) | `WS_EX_TOOLWINDOW` via native API | X11: `_NET_WM_STATE_SKIP_TASKBAR` |
| **Tray icon display** | Works | Works | Works (may need empty menu) |
| **Tray click events** | Works | Works (+ DoubleClick) | **No click events** (right-click menu works) |
| **Tray-relative positioning** | Works (below tray) | Works (above tray) | Partial (no click events) |
| **CSS `backdrop-filter: blur()`** | Works | **Broken** with `transparent: true` ([#10064]) | Works |
| **Rounded corners** | Works (`shadow: false` needed) | Win11: native rounded; Win10: rectangular | Works (needs compositor) |
| **Click-through (`setIgnoreCursorEvents`)** | Works | Works | Works on X11, **unreliable on Wayland** ([#11461]) |
| **Window shadows** | Disabled for transparent windows | 1px white border on undecorated | Compositor-controlled |
| **Tray tooltip** | Works | Works | **Not supported** |

### Critical Risks and Workarounds

| Risk | Severity | Workaround |
|------|----------|------------|
| **`focusable: false` broken (macOS)** | High | Use `tauri-nspanel` to convert to NSPanel with `NSWindowStyleMaskNonActivatingPanel`. Or use `ActivationPolicy::Accessory` (hides from Dock). |
| **`focusable: false` broken (Windows)** | High | Post-creation, set `WS_EX_NOACTIVATE` extended window style via `windows-rs` / `winapi` crate on the HWND. |
| **`skipTaskbar` unreliable (Windows)** | Medium | Set `WS_EX_TOOLWINDOW` via native API. Tool windows have different focus behavior. |
| **Transparency lost in DMG (macOS)** | Medium | Test with release builds (`tauri build`) early and often. Do not wait until shipping. |
| **`backdrop-filter: blur()` broken (Windows)** | Low | Use native acrylic/mica effects instead of CSS blur, OR use solid backgrounds. |
| **Always-on-top not over fullscreen** | Low | Accept this limitation for MVP. For Phase 3, NSPanel + `CanJoinAllSpaces` on macOS. |
| **Linux Wayland tray issues** | Low | Target X11 as primary Linux backend. Recommend `GDK_BACKEND=x11` for Wayland users. Require webkit2gtk >= 2.48.0. |

[#13415]: https://github.com/tauri-apps/tauri/issues/13415
[#14102]: https://github.com/tauri-apps/tauri/issues/14102
[#7519]: https://github.com/tauri-apps/tauri/issues/7519
[#10422]: https://github.com/tauri-apps/tauri/issues/10422
[#9829]: https://github.com/tauri-apps/tauri/issues/9829
[#10064]: https://github.com/tauri-apps/tauri/issues/10064
[#11461]: https://github.com/tauri-apps/tauri/issues/11461

---

## E) Window State Machine

### Window Definitions

| Window | Label | Startup State | Purpose |
|--------|-------|---------------|---------|
| **Main** | `"main"` | Hidden | Full dashboard app |
| **Widget** | `"widget"` | Hidden | Floating credit card / mini-app |
| **Toast** | `"toast"` | Hidden | Notification overlay |

### Main Window States

```
[Hidden] ──show()──→ [Visible/Focused]
    ↑                       │
    └── hide() ◄────────────┘ (close-requested intercepted)
```

- On close-requested: `event.preventDefault()` then `window.hide()`.
- Shown when: user clicks "Open Tally Wallet" in tray menu, or toast click navigates here.

### Widget Window States

```
[Hidden] ──tray click──→ [Compact Card] ──expand btn──→ [Expanded Mini-App]
    ↑                          │                               │
    │                          │ tray click                    │ collapse btn
    └── tray click ◄───────────┘                               │
    └── blur (focus loss) ◄────────────────────────────────────┘
```

- **Compact**: 360x240, shows credit card with balance.
- **Expanded**: 360x520, shows card + recent transactions + quick actions.
- On blur (click outside): hide the widget.
- Position: always anchored near tray icon via positioner plugin.

### Toast Window States

```
[Hidden] ──agent-charge event──→ [Showing N cards] ──all dismissed──→ [Hidden]
                                       │
                                       │ new event arrives
                                       └──→ [Showing N+1 cards] (max 3, rest queued)
```

- Created at startup, kept hidden.
- Shown when first notification arrives.
- Hidden when last notification dismisses.
- Window height dynamically resized to fit visible cards.

### Event Flows

#### 1. App Launch

```
App starts
  → Create main window (hidden)
  → Create widget window (hidden)
  → Create toast window (hidden)
  → Create tray icon with menu
  → Tray icon appears in system tray / menu bar
  → User sees only the tray icon
```

#### 2. Tray Click -- Toggle Widget

```
User left-clicks tray icon
  → on_tray_icon_event fires
  → Forward event to positioner plugin (on_tray_event)
  → Check widget.is_visible()
  → If hidden:
      → widget.move_window(Position::TrayBottomCenter)
      → widget.show()
      → widget.set_focus()
  → If visible:
      → widget.hide()
```

#### 3. Agent Charge Event -- Toast Appears

```
Rust service detects agent charge (MCP handler / polling)
  → app.emit_to("toast", "agent-charge", payload)
  → Toast webview listener fires
  → notificationStore.addNotification(payload)
  → If notifications.length was 0:
      → toast_window.show()
  → CSS slideIn animation plays
  → After 5 seconds: slideOut animation, remove from store
  → If notifications.length drops to 0:
      → toast_window.hide()
```

#### 4. Toast Click -- Open Transaction Detail

```
User clicks a toast card
  → emitTo("main", "navigate-to-transaction", { txHash })
  → Main window listener fires
  → React Router navigates to /transactions/{txHash}
  → main_window.show()
  → main_window.set_focus()
  → (Optional) dismiss the clicked toast
```

#### 5. Widget Expand

```
User clicks expand button in widget
  → setWidgetMode("expanded")
  → widget.setSize(new LogicalSize(360, 520))
  → React renders expanded view (recent transactions, quick actions)
```

#### 6. Close/Minimize -- Hide to Tray

```
User clicks close button on main window
  → onCloseRequested fires
  → event.preventDefault()
  → main_window.hide()
  → App continues running in tray
```

---

## F) Suggested Directory Structure

```
src-tauri/
├── Cargo.toml
├── tauri.conf.json
├── capabilities/
│   ├── main-window.json        # Permissions for main window
│   ├── widget-window.json      # Permissions for widget window
│   └── toast-window.json       # Permissions for toast window
├── icons/
│   ├── icon.png                # App icon
│   └── tray-icon.png           # Tray icon (consider template icon for macOS)
├── src/
│   ├── lib.rs                  # App builder, plugin registration, tray setup
│   ├── main.rs                 # Entry point
│   ├── commands/
│   │   ├── mod.rs
│   │   ├── wallet.rs           # Wallet commands (existing)
│   │   ├── auth.rs             # Auth commands (existing)
│   │   └── notifications.rs    # Notification-related commands
│   ├── services/
│   │   ├── mod.rs
│   │   ├── wallet_service.rs   # (existing)
│   │   ├── auth_service.rs     # (existing)
│   │   └── notification_service.rs  # Manages charge detection, emits events
│   ├── state/
│   │   └── app_state.rs        # AppState with all services (existing)
│   ├── tray.rs                 # Tray icon setup, menu, click handlers
│   └── windows.rs              # Window creation, close interception, helpers
│
src/                            # Frontend (React)
├── main.tsx                    # Main window entry point (existing)
├── widget.tsx                  # Widget window entry point (NEW)
├── toast.tsx                   # Toast window entry point (NEW)
├── App.tsx                     # Main app component (existing)
├── WidgetApp.tsx               # Widget root component (NEW)
├── ToastApp.tsx                # Toast root component (NEW)
├── pages/                      # Main window pages (existing)
├── components/
│   ├── widget/                 # Widget-specific components (NEW)
│   │   ├── CreditCard.tsx
│   │   ├── CompactView.tsx
│   │   └── ExpandedView.tsx
│   └── toast/                  # Toast-specific components (NEW)
│       ├── ToastCard.tsx
│       └── ToastStack.tsx
├── stores/
│   ├── walletStore.ts          # (existing)
│   ├── authStore.ts            # (existing)
│   └── notificationStore.ts    # Toast notification queue (NEW)
├── styles/
│   ├── globals.css             # Main window styles (existing)
│   ├── widget.css              # Widget window styles (NEW)
│   └── toast.css               # Toast window styles (NEW)
│
index.html                      # Main window HTML entry
widget.html                     # Widget window HTML entry (NEW)
toast.html                      # Toast window HTML entry (NEW)
```

### Multi-Entry-Point Vite Config

Each window needs its own HTML entry point. Configure Vite for multi-page:

```typescript
// vite.config.ts
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { resolve } from 'path';

export default defineConfig({
  plugins: [react()],
  build: {
    rollupOptions: {
      input: {
        main: resolve(__dirname, 'index.html'),
        widget: resolve(__dirname, 'widget.html'),
        toast: resolve(__dirname, 'toast.html'),
      },
    },
  },
});
```

Each HTML file loads its own React root:

```html
<!-- widget.html -->
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <link rel="stylesheet" href="/src/styles/widget.css" />
</head>
<body>
  <div id="root"></div>
  <script type="module" src="/src/widget.tsx"></script>
</body>
</html>
```

---

## G) Milestone Plan

### MVP (Phase 1): Tray + Widget Window

**Goal**: Tray icon appears at launch. Left-click toggles a floating credit card widget near the tray. Right-click shows a menu with "Open Wallet" and "Quit".

**Tasks**:
1. Add `tray-icon` feature to `tauri` in Cargo.toml
2. Add `tauri-plugin-positioner` with `tray-icon` feature
3. Create `src-tauri/src/tray.rs` -- tray icon, menu, click handler
4. Create `src-tauri/src/windows.rs` -- widget window creation, close interception
5. Add widget window config to `tauri.conf.json`
6. Create `widget.html`, `src/widget.tsx`, `src/WidgetApp.tsx`
7. Build compact credit card component (360x240)
8. Wire tray left-click to toggle widget visibility with `TrayBottomCenter` positioning
9. Intercept main window close to hide instead of quit
10. Add per-window capability files
11. Test on macOS (primary dev platform)

**Estimated effort**: 2-3 days

### Phase 2: Toast Notifications

**Goal**: When an agent charge occurs, a toast card slides in from the top-right. Auto-dismisses after 5s. Clicking opens the main app to the transaction.

**Tasks**:
1. Create `toast.html`, `src/toast.tsx`, `src/ToastApp.tsx`
2. Add toast window config to `tauri.conf.json`
3. Build `ToastCard` component with CSS slide-in/slide-out animations
4. Build `notificationStore` (Zustand) with queue, max 3 visible, auto-dismiss
5. Create `notification_service.rs` in Rust backend -- emits `agent-charge` events
6. Wire `emit_to("toast", ...)` from Rust charge detection
7. Implement click-to-navigate: toast emits to main window, main window navigates
8. Dynamic toast window resizing based on notification count
9. Implement `setIgnoreCursorEvents` toggling for click-through

**Estimated effort**: 3-4 days

### Phase 3: Polish & Platform Workarounds

**Goal**: Fix the focus-stealing and platform-specific issues found in research.

**Tasks**:
1. **macOS NSPanel workaround**: Integrate `tauri-nspanel` (v2 branch) for widget window -- prevents focus stealing
2. **macOS**: Set `ActivationPolicy::Accessory` to hide from Dock/Cmd+Tab (tray-only app)
3. **Windows**: Post-creation `WS_EX_NOACTIVATE` on widget and toast windows via native API
4. **Windows**: `WS_EX_TOOLWINDOW` for `skipTaskbar` reliability
5. CSS animation polish: spring-based easing, micro-interactions
6. Toast notification stacking refinement, collapse-by-agent for bursty events
7. Test transparency in macOS DMG builds -- verify no opaque regression
8. Multi-monitor testing with positioner plugin

**Estimated effort**: 3-5 days

### Phase 4: Autostart, Persistence, Linux

**Goal**: Production readiness across all desktop platforms.

**Tasks**:
1. Add `tauri-plugin-autostart` with `--minimized` flag
2. Add "Launch at login" toggle in Settings page
3. Add `tauri-plugin-window-state` for main window position persistence (exclude widget/toast)
4. Linux testing and fixes:
   - X11 primary, Wayland fallback with `GDK_BACKEND=x11`
   - Require webkit2gtk >= 2.48.0 in docs/installer
   - Test `libayatana-appindicator3-1` dependency
5. Windows testing:
   - Verify WebView2 transparency
   - Test acrylic/mica effects as alternative to CSS blur
6. Compact mode as default on autostart (no main window until explicitly opened)

**Estimated effort**: 3-4 days

---

## H) Source Citations

### Official Tauri Documentation
- [Tauri v2 Official Docs](https://v2.tauri.app/)
- [Tauri v2 Configuration Reference](https://v2.tauri.app/reference/config/)
- [Tauri v2 Window JS API](https://v2.tauri.app/reference/javascript/api/namespacewindow/)
- [Tauri v2 Event JS API](https://v2.tauri.app/reference/javascript/api/namespaceevent/)
- [Tauri v2 Calling Frontend from Rust](https://v2.tauri.app/develop/calling-frontend/)
- [Tauri v2 Window Customization Guide](https://v2.tauri.app/learn/window-customization/)
- [Tauri v2 Security / Capabilities](https://v2.tauri.app/security/capabilities/)
- [Tauri v2 System Tray](https://v2.tauri.app/learn/system-tray/)
- [Tauri v2 Positioner Plugin](https://v2.tauri.app/plugin/positioner/)
- [Tauri v2 Autostart Plugin](https://v2.tauri.app/plugin/autostart/)
- [Tauri v2 Window State Plugin](https://v2.tauri.app/plugin/window-state/)
- [Tauri v2 Notification Plugin](https://v2.tauri.app/plugin/notification/)
- [Tauri v2 Release Page](https://v2.tauri.app/release/)
- [Tauri 2.0 Stable Release Announcement](https://v2.tauri.app/blog/tauri-20/)

### Rust API Docs
- [WindowConfig (docs.rs)](https://docs.rs/tauri-utils/latest/tauri_utils/config/struct.WindowConfig.html)
- [WebviewWindowBuilder (docs.rs)](https://docs.rs/tauri/latest/tauri/webview/struct.WebviewWindowBuilder.html)
- [Window (docs.rs)](https://docs.rs/tauri/latest/tauri/window/struct.Window.html)
- [Emitter trait (docs.rs)](https://docs.rs/tauri/latest/tauri/trait.Emitter.html)

### GitHub Issues (Referenced in Risk Matrix)
- [#14102 -- `focusable: false` broken on macOS](https://github.com/tauri-apps/tauri/issues/14102)
- [#7519 -- `focus: false` ignored on Windows](https://github.com/tauri-apps/tauri/issues/7519)
- [#10422 -- `skipTaskbar` unreliable on Windows](https://github.com/tauri-apps/tauri/issues/10422)
- [#9829 -- `skipTaskbar` broken on Wayland](https://github.com/tauri-apps/tauri/issues/9829)
- [#13415 -- Transparency lost in DMG build on macOS](https://github.com/tauri-apps/tauri/issues/13415)
- [#8308 -- `transparent: true` broken in v2 on Windows](https://github.com/tauri-apps/tauri/issues/8308)
- [#11488 -- Not visible over fullscreen on macOS](https://github.com/tauri-apps/tauri/issues/11488)
- [#14234 -- Tray icon missing on Wayland](https://github.com/tauri-apps/tauri/issues/14234)
- [#12800 -- Ghost renders on Linux](https://github.com/tauri-apps/tauri/issues/12800)
- [#10064 -- CSS blur broken with transparent on Windows](https://github.com/tauri-apps/tauri/issues/10064)
- [#8255 -- Transparent glitch on macOS Sonoma](https://github.com/tauri-apps/tauri/issues/8255)
- [#6162 -- Window properties ignored on Linux](https://github.com/tauri-apps/tauri/issues/6162)
- [#13157 -- Glitchy rendering on Linux](https://github.com/tauri-apps/tauri/issues/13157)
- [#9287 -- Rounded corners + shadows conflict](https://github.com/tauri-apps/tauri/issues/9287)
- [#12834 -- `setFocus()` broken 2.3+ on macOS](https://github.com/tauri-apps/tauri/issues/12834)
- [#11461 -- `setIgnoreCursorEvents` broken on Linux](https://github.com/tauri-apps/tauri/issues/11461)
- [#13070 -- Transparent window click-through workaround](https://github.com/tauri-apps/tauri/issues/13070)
- [#11718 -- PhysicalPosition bug on Windows multi-monitor](https://github.com/tauri-apps/tauri/issues/11718)

### GitHub Discussions
- [Discussion #7951 -- Overlay notification](https://github.com/tauri-apps/tauri/discussions/7951)
- [Discussion #4810 -- How to make an Overlay](https://github.com/tauri-apps/tauri/discussions/4810)
- [Discussion #6604 -- Notification System for Tauri](https://github.com/tauri-apps/tauri/discussions/6604)

### Community / Third-Party
- [tauri-nspanel (v2 branch)](https://github.com/ahkohd/tauri-nspanel) -- macOS NSPanel integration
- [tauri-plugin-polygon](https://crates.io/crates/tauri-plugin-polygon) -- Polygon-based click-through regions
- [window-vibrancy crate](https://github.com/tauri-apps/window-vibrancy) -- Platform vibrancy effects
- [Tauri Releases (GitHub)](https://github.com/tauri-apps/tauri/releases)
- [Building a system tray app with Tauri](https://tauritutorials.com/blog/building-a-system-tray-app-with-tauri)
