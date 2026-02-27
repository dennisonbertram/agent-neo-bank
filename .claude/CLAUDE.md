# Agent Neo Bank

## Project Overview

Agent Neo Bank is a new project. Update this section as the project takes shape.

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
