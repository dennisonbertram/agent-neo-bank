# Tauri v2 Plugins: Tray, Positioner, and Related Plugins

> Research date: 2026-02-28
> Sources: Official Tauri v2 docs, docs.rs, GitHub plugins-workspace, context7

---

## Table of Contents

1. [System Tray (Built-in)](#1-system-tray-built-in)
2. [Positioner Plugin](#2-positioner-plugin)
3. [Window State Plugin](#3-window-state-plugin)
4. [Autostart Plugin](#4-autostart-plugin)
5. [Notification Plugin](#5-notification-plugin)
6. [Summary: Package Names and Versions](#6-summary-package-names-and-versions)
7. [Platform Support Matrix](#7-platform-support-matrix)
8. [Integration Recipe: Tray App with Positioned Window](#8-integration-recipe-tray-app-with-positioned-window)

---

## 1. System Tray (Built-in)

The system tray is **not a plugin** -- it is built into the `tauri` crate itself behind a feature flag. No separate crate or npm package is needed.

**Source:** https://v2.tauri.app/learn/system-tray/

### 1.1 Installation

Enable the `tray-icon` feature in `src-tauri/Cargo.toml`:

```toml
[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
```

On the frontend, the tray API is part of the core `@tauri-apps/api` package:

```typescript
import { TrayIcon } from '@tauri-apps/api/tray';
import { Menu, MenuItem } from '@tauri-apps/api/menu';
```

No additional npm install is required beyond `@tauri-apps/api`.

### 1.2 Capabilities / Permissions

Add to `src-tauri/capabilities/default.json`:

```json
{
  "permissions": [
    "core:tray:default",
    "core:tray:allow-set-icon",
    "core:menu:default"
  ]
}
```

### 1.3 Creating a Tray Icon

#### Rust (recommended for desktop apps)

```rust
use tauri::tray::TrayIconBuilder;

tauri::Builder::default()
    .setup(|app| {
        let tray = TrayIconBuilder::new()
            .icon(app.default_window_icon().unwrap().clone())
            .tooltip("Tally Wallet")
            .build(app)?;
        Ok(())
    })
```

#### JavaScript/TypeScript

```typescript
import { TrayIcon } from '@tauri-apps/api/tray';
import { defaultWindowIcon } from '@tauri-apps/api/app';

const tray = await TrayIcon.new({
    icon: await defaultWindowIcon(),
    tooltip: 'Tally Wallet',
});
```

### 1.4 Tray Menu

#### Rust

```rust
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::TrayIconBuilder;

tauri::Builder::default()
    .setup(|app| {
        let show = MenuItemBuilder::with_id("show", "Show Wallet").build(app)?;
        let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
        let menu = MenuBuilder::new(app).items(&[&show, &quit]).build()?;

        let tray = TrayIconBuilder::new()
            .icon(app.default_window_icon().unwrap().clone())
            .tooltip("Tally Wallet")
            .menu(&menu)
            .show_menu_on_left_click(false) // only show menu on right-click
            .on_menu_event(move |app, event| match event.id().as_ref() {
                "show" => {
                    if let Some(w) = app.get_webview_window("main") {
                        let _ = w.show();
                        let _ = w.set_focus();
                    }
                }
                "quit" => app.exit(0),
                _ => {}
            })
            .build(app)?;

        Ok(())
    })
```

#### JavaScript/TypeScript

```typescript
import { TrayIcon } from '@tauri-apps/api/tray';
import { Menu } from '@tauri-apps/api/menu';

const menu = await Menu.new({
    items: [
        { id: 'show', text: 'Show Wallet', action: () => { /* show window */ } },
        { id: 'quit', text: 'Quit', action: () => { /* exit */ } },
    ],
});

const tray = await TrayIcon.new({
    menu,
    showMenuOnLeftClick: false,
});
```

### 1.5 Tray Icon Click Event Handling

Tauri v2 emits five tray icon event types:

| Event | Description | Fields | Platform Notes |
|-------|-------------|--------|----------------|
| `Click` | Single click (any button) | `id`, `position`, `rect`, `button`, `button_state` | Not emitted on Linux |
| `DoubleClick` | Double-click | `id`, `position`, `rect`, `button` | Windows only |
| `Enter` | Cursor enters tray region | `id`, `position`, `rect` | Not emitted on Linux |
| `Move` | Cursor moves over tray | `id`, `position`, `rect` | Not emitted on Linux |
| `Leave` | Cursor exits tray region | `id`, `position`, `rect` | Not emitted on Linux |

- `button`: `MouseButton::Left`, `MouseButton::Right`, `MouseButton::Middle`
- `button_state`: `MouseButtonState::Up`, `MouseButtonState::Down`
- `position`: `PhysicalPosition<f64>` -- physical screen coordinates of the click
- `rect`: `Rect` -- position and size of the tray icon itself

**Important:** The `TrayIconEvent` enum is `#[non_exhaustive]`, so always include a wildcard `_` match arm.

#### Rust -- Left-click to show window, right-click for menu

```rust
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};

TrayIconBuilder::new()
    .show_menu_on_left_click(false)
    .on_tray_icon_event(|tray, event| {
        match event {
            TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } => {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.unminimize();
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            _ => {}
        }
    })
    .build(app)?;
```

#### JavaScript

```typescript
const tray = await TrayIcon.new({
    showMenuOnLeftClick: false,
    action: (event) => {
        if (event.type === 'Click') {
            if (event.button === 'Left' && event.buttonState === 'Up') {
                // show/toggle the window
            }
            // event.button can also be 'Right' or 'Middle'
        }
    },
});
```

### 1.6 Tooltip

```rust
// At build time
TrayIconBuilder::new().tooltip("Tally Wallet").build(app)?;

// Dynamic update
tray.set_tooltip(Some("Tally Wallet - 3 pending"))?;
tray.set_tooltip(None)?; // remove tooltip
```

```typescript
const tray = await TrayIcon.new({ tooltip: 'Tally Wallet' });
await tray.setTooltip('Tally Wallet - 3 pending');
await tray.setTooltip(null); // remove
```

**Platform note:** Tooltip is **unsupported on Linux**.

### 1.7 Dynamic Tray Icon Updates

All `TrayIcon` properties can be updated at runtime via setter methods.

#### Rust

```rust
// Change icon
tray.set_icon(Some(new_icon))?;
tray.set_icon(None)?; // remove icon

// Change tooltip
tray.set_tooltip(Some("Updated tooltip"))?;

// Change title (macOS/Linux only)
tray.set_title(Some("Title"))?;

// Show/hide
tray.set_visible(true)?;

// macOS template icon (adapts to dark/light mode)
tray.set_icon_as_template(true)?;

// Replace menu
tray.set_menu(Some(new_menu))?;

// Toggle menu on left click
tray.set_show_menu_on_left_click(false)?;
```

#### JavaScript

```typescript
await tray.setIcon(newIcon);
await tray.setTooltip('new tooltip');
await tray.setTitle('Title');        // macOS/Linux only
await tray.setVisible(false);
await tray.setIconAsTemplate(true);  // macOS only
await tray.setMenu(newMenu);
await tray.setShowMenuOnLeftClick(false);
```

#### Static Methods

```typescript
// Look up an existing tray icon by ID
const tray = await TrayIcon.getById('my-tray');

// Remove a tray icon
await TrayIcon.removeById('my-tray');
```

### 1.8 Known Limitations and Gotchas

| Issue | Details |
|-------|---------|
| **Linux: no click/move/leave events** | The tray icon is shown and right-click menus work, but `Click`, `DoubleClick`, `Enter`, `Move`, `Leave` events are **not emitted** on Linux. |
| **Linux: icon may not appear** | On Linux, the icon may not show unless a `Menu` is set. Setting an empty menu is sufficient. |
| **Linux: cannot remove menu** | Once a menu is set on Linux, it cannot be removed (calling `set_menu(None)` has no effect). |
| **macOS: icon_as_template resets** | When calling `set_icon()`, the `icon_as_template` flag resets. You need to call `set_icon_as_template(true)` again afterwards. This causes a brief visual blink. |
| **macOS: title only** | `set_title()` only works on macOS and Linux. On Windows it is unsupported. |
| **Windows: DoubleClick only** | The `DoubleClick` event only fires on Windows. |

---

## 2. Positioner Plugin

Positions windows at well-known screen locations, including **relative to the system tray icon**. Essential for building a tray-anchored floating widget.

**Source:** https://v2.tauri.app/plugin/positioner/

### 2.1 Installation

**Cargo crate:** `tauri-plugin-positioner`
**npm package:** `@tauri-apps/plugin-positioner`

```bash
# Automatic (recommended)
npm run tauri add positioner

# Manual
cargo add tauri-plugin-positioner --target 'cfg(any(target_os = "macos", windows, target_os = "linux"))'
npm install @tauri-apps/plugin-positioner
```

For tray-relative positioning, enable the `tray-icon` feature:

```toml
[dependencies]
tauri-plugin-positioner = { version = "2", features = ["tray-icon"] }
```

### 2.2 Rust Setup

```rust
use tauri::tray::TrayIconBuilder;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_positioner::init())
        .setup(|app| {
            #[cfg(desktop)]
            {
                TrayIconBuilder::new()
                    .on_tray_icon_event(|tray_handle, event| {
                        // REQUIRED: forward tray events to positioner plugin
                        tauri_plugin_positioner::on_tray_event(
                            tray_handle.app_handle(),
                            &event,
                        );
                    })
                    .build(app)?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**Critical:** You **must** call `tauri_plugin_positioner::on_tray_event()` inside `on_tray_icon_event` for tray-relative positions to work. Without this, `TrayCenter` etc. will not know the tray icon location.

### 2.3 Capabilities / Permissions

```json
{
  "permissions": ["positioner:default"]
}
```

Default includes: `allow-move-window`, `allow-move-window-constrained`, `allow-set-tray-icon-state`.

### 2.4 Position Enum

All 15 available positions:

| Position | Value | Description |
|----------|-------|-------------|
| `TopLeft` | 0 | Top-left corner of screen |
| `TopRight` | 1 | Top-right corner of screen |
| `BottomLeft` | 2 | Bottom-left corner of screen |
| `BottomRight` | 3 | Bottom-right corner of screen |
| `TopCenter` | 4 | Top-center of screen |
| `BottomCenter` | 5 | Bottom-center of screen |
| `LeftCenter` | 6 | Left-center of screen |
| `RightCenter` | 7 | Right-center of screen |
| `Center` | 8 | Center of screen |
| `TrayLeft` | 9 | Left-aligned with tray icon |
| `TrayBottomLeft` | 10 | Below tray, left-aligned |
| `TrayRight` | 11 | Right-aligned with tray icon |
| `TrayBottomRight` | 12 | Below tray, right-aligned |
| `TrayCenter` | 13 | Centered on tray icon |
| `TrayBottomCenter` | 14 | Below tray, centered |

### 2.5 JavaScript API

```typescript
import {
    moveWindow,
    moveWindowConstrained,
    handleIconState,
    Position,
} from '@tauri-apps/plugin-positioner';

// Position window centered below tray icon
await moveWindow(Position.TrayBottomCenter);

// Same but constrained to screen boundaries (prevents window going off-screen)
await moveWindowConstrained(Position.TrayCenter);

// Handle tray icon state (call from tray event handler)
// This is the JS equivalent of on_tray_event
await handleIconState(event);
```

### 2.6 Rust API

```rust
use tauri_plugin_positioner::{WindowExt, Position};

// Get the window and move it
let window = app.get_webview_window("main").unwrap();
window.as_ref().window().move_window(Position::TrayCenter)?;

// Constrained version
window.as_ref().window().move_window_constrained(Position::TrayCenter)?;
```

### 2.7 Platform Behavior for Tray Positions

| Platform | Tray Location | Window Appears |
|----------|---------------|----------------|
| **macOS** | Top menu bar (right side) | Below the tray icon, hanging down |
| **Windows** | Bottom taskbar (right side, system tray) | Above the tray icon, popping up |
| **Linux** | Varies by DE | Varies; click events may not work |

The `moveWindowConstrained` function is preferred for tray positions because it clamps the window to screen bounds, preventing it from going off-screen on edge cases (tray icon at far right of screen, etc.).

### 2.8 Known Limitations

- **Port of electron-positioner** -- behavior mirrors Electron's positioner module.
- **Desktop only** -- no mobile support.
- **Requires `tray-icon` feature** -- tray positions (`TrayLeft`, `TrayCenter`, etc.) will not work without enabling the feature and forwarding tray events.
- **No custom offset API** -- you cannot add pixel offsets to positions. If you need a custom offset, you must calculate and set the window position manually using the `rect` from `TrayIconEvent`.
- **Minimum Rust 1.77.2**.

---

## 3. Window State Plugin

Persists and restores window position, size, and display state across app launches.

**Source:** https://v2.tauri.app/plugin/window-state/

### 3.1 Installation

**Cargo crate:** `tauri-plugin-window-state`
**npm package:** `@tauri-apps/plugin-window-state`

```bash
cargo add tauri-plugin-window-state --target 'cfg(any(target_os = "macos", windows, target_os = "linux"))'
npm install @tauri-apps/plugin-window-state
```

### 3.2 Rust Setup

```rust
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

Once registered, **all windows automatically save their state on close and restore on next launch**. No additional code needed for basic use.

### 3.3 Capabilities / Permissions

```json
{
  "permissions": ["window-state:default"]
}
```

### 3.4 StateFlags

Controls which properties are saved/restored:

```rust
use tauri_plugin_window_state::StateFlags;

StateFlags::all()        // Everything
StateFlags::POSITION     // x, y
StateFlags::SIZE         // width, height
StateFlags::MAXIMIZED    // maximized state
StateFlags::VISIBLE      // visible state
StateFlags::DECORATIONS  // window decorations
StateFlags::FULLSCREEN   // fullscreen state
```

### 3.5 JavaScript API

```typescript
import {
    saveWindowState,
    restoreStateCurrent,
    StateFlags,
} from '@tauri-apps/plugin-window-state';

// Manual save
await saveWindowState(StateFlags.ALL);

// Manual restore
await restoreStateCurrent(StateFlags.ALL);
```

### 3.6 Rust API

```rust
use tauri_plugin_window_state::{AppHandleExt, WindowExt, StateFlags};

// Save all window states
app.save_window_state(StateFlags::all())?;

// Restore a specific window
window.restore_state(StateFlags::all())?;
```

### 3.7 Preventing Flash on Launch

Set `visible: false` in `tauri.conf.json` for the window, then manually restore and show:

```json
{
  "windows": [
    {
      "label": "main",
      "visible": false
    }
  ]
}
```

```rust
// After plugin restores state, make window visible
if let Some(window) = app.get_webview_window("main") {
    window.restore_state(StateFlags::all())?;
    window.show()?;
}
```

### 3.8 Known Limitations

- **Desktop only** (macOS, Windows, Linux).
- State is stored per window label in a local file.
- For a tray-only app where the window position is always computed from tray icon position, this plugin may conflict. Consider whether you want remembered positions or always-computed positions.

---

## 4. Autostart Plugin

Launch the application at system login/boot.

**Source:** https://v2.tauri.app/plugin/autostart/

### 4.1 Installation

**Cargo crate:** `tauri-plugin-autostart`
**npm package:** `@tauri-apps/plugin-autostart`

```bash
cargo add tauri-plugin-autostart --target 'cfg(any(target_os = "macos", windows, target_os = "linux"))'
npm install @tauri-apps/plugin-autostart
```

### 4.2 Rust Setup

```rust
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),  // optional CLI args passed on launch
        ))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

#### MacosLauncher Enum

| Variant | Mechanism | Notes |
|---------|-----------|-------|
| `MacosLauncher::LaunchAgent` | Creates a LaunchAgent plist in `~/Library/LaunchAgents/` | Preferred; more reliable |
| `MacosLauncher::AppleScript` | Uses AppleScript-based login item | Legacy approach |

The second argument is `Option<Vec<&str>>` for command-line arguments passed to the app when auto-started. Useful for passing `--minimized` to start hidden.

### 4.3 Capabilities / Permissions

```json
{
  "permissions": [
    "autostart:allow-enable",
    "autostart:allow-disable",
    "autostart:allow-is-enabled"
  ]
}
```

### 4.4 JavaScript API

```typescript
import { enable, disable, isEnabled } from '@tauri-apps/plugin-autostart';

// Enable auto-start
await enable();

// Check status
const enabled = await isEnabled();
console.log('Autostart enabled:', enabled);

// Disable
await disable();
```

### 4.5 Platform Support

- **macOS:** LaunchAgent (recommended) or AppleScript
- **Windows:** Registry-based (`HKCU\Software\Microsoft\Windows\CurrentVersion\Run`)
- **Linux:** XDG autostart (`.desktop` file in `~/.config/autostart/`)

### 4.6 Known Limitations

- Desktop only.
- On macOS, the LaunchAgent approach is more reliable than AppleScript.
- The auto-started app receives the CLI args specified in `init()`, not the args from the last manual launch.

---

## 5. Notification Plugin

Native OS notifications. Included for completeness; may not be needed for a tray widget.

**Source:** https://v2.tauri.app/plugin/notification/

### 5.1 Installation

**Cargo crate:** `tauri-plugin-notification`
**npm package:** `@tauri-apps/plugin-notification`

```bash
cargo add tauri-plugin-notification
npm install @tauri-apps/plugin-notification
```

### 5.2 Rust Setup

```rust
.plugin(tauri_plugin_notification::init())
```

### 5.3 JavaScript API

```typescript
import {
    isPermissionGranted,
    requestPermission,
    sendNotification,
} from '@tauri-apps/plugin-notification';

// Check + request permission
let granted = await isPermissionGranted();
if (!granted) {
    const permission = await requestPermission();
    granted = permission === 'granted';
}

// Send
if (granted) {
    sendNotification({
        title: 'Agent Activity',
        body: 'Agent "GPT-5" requested $50.00 spend approval',
    });
}
```

### 5.4 Capabilities / Permissions

```json
{
  "permissions": [
    "notification:default",
    "notification:allow-is-permission-granted",
    "notification:allow-request-permission",
    "notification:allow-notify"
  ]
}
```

### 5.5 Platform Notes

- **Windows:** In development mode, notifications appear branded as "PowerShell" because the app is not installed. Only installed apps show proper branding.
- **macOS / Linux:** Work as expected.
- **Mobile:** Full support with additional features (actions, channels, push).

---

## 6. Summary: Package Names and Versions

| Feature | Cargo Crate | npm Package | Feature Flag |
|---------|-------------|-------------|-------------|
| System Tray | Built into `tauri` | Built into `@tauri-apps/api` | `tray-icon` in `tauri` |
| Positioner | `tauri-plugin-positioner` | `@tauri-apps/plugin-positioner` | `tray-icon` for tray positions |
| Window State | `tauri-plugin-window-state` | `@tauri-apps/plugin-window-state` | None |
| Autostart | `tauri-plugin-autostart` | `@tauri-apps/plugin-autostart` | None |
| Notification | `tauri-plugin-notification` | `@tauri-apps/plugin-notification` | None |

All plugins require **Rust >= 1.77.2** and **Tauri v2**.

---

## 7. Platform Support Matrix

| Feature | macOS | Windows | Linux |
|---------|-------|---------|-------|
| Tray icon display | Yes | Yes | Yes (may need empty menu) |
| Tray click events | Yes | Yes | No (right-click menu works) |
| Tray DoubleClick | No | Yes | No |
| Tray tooltip | Yes | Yes | No |
| Tray title | Yes | No | Yes |
| Tray template icon | Yes | N/A | N/A |
| Positioner (screen) | Yes | Yes | Yes |
| Positioner (tray) | Yes | Yes | Partial (no click events) |
| Window State | Yes | Yes | Yes |
| Autostart | Yes (LaunchAgent) | Yes (Registry) | Yes (XDG) |
| Notification | Yes | Yes (dev branding issue) | Yes |

---

## 8. Integration Recipe: Tray App with Positioned Window

This is the combined setup for a tray-anchored floating window app (our target architecture).

### Cargo.toml

```toml
[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-positioner = { version = "2", features = ["tray-icon"] }
tauri-plugin-autostart = "2"
# tauri-plugin-window-state = "2"  # Optional; may conflict with tray positioning
```

### lib.rs

```rust
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};
use tauri_plugin_positioner::{Position, WindowExt};

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .setup(|app| {
            // Build tray menu
            let show = MenuItemBuilder::with_id("show", "Show Wallet").build(app)?;
            let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
            let menu = MenuBuilder::new(app).items(&[&show, &quit]).build()?;

            // Build tray icon
            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("Tally Wallet")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "show" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    // Forward to positioner plugin (REQUIRED for tray positions)
                    tauri_plugin_positioner::on_tray_event(tray.app_handle(), &event);

                    // Handle left-click: position window near tray and show
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            // Position centered below tray icon
                            let _ = window
                                .as_ref()
                                .window()
                                .move_window(Position::TrayBottomCenter);
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### tauri.conf.json (window config for tray app)

```json
{
  "windows": [
    {
      "label": "main",
      "title": "Tally Wallet",
      "width": 380,
      "height": 600,
      "visible": false,
      "skipTaskbar": true,
      "decorations": false,
      "alwaysOnTop": true,
      "resizable": false
    }
  ]
}
```

Key window properties for a tray widget:
- `visible: false` -- start hidden, show on tray click
- `skipTaskbar: true` -- do not show in taskbar/dock
- `decorations: false` -- no title bar
- `alwaysOnTop: true` -- float above other windows
- `resizable: false` -- fixed widget size

### capabilities/default.json

```json
{
  "identifier": "default",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "core:tray:default",
    "core:menu:default",
    "positioner:default",
    "autostart:allow-enable",
    "autostart:allow-disable",
    "autostart:allow-is-enabled"
  ]
}
```

### Frontend: Toggle Window on Tray Click

If you want to handle positioning from the frontend instead of Rust:

```typescript
import { getCurrentWindow } from '@tauri-apps/api/window';
import { moveWindow, Position } from '@tauri-apps/plugin-positioner';

export async function toggleWindow() {
    const window = getCurrentWindow();
    const visible = await window.isVisible();

    if (visible) {
        await window.hide();
    } else {
        await moveWindow(Position.TrayBottomCenter);
        await window.show();
        await window.setFocus();
    }
}
```

---

## 9. Community Plugins and Native Code Notes

### No Community Plugins Required

All functionality needed for a tray-based floating widget is available through official Tauri v2 APIs and plugins:
- Tray icon: built-in
- Window positioning near tray: `tauri-plugin-positioner`
- Auto-start: `tauri-plugin-autostart`
- Window state persistence: `tauri-plugin-window-state`

### Native OS Code Considerations

For most use cases, **no native OS code is needed**. However, some edge cases may require platform-specific code:

| Scenario | Requires Native Code? | Details |
|----------|-----------------------|---------|
| Basic tray icon + menu | No | Built-in Tauri API |
| Position window near tray | No | Positioner plugin |
| Auto-dismiss on click-away (blur) | No | Use Tauri's `on_window_event` for `Focused(false)` |
| Rounded corners on window | No | CSS `border-radius` + transparent window (`transparent: true` in config) |
| macOS vibrancy/blur behind window | Possibly | May need `NSVisualEffectView` via Cocoa APIs (not built into Tauri) |
| Custom tray icon animations | Yes | Requires rapid `set_icon()` calls; no built-in animation API |
| Panel-style window (macOS) | Yes | `NSPanel` behavior (auto-hide, no dock icon) requires Cocoa APIs or `cocoa` crate |
| Click-through regions | Possibly | `setIgnoreMouseEvents` may need platform-specific handling |

For a **macOS NSPanel-style tray app** (the gold standard for status bar apps), the `tauri-nspanel` community crate exists but is not officially maintained. The built-in approach (hide on blur, `alwaysOnTop`, `skipTaskbar`) gets close to NSPanel behavior without native code.
