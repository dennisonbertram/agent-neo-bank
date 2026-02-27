# Documentation Organization Standards

## Folder Structure

```
docs/
├── README.md              # Index of all documentation
├── investigations/        # Research and analysis
│   └── README.md
├── implementation/        # Code changes and features
│   └── README.md
├── testing/              # Test results and validation
│   └── README.md
├── exploration/          # Codebase exploration findings
│   └── README.md
├── decisions/            # Technical decisions (ADRs)
│   └── README.md
└── reference/            # Standards, guides, reference
    └── README.md
```

## Naming Conventions

- Use kebab-case for all filenames: `auth-flow-analysis.md`
- Prefix with date for time-sensitive docs: `2026-02-26-deploy-postmortem.md`
- Use descriptive names that indicate content, not status

## Document Format

Every document should include:

1. **Title** (H1) - Clear, descriptive title
2. **Summary** - 1-3 sentence overview at the top
3. **Body** - Organized with H2/H3 headings
4. **References** - Links to related docs, code, or external resources

## When to Create Documentation

- **Investigations**: When researching a problem, evaluating options, or analyzing behavior
- **Implementation**: When building a feature, documenting architecture, or recording design choices
- **Testing**: When running test suites, validating deployments, or benchmarking
- **Exploration**: When mapping codebases, tracing execution, or understanding systems
- **Decisions**: When making architectural choices that affect the project long-term

## Subagent Output

All subagent work products go into the appropriate `docs/` subfolder. Subagents must:
1. Write findings to a markdown file in the correct folder
2. Return the file path and a brief summary to the top-level agent
3. Never dump large outputs into the conversation context

## Maintenance

- Review and archive stale docs quarterly
- Keep README.md indexes up to date when adding/removing docs
- Remove or update docs that become inaccurate
