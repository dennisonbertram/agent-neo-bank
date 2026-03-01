# Cross-Platform Limitations & Gotchas: Tauri v2 Floating Widget / Tray App

> Research date: 2026-02-28
> Sources: Tauri v2 official docs, GitHub issues, community discussions, WebView2 feedback

---

## 1. Transparent Windows

### macOS

**Basic transparency works**, but there are significant caveats:

- **`transparent: true` + `decorations: false`** is the standard approach. Requires `html, body { background: transparent }` in CSS.
- **`macOSPrivateApi: true`** is required in `tauri.conf.json` for full transparency support (enables `NSVisualEffectView` and related private APIs).
- **Shadows are always disabled for transparent windows on macOS.** This is by design to avoid shadow artifacts ([#5494](https://github.com/tauri-apps/tauri/issues/5494)).
- **DMG build transparency loss**: Transparent windows may render as solid white after bundling into a DMG, even though they work in `tauri dev`. Reported in [#13415](https://github.com/tauri-apps/tauri/issues/13415) (May 2025). No confirmed fix as of writing. Workaround: test with the built artifact early.
- **Focus change glitch on Sonoma**: Transparent windows exhibit visual glitches after focus changes on macOS Sonoma 14 ([#8255](https://github.com/tauri-apps/tauri/issues/8255)).
- **Vibrancy/blur effects**: Use the `window-vibrancy` crate (now integrated into Tauri v2). Call `apply_vibrancy` with an `NSVisualEffectMaterial` variant (e.g., `HudWindow`, `Sidebar`). Requires `macOSPrivateApi: true`.

### Windows

- **Basic transparency works** with `transparent: true` + `decorations: false`.
- **WebView2 transparency quirk**: Setting `DefaultBackgroundColor` to transparent may not yield true transparency; it can render as the theme's background brush instead ([WebView2 #6527](https://github.com/microsoft/microsoft-ui-xaml/issues/6527), [WebView2 #2419](https://github.com/MicrosoftEdge/WebView2Feedback/issues/2419)).
- **`backdrop-filter: blur()` does not work** when window transparency is enabled ([#10064](https://github.com/tauri-apps/tauri/issues/10064)). CSS blur and transparent windows are mutually exclusive on Windows.
- **Acrylic/Mica effects** (native Windows blur): Use `apply_acrylic` (Windows 10/11) or `apply_mica` (Windows 11 only) from the vibrancy integration. These work as native DWM effects, NOT CSS blur.
  - **Performance issue**: Bad resize/drag performance on Windows 11 build 22621+ when acrylic is enabled.
  - `apply_blur` is available for Windows 7/10/11 as a simpler alternative.
- **Transparent v2 regression**: The same `transparent: true` config that worked in Tauri v1 may not work in v2 on Windows ([#8308](https://github.com/tauri-apps/tauri/issues/8308)).

### Linux

- **X11**: Transparency works if a compositor is running (e.g., picom, compton, or a compositing WM like KWin/Mutter). Without a compositor, the transparent regions will be black or garbage pixels. Most modern desktop environments (GNOME, KDE, XFCE with compositor enabled) support this.
- **Wayland**: Transparency generally works since all Wayland compositors are compositing by nature. However:
  - **Rendering ghost artifacts**: On older webkit2gtk versions (< 2.48.0), transparent windows may fail to update properly, showing "ghost" renders of previous content until the window is resized ([#12800](https://github.com/tauri-apps/tauri/issues/12800)). **Fix**: Require webkit2gtk >= 2.48.0.
  - **Window properties ignored**: `visible`, `decorations`, and other properties may be ignored on some Linux DEs, especially KDE on Wayland ([#6162](https://github.com/tauri-apps/tauri/issues/6162)).
- **Vibrancy/blur effects**: Not supported on Linux. Blur and vibrancy are controlled entirely by the compositor; Tauri has no API to request them.
- **Glitchy rendering**: Maximize/unmaximize cycles can cause "shadow" copies of DOM elements on Linux ([#13157](https://github.com/tauri-apps/tauri/issues/13157)).

---

## 2. Always-on-Top Behavior

### API

Tauri v2 provides `alwaysOnTop: true` in config and `window.setAlwaysOnTop(true)` at runtime.

### macOS

- **Works for normal windows** but **does NOT appear above full-screen apps** by default. Full-screen apps on macOS use a separate Space, and `alwaysOnTop` does not cross Space boundaries.
- **Workaround to appear over full-screen apps**: Use `ActivationPolicy::Accessory` + convert window to an `NSPanel` with `NSWindowCollectionBehavior::CanJoinAllSpaces` and an appropriate window level. The community [`tauri-nspanel`](https://github.com/ahkohd/tauri-nspanel) plugin (v2 branch) implements this, but it causes the Dock icon to disappear (since `Accessory` hides the app from the Dock). There is minor Dock icon blinking if you toggle policies dynamically ([#11488](https://github.com/tauri-apps/tauri/issues/11488)).
- **No user permission required** for always-on-top itself. Screen Recording permission is NOT needed unless you are capturing screen content.

### Windows

- **Works reliably** for normal desktop windows.
- **Full-screen apps**: `alwaysOnTop` windows are typically hidden when another app goes truly full-screen (exclusive fullscreen). For borderless-fullscreen (windowed-fullscreen) games/apps, the overlay will appear.
- **No special permissions required.**

### Linux

- **`alwaysOnTop` is documented as unsupported on Linux** in the Tauri API reference. Behavior depends entirely on the window manager/compositor. GNOME and KDE may or may not honor the hint. Tiling WMs often ignore it entirely.
- **No user permission required**, but behavior is unreliable.

---

## 3. Focus Behavior (Show Without Stealing Focus)

### macOS

- **`focusable: false` is broken on macOS**: The window still steals focus when clicked ([#14102](https://github.com/tauri-apps/tauri/issues/14102), open as of August 2025).
- **Workaround**: Use [`tauri-nspanel`](https://github.com/ahkohd/tauri-nspanel) (v2 branch) to convert the window to an `NSPanel` with `NSWindowStyleMaskNonActivatingPanel`. This creates a panel that can be shown/hidden without stealing focus from the previously focused application. This is how macOS Spotlight, Alfred, and Raycast work.
- **Alternative workaround**: Set `ActivationPolicy::Accessory` before entering the event loop. This prevents focus stealing but also hides the app from the Dock and Cmd+Tab switcher.
- **`window.setFocus()`** may not work reliably after upgrading to Tauri 2.3+ ([#12834](https://github.com/tauri-apps/tauri/issues/12834)).

### Windows

- **`focus: false`** in window config is ignored. The default focus state of all windows is `true`, which overrides any `focus: false` setting ([#7519](https://github.com/tauri-apps/tauri/issues/7519)).
- **No built-in "no focus" window type** in Tauri for Windows. The Windows API supports `WS_EX_NOACTIVATE` extended style, but Tauri does not expose this.
- **Workaround**: Use native Windows API calls (via `windows-rs` or `winapi` crate) to set `WS_EX_NOACTIVATE` on the HWND after window creation.

### Linux

- **`focus: false`** behavior varies by WM/compositor. On GNOME Wayland, windows may not be properly focused on creation ([#10746](https://github.com/tauri-apps/tauri/issues/10746)), which is ironically helpful for "no focus" scenarios but is a bug, not a feature.
- **X11**: Some WMs support `_NET_WM_STATE_SKIP_PAGER` and related hints, but Tauri does not expose these.
- **Wayland**: Focus behavior is compositor-controlled. Applications cannot prevent focus assignment on most Wayland compositors (this is by design in the Wayland security model).

---

## 4. Tray Icon Positioning

### Plugin: `tauri-plugin-positioner`

The official [Positioner plugin](https://v2.tauri.app/plugin/positioner/) provides pre-defined window positions including tray-relative positions (`TrayLeft`, `TrayRight`, `TrayCenter`, `TrayBottomLeft`, `TrayBottomCenter`, `TrayBottomRight`).

**Setup requirements:**
1. Add `tray-icon` feature flag to `tauri-plugin-positioner` in `Cargo.toml`
2. Register `on_tray_event` handler that calls `tauri_plugin_positioner::on_tray_event(app, &event)`
3. After the tray icon has been clicked at least once, the plugin can position windows relative to the tray icon

**Limitation**: Tray-relative positioning only works AFTER the user has clicked the tray icon at least once (the plugin needs the click event to capture the tray icon's rectangle).

### macOS

- The tray icon click event provides position and size data (`position`, `rect`).
- Menu bar is always at the top of the screen. The positioner plugin places the window directly below the tray icon.
- **Multi-monitor issue**: Positioning may be incorrect across monitors ([#7139](https://github.com/tauri-apps/tauri/issues/7139)).

### Windows

- System tray is typically in the bottom-right corner.
- The tray click event provides coordinates. The positioner plugin positions the window above the tray icon.
- Works with both traditional and "overflow" tray areas.

### Linux

- **Status area behavior varies dramatically** by desktop environment:
  - GNOME: Uses `libappindicator` / `libayatana-appindicator` for tray icons. Position data is available through click events.
  - KDE: System tray is generally reliable.
  - **Wayland on GNOME**: Tray icons may not appear in dev mode; only work in AppImage builds ([#14234](https://github.com/tauri-apps/tauri/issues/14234)).
  - **Flatpak**: Tray icon may fail entirely due to image format issues ([#13599](https://github.com/tauri-apps/tauri/issues/13599)).
- **Required dependency**: `libayatana-appindicator3-1` (preferred) or `libappindicator3-1`.

---

## 5. Click-Through Windows

### API

`window.setIgnoreCursorEvents(ignore: boolean)` -- toggles whether the entire window ignores cursor events.

### How It Works

When enabled, ALL mouse events pass through the window to whatever is underneath. When disabled, the window captures events normally. There is **no built-in per-pixel or per-region click-through** based on transparency.

### Cross-Platform Workaround for Partial Click-Through

Community-developed approach ([#13070](https://github.com/tauri-apps/tauri/issues/13070)):
1. Track cursor position from the backend
2. Determine if cursor is over a transparent or opaque region
3. Dynamically call `setIgnoreCursorEvents(true/false)` based on hit-testing
4. Minimal performance impact reported

### Plugin: `tauri-plugin-polygon`

The [`tauri-plugin-polygon`](https://crates.io/crates/tauri-plugin-polygon) crate allows defining polygon regions for mouse response, enabling more granular click-through behavior on Tauri v2.

### Platform Differences

- **macOS**: `setIgnoreCursorEvents` works. No automatic transparent-region pass-through.
- **Windows**: `setIgnoreCursorEvents` works. Same limitation -- all-or-nothing.
- **Linux**: `setIgnoreCursorEvents` works on X11. Behavior on Wayland may vary. The `setIgnoreCursorEvents` API has had reports of not working on some configurations ([#11461](https://github.com/tauri-apps/tauri/issues/11461)).

### CSS `pointer-events: none`

Only works within the webview. It prevents the webview from handling the event but does **NOT** forward events to underlying OS windows/applications. Useless for click-through to the desktop.

---

## 6. Alt-Tab / Cmd-Tab / Task Switcher

### API

`skipTaskbar: true` in window config, or `window.setSkipTaskbar(true)` at runtime.

### macOS

- **Cmd+Tab**: To hide from Cmd+Tab, use `ActivationPolicy::Accessory`:
  ```rust
  #[cfg(target_os = "macos")]
  app.set_activation_policy(tauri::ActivationPolicy::Accessory);
  ```
  This also hides the Dock icon. There is no way to skip Cmd+Tab while keeping the Dock icon.
- `skipTaskbar` alone does NOT hide from Cmd+Tab on macOS (macOS does not have a "taskbar" in the Windows sense).

### Windows

- **`skipTaskbar: true`**: Supposed to hide from the taskbar, but **is unreliable on Windows** ([#10422](https://github.com/tauri-apps/tauri/issues/10422), open). Behavior is inconsistent across Windows 10/11.
- **Alt+Tab**: `skipTaskbar` does not necessarily hide from Alt+Tab. To fully hide from Alt+Tab, you need to set the `WS_EX_TOOLWINDOW` extended window style via native API calls, but tool windows behave differently (no taskbar entry, different focus behavior).
- **Headless environments**: `set_skip_taskbar` can crash in Docker/headless Windows ("Failed to create taskbarlist"). Fixed in recent Tauri versions by converting panics to errors.

### Linux

- **X11**: `skipTaskbar` works via `_NET_WM_STATE_SKIP_TASKBAR` hint. Generally reliable on GNOME and KDE.
- **Wayland**: `set_skip_taskbar(true)` **does not work** ([#9829](https://github.com/tauri-apps/tauri/issues/9829)). Workaround: force the app to run under X11 with `GDK_BACKEND=x11`, but this defeats the purpose of Wayland.

---

## 7. Window Shadows and Rounded Corners

### Shadows

| Platform | Decorated Windows | Undecorated + `shadow: true` | Transparent Windows |
|----------|-------------------|------------------------------|---------------------|
| macOS    | Native shadow (system-controlled) | Native shadow | **No shadow** (always disabled) |
| Windows  | Native shadow (always ON, cannot disable) | 1px white border + shadow; rounded corners on Win11 | Varies; may show artifacts |
| Linux    | Compositor-controlled | Compositor-controlled | Compositor-controlled |

- The `window-shadows` crate is deprecated for Tauri v2; shadow control is now built into `tao`/`wry`.
- On macOS, if you need shadows on a transparent/custom window, you must implement them in CSS (e.g., `box-shadow`).

### Rounded Corners

- **macOS**: CSS `border-radius` on the root element mostly works when `transparent: true` + `decorations: false` + `shadow: false`. However, the top corners may remain sharp due to the webview's native frame ([#9287](https://github.com/tauri-apps/tauri/issues/9287)). Setting `shadow: false` fixes the border artifact. `macOSPrivateApi: true` helps.
- **Windows**: On Windows 11, undecorated windows with `shadow: true` automatically get native rounded corners. CSS `border-radius` works independently in the webview but the native window frame is rectangular on Windows 10. You may see a rectangular window with rounded content inside (visible corners).
- **Linux**: CSS `border-radius` works in the webview, but the native window frame is rectangular. With `transparent: true` + `decorations: false`, the rounded CSS corners are visible against the transparent background, so it works correctly when a compositor is running. Without a compositor, corners show as black.

### Recommended Recipe for Rounded Transparent Window

```json
// tauri.conf.json
{
  "windows": [{
    "transparent": true,
    "decorations": false,
    "shadow": false
  }],
  "app": {
    "macOSPrivateApi": true
  }
}
```

```css
html, body {
  background: transparent;
  margin: 0;
  padding: 0;
  overflow: hidden;
}

.app-container {
  border-radius: 12px;
  background: white;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.12);
  overflow: hidden;
}
```

---

## 8. Known Tauri v2 Bugs & Issues Summary

### Critical for Floating Widget / Tray App

| Issue | Platforms | Status | Impact | Workaround |
|-------|-----------|--------|--------|------------|
| [#14102](https://github.com/tauri-apps/tauri/issues/14102) `focusable: false` broken | macOS | Open | Window steals focus | Use `tauri-nspanel` or `ActivationPolicy::Accessory` |
| [#7519](https://github.com/tauri-apps/tauri/issues/7519) `focus: false` ignored | Windows | Open | Window always takes focus | Native API `WS_EX_NOACTIVATE` |
| [#10422](https://github.com/tauri-apps/tauri/issues/10422) `skipTaskbar` unreliable | Windows | Open | Appears in taskbar intermittently | `WS_EX_TOOLWINDOW` via native API |
| [#9829](https://github.com/tauri-apps/tauri/issues/9829) `skipTaskbar` broken on Wayland | Linux | Open | Cannot hide from taskbar on Wayland | Force X11 backend |
| [#13415](https://github.com/tauri-apps/tauri/issues/13415) Transparency lost in DMG | macOS | Open | Built app has opaque background | Test with built artifacts early |
| [#8308](https://github.com/tauri-apps/tauri/issues/8308) `transparent: true` broken v2 | Windows | Open | No transparency on Windows | Verify webview background color is set |
| [#11488](https://github.com/tauri-apps/tauri/issues/11488) Not visible over fullscreen | macOS | Closed (not planned) | Cannot overlay fullscreen apps | `tauri-nspanel` + `Accessory` policy |
| [#14234](https://github.com/tauri-apps/tauri/issues/14234) Tray icon missing on Wayland | Linux | Open | No tray icon in dev mode | Use AppImage or force X11 |
| [#12800](https://github.com/tauri-apps/tauri/issues/12800) Ghost renders | Linux | Closed (upstream) | Stale content until resize | Upgrade webkit2gtk >= 2.48.0 |
| [#10064](https://github.com/tauri-apps/tauri/issues/10064) CSS blur broken with transparent | Windows | Open | `backdrop-filter: blur()` fails | Use native acrylic/mica instead |
| [#8255](https://github.com/tauri-apps/tauri/issues/8255) Transparent glitch on Sonoma | macOS | Open | Visual artifacts after focus change | Minimize focus transitions |

### Less Critical but Relevant

| Issue | Platforms | Status | Impact |
|-------|-----------|--------|--------|
| [#6162](https://github.com/tauri-apps/tauri/issues/6162) Window properties ignored on Linux | Linux | Open | `visible`, `decorations` may not work |
| [#13157](https://github.com/tauri-apps/tauri/issues/13157) Glitchy rendering | Linux | Open | Shadow DOM copies on maximize/unmaximize |
| [#9287](https://github.com/tauri-apps/tauri/issues/9287) Rounded corners + shadows conflict | macOS/Windows | Open | Corners not rounded when shadow enabled |
| [#12834](https://github.com/tauri-apps/tauri/issues/12834) `setFocus()` broken 2.3+ | macOS | Open | Cannot programmatically focus window |
| [#11461](https://github.com/tauri-apps/tauri/issues/11461) `setIgnoreCursorEvents` broken | Linux | Open | Click-through may not work |

---

## 9. Recommendations for a Floating Widget / Tray App

### Architecture Recommendations

1. **macOS**: Use `tauri-nspanel` (v2 branch) for the floating widget window. This gives you non-focus-stealing panels that can appear over full-screen apps. Set `ActivationPolicy::Accessory` to hide from Dock/Cmd+Tab.

2. **Windows**: Use native API calls post-window-creation to set `WS_EX_NOACTIVATE` and `WS_EX_TOOLWINDOW` for proper no-focus, no-taskbar behavior. Do not rely on Tauri's `focusable` or `skipTaskbar` config.

3. **Linux**: Target X11 as the primary backend. Wayland support for tray apps has too many unresolved issues (no `skipTaskbar`, missing tray icons, property ignoring). Consider setting `GDK_BACKEND=x11` as a fallback. Require webkit2gtk >= 2.48.0 in documentation.

### Configuration Baseline

```json
{
  "app": {
    "macOSPrivateApi": true
  },
  "windows": [{
    "label": "widget",
    "transparent": true,
    "decorations": false,
    "shadow": false,
    "alwaysOnTop": true,
    "skipTaskbar": true,
    "visible": false,
    "width": 360,
    "height": 480
  }]
}
```

### Key Trade-offs

- **Transparency + CSS blur**: Mutually exclusive on Windows. Choose either native acrylic/mica or CSS-only styling with a solid background.
- **No-focus + Dock visibility**: Mutually exclusive on macOS. `ActivationPolicy::Accessory` hides from both Dock and Cmd+Tab.
- **Wayland + Tray**: Linux tray icon support on Wayland is fragile. AppImage format works better than .deb for Wayland tray icons.
- **Rounded corners + shadows**: Disable native shadows (`shadow: false`) and use CSS `box-shadow` + `border-radius` for consistent cross-platform appearance.

---

## Sources

- [Tauri v2 Window Customization](https://v2.tauri.app/learn/window-customization/)
- [Tauri v2 Positioner Plugin](https://v2.tauri.app/plugin/positioner/)
- [Tauri v2 System Tray](https://v2.tauri.app/learn/system-tray/)
- [Tauri v2 Configuration Reference](https://v2.tauri.app/reference/config/)
- [tauri-nspanel (v2 branch)](https://github.com/ahkohd/tauri-nspanel)
- [tauri-plugin-polygon](https://crates.io/crates/tauri-plugin-polygon)
- [window-vibrancy crate](https://github.com/tauri-apps/window-vibrancy)
- [Building a system tray app with Tauri](https://tauritutorials.com/blog/building-a-system-tray-app-with-tauri)
