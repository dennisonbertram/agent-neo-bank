# Tauri v2 Core Window & Multi-Window APIs

> Research compiled 2026-02-28. All APIs verified against official Tauri docs and context7.

---

## 1. Latest Stable Tauri Version

| Package | Version | Notes |
|---------|---------|-------|
| `tauri` (Rust crate) | **2.10.2** | Released ~Feb 4, 2025 ([GitHub Releases](https://github.com/tauri-apps/tauri/releases)) |
| `@tauri-apps/api` (JS) | **2.10.1** | JS/TS frontend bindings |
| `tauri-cli` | **2.10.0** | CLI tooling |
| `@tauri-apps/cli` | **2.10.0** | npm wrapper for CLI |
| `wry` | **0.54.2** | WebView rendering library |
| `tao` | **0.34.5** | Windowing library |

Tauri v2.0 stable was initially released on **October 2, 2024** ([announcement](https://v2.tauri.app/blog/tauri-20/)). The v2.10.x line is the latest as of this research.

**Release page**: https://v2.tauri.app/release/

---

## 2. Multi-Window Architecture

Tauri v2 supports multiple windows, each identified by a unique **label** (alphanumeric string). Windows can be created via configuration or programmatically.

### 2a. Static Window Creation (tauri.conf.json)

Define windows in `app.windows` array. Each entry is a `WindowConfig` object:

```jsonc
// tauri.conf.json
{
  "app": {
    "windows": [
      {
        "label": "main",
        "title": "Main Window",
        "url": "/",
        "width": 1200,
        "height": 800
      },
      {
        "label": "settings",
        "title": "Settings",
        "url": "/settings",
        "width": 600,
        "height": 400,
        "visible": false
      }
    ]
  }
}
```

Set `"create": false` to define a window config without auto-creating it at startup (useful for deferred creation).

### 2b. Programmatic Window Creation (Rust)

Use `WebviewWindowBuilder` in Rust:

```rust
use tauri::Manager;

// From the setup hook:
tauri::Builder::default()
    .setup(|app| {
        let webview_url = tauri::WebviewUrl::App("index.html".into());

        // Create first window
        tauri::WebviewWindowBuilder::new(app, "first", webview_url.clone())
            .title("First Window")
            .build()?;

        // Create second window
        tauri::WebviewWindowBuilder::new(app, "second", webview_url)
            .title("Second Window")
            .build()?;

        Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
```

You can also build from a pre-defined config entry:

```rust
tauri::WebviewWindowBuilder::from_config(app.handle(), &app.config().app.windows[0])?.build()?;
```

### 2c. Programmatic Window Creation (JavaScript/TypeScript)

```typescript
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';

const webview = new WebviewWindow('settings', {
  url: '/settings',
  title: 'Settings',
  width: 600,
  height: 400,
});

webview.once('tauri://created', () => {
  console.log('window created');
});

webview.once('tauri://error', (e) => {
  console.error('window creation error', e);
});
```

**Required permission**: `core:webview:allow-create-webview-window` in your capabilities file.

### 2d. Getting References to Existing Windows

**Rust:**
```rust
use tauri::Manager;

// From AppHandle or App
let window = app.get_webview_window("main").unwrap();
```

**JavaScript:**
```typescript
import { getCurrentWindow } from '@tauri-apps/api/window';
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';

// Current window
const current = getCurrentWindow();

// Other window by label
const settings = WebviewWindow.getByLabel('settings');
```

### 2e. Capabilities Per Window

Tauri v2 uses an ACL-based capability system. Each capability can target specific windows:

```jsonc
// src-tauri/capabilities/main.json
{
  "identifier": "main-capability",
  "description": "Permissions for the main window",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "core:window:default",
    "core:window:allow-start-dragging"
  ]
}
```

```jsonc
// src-tauri/capabilities/settings.json
{
  "identifier": "settings-capability",
  "windows": ["settings"],
  "permissions": [
    "core:default",
    "core:window:allow-close",
    "core:window:allow-hide"
  ],
  "platforms": ["macOS", "windows"]
}
```

**Docs**: https://v2.tauri.app/security/capabilities/

---

## 3. Window Configuration Options

### Complete WindowConfig Reference

All fields of `WindowConfig` (Rust struct in `tauri_utils::config`). JSON keys use `camelCase`; Rust fields use `snake_case`.

**Rust docs**: https://docs.rs/tauri-utils/latest/tauri_utils/config/struct.WindowConfig.html

| JSON Key (`tauri.conf.json`) | Rust Field | Type | Default | Description |
|---|---|---|---|---|
| `label` | `label` | `String` | (required) | Unique window identifier (alphanumeric) |
| `url` | `url` | `WebviewUrl` | `"/"` | Window webview URL |
| `create` | `create` | `bool` | `true` | Whether Tauri creates the window at startup |
| `title` | `title` | `String` | `""` | Window title |
| `width` | `width` | `f64` | `800.0` | Initial width in logical pixels |
| `height` | `height` | `f64` | `600.0` | Initial height in logical pixels |
| `minWidth` | `min_width` | `Option<f64>` | `None` | Minimum width constraint |
| `minHeight` | `min_height` | `Option<f64>` | `None` | Minimum height constraint |
| `maxWidth` | `max_width` | `Option<f64>` | `None` | Maximum width constraint |
| `maxHeight` | `max_height` | `Option<f64>` | `None` | Maximum height constraint |
| `x` | `x` | `Option<f64>` | `None` | Initial horizontal position (requires `y` too) |
| `y` | `y` | `Option<f64>` | `None` | Initial vertical position (requires `x` too) |
| `center` | `center` | `bool` | `false` | Start centered on screen |
| `resizable` | `resizable` | `bool` | `true` | Whether the window is resizable |
| `maximizable` | `maximizable` | `bool` | `true` | Native maximize button enabled |
| `minimizable` | `minimizable` | `bool` | `true` | Native minimize button enabled |
| `closable` | `closable` | `bool` | `true` | Native close button enabled |
| `visible` | `visible` | `bool` | `true` | Visible on creation |
| `focus` | `focus` | `bool` | `true` | Initially focused |
| `focusable` | `focusable` | `bool` | `true` | Whether the window can receive focus |
| `fullscreen` | `fullscreen` | `bool` | `false` | Start fullscreen |
| `maximized` | `maximized` | `bool` | `false` | Start maximized |
| `decorations` | `decorations` | `bool` | `true` | Window borders and title bar |
| `transparent` | `transparent` | `bool` | `false` | Transparent window (see notes below) |
| `alwaysOnTop` | `always_on_top` | `bool` | `false` | Always above other windows |
| `alwaysOnBottom` | `always_on_bottom` | `bool` | `false` | Always below other windows |
| `skipTaskbar` | `skip_taskbar` | `bool` | `false` | Hide from taskbar/dock |
| `shadow` | `shadow` | `bool` | `true` | Window shadow |
| `theme` | `theme` | `Option<Theme>` | `None` | Light/Dark/System theme |
| `titleBarStyle` | `title_bar_style` | `TitleBarStyle` | `Visible` | macOS title bar style |
| `trafficLightPosition` | `traffic_light_position` | `Option<LogicalPosition>` | `None` | macOS traffic light position (since 2.4.0) |
| `hiddenTitle` | `hidden_title` | `bool` | `false` | Hide title text |
| `contentProtected` | `content_protected` | `bool` | `false` | Prevent screen capture |
| `parent` | `parent` | `Option<String>` | `None` | Parent window label |
| `visibleOnAllWorkspaces` | `visible_on_all_workspaces` | `bool` | `false` | Visible on all virtual desktops |
| `acceptFirstMouse` | `accept_first_mouse` | `bool` | `false` | macOS: accept click that focuses |
| `tabbingIdentifier` | `tabbing_identifier` | `Option<String>` | `None` | macOS tab grouping |
| `dragDropEnabled` | `drag_drop_enabled` | `bool` | `true` | Enable file drag-and-drop |
| `backgroundColor` | `background_color` | `Option<Color>` | `None` | Window and webview background color |
| `windowEffects` | `window_effects` | `Option<WindowEffectsConfig>` | `None` | Platform visual effects (mica, acrylic, vibrancy) |
| `incognito` | `incognito` | `bool` | `false` | Incognito/private webview |
| `userAgent` | `user_agent` | `Option<String>` | `None` | Custom user agent |
| `additionalBrowserArgs` | `additional_browser_args` | `Option<String>` | `None` | Extra browser arguments |
| `proxyUrl` | `proxy_url` | `Option<Url>` | `None` | Proxy for network requests |
| `zoomHotkeysEnabled` | `zoom_hotkeys_enabled` | `bool` | `true` | Ctrl+/- zoom hotkeys |
| `browserExtensionsEnabled` | `browser_extensions_enabled` | `bool` | `false` | Browser extension support |
| `useHttpsScheme` | `use_https_scheme` | `bool` | `false` | Use HTTPS custom protocol |
| `devtools` | `devtools` | `Option<bool>` | `None` | DevTools availability |
| `javascriptDisabled` | `javascript_disabled` | `bool` | `false` | Disable JS execution |
| `scrollBarStyle` | `scroll_bar_style` | `ScrollBarStyle` | `Default` | Native scrollbar style |
| `preventOverflow` | `prevent_overflow` | `Option<PreventOverflowConfig>` | `None` | Prevent window exceeding monitor bounds |

---

### 3a. Frameless/Chromeless Windows (Decorations)

Remove native title bar and borders.

**Config (`tauri.conf.json`):**
```jsonc
{
  "app": {
    "windows": [{
      "label": "main",
      "decorations": false
    }]
  }
}
```

**Rust API:**
```rust
// WebviewWindowBuilder
tauri::WebviewWindowBuilder::new(app, "popup", url)
    .decorations(false)
    .build()?;

// Runtime toggle
window.set_decorations(false)?;
```

**JavaScript API:**
```typescript
import { getCurrentWindow } from '@tauri-apps/api/window';
await getCurrentWindow().setDecorations(false);
```

**Platform notes:**
- On Windows: `shadow: false` has no effect on decorated windows (shadows always ON). `shadow: true` on undecorated windows gives 1px white border; on Windows 11 you get rounded corners.
- On Linux (GTK): The window manager may still show some decoration.

---

### 3b. Transparent Background Windows

**Config:**
```jsonc
{
  "app": {
    "windows": [{
      "label": "overlay",
      "transparent": true,
      "decorations": false
    }]
  }
}
```

**macOS requirement:** You must enable the `macos-private-api` feature flag:

```jsonc
// tauri.conf.json
{
  "app": {
    "macOSPrivateApi": true
  }
}
```

> WARNING: Using macOS private APIs prevents App Store submission.

**CSS requirements for transparency:**
```css
html, body {
  background: transparent;
}
```

Without this, the webview will render an opaque white/dark background even though the window itself is transparent.

**Platform notes:**
- **Windows 7**: Transparency is not supported; alpha is ignored.
- **Windows 8+**: Translucent colors not supported -- any alpha other than `0` is treated as `255`.
- **macOS**: Requires `macOSPrivateApi: true`.
- **Linux**: Limited support depending on compositor.

**Window effects (alternative):** Instead of raw transparency, use platform-native effects:
```jsonc
{
  "windowEffects": {
    "effects": ["mica"],
    "state": "active"
  }
}
```
Available effects: `mica`, `acrylic`, `blur`, `vibrancy` (platform-dependent). Requires `transparent: true`.

---

### 3c. Always-on-Top

**Config:**
```jsonc
{ "alwaysOnTop": true }
```

**Rust API:**
```rust
window.set_always_on_top(true)?;
```

**JavaScript API:**
```typescript
import { getCurrentWindow } from '@tauri-apps/api/window';
await getCurrentWindow().setAlwaysOnTop(true);
```

**Platform notes:** Unsupported on Linux (Wayland), iOS, Android.

---

### 3d. Skip Taskbar/Dock

Hide the window from the taskbar (Windows) or dock (macOS).

**Config:**
```jsonc
{ "skipTaskbar": true }
```

**Rust API:**
```rust
window.set_skip_taskbar(true)?;
```

**JavaScript API:**
```typescript
await getCurrentWindow().setSkipTaskbar(true);
```

---

### 3e. Hide from Alt-Tab / Cmd-Tab

There is **no dedicated `skipAltTab` or `hideFromSwitcher` config key** in Tauri v2. However, this behavior can be achieved through a combination of techniques:

1. **`skipTaskbar: true`** -- On Windows, this often removes the window from Alt-Tab as well (behavior depends on Windows version).
2. **`parent` window** -- On Windows, owned windows (windows with a parent) are excluded from the Alt-Tab switcher. Set a parent window to achieve this:
   ```jsonc
   { "parent": "main" }
   ```
   Platform-specific behavior:
   - **Windows**: Owned windows are always above their owner, destroyed when owner is destroyed, hidden when owner is minimized. They do NOT appear in Alt-Tab.
   - **Linux**: `set_transient_for` (GTK3). May or may not hide from app switcher depending on WM.
   - **macOS**: Adds as a child window. Does not inherently hide from Cmd-Tab.

3. **macOS**: The app-level Cmd-Tab behavior is controlled by the `LSUIElement` key in `Info.plist` (setting it to `true` hides the app entirely from the dock and Cmd-Tab). This is outside Tauri config but can be set in the bundle config.

---

### 3f. Window Sizing and Positioning (Programmatic)

**Rust API:**
```rust
use tauri::LogicalSize;
use tauri::LogicalPosition;

// Set size
window.set_size(LogicalSize::new(800.0, 600.0))?;

// Set position
window.set_position(LogicalPosition::new(100.0, 100.0))?;

// Set minimum size
window.set_min_size(Some(LogicalSize::new(400.0, 300.0)))?;

// Center on screen
window.center()?;
```

**JavaScript API:**
```typescript
import { getCurrentWindow } from '@tauri-apps/api/window';
import { LogicalSize, LogicalPosition } from '@tauri-apps/api/dpi';

const win = getCurrentWindow();
await win.setSize(new LogicalSize(800, 600));
await win.setPosition(new LogicalPosition(100, 100));
await win.setMinSize(new LogicalSize(400, 300));
await win.center();
```

**Config:**
```jsonc
{
  "width": 800,
  "height": 600,
  "x": 100,
  "y": 100,
  "center": true,
  "minWidth": 400,
  "minHeight": 300,
  "preventOverflow": true
}
```

---

### 3g. Draggable Regions (data-tauri-drag-region)

For frameless windows, enable window dragging from custom titlebar regions.

**HTML:**
```html
<div data-tauri-drag-region class="titlebar">
  <span>My App</span>
  <div class="controls">
    <button id="btn-minimize">-</button>
    <button id="btn-maximize">[]</button>
    <button id="btn-close">X</button>
  </div>
</div>
```

**Critical behavior:** `data-tauri-drag-region` only applies to the element it is directly on. Child elements do NOT inherit the drag behavior. This is intentional so buttons/inputs inside the titlebar remain clickable.

**Required permission:**
```jsonc
{
  "permissions": [
    "core:window:allow-start-dragging"
  ]
}
```

**Programmatic drag (alternative):**

For more control, use `startDragging()` directly:

```typescript
import { getCurrentWindow } from '@tauri-apps/api/window';

const appWindow = getCurrentWindow();
const titlebar = document.getElementById('titlebar');

let lastClickTime = 0;
titlebar.addEventListener('mousedown', async (e) => {
  const now = Date.now();
  if (now - lastClickTime < 300) {
    // Double-click: toggle maximize
    await appWindow.toggleMaximize();
  } else {
    await appWindow.startDragging();
  }
  lastClickTime = now;
});
```

**JavaScript window controls:**
```typescript
import { getCurrentWindow } from '@tauri-apps/api/window';

const appWindow = getCurrentWindow();

document.getElementById('btn-minimize')?.addEventListener('click', () => appWindow.minimize());
document.getElementById('btn-maximize')?.addEventListener('click', () => appWindow.toggleMaximize());
document.getElementById('btn-close')?.addEventListener('click', () => appWindow.close());
```

---

### 3h. Focus Control (Show Without Stealing Focus)

**Config:**
```jsonc
{
  "focus": false,
  "focusable": true,
  "visible": true
}
```

- `focus: false` -- window does not grab focus on creation.
- `focusable: false` -- window can NEVER receive focus (useful for overlay widgets).

**Rust API:**
```rust
// Create without focus
tauri::WebviewWindowBuilder::new(app, "tooltip", url)
    .focused(false)
    .build()?;

// Runtime: bring to front and focus
window.set_focus()?;

// Show without focusing (show only makes it visible)
window.show()?;
```

**JavaScript API:**
```typescript
await getCurrentWindow().setFocus(); // Focus the window
await getCurrentWindow().show();     // Show without stealing focus
```

---

### 3i. Close Interception (Prevent Close, Hide Instead)

In Tauri v2, `WebviewWindow::close()` triggers a **close-requested event** instead of force-closing. Use `WebviewWindow::destroy()` to force-close.

**JavaScript -- intercept close:**
```typescript
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';

const appWindow = getCurrentWebviewWindow();

appWindow.onCloseRequested(async (event) => {
  // Prevent the window from closing
  event.preventDefault();

  // Hide instead of close
  await appWindow.hide();
});
```

**Rust -- intercept close via window event:**
```rust
use tauri::Manager;

tauri::Builder::default()
    .setup(|app| {
        let window = app.get_webview_window("main").unwrap();
        window.on_window_event(|event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // Prevent close
                api.prevent_close();
                // Optionally hide the window
                // window.hide().unwrap();
            }
        });
        Ok(())
    });
```

**Note:** Listening to `tauri://close-requested` only works on the specific `WebviewWindow` instance, NOT with the global `listen` function.

---

### 3j. Window Visibility (Show/Hide)

**Rust API:**
```rust
window.show()?;   // Make visible
window.hide()?;   // Make invisible
```

**JavaScript API:**
```typescript
await getCurrentWindow().show();
await getCurrentWindow().hide();
```

**Config:**
```jsonc
{ "visible": false }
```

Set `visible: false` to create a hidden window, then call `show()` when ready. This avoids flash-of-content on startup.

---

## 4. Tauri Event System

The event system provides pub-sub communication between Rust backend and frontend webviews. It is separate from the command/IPC system.

**Docs**: https://v2.tauri.app/develop/calling-frontend/

### Core Traits

| Trait | Implemented By | Purpose |
|-------|---------------|---------|
| `Emitter` | `AppHandle`, `WebviewWindow` | Send events |
| `Listener` | `AppHandle`, `WebviewWindow` | Receive events |

### 4a. Global Events (Broadcast to All Listeners)

**Rust -- emit globally:**
```rust
use tauri::{AppHandle, Emitter};

#[tauri::command]
fn download(app: AppHandle, url: String) {
    app.emit("download-started", &url).unwrap();
    for progress in [1, 15, 50, 80, 100] {
        app.emit("download-progress", progress).unwrap();
    }
}
```

**JavaScript -- listen globally:**
```typescript
import { listen } from '@tauri-apps/api/event';

type DownloadStarted = {
  url: string;
  downloadId: number;
  contentLength: number;
};

const unlisten = await listen<DownloadStarted>('download-started', (event) => {
  console.log(`downloading from ${event.payload.url}`);
});

// Stop listening:
unlisten();
```

### 4b. Webview-Specific Events (Target a Specific Window)

**Rust -- emit to specific webview:**
```rust
use tauri::{AppHandle, Emitter};

#[tauri::command]
fn login(app: AppHandle, user: String, password: String) {
    let result = if user == "tauri-apps" && password == "tauri" {
        "loggedIn"
    } else {
        "invalidCredentials"
    };
    // Emit ONLY to the "login" webview
    app.emit_to("login", "login-result", result).unwrap();
}
```

**JavaScript -- listen on specific webview:**
```typescript
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';

const appWebview = getCurrentWebviewWindow();
appWebview.listen<string>('login-result', (event) => {
    console.log('login result:', event.payload);
});
```

**JavaScript -- emit to specific webview:**
```typescript
import { emitTo } from '@tauri-apps/api/event';

emitTo('settings', 'settings-update-requested', {
  key: 'notification',
  value: 'all',
});
```

### 4c. Filtered Events (Emit to Multiple Specific Windows)

```rust
use tauri::{AppHandle, Emitter, EventTarget};

#[tauri::command]
fn open_file(app: AppHandle, path: std::path::PathBuf) {
    app.emit_filter("open-file", &path, |target| match target {
        EventTarget::WebviewWindow { label } =>
            label == "main" || label == "file-viewer",
        _ => false,
    }).unwrap();
}
```

### 4d. Event Payloads

Payloads must implement `Serialize + Clone`:

```rust
use serde::Serialize;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct DownloadStarted<'a> {
    url: &'a str,
    download_id: usize,
    content_length: usize,
}
```

> **Limitation:** Event payloads are always serialized as JSON strings, making them unsuitable for large binary data. Use commands for larger payloads.

### 4e. One-Time Listeners and Unlisten

**Rust:**
```rust
use tauri::Listener;

let event_id = app.listen("download-started", |event| { /* ... */ });
app.unlisten(event_id);

// Listen once
app.once("ready", |event| {
    println!("app is ready");
});
```

**JavaScript:**
```typescript
import { once } from '@tauri-apps/api/event';

await once('ready', (event) => {
    console.log('ready!');
});
```

### 4f. Important: Global vs Webview-Specific Event Isolation

Webview-specific events are **NOT** delivered to global listeners. If you need to catch all events regardless of target, use `listen_any` instead of `listen`:

```rust
app.listen_any("some-event", |event| {
    // catches both global and webview-targeted events
});
```

---

## 5. Security Model

**Docs**: https://v2.tauri.app/security/

### Architecture Layers

```
+---------------------------+
|   Frontend (WebView)      |  -- Untrusted: HTML/JS/CSS
|   System WebView Runtime  |  -- OS-provided (Edge WebView2 / WKWebView / WebKitGTK)
+---------------------------+
|   IPC Bridge              |  -- Serialized JSON messages, filtered by capabilities
+---------------------------+
|   Tauri Core (Rust)       |  -- Trusted: full system access
|   TAO (windowing)         |
|   WRY (webview rendering) |
+---------------------------+
```

### Trust Boundaries

1. **Rust backend** = trusted. Has full system access (filesystem, network, processes, etc.).
2. **WebView frontend** = untrusted. Can only access system resources explicitly exposed via the IPC layer.
3. **All IPC routes through the Core process**, allowing interception, filtering, and manipulation.

### Capabilities System (v2 ACL)

Replaced v1's flat allowlist. Capabilities are JSON files in `src-tauri/capabilities/`:

```jsonc
{
  "identifier": "main-user-files-write",
  "description": "Main window can write to user files",
  "windows": ["main"],           // Which windows get these permissions
  "permissions": [
    "core:default",
    "dialog:open",
    {
      "identifier": "fs:allow-write-text-file",
      "allow": [{ "path": "$HOME/test.txt" }]
    }
  ],
  "platforms": ["macOS", "windows"]  // Optional platform scoping
}
```

### Security Best Practices

- **Never expose secrets in the frontend.** API keys, wallet keys, tokens must stay in Rust. The webview has access to DevTools in development.
- **Scope permissions narrowly.** Use per-window capabilities. A popup does not need filesystem access.
- **Use `contentProtected: true`** if the window displays sensitive data (prevents screen capture).
- **Validate all IPC inputs** in your Tauri commands. The frontend is untrusted.

---

## 6. CSS / WebView Constraints

### WebView Engines by Platform

| Platform | Engine | Version Notes |
|----------|--------|---------------|
| Windows | Edge WebView2 | Ships with Windows 10/11; must be installed separately on older |
| macOS | WKWebView | Ships with macOS |
| Linux | WebKitGTK | Distribution-dependent version |

These are **not bundled** -- they are dynamically linked at runtime from the OS.

**Docs**: https://v2.tauri.app/reference/webview-versions/

### Transparent Body Requirements

For transparent windows to work, you MUST set the CSS background to transparent:

```css
html, body {
  background: transparent;
  margin: 0;
  padding: 0;
}
```

Without this, the webview renders an opaque background behind your content even when the window itself is transparent.

### Platform-Specific CSS Issues

**Windows:**
- Alpha channel in `background-color` is problematic. On Windows 8+, any alpha value other than `0` is treated as `255` (fully opaque).
- `shadow: true` on undecorated windows adds a 1px white border. On Windows 11, this also rounds the corners.
- For window effects (mica/acrylic), you may need workarounds. See: https://github.com/tauri-apps/tao/issues/72#issuecomment-975607891

**macOS:**
- Custom titlebars lose native features (window snapping, drag-to-arrange).
- Alternative: use `titleBarStyle: "overlay"` or `"transparent"` to keep native controls while customizing the title bar area.
- The `trafficLightPosition` config (since 2.4.0) lets you reposition the native red/yellow/green buttons when using `titleBarStyle: "overlay"`.

**Linux:**
- Window effects (`windowEffects`) are unsupported.
- `shadow` setting is unsupported.
- `visibleOnAllWorkspaces` is supported (unlike Windows/iOS/Android where it is not).

**Cross-platform scrollbar styling:**
- Use `scrollBarStyle` config for native scrollbar appearance (currently only `fluentOverlay` on Windows with WebView2 >= 125.0.2535.41).
- CSS scrollbar styles (`::webkit-scrollbar`) apply on top of the native style.

### `data-tauri-drag-region` CSS Interaction

- The drag region attribute disables pointer events on child elements during the drag. Buttons/inputs inside a drag region work normally when clicked (not dragged).
- If you need custom drag behavior, use `appWindow.startDragging()` on mousedown events instead of the attribute.

---

## Quick Reference: API Mapping

| Feature | Config Key | Rust Method | JS/TS Method |
|---------|-----------|-------------|-------------|
| Show window | `visible` | `window.show()` | `getCurrentWindow().show()` |
| Hide window | `visible` | `window.hide()` | `getCurrentWindow().hide()` |
| Always on top | `alwaysOnTop` | `window.set_always_on_top(bool)` | `getCurrentWindow().setAlwaysOnTop(bool)` |
| Skip taskbar | `skipTaskbar` | `window.set_skip_taskbar(bool)` | `getCurrentWindow().setSkipTaskbar(bool)` |
| Decorations | `decorations` | `window.set_decorations(bool)` | `getCurrentWindow().setDecorations(bool)` |
| Transparent | `transparent` | builder `.transparent(true)` | `new WebviewWindow('x', { transparent: true })` |
| Set size | `width`/`height` | `window.set_size(LogicalSize)` | `getCurrentWindow().setSize(LogicalSize)` |
| Set position | `x`/`y` | `window.set_position(LogicalPosition)` | `getCurrentWindow().setPosition(LogicalPosition)` |
| Center | `center` | `window.center()` | `getCurrentWindow().center()` |
| Focus | `focus` | `window.set_focus()` | `getCurrentWindow().setFocus()` |
| Close intercept | -- | `window.on_window_event(CloseRequested)` | `appWindow.onCloseRequested(cb)` |
| Drag region | -- (HTML attr) | `window.start_dragging()` | `getCurrentWindow().startDragging()` |
| Emit global | -- | `app.emit(name, payload)` | `emit(name, payload)` |
| Emit to window | -- | `app.emit_to(label, name, payload)` | `emitTo(label, name, payload)` |
| Listen global | -- | `app.listen(name, cb)` | `listen(name, cb)` |
| Listen on window | -- | `webview.listen(name, cb)` | `getCurrentWebviewWindow().listen(name, cb)` |

---

## Sources

- [Tauri v2 Official Docs](https://v2.tauri.app/)
- [Tauri v2 Configuration Reference](https://v2.tauri.app/reference/config/)
- [Tauri v2 Window JS API](https://v2.tauri.app/reference/javascript/api/namespacewindow/)
- [Tauri v2 Event JS API](https://v2.tauri.app/reference/javascript/api/namespaceevent/)
- [Tauri v2 Calling Frontend from Rust](https://v2.tauri.app/develop/calling-frontend/)
- [Tauri v2 Window Customization Guide](https://v2.tauri.app/learn/window-customization/)
- [Tauri v2 Security / Capabilities](https://v2.tauri.app/security/capabilities/)
- [Tauri v2 Capabilities for Windows and Platforms](https://v2.tauri.app/learn/security/capabilities-for-windows-and-platforms/)
- [WindowConfig Rust Docs (docs.rs)](https://docs.rs/tauri-utils/latest/tauri_utils/config/struct.WindowConfig.html)
- [Tauri Releases (GitHub)](https://github.com/tauri-apps/tauri/releases)
- [Tauri Core Ecosystem Releases](https://v2.tauri.app/release/)
- [Tauri 2.0 Stable Release Announcement](https://v2.tauri.app/blog/tauri-20/)
