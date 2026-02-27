# Custom Commands

Place custom Claude Code slash commands in this directory as `.md` files.

## What Are Commands?

Commands are markdown files that define slash commands (e.g., `/my-command`) you can invoke in Claude Code conversations.

## Creating a Command

1. Create a `.md` file in this directory (filename becomes the command name)
2. Add optional YAML frontmatter for arguments
3. Write the command prompt as markdown

### Example

`deploy-check.md`:
```markdown
---
description: "Check deployment readiness"
arguments:
  - name: environment
    description: "Target environment (staging/production)"
    required: true
---

Check if the current codebase is ready to deploy to $ARGUMENTS.environment:

1. Run the test suite
2. Check for hardcoded dev values
3. Verify environment variables
4. Report findings
```

## Frontmatter Fields

| Field | Required | Description |
|-------|----------|-------------|
| `description` | No | Shown in command help |
| `arguments` | No | Named arguments accessible via `$ARGUMENTS.name` |

## Tips

- Use descriptive filenames — they become the `/command-name`
- Keep commands focused on a single workflow
- Reference project-specific conventions from CLAUDE.md
