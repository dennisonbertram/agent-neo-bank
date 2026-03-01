# Status Bar (Menu Bar) App Research for Tauri v2 on macOS

> **Date**: 2026-02-28
> **Goal**: Determine how to convert Tally Agentic Wallet into a macOS status bar (menu bar) app that runs in the background, hides from the Dock, and supports native notifications.

---

## Table of Contents

1. [System Tray / Status Bar Icon](#1-system-tray--status-bar-icon)
2. [Hiding from the Dock and Cmd+Tab](#2-hiding-from-the-dock-and-cmdtab)
3. [Background Running (Close-to-Tray)](#3-background-running-close-to-tray)
4. [Window Positioning with Positioner Plugin](#4-window-positioning-with-positioner-plugin)
5. [Native macOS Notifications](#5-native-macos-notifications)
6. [Configuration Changes Required](#6-configuration-changes-required)
7. [Gotchas and Limitations](#7-gotchas-and-limitations)
8. [Current Project State](#8-current-project-state)
9. [Implementation Plan](#9-implementation-plan)
10. [Sources](#10-sources)

---

## 1. System Tray / Status Bar Icon

Tauri v2 has **built-in system tray support** via the `tray-icon` feature (no separate plugin needed). The project already has this feature enabled in `Cargo.toml`.

### Cargo.toml (already configured)

```toml
tauri = { version = "2", features = ["tray-icon"] }
```

### Creating the Tray Icon (Rust)

```rust
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager,
};

tauri::Builder::default()
    .setup(|app| {
        // Build menu items
        let show_i = MenuItem::with_id(app, "show", "Show Window", true, None::<&str>)?;
        let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
        let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

        // Build tray icon
        let _tray = TrayIconBuilder::new()
            .icon(app.default_window_icon().unwrap().clone())
            .menu(&menu)
            .menu_on_left_click(false) // left click toggles window, right click shows menu
            .on_menu_event(|app, event| match event.id.as_ref() {
                "show" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.unminimize();
                        let _ = window.set_focus();
                    }
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            })
            .on_tray_icon_event(|tray, event| {
                use tauri::tray::{MouseButton, MouseButtonState, TrayIconEvent};
                if let TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                } = event
                {
                    let app = tray.app_handle();
                    if let Some(window) = app.get_webview_window("main") {
                        if window.is_visible().unwrap_or(false) {
                            let _ = window.hide();
                        } else {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }
                }
            })
            .build(app)?;

        Ok(())
    })
```

### Creating the Tray Icon (JavaScript alternative)

```javascript
import { TrayIcon } from '@tauri-apps/api/tray';
import { Menu } from '@tauri-apps/api/menu';
import { defaultWindowIcon } from '@tauri-apps/api/app';

const menu = await Menu.new({
  items: [
    { id: 'show', text: 'Show Window', action: () => { /* show window */ } },
    { id: 'quit', text: 'Quit', action: () => { /* exit app */ } },
  ],
});

const tray = await TrayIcon.new({
  icon: await defaultWindowIcon(),
  menu,
  menuOnLeftClick: false,
  action: (event) => {
    if (event.type === 'Click' && event.button === 'Left') {
      // Toggle window visibility
    }
  },
});
```

### Template Images (macOS)

On macOS, tray icons should be **template images** (monochrome, 22x22 points). Tauri supports setting the icon as a template via `TrayIconBuilder`:

```rust
TrayIconBuilder::new()
    .icon(app.default_window_icon().unwrap().clone())
    .icon_as_template(true) // macOS only: treats icon as template image
    .build(app)?;
```

Template images automatically adapt to light/dark menu bar themes.

### Tray Icon Events

Available event types:
- `Click` (with button type: Left/Right/Middle, and button state: Up/Down)
- `DoubleClick`
- `Enter` (mouse hover)
- `Move` (mouse move over icon)
- `Leave` (mouse leaves icon area)

**Note**: On Linux, tray events are not supported; only right-click context menus work.

---

## 2. Hiding from the Dock and Cmd+Tab

### `set_activation_policy` API

Tauri v2 exposes `set_activation_policy()` on the app handle, which maps to macOS's `NSApplicationActivationPolicy`. Three policies are available:

| Policy | Dock Icon | Cmd+Tab | Menu Bar |
|--------|-----------|---------|----------|
| `Regular` (default) | Yes | Yes | Yes |
| `Accessory` | No | No | No (unless window focused) |
| `Prohibited` | No | No | No |

For a status bar app, use **`Accessory`**:

```rust
use tauri::ActivationPolicy;

tauri::Builder::default()
    .setup(|app| {
        #[cfg(target_os = "macos")]
        app.set_activation_policy(ActivationPolicy::Accessory);

        // ... rest of setup
        Ok(())
    })
```

### Dynamic Toggle (Show in Dock When Window Opens)

You can switch the policy at runtime to show/hide the Dock icon dynamically:

```rust
use tauri::{ActivationPolicy, Manager, WindowEvent};

// In setup, after creating the main window:
if let Some(main_win) = app.get_webview_window("main") {
    let app_handle = app.handle().clone();
    let win_clone = main_win.clone();
    main_win.on_window_event(move |event| {
        #[cfg(target_os = "macos")]
        match event {
            WindowEvent::CloseRequested { api, .. } => {
                api.prevent_close();
                let _ = win_clone.hide();
                let _ = app_handle.set_activation_policy(ActivationPolicy::Accessory);
            }
            WindowEvent::Focused(true) => {
                // Optionally show in Dock when window is focused
                // let _ = app_handle.set_activation_policy(ActivationPolicy::Regular);
            }
            _ => {}
        }
    });
}
```

### Alternative: `LSUIElement` in Info.plist

You can permanently hide the app from the Dock by setting `LSUIElement` to `true` in the macOS bundle's `Info.plist`. However, this is **static** (cannot be toggled at runtime) and is generally less flexible than `set_activation_policy`.

In Tauri, you would add this to the Info.plist template or via `tauri.conf.json` bundle config. The `set_activation_policy` approach is preferred because it allows runtime toggling.

---

## 3. Background Running (Close-to-Tray)

By default, Tauri exits when all windows are closed. To keep the app running in the background:

### Approach 1: Hide Window on Close (Recommended)

Intercept the `CloseRequested` event, prevent the close, and hide the window instead:

```rust
tauri::Builder::default()
    .setup(|app| {
        if let Some(main_win) = app.get_webview_window("main") {
            let win_clone = main_win.clone();
            main_win.on_window_event(move |event| {
                if let WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = win_clone.hide();
                }
            });
        }
        Ok(())
    })
```

This is the cleanest approach: the window is never destroyed, so no exit event fires.

### Approach 2: `prevent_exit()` on `RunEvent::ExitRequested`

```rust
tauri::Builder::default()
    // ... setup, invoke_handler, etc.
    .build(tauri::generate_context!())
    .expect("error while building tauri app")
    .run(|app, event| {
        if let tauri::RunEvent::ExitRequested { api, code, .. } = &event {
            if code.is_none() {
                // Only prevent exit when it's from window close, not explicit exit
                api.prevent_exit();
            }
        }
    });
```

**Warning**: This approach has a known issue on macOS where `prevent_exit()` can prevent normal process termination, requiring force-kill. It also conflicts with `tauri-plugin-window-state` (causes an infinite loop of `windowDidMove` events).

### Recommended Pattern

Use **Approach 1** (hide on close). Combined with a "Quit" option in the tray menu that calls `app.exit(0)`, this gives users a clean way to actually quit.

---

## 4. Window Positioning with Positioner Plugin

The `tauri-plugin-positioner` plugin positions the window relative to the tray icon, which is essential for a menu bar app UX.

### Installation

```bash
# In project root
npm run tauri add positioner
```

Or manually:

**Cargo.toml** (with tray-icon feature):
```toml
tauri-plugin-positioner = { version = "2", features = ["tray-icon"] }
```

**npm**:
```bash
npm install @tauri-apps/plugin-positioner
```

### Rust Setup

```rust
use tauri_plugin_positioner::{Position, WindowExt};

tauri::Builder::default()
    .plugin(tauri_plugin_positioner::init())
    .setup(|app| {
        // Build tray with positioner integration
        let _tray = TrayIconBuilder::new()
            .icon(app.default_window_icon().unwrap().clone())
            .on_tray_icon_event(|tray, event| {
                // Forward tray events to positioner
                tauri_plugin_positioner::on_tray_event(tray.app_handle(), &event);

                use tauri::tray::{MouseButton, MouseButtonState, TrayIconEvent};
                if let TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                } = event
                {
                    let app = tray.app_handle();
                    if let Some(window) = app.get_webview_window("main") {
                        if window.is_visible().unwrap_or(false) {
                            let _ = window.hide();
                        } else {
                            // Position window under tray icon
                            let _ = window.as_ref().window().move_window(Position::TrayCenter);
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                }
            })
            .build(app)?;

        Ok(())
    })
```

### JavaScript Usage

```javascript
import { moveWindow, Position } from '@tauri-apps/plugin-positioner';

// Position window relative to tray icon
await moveWindow(Position.TrayCenter);
// Other options: Position.TrayLeft, Position.TrayRight, Position.TrayBottomCenter
```

### Capabilities

Add to `src-tauri/capabilities/default.json`:
```json
{
  "permissions": [
    "positioner:default"
  ]
}
```

### Available Positions

- `TrayLeft` - Aligned to the left edge of the tray icon
- `TrayCenter` - Centered under the tray icon
- `TrayRight` - Aligned to the right edge of the tray icon
- `TrayBottomCenter` - Centered below the tray icon
- `TopRight`, `TopLeft`, `BottomRight`, `BottomLeft`, `Center` (screen-relative)

---

## 5. Native macOS Notifications

The project already has `tauri-plugin-notification` installed and configured.

### Current State (already in place)

**Cargo.toml**: `tauri-plugin-notification = "2"` (already present)

**lib.rs**: `.plugin(tauri_plugin_notification::init())` (already present)

**capabilities/default.json**: `"notification:default"` (already present)

### JavaScript Usage

```javascript
import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from '@tauri-apps/plugin-notification';

// Check and request permission
let permissionGranted = await isPermissionGranted();
if (!permissionGranted) {
  const permission = await requestPermission();
  permissionGranted = permission === 'granted';
}

// Send notification
if (permissionGranted) {
  sendNotification({
    title: 'Tally Agentic Wallet',
    body: 'Agent transaction approved',
    sound: 'Ping', // macOS system sound
  });
}
```

### macOS-Specific Notes

- Notifications use the native macOS Notification Center
- Sound can be any macOS system sound name (e.g., `'Ping'`, `'Basso'`, `'Glass'`)
- The app must be properly signed for notifications to work in production builds
- On macOS, notifications are fully supported with banners and alerts
- **Important**: Only functional for installed apps on Windows, but works in dev mode on macOS

---

## 6. Configuration Changes Required

### `tauri.conf.json` Changes

The window configuration should be updated for a status bar app:

```json
{
  "app": {
    "windows": [
      {
        "title": "Tally Agentic Wallet",
        "width": 390,
        "height": 844,
        "resizable": false,
        "fullscreen": false,
        "decorations": false,
        "transparent": false,
        "center": false,
        "visible": false,
        "skipTaskbar": true,
        "alwaysOnTop": true
      }
    ]
  }
}
```

Key changes:
- `"visible": false` - Start hidden (tray click shows it)
- `"skipTaskbar": true` - Hide from Windows/Linux taskbar
- `"decorations": false` - No title bar (menu bar popup style)
- `"alwaysOnTop": true` - Keep popup above other windows
- `"center": false` - Position will be set by positioner plugin

### `Cargo.toml` Additions

```toml
tauri-plugin-positioner = { version = "2", features = ["tray-icon"] }
```

### Capabilities (`default.json`) Addition

```json
{
  "permissions": [
    "core:default",
    "notification:default",
    "clipboard-manager:default",
    "positioner:default"
  ]
}
```

### Tray Icon Asset

Add a tray icon image at `src-tauri/icons/tray-icon.png`:
- Size: 22x22 pixels (44x44 for @2x Retina)
- Format: PNG with transparency
- Style: Monochrome (template image) for macOS menu bar consistency

---

## 7. Gotchas and Limitations

### Known Issues

1. **`tauri-plugin-window-state` conflict**: Using the window-state plugin with `prevent_exit()` causes an infinite loop of `windowDidMove` events on macOS. **Workaround**: Do not use window-state plugin with close-to-tray, or disable it conditionally.

2. **`prevent_exit()` blocks graceful shutdown**: Calling `api.prevent_exit()` on `RunEvent::ExitRequested` prevents normal process termination. Users must force-kill via Activity Monitor. **Workaround**: Use the hide-on-close approach instead, and provide an explicit "Quit" menu item in the tray.

3. **`set_activation_policy` + `window.show()` bug**: There's a known issue (tauri-apps/tauri#5122) where calling `set_activation_policy(Accessory)` and then `window.show()` can fail to bring the window to the foreground. **Workaround**: Call `set_activation_policy(Regular)` before showing the window, then switch back to `Accessory` when hiding.

4. **macOS menu bar quirks**: When using submenus on macOS, all items must be grouped under a submenu. Top-level items are ignored, and the first submenu is placed under the application's "About" menu by default.

5. **Close-all-windows exit loop (Tauri v2)**: In Tauri v2, closing all webview windows emits `ExitRequested`, which if prevented, can create a loop. The recommended approach is to **never close** the window -- only hide it.

6. **Focus loss behavior**: For a true menu bar popup, you may want to hide the window when it loses focus. Handle `WindowEvent::Focused(false)`:

   ```rust
   WindowEvent::Focused(false) => {
       let _ = win_clone.hide();
       #[cfg(target_os = "macos")]
       let _ = app_handle.set_activation_policy(ActivationPolicy::Accessory);
   }
   ```

7. **Feature request pending**: There is an open feature request (tauri-apps/tauri#13511) for a proper "prevent exit when all windows close" API that distinguishes between user-initiated exit and window-close-triggered exit. As of February 2026, this is still open.

### Platform Considerations

- **Linux**: Tray icon events (click, hover) are not supported on Linux. Only right-click context menus work. The tray icon itself is displayed.
- **Windows**: Use `skipTaskbar: true` in window config instead of activation policy. The `set_activation_policy` API is macOS-only.
- **Signing**: On macOS, notifications require proper code signing for production. Development builds work without signing.

---

## 8. Current Project State

The project is already partially configured for tray support:

| Feature | Status |
|---------|--------|
| `tray-icon` feature in Cargo.toml | Already enabled |
| `tauri-plugin-notification` | Already installed and configured |
| `tauri-plugin-positioner` | **Not installed** - needs to be added |
| Tray icon creation in `lib.rs` | **Not implemented** - needs to be added in `.setup()` |
| `set_activation_policy` call | **Not implemented** |
| Close-to-tray (hide on close) | **Not implemented** |
| Tray icon asset | **Not created** - needs a 22x22 template PNG |
| Window config for popup style | **Not configured** - current config is for a regular window |

---

## 9. Implementation Plan

### Step 1: Add Dependencies

Add `tauri-plugin-positioner` to `Cargo.toml` and `package.json`.

### Step 2: Create Tray Icon Asset

Create a 22x22 (and 44x44 @2x) monochrome PNG icon for the menu bar.

### Step 3: Update `lib.rs`

In the `.setup()` closure:
1. Set activation policy to `Accessory`
2. Create tray icon with menu (Show, Quit)
3. Handle tray click to toggle window visibility
4. Handle close-requested to hide instead of close
5. Initialize positioner plugin and position window on tray click

### Step 4: Update `tauri.conf.json`

Change window config: `visible: false`, `decorations: false`, `skipTaskbar: true`, `alwaysOnTop: true`.

### Step 5: Update Capabilities

Add `"positioner:default"` to `default.json`.

### Step 6: Frontend Adjustments

- Handle the borderless window (add custom drag region if needed)
- Add a close/hide button since there are no native window controls
- Consider adding a small arrow/triangle pointing up toward the tray icon

### Complete Rust Implementation Sketch

```rust
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, WindowEvent,
};

#[cfg(target_os = "macos")]
use tauri::ActivationPolicy;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_positioner::init())
        .setup(|app| {
            // Hide from Dock on macOS
            #[cfg(target_os = "macos")]
            app.set_activation_policy(ActivationPolicy::Accessory);

            // Create tray menu
            let show_i = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

            // Create tray icon
            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .icon_as_template(true)
                .menu(&menu)
                .menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    tauri_plugin_positioner::on_tray_event(tray.app_handle(), &event);

                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                            } else {
                                use tauri_plugin_positioner::{Position, WindowExt};
                                let _ = window.as_ref().window().move_window(Position::TrayCenter);
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;

            // Hide window on close instead of quitting
            if let Some(main_win) = app.get_webview_window("main") {
                let win_clone = main_win.clone();
                main_win.on_window_event(move |event| {
                    if let WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = win_clone.hide();
                    }
                });
            }

            // ... existing AppState setup, config, etc.

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // ... existing commands
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

---

## 10. Sources

- [Tauri v2 System Tray Guide](https://v2.tauri.app/learn/system-tray/) - Official documentation
- [Tauri v2 Tray JavaScript API Reference](https://v2.tauri.app/reference/javascript/api/namespacetray/)
- [Tauri v2 Notification Plugin](https://v2.tauri.app/plugin/notification/) - Official documentation
- [Tauri v2 Positioner Plugin](https://v2.tauri.app/plugin/positioner/) - Official documentation
- [Tauri v2 Configuration Reference](https://v2.tauri.app/reference/config/)
- [Discussion #6038: Hide app icon from dock on macOS](https://github.com/tauri-apps/tauri/discussions/6038)
- [Discussion #10774: Toggle dock icon on macOS](https://github.com/tauri-apps/tauri/discussions/10774)
- [Discussion #11489: System tray only app in Tauri v2](https://github.com/tauri-apps/tauri/discussions/11489)
- [Issue #13511: Prevent exit when all windows closed](https://github.com/tauri-apps/tauri/issues/13511)
- [Issue #5122: set_activation_policy breaks window.show()](https://github.com/tauri-apps/tauri/issues/5122)
- [Issue #2258: Expose set_activation_policy](https://github.com/tauri-apps/tauri/issues/2258)
- [Building a system tray app with Tauri (tutorial)](https://tauritutorials.com/blog/building-a-system-tray-app-with-tauri)
- [ahkohd/tauri-macos-menubar-app-example](https://github.com/ahkohd/tauri-macos-menubar-app-example)
- [tauri-apps/tray-icon (underlying crate)](https://github.com/tauri-apps/tray-icon)
