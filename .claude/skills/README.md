# Custom Skills

Place custom Claude Code skills in this directory as `.md` files.

## What Are Skills?

Skills are markdown files that provide specialized knowledge or workflows to Claude Code. They are loaded into context when triggered by matching phrases.

## Creating a Skill

1. Create a `.md` file in this directory
2. Add YAML frontmatter with trigger configuration
3. Write the skill content as markdown instructions

### Example

```markdown
---
description: "Describe when this skill should trigger. Include trigger phrases."
---

# Skill Name

Instructions for Claude Code to follow when this skill is activated.
```

## Frontmatter Fields

| Field | Required | Description |
|-------|----------|-------------|
| `description` | Yes | When to trigger this skill. Include keywords and phrases. |

## Tips

- Keep skills focused on a single task or domain
- Include concrete examples in instructions
- Reference project-specific paths and conventions
- Test skills by using the trigger phrases in conversation
