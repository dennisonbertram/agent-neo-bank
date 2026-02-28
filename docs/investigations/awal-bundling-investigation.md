# awal CLI Bundling Investigation

**Date**: 2026-02-28
**Status**: Complete

## 1. Current Invocation Method

The app currently invokes awal via `npx awal@latest`. The Rust backend uses a two-part approach:

- **Binary**: `"npx"` (configured in `AppConfig.awal_binary_path`, defaults to `"npx"`)
- **Args prefix**: `["awal@latest"]` (hardcoded in `AppState::new()`)

So every CLI call becomes: `npx awal@latest <subcommand> --json`

**Key files**:
- `src-tauri/src/config.rs` — `awal_binary_path` field, defaults to `"npx"`
- `src-tauri/src/state/app_state.rs` — constructs `RealCliExecutor::new(&config.awal_binary_path, vec!["awal@latest".to_string()], &config.network)`
- `src-tauri/src/cli/executor.rs` — `RealCliExecutor` spawns `tokio::process::Command` with binary + args_prefix + command args
- `src-tauri/src/cli/commands.rs` — `AwalCommand` enum with `to_args()` producing subcommand args like `["auth", "login", "user@example.com", "--json"]`

## 2. awal npm Package Details

| Field | Value |
|-------|-------|
| **Package name** | `awal` |
| **Latest version** | `2.0.3` |
| **All versions** | 1.0.0, 1.3.0, 1.3.1, 1.3.2, 2.0.3 |
| **License** | Apache-2.0 |
| **Maintainer** | `erik_cb <erik.reppel@coinbase.com>` |
| **Binary name** | `awal` |
| **Unpacked size** | 628.5 kB |
| **Type** | Pure Node.js (no native addons) |

### Dependencies (10 total, all pure JS)
- `@langchain/community` ^1.1.12
- `@langchain/core` ^1.1.20
- `@x402/core` 2.3.0
- `@x402/extensions` 2.3.0
- `chalk` ^5.3.0
- `commander` ^12.0.0
- `env-paths` ^3.0.0
- `ora` ^8.0.0
- `viem` ^2.37.3
- `zod` ^3.23.8

**Not currently in package.json** — there are zero references to awal in the project's `package.json`.

## 3. Problems with Current Approach (`npx awal@latest`)

1. **Requires Node.js/npm on the user's machine** — end users of this Tauri desktop app may not have Node.js installed.
2. **Network dependency** — `npx` downloads the package on first use and may re-download on updates. Cold start can take 5-10+ seconds.
3. **Version unpinned** — `@latest` means the CLI version can change without notice, potentially breaking the app.
4. **Startup latency** — every `npx` invocation has overhead: package resolution, extraction, spawning Node.js.
5. **No offline support** — fails without internet on first run.

## 4. Bundling Options

### Option A: Install as Local npm Dependency (Pinned Version)

**Approach**: Add `"awal": "2.0.3"` to `package.json` dependencies. Change invocation from `npx awal@latest` to calling the local `node_modules/.bin/awal`.

**Pros**:
- Pinned version, reproducible builds
- No `npx` download overhead after `npm install`
- Simplest change — just update `awal_binary_path` and `args_prefix`

**Cons**:
- Still requires Node.js on the user's machine
- `node_modules` must be present at runtime
- Not truly bundled — depends on the user's npm install

**Implementation**:
```bash
npm install awal@2.0.3
```
Change `config.rs` default to path to `node_modules/.bin/awal` and remove `args_prefix`.

### Option B: Bundle as Tauri Sidecar (Compiled Node.js Binary)

**Approach**: Use `pkg`, `nexe`, or `bun build --compile` to compile awal + Node.js runtime into a standalone binary, then bundle it as a Tauri sidecar.

**Pros**:
- No Node.js dependency for end users
- Fast startup (no npx overhead)
- Pinned and offline-capable
- Tauri has first-class sidecar support

**Cons**:
- Must compile per-platform (x86_64-apple-darwin, aarch64-apple-darwin, x86_64-unknown-linux-gnu, x86_64-pc-windows-msvc)
- Binary size increase (~40-80MB for Node.js runtime + awal)
- Build complexity — need CI scripts to produce platform binaries
- Must re-compile when updating awal version
- `pkg` is deprecated; `bun build --compile` or `nexe` are alternatives

**Implementation**:
1. Compile: `bun build ./node_modules/awal/bin/awal.js --compile --outfile src-tauri/binaries/awal`
2. Rename with target triple: `awal-aarch64-apple-darwin`
3. Add to `tauri.conf.json`:
   ```json
   { "bundle": { "externalBin": ["binaries/awal"] } }
   ```
4. Update Rust code to use `app.shell().sidecar("awal")` or change `awal_binary_path` to the resolved sidecar path
5. Add shell permissions in `src-tauri/capabilities/default.json`

### Option C: Tauri Node.js Sidecar (Bundle Node Runtime)

**Approach**: Bundle a lightweight Node.js runtime (or Bun runtime) as a sidecar, and ship the awal JS source as a Tauri resource.

**Pros**:
- Smaller than full compilation — share one runtime across updates
- Can update awal JS without recompiling the runtime
- More flexible than Option B

**Cons**:
- Still large (Node.js runtime ~30-50MB, Bun ~40MB)
- More complex resource management
- JS source is readable in the app bundle

### Option D: Direct Binary from Coinbase (Future)

**Approach**: If Coinbase releases a pre-compiled awal binary (Go, Rust, etc.), use it directly as a Tauri sidecar.

**Current status**: awal is a pure Node.js CLI. No standalone binary exists. This option is not currently viable.

## 5. Recommendation

**Short-term (immediate)**: **Option A** — Pin awal as an npm dependency.
- Add `"awal": "2.0.3"` to `package.json`
- Change `awal_binary_path` default from `"npx"` to the resolved binary path
- Remove `"awal@latest"` from `args_prefix`
- This eliminates version drift and reduces startup latency

**Medium-term (before distribution)**: **Option B** — Compile to standalone sidecar.
- Use `bun build --compile` to produce platform-specific binaries
- Bundle as Tauri sidecar with `externalBin`
- Add to CI/CD build matrix for each target platform
- This eliminates the Node.js dependency for end users

## 6. Required Code Changes (for Option A)

### package.json
```json
"dependencies": {
  "awal": "2.0.3",
  ...
}
```

### src-tauri/src/config.rs
Change default `awal_binary_path` or add logic to resolve `node_modules/.bin/awal`.

### src-tauri/src/state/app_state.rs
Change from:
```rust
RealCliExecutor::new(
    &config.awal_binary_path,       // "npx"
    vec!["awal@latest".to_string()], // args prefix
    &config.network,
)
```
To:
```rust
RealCliExecutor::new(
    &config.awal_binary_path,  // "/path/to/node_modules/.bin/awal"
    vec![],                     // no args prefix needed
    &config.network,
)
```

## 7. Required Code Changes (for Option B — Sidecar)

### tauri.conf.json
```json
{
  "bundle": {
    "externalBin": ["binaries/awal"]
  }
}
```

### src-tauri/capabilities/default.json
```json
{
  "permissions": [
    {
      "identifier": "shell:allow-execute",
      "allow": [
        {
          "name": "binaries/awal",
          "sidecar": true,
          "args": true
        }
      ]
    }
  ]
}
```

### Build script (package.json)
```json
"scripts": {
  "build:sidecar": "bun build ./node_modules/awal/bin/awal.js --compile --outfile src-tauri/binaries/awal-$(rustc --print host-tuple)"
}
```

### Rust executor changes
Either use `tauri_plugin_shell::ShellExt` sidecar API, or resolve the sidecar path at startup and pass it to `RealCliExecutor`.
