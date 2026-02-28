# Phase 4: Polish & Production

```
Implement Phase 4 (Polish & Production) of Tally Agentic Wallet using the swarm skill.

## Context
- Architecture plan: docs/architecture/architecture-plan.md (Section 12, Phase 4)
- Testing spec: docs/architecture/testing-specification.md
- Phases 1a, 1b, 2, 2.5, 3 complete. Full send/receive/earn lifecycle working. Coinbase Onramp, chain monitoring, Unix socket, spending analytics, transaction export all functional. CI pipeline enforcing coverage.

## Important: Write Phase 4 Test Cases First
The FIRST task is writing test cases for Phase 4 components (new sections in testing-specification.md) BEFORE any implementation begins.

## Phase 4 Tasks

### Wave 1 (parallelize — write tests + independent work)

1. **Write Phase 4 test cases** (`docs/architecture/testing-specification.md`)
   - Add Section 3.19: Token rotation tests (new token generated, old token invalidated, agent must re-auth)
   - Add Section 3.20: Error recovery tests (CLI crash retry, WebSocket reconnect, stale session recovery)
   - Add Section 3.21: Auto-updater tests (check for update, download, install prompt)
   - Add Section 3.22: Backup/restore tests (export DB, import DB, data integrity verification)
   - Add Section 3.23: Onboarding tour tests (step progression, skip, completion state)
   - Add Section 3.24: Agent platform auto-discovery tests (platform detection, skill installation, re-scan on focus, skip/manual configure)

2. **Dark mode** (`src/`, `tailwind.config.ts`)
   - System-aware theme toggle (light/dark/system)
   - Toggle in Settings page header or app titlebar
   - shadcn/ui already supports dark mode — wire up the CSS variables
   - Persist preference to app_config
   - Tests: toggle switches theme, system preference respected, persistence works

3. **Keyboard shortcuts** (`src/hooks/useKeyboardShortcuts.ts`, `src/components/CommandPalette.tsx`)
   - Cmd+K: open command palette
   - Command palette: search across agents, transactions, settings, actions
   - Actions: "New invitation code", "Toggle kill switch", "Export transactions", "Go to agent X"
   - Esc: close any modal/dialog
   - Tests: Cmd+K opens palette, search filters results, action executes, Esc closes

### Wave 2 (depends on Wave 1 test cases)

4. **Token rotation** (`src-tauri/src/core/agent_registry.rs`, `src/pages/AgentDetail.tsx`)
   - "Rotate token" button on agent detail page
   - Generates new token, invalidates old one immediately
   - Agent must re-authenticate with new token
   - Confirmation dialog: "This will disconnect the agent until it uses the new token"
   - New token shown once (same delivery model as initial token — but displayed in UI since user initiated)
   - Tests: Section 3.19

5. **Error recovery** (multiple modules)
   - CLI wrapper: retry with exponential backoff on transient failures (timeout, connection refused)
   - Max 3 retries, then fail permanently
   - WebSocket to Transaction Monitor: auto-reconnect with backoff
   - Stale auth session: detect expired OTP session, prompt re-auth in UI
   - Tests: Section 3.20

6. **Agent performance metrics** (`src/pages/AgentDetail.tsx`, `src/components/dashboard/`)
   - ROI tracking per agent: total earned vs total spent
   - Net position indicator (positive = earning, negative = spending)
   - Spending velocity: average per day/week
   - Top services used by this agent
   - Tests: ROI calculation, handles agents with no earnings, velocity with sparse data

7. **Spending analytics dashboard** (`src/pages/Dashboard.tsx`)
   - Time-series chart: daily spending over last 30/90 days
   - Comparison: this week vs last week
   - Top agents by spend (leaderboard)
   - Top services by spend
   - Budget burn rate: at current velocity, when will global budget run out?
   - Tests: renders charts, handles zero data, time range switching, burn rate calculation

### Wave 3 (depends on Wave 2)

8. **Backup/restore** (`src-tauri/src/commands/backup.rs`, `src/pages/Settings/Backup.tsx`)
   - Export: copy SQLite DB file to user-chosen location via Tauri file dialog
   - Include metadata: app version, export timestamp, agent count, transaction count
   - Import: user selects backup file, app validates schema version, confirms overwrite, restarts
   - Tests: Section 3.22

9. **Auto-updater** (`src-tauri/`, Tauri updater plugin)
   - Check for updates on app launch (non-blocking)
   - If update available: show banner in UI with version notes
   - "Update now" button downloads and installs
   - Uses GitHub Releases as update source
   - Tests: Section 3.21

10. **Onboarding tour** (`src/components/OnboardingTour.tsx`)
    - First-launch detection (flag in app_config)
    - Step-by-step overlay highlighting: balance display, agent list, invitation codes, settings
    - Skip button, progress dots
    - Marked complete in app_config so it doesn't show again
    - "Show tour again" button in Settings
    - Tests: Section 3.23

11. **Agent platform auto-discovery & skill installation** (`src-tauri/src/core/platform_discovery.rs`, `src-tauri/src/commands/platform_discovery.rs`, `src/components/onboarding/PlatformDiscovery.tsx`)
    - **Platform scanner:** detect installed AI agent platforms on the user's machine:
      - Claude Code: check for `~/.claude/` directory and `~/.claude/skills/` directory
      - Codex: check for codex CLI installation (`which codex`), config directories
      - Other agents: extensible registry of known agent platforms with skill/plugin systems
    - **Skill auto-installer:** for each discovered platform, install the Tally Agentic Wallet skill:
      - Claude Code: copy `tally-agentic-wallet.md` skill file into `~/.claude/skills/` (global) or project-level `.claude/skills/`
      - Codex: install equivalent configuration into codex config directory
      - Skill file tells agents how to discover and use the local API (REST endpoint, auth flow, available commands)
    - **First-launch UI step** (integrated into onboarding tour or shown post-onboarding):
      - "We found Claude Code installed on your machine"
      - "We've installed the Tally Agentic Wallet skill so your agents know how to use your wallet"
      - List of discovered platforms with install status (installed / skipped / failed)
      - "Skip" button and "Configure manually later" option in Settings
    - **Periodic re-scan:**
      - Re-check for new agent platform installations on app focus (`tauri::WindowEvent::Focused`)
      - If new platform discovered: show non-intrusive toast notification prompting skill install
      - Settings page section: "Connected Agent Platforms" — view discovered platforms, manually trigger re-scan, install/uninstall skills
    - **Persistence:** store discovered platforms and install status in `app_config` table
    - Tests: Section 3.24 — platform detection (present/absent), skill file installation (success/permission denied), re-scan triggers, skip flow, manual configure flow

12. **Production network support** (`src/pages/Settings/Network.tsx`)
    - Enhanced mainnet toggle with safety checklist:
      - "I understand this uses real funds" checkbox
      - "I have set spending limits" checkbox
      - "I have tested on testnet" checkbox
    - All three required before mainnet can be enabled
    - Visual indicator in app header when on mainnet (e.g., green dot = testnet, orange dot = mainnet)
    - Tests: checklist enforcement, indicator display, persistence

### Wave 4 (depends on Wave 3)

13. **Performance optimization**
    - SQLite query optimization: add indexes on frequently queried columns (agent_id, status, created_at, period)
    - EXPLAIN ANALYZE on slow queries, optimize
    - Frontend lazy loading: React.lazy for Settings, AgentDetail, Fund pages
    - Virtual scrolling for long transaction lists (if > 1000 rows)
    - Tests: query performance benchmarks, lazy loading renders correctly

14. **Production Transaction Monitor deployment**
    - Dockerfile for the Transaction Monitor Service
    - Railway deployment config (or similar)
    - Environment variables: ALCHEMY_API_KEY, WEBSOCKET_PORT, DATABASE_URL
    - Health check endpoint
    - Desktop app config to point at production monitor URL (vs localhost for dev)
    - Tests: health check, Docker build succeeds

15. **Final integration tests + E2E**
    - Token rotation E2E: rotate in UI → old token rejected → new token works
    - Backup/restore E2E: export → delete data → import → data restored
    - Onboarding tour E2E: first launch → tour shows → complete → doesn't show again
    - Mainnet toggle E2E: checklist → enable → indicator changes
    - Command palette E2E: Cmd+K → search → execute action
    - Platform auto-discovery E2E: first launch detects platforms → skill installed → re-scan on focus detects new platform → toast prompt
    - Full regression: re-run all Phase 2.5 integration scenarios to confirm nothing broke

16. **Update CI pipeline**
    - Add Docker build step for Transaction Monitor
    - Add performance benchmark job (fail if query time regresses > 20%)
    - Final coverage check: ensure 80% Rust, 70% React maintained
    - Add Playwright E2E for all new UI (dark mode, command palette, onboarding, backup, token rotation, platform discovery)

## Rules
- STRICT TDD: Wave 1 Task #1 (write test cases) must complete before ANY implementation starts.
- All other TDD rules apply: tests first for every module.
- Use worktree isolation for all code-writing agents.
- Commit after each passing wave.
- Dark mode must not break any existing Playwright E2E tests — update selectors if needed.
- Performance benchmarks are assertions, not just measurements. Set baselines and fail on regression.
- The final integration test wave re-runs ALL previous scenarios. Nothing is allowed to regress.

## Phase 4 Deliverable
Production-ready desktop app: dark mode, command palette, token rotation, error recovery with retry, agent ROI metrics, spending analytics, backup/restore, auto-updater, onboarding tour, agent platform auto-discovery & skill installation, mainnet safety checklist, performance optimized, Transaction Monitor deployed to production. Full E2E coverage. CI pipeline complete. Ready to ship.
```
