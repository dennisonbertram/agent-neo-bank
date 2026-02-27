# Worktree Setup Optimization on macOS (APFS)

**Date**: 2026-02-26
**Status**: Research complete
**Platform**: macOS Darwin 24.1.0 / APFS

---

## Summary

The fastest safe strategy for macOS worktree setup is:

1. **node_modules**: Use `cp -cR` (APFS copy-on-write clone) — nearly instant, safe, fully isolated
2. **.env files**: Copy, never symlink — isolation is the whole point of worktrees
3. **Build artifacts** (.next/, dist/, build/): Regenerate, do not copy
4. **pnpm projects**: Run `pnpm install --frozen-lockfile` — already uses a content-addressable store, install is fast and produces correct symlinks
5. **Wire everything up** with a `WorktreeCreate` hook in `.claude/settings.json`

---

## 1. node_modules Symlinking

### How it works
Symlink the entire `node_modules` directory from the main checkout into the worktree:
```bash
ln -s /main/node_modules /worktree/node_modules
```

### Pros
- Zero disk cost
- Nearly instant setup

### What breaks
Node.js resolves `__dirname` via `fs.realpathSync()` before returning it to packages. This means `__dirname` inside a package always resolves to the **real path** through the symlink target, not the worktree path.

**Observed behavior** (tested):
```
realpath:     /private/.../main/node_modules/somepackage
symlink path: /.../worktree/node_modules/somepackage
Same? false
```

This causes problems for:
- **Packages that use `__dirname` to locate sibling files** (many build tools: webpack, esbuild, rollup plugins)
- **Native addons (`.node` files)**: dlopen'd with absolute paths derived from `__dirname`; two worktrees running different versions will conflict
- **Packages that write state relative to their install location** (e.g., some caches, prisma schema resolution)
- **`require.resolve()` calls** that walk up from `__dirname`
- **Workspace-aware tools** (Turborepo, Nx, Lerna) that check `node_modules` location relative to the workspace root

### Verdict
**Do not symlink node_modules.** The isolation breaks in subtle and hard-to-debug ways. The speed gain is not worth it when `cp -cR` is nearly as fast (see below).

---

## 2. APFS Clonefile / `cp -c`

### How it works
APFS (Apple File System) supports **copy-on-write (COW) cloning** via `clonefile(2)`. The macOS `cp` command exposes this via the `-c` flag. When you clone a file, both the original and clone initially share the same disk blocks. Only when one is modified does the OS allocate new blocks for the modified pages.

### Benchmark results (tested on APFS home volume)

| Method | Time (500 packages, ~2MB) | Time (single 50MB file) |
|--------|--------------------------|------------------------|
| `cp -cR` (APFS clone) | **37ms** | **4ms** |
| `cp -R` (regular copy) | 199ms | 17ms |
| `cp -al` (hardlinks) | 235ms | ~4ms (but see below) |

**cp -cR is 3-5x faster than cp -R for many-small-files workloads**, which is exactly what node_modules is.

### Key properties
- **Instant for large files**: A 50MB file clones in 4ms regardless of size — it's a metadata operation
- **COW isolation**: Modifying the clone (e.g., `npm rebuild`) does not affect the original
- **Works on APFS volumes**: Requires both source and destination to be on the same APFS volume (standard for macOS home directories)
- **Disk usage**: Both files appear as full size in `du` output, but the OS only stores unique blocks once until copy-on-write occurs
- **Fallback**: If not on APFS (e.g., network share, HFS+), `cp -c` falls back to a regular copy silently

### Usage
```bash
# Clone node_modules into a new worktree
cp -cR /path/to/main/node_modules /path/to/worktree/node_modules
```

### Caveats
- Does **not** work across volumes (e.g., copying from an external drive)
- `/var/folders` (macOS temp dir) is not APFS — test on actual project directories
- Native modules may need `npm rebuild` after cloning if they contain paths baked into binaries

### Verdict
**Preferred approach for npm/bun projects.** Fast, safe, fully isolated.

---

## 3. Hardlinks (`cp -al`)

### Does `cp -al` work on macOS?
Yes, `cp -al` works on macOS (tested). The `-a` flag preserves attributes, `-l` creates hardlinks instead of copies.

### Why hardlinks are worse than COW clones for node_modules

| Property | Hardlinks (`cp -al`) | COW Clone (`cp -cR`) |
|----------|---------------------|---------------------|
| Setup speed | Slower (235ms) | Faster (37ms) |
| Write isolation | **None** — modification in worktree modifies original | Full — writes diverge at page level |
| Cross-directory safety | No — same inode means same file | Yes — distinct inodes |
| Directories | Cannot hardlink directories | Full recursive clone |

Hardlinks cannot span directories for the directory node itself — `cp -al` creates hardlinks for the **files inside** the directory tree, but the directories themselves are new. This means:
- Adding a file in the worktree's `node_modules` does NOT appear in the main checkout (directory entries differ)
- **But modifying an existing file** (`npm rebuild`, patching) **changes the file in both locations** because they share an inode

### Verdict
**Do not use hardlinks for node_modules.** The write-aliasing behavior is a correctness hazard. COW clones are faster anyway.

---

## 4. pnpm and bun Considerations

### pnpm
pnpm uses a **content-addressable store** at `~/.pnpm/store/v10` (this machine's location confirmed). The `node_modules` in a pnpm project contains symlinks into the `.pnpm` virtual store, which itself symlinks into the global store.

```
node_modules/
  express -> .pnpm/express@4.18.2/node_modules/express
  .pnpm/
    express@4.18.2/node_modules/
      express/ -> ~/.pnpm/store/v10/.../express/
```

**Cloning a pnpm node_modules with `cp -cR` breaks the symlink structure** — the clone will contain copies of symlinks pointing back to `.pnpm` in the original location, not a new self-contained tree. The `.pnpm` virtual store contains yet more symlinks.

**Best approach for pnpm worktrees**:
```bash
cd /path/to/worktree
pnpm install --frozen-lockfile
```
pnpm installs are fast because packages are already in the global store — it just creates symlinks. A typical pnpm install in a project with a warm store takes 2-10 seconds. No file data is copied.

**Do not `cp -cR` a pnpm node_modules.** The symlink graph will be broken.

### bun
bun has a global cache similar to pnpm and installs are fast (often sub-second for warm caches). bun's `node_modules` layout is more like npm's (real files, not symlinks), so `cp -cR` works correctly.

```bash
# For bun projects — clone is fine
cp -cR /path/to/main/node_modules /path/to/worktree/node_modules
# OR just reinstall (also fast)
cd /path/to/worktree && bun install --frozen-lockfile
```

### npm / yarn (classic)
Standard `node_modules` with real files. `cp -cR` is the correct and fast approach.

---

## 5. .env Files

### Symlink vs Copy

**Symlinking `.env`**:
```bash
ln -s /main/.env /worktree/.env
```
This means changes to `.env` in one worktree affect all worktrees. This defeats the purpose of isolation — if a worktree needs a different `DATABASE_URL`, `PORT`, or feature flag, you cannot set it without affecting other worktrees.

**Copying `.env`**:
```bash
cp /main/.env /worktree/.env
```
Each worktree gets its own copy. Changes are isolated. This is almost always the correct behavior.

### Verdict
**Always copy `.env` files, never symlink them.** The typical reason for a worktree is to run a parallel instance or test a different configuration — sharing environment variables undermines this.

**Recommended `.env` copy strategy**:
```bash
# Copy all .env variants
for f in .env .env.local .env.development .env.test; do
  [ -f "/main/$f" ] && cp "/main/$f" "/worktree/$f"
done
```

---

## 6. Build Artifacts (.next/, dist/, build/)

### Should these be copied or regenerated?

**Do not copy build artifacts to worktrees.** Reasons:

1. **Stale artifacts**: The worktree may be on a different branch with different source. A copied `.next/` from main will be wrong.
2. **Build tools embed absolute paths**: Next.js, Vite, and webpack embed absolute paths and content hashes into build outputs. A copy from a different directory will have mismatched paths.
3. **Large size**: `.next/` can be 200MB+. Even with COW cloning, this adds metadata overhead.
4. **Fast to regenerate**: `next build` / `vite build` with a warm dependency cache is typically 15-60 seconds — much faster than debugging artifacts from a wrong branch.
5. **Dev mode doesn't need them**: For `next dev` or `vite dev`, no build artifacts are needed at all.

### Verdict

| Artifact | Strategy | Reason |
|----------|----------|--------|
| `.next/` | Regenerate | Branch-specific, embeds paths |
| `dist/` | Regenerate | Branch-specific, stale risk |
| `build/` | Regenerate | Branch-specific, stale risk |
| `.turbo/` | Skip | Turborepo uses its own caching |
| `node_modules/.cache/` | Optionally clone | Build caches (babel, webpack) are safe to COW clone |

---

## 7. Claude Code WorktreeCreate Hooks

Claude Code version 2.1.50+ supports `WorktreeCreate` and `WorktreeRemove` hook events. These hooks **replace the default git worktree behavior entirely** when present.

### How WorktreeCreate works

When `--worktree` is passed to `claude` or a subagent uses `isolation: "worktree"`, Claude Code:
1. Fires the `WorktreeCreate` hook (if configured)
2. The hook script receives JSON on stdin with `name` (a slug like `"feature-auth"`)
3. The hook must print the **absolute path** to the created worktree directory on stdout
4. Claude Code uses that path as the working directory for the isolated session
5. A non-zero exit code causes worktree creation to fail

### WorktreeCreate input (stdin JSON)
```json
{
  "session_id": "abc123",
  "transcript_path": "/Users/.../.claude/projects/.../session.jsonl",
  "cwd": "/Users/myproject",
  "hook_event_name": "WorktreeCreate",
  "name": "feature-auth"
}
```

### WorktreeCreate output
The hook prints only the absolute path to stdout:
```
/Users/myproject/.claude/worktrees/feature-auth
```

### WorktreeRemove
Fires when Claude Code removes a worktree (session exit or subagent finish). Receives `worktree_path` in the JSON input. Cannot block removal but can perform cleanup. Failures are logged in debug mode only.

```json
{
  "hook_event_name": "WorktreeRemove",
  "worktree_path": "/Users/myproject/.claude/worktrees/feature-auth"
}
```

### Hook configuration location
Hooks can be defined in:
- `~/.claude/settings.json` — all projects (not committed)
- `.claude/settings.json` — this project (committable)
- `.claude/settings.local.json` — this project (gitignored)

### Important behavior notes
- `WorktreeCreate` and `WorktreeRemove` do **not** support matchers — they always fire
- Only `type: "command"` hooks are supported for these events (not prompt or agent hooks)
- Hook scripts must redirect non-path output to stderr to avoid corrupting the path output

---

## 8. Recommended Approach for macOS

### Decision matrix by package manager

| Package Manager | node_modules strategy | Speed |
|----------------|----------------------|-------|
| npm / bun | `cp -cR` (APFS clone) | ~40ms for typical project |
| pnpm | `pnpm install --frozen-lockfile` | 2-10s (warm store) |
| yarn (classic) | `cp -cR` | ~40ms |
| yarn (PnP) | No node_modules; clone `.yarn/cache` | Fast |

### Recommended WorktreeCreate hook script

Place at `.claude/hooks/worktree-create.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail

# Read the worktree name from JSON input
NAME=$(jq -r '.name' < /dev/stdin)
MAIN_DIR=$(pwd)  # or use a fixed project root
WORKTREE_BASE="${MAIN_DIR}/.claude/worktrees"
WORKTREE_DIR="${WORKTREE_BASE}/${NAME}"

mkdir -p "$WORKTREE_BASE"

# 1. Create git worktree (default behavior we're augmenting)
git worktree add "$WORKTREE_DIR" HEAD >&2

# 2. Handle node_modules
PKG_MANAGER="npm"  # detect from lockfile
if [ -f "${MAIN_DIR}/pnpm-lock.yaml" ]; then PKG_MANAGER="pnpm"; fi
if [ -f "${MAIN_DIR}/bun.lockb" ]; then PKG_MANAGER="bun"; fi

if [ "$PKG_MANAGER" = "pnpm" ]; then
  # pnpm: reinstall using global store (fast with warm cache)
  (cd "$WORKTREE_DIR" && pnpm install --frozen-lockfile) >&2
else
  # npm/bun/yarn: APFS COW clone (instant)
  if [ -d "${MAIN_DIR}/node_modules" ]; then
    cp -cR "${MAIN_DIR}/node_modules" "${WORKTREE_DIR}/node_modules" >&2
  fi
fi

# 3. Copy .env files (never symlink)
for envfile in .env .env.local .env.development .env.test .env.production; do
  if [ -f "${MAIN_DIR}/${envfile}" ]; then
    cp "${MAIN_DIR}/${envfile}" "${WORKTREE_DIR}/${envfile}" >&2
  fi
done

# 4. Print the path (required — this is what Claude Code reads)
echo "$WORKTREE_DIR"
```

### Recommended WorktreeRemove hook script

Place at `.claude/hooks/worktree-remove.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail

WORKTREE_PATH=$(jq -r '.worktree_path' < /dev/stdin)

# Remove the git worktree (prune the reference)
git worktree remove --force "$WORKTREE_PATH" >&2 || true

# Belt-and-suspenders cleanup
rm -rf "$WORKTREE_PATH" >&2 || true
```

### Register the hooks in `.claude/settings.json`

```json
{
  "hooks": {
    "WorktreeCreate": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "\"$CLAUDE_PROJECT_DIR\"/.claude/hooks/worktree-create.sh",
            "timeout": 120,
            "statusMessage": "Setting up worktree..."
          }
        ]
      }
    ],
    "WorktreeRemove": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "\"$CLAUDE_PROJECT_DIR\"/.claude/hooks/worktree-remove.sh",
            "timeout": 30
          }
        ]
      }
    ]
  }
}
```

---

## Performance Summary

| Operation | Time | Method |
|-----------|------|--------|
| Clone 2MB node_modules (500 pkgs) | ~37ms | `cp -cR` |
| Clone 1GB node_modules (estimate) | ~200-400ms | `cp -cR` (metadata-bounded) |
| pnpm install (warm store) | 2-10s | `pnpm install --frozen-lockfile` |
| Copy .env | <1ms | `cp` |
| git worktree add | ~100ms | `git worktree add` |
| **Total for npm project** | **~200ms** | git + cp -cR + cp |
| **Total for pnpm project** | **3-12s** | git + pnpm install |

---

## References

- [APFS Reference: clonefile(2)](https://developer.apple.com/documentation/kernel/1645397-clonefile)
- [Claude Code Hooks Reference](https://code.claude.com/docs/en/hooks) — WorktreeCreate added in v2.1.50
- [Claude Code Changelog](https://code.claude.com/docs/en/changelog) — v2.1.50: "Added WorktreeCreate and WorktreeRemove hook events"
- Tested on: macOS Darwin 24.1.0, APFS, Node.js v22.21.1, pnpm 9.x, bun 1.3.5
