# Frameless Transparent Window Investigation

**Date**: 2026-02-28
**Status**: Reverted — needs deeper investigation

## Goal

Test a frameless Tauri window with transparent background so the app floats on screen like a phone mockup with rounded corners.

## What We Tried

### 1. Config-only approach
- Set `decorations: false` and `transparent: true` in `tauri.conf.json`
- Set `html, body { background: transparent }` in CSS
- Kept `#root` with `background-color: var(--bg-primary)` and `border-radius: 16px`

**Result**: Window became frameless but background stayed solid white with a thin black border line around the window. The webview transparency config alone doesn't clear the native macOS window background.

### 2. Rust-side `set_background_color`
- Added `window.set_background_color(Some(Color(0, 0, 0, 0)))` in the Tauri setup
- Used `tauri::webview::Color` (Tauri v2 API — `Color(r, g, b, a)` tuple struct from `tauri_utils::config::Color`)

**Result**: Still white background. The webview background color API doesn't affect the native `NSWindow` backing layer on macOS.

### 3. Inset `#root` for visible rounded corners
- Shrunk `#root` with `calc(100% - 16px)` / `calc(100vh - 16px)` and `margin: 8px auto`
- Added `box-shadow` for floating effect

**Result**: Rounded corners were visible but the surrounding area was white (not transparent). Also the `data-tauri-drag-region` drag handle didn't work — window wasn't draggable.

## Root Cause

True window transparency on macOS requires accessing private AppKit APIs:
- The `NSWindow` needs `setBackgroundColor: [NSColor clearColor]`
- The `NSWindow` needs `setOpaque: NO`
- Tauri v2's `transparent: true` config sets the *webview* transparent but doesn't fully clear the native window backing

## What Would Be Needed

1. **Enable `macos-private-api`** feature flag in `Cargo.toml` for the tauri dependency
2. **Use `NSVisualEffectView`** or direct `objc` calls to set the window background to clear
3. **Alternative**: Use Tauri's `WindowEffect` API (vibrancy/blur effects) which does work on macOS — gives a frosted glass look rather than full transparency

## Drag Region Issue

The `data-tauri-drag-region` attribute on a `<div>` in App.tsx didn't enable window dragging. May need:
- The `core:window:allow-start-dragging` permission in capabilities
- Or the drag region attribute may not propagate through React's virtual DOM correctly

## Decision

Reverted all changes. The frameless transparent look is achievable but requires native macOS API integration that's beyond a quick config toggle. Worth revisiting if the design direction calls for it.

## Files That Were Modified (all reverted)

- `src-tauri/tauri.conf.json` — decorations/transparent flags
- `src-tauri/src/lib.rs` — set_background_color call
- `src/styles/globals.css` — transparent backgrounds, border-radius, inset sizing
- `src/App.tsx` — drag region div
