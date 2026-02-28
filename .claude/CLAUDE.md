# Agent Neo Bank

## Project Overview

Agent Neo Bank is a Tauri v2 desktop app (Rust backend + React frontend) for managing agent wallets with spending policies, approvals, and on-chain transactions.

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
