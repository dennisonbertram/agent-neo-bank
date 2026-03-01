# Tally Agentic Wallet

## Project Overview

Tally Agentic Wallet is a Tauri v2 desktop app (Rust backend + React frontend) for managing agent wallets with spending policies, approvals, and on-chain transactions.

**Important: The end user never interacts with the CLI or terminal.** This is a GUI-only desktop application. All user-facing flows (onboarding, OTP verification, funding, etc.) happen entirely within the app window. Never reference the CLI, terminal, or command line in any user-facing UI text or copy.

## Tech Stack

<!-- Update once dependencies are chosen -->
- **Framework**: TBD
- **Language**: TypeScript (recommended)
- **Package Manager**: TBD

## Development

### Getting Started

```bash
# Install dependencies
npm install  # or bun install, pnpm install

# Run development server
npm run dev
```

### Project Structure

```
src/           # Application source code
docs/          # Documentation and reports
.claude/       # Claude Code configuration
  skills/      # Custom skills
  commands/    # Custom slash commands
```

## AWAL CLI Dependency

The app depends on `awal` (Coinbase Agent Wallet CLI) for all wallet operations. Currently installed as a pinned npm dependency (`awal@2.0.3`) and invoked via `node_modules/.bin/awal`.

**PRODUCTION TODO**: For distribution, awal must be compiled into a standalone binary using `bun build --compile` and bundled as a Tauri sidecar via `externalBin` in `tauri.conf.json`. This eliminates the Node.js requirement for end users. See `docs/investigations/awal-bundling-investigation.md` for the full plan.

## Code Conventions

- Use TypeScript with strict mode
- Prefer named exports over default exports
- Use async/await over raw promises
- Handle errors at system boundaries (user input, external APIs)
- Keep functions small and focused

## Architecture Rules (NON-NEGOTIABLE)

- **Only architecturally correct implementations.** Never monkeypatch, bypass service layers, or scatter logic across the codebase.
- **Respect the service layer.** Tauri commands call services, services call the CLI executor. Commands never call the CLI directly. If a service method is missing, add it. If a service isn't on AppState, add it.
- **No hardcoded fallback data in the frontend.** If data comes from the backend, it must come from the backend. Placeholder JSON files are for development mocking only and must never be the primary data source in production paths.
- **Follow existing patterns.** Before implementing anything, read how adjacent features are structured and follow the same architecture. Don't invent new patterns when one exists.
- **No stub Tauri commands.** Every `#[tauri::command]` must be wired to its service. Never leave commands returning `{ "status": "not_implemented" }`. If the service exists, wire it. If it doesn't, build it.

### Data Flow (NON-NEGOTIABLE)

The app follows a strict layered data flow. **Every layer must be wired end-to-end. No stubs, no shortcuts.**

```
awal CLI  →  Rust Service (WalletService, AuthService, etc.)
          →  Rust Tauri Command (injects State<AppState>, calls service)
          →  React Zustand Store (single source of truth, initialized once at app level)
          →  React Pages/Components (consume store, never fetch independently)
```

**Rules:**
1. **Backend data is fetched ONCE at the app level.** Wallet data (address, balances) is loaded by `walletStore.initialize()` in `App.tsx` after authentication. Balance polling (15s) lives in the store, not in page components.
2. **Pages never fetch backend data independently.** Pages consume Zustand stores via hooks (`useWalletStore()`, etc.). A page must NEVER call `tauriApi.*` or `safeTauriCall()` for data that a store already provides. If you need data in a page, check if a store provides it first. If no store exists, create one and initialize it at the app level.
3. **One store per domain.** `walletStore` owns address + balances. `authStore` owns auth state. Future stores (agentStore, transactionStore) follow the same pattern: initialized once in `App.tsx`, consumed everywhere.
4. **No loading screens for globally-available data.** If data is fetched at the app level, it's available by the time a protected page renders. Pages should not show "Loading wallet address..." for data the app already has.
5. **Polling lives in stores, not components.** If data needs periodic refresh, the store manages the interval. Components are pure consumers.

### Why This Architecture (Design Rationale)

**Why Zustand stores (not React Query, not Context):**
- The app has a small, well-defined data surface (address, balances, auth state) — not dozens of REST endpoints. Zustand is the right weight.
- React Query / TanStack Query is overkill here. Those libraries shine with many REST endpoints needing complex caching, deduplication, and stale-while-revalidate. We have 2-3 Tauri IPC calls.
- React Context causes unnecessary re-renders. Zustand's selector-based subscriptions avoid this.

**Why poll-based updates (for now):**
- Simple and predictable. The store polls balance every 15s via `setInterval`.
- Sufficient for a desktop wallet where balances change infrequently.

**Future evolution — event-driven updates:**
- Tauri has a built-in event system (`emit`/`listen`) where the Rust backend can push updates to the frontend.
- Instead of polling "what's the balance?", the backend could emit `balance-updated` events when it detects changes.
- This is more efficient and more native to Tauri's architecture.
- Migrate to this when polling becomes a bottleneck or when we need real-time responsiveness (e.g., watching pending transactions).

**What NOT to introduce:**
- Do not add React Query, SWR, or similar data-fetching libraries. Zustand handles our needs.
- Do not add React Context for shared state. Zustand stores are the single mechanism.
- Do not add per-page data fetching for data a store already owns.

## Testing (NON-NEGOTIABLE TDD REQUIREMENT)

**All code in this project follows strict TDD. No exceptions.**

1. **Tests first, always.** Write failing tests BEFORE writing implementation code. Red → Green → Refactor.
2. **No implementation without a test.** Every function, endpoint, and component must have a corresponding test that was written first.
3. **No merges without passing tests.** All tests must pass and coverage thresholds must be met (80% Rust, 70% React).
4. **Test files live next to code.** Rust: `#[cfg(test)] mod tests` inline. React: colocated `*.test.tsx`. Integration: `src-tauri/tests/`.
5. **See `docs/architecture/testing-specification.md`** for the full test plan with 100+ concrete test cases, integration scenarios, fixtures, and CI requirements.

## Documentation

- Keep docs organized in the `docs/` folder structure
- See `docs/reference/documentation-organization.md` for standards

## GitHub Workflow

### Branch Naming
- `feature/xxx` -- new features
- `fix/xxx` -- bug fixes
- `docs/xxx` -- documentation updates

### Commit Message Conventions
- Use imperative mood: "Add feature" not "Added feature"
- Keep the subject line under 72 characters
- Reference issues when applicable: "Fix #12: correct balance calculation"

### PR Workflow
1. Create a branch from `main` using the naming convention above
2. Implement with TDD (tests first, always)
3. Push branch and open a PR using the PR template
4. Request review (or self-review for solo work)
5. Merge to `main` after approval and passing checks

### Labels
- `bug` -- something is broken
- `feature` -- new feature or request
- `enhancement` -- improvement to existing functionality
- `docs` -- documentation updates
- `agent` -- agent-related functionality
- `blocked` -- blocked by external dependency
