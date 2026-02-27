# Claude Code Native Worktree Support - Research Findings

**Date**: 2026-02-26
**Status**: Complete
**Sources**: Official Claude Code documentation (code.claude.com), Context7 docs, web search, Boris Cherny announcement (Feb 21, 2026)

---

## Overview

Claude Code shipped built-in git worktree support on **February 21, 2026** (announced by Boris Cherny, Anthropic). This feature allows multiple Claude Code agents to work in parallel on the same repository without interfering with each other. Each agent gets its own isolated worktree (separate working directory and branch) while sharing the same repository history and remote connections.

The feature was already available in the Claude Code Desktop app and has now been brought to the CLI, IDE extensions, web, and mobile app.

---

## How It Works

### CLI Usage

Use the `--worktree` (`-w`) flag when launching Claude Code:

```bash
# Start Claude in a named worktree
claude --worktree feature-auth
# Creates .claude/worktrees/feature-auth/ with branch worktree-feature-auth

# Start another session in a separate worktree
claude --worktree bugfix-123

# Auto-generate a random name (e.g., "bright-running-fox")
claude --worktree
```

### Worktree Location and Branching

- Worktrees are created at `<repo>/.claude/worktrees/<name>`
- Each worktree branches from the **default remote branch**
- The worktree branch is named `worktree-<name>`
- You should add `.claude/worktrees/` to your `.gitignore`

### In-Session Creation

You can also ask Claude to create a worktree during an active session:
- Say "work in a worktree" or "start a worktree"
- Claude will create one automatically via the `EnterWorktree` tool

---

## Subagent Worktree Isolation

### Yes, Subagents Can Run in Isolated Worktrees

This is one of the most powerful aspects of the feature. Subagents (Task agents) support worktree isolation natively.

### Configuration

There are two ways to enable worktree isolation for subagents:

#### 1. Ask Claude at Runtime
Simply tell Claude: "use worktrees for your agents" and it will spawn subagents in isolated worktrees.

#### 2. Custom Subagent Frontmatter (`isolation` field)
Add `isolation: worktree` to a custom subagent's YAML frontmatter:

```markdown
---
name: migration-worker
description: Handles code migration tasks in isolation
isolation: worktree
tools: Read, Write, Edit, Bash, Grep, Glob
model: sonnet
---

You are a code migration specialist. Work on the assigned migration task.
```

### The `isolation` Frontmatter Field

From the official documentation's supported frontmatter fields table:

| Field       | Required | Description |
|-------------|----------|-------------|
| `isolation` | No       | Set to `worktree` to run the subagent in a temporary git worktree, giving it an isolated copy of the repository. The worktree is automatically cleaned up if the subagent makes no changes. |

### Important: No `isolation` Parameter on the Task Tool Itself

The `isolation` parameter is configured on the **subagent definition** (in frontmatter), not passed as a parameter to the Task tool at invocation time. You either:
1. Define `isolation: worktree` in the agent's `.md` file
2. Ask Claude verbally to "use worktrees for your agents"

---

## Worktree Cleanup / Merging Changes Back

### Automatic Cleanup Behavior

When you exit a worktree session, Claude handles cleanup based on whether changes were made:

| Scenario | Behavior |
|----------|----------|
| **No changes** | Worktree and its branch are removed automatically |
| **Changes or commits exist** | Claude prompts you to keep or remove the worktree |
| **Keep** | Preserves the directory and branch so you can return later |
| **Remove** | Deletes the worktree directory and its branch, discarding all uncommitted changes and commits |

### Subagent Worktree Cleanup

For subagents with `isolation: worktree`:
- The worktree is **automatically cleaned up** if the subagent makes no changes
- If changes were committed, they remain on the worktree branch for review/merge

### Merging Changes Back

The documentation does **not** describe an automatic merge mechanism. The workflow is:
1. Subagent works in its worktree on branch `worktree-<name>`
2. Subagent commits changes to that branch
3. You (or a coordinating agent) merge the branch back to the target branch using standard git operations (`git merge`, `git rebase`, or PR creation)
4. Clean up the worktree manually or let Claude prompt you

For manual worktree management outside Claude sessions:
```bash
git worktree list        # See all worktrees
git worktree remove <path>  # Clean up a specific worktree
```

---

## Workflow for Parallel Agents in Separate Worktrees

### Pattern 1: Multiple CLI Sessions

```bash
# Terminal 1: Auth feature
claude --worktree feature-auth

# Terminal 2: Bug fix
claude --worktree bugfix-123

# Terminal 3: Documentation
claude --worktree docs-update
```

Each session has full, isolated access to the codebase. Agent A can rewrite `src/auth.ts` while Agent B rewrites the same file with a different approach.

### Pattern 2: Subagent Swarm with Worktree Isolation

Define agents with `isolation: worktree` and have a coordinator spawn them:

```markdown
---
name: migration-worker
description: Handles file migration tasks
isolation: worktree
tools: Read, Write, Edit, Bash, Grep, Glob
---
```

The coordinator agent spawns multiple migration workers, each in their own worktree, working in parallel without conflicts.

### Pattern 3: Agent Teams + Worktrees

For sustained parallelism with inter-agent communication, combine agent teams with worktrees:
- Each team member gets its own worktree
- Team members communicate via the messaging system
- Task lists coordinate work assignment
- Each agent can build, test, and modify files simultaneously

### Key Benefit Over Non-Worktree Parallel Agents

**Without worktree isolation**: Parallel subagents are limited to reading files or writing to non-overlapping paths. Two agents editing the same file will conflict.

**With worktree isolation**: Each agent has the entire codebase to itself. Multiple agents can modify the same files independently. This is "especially powerful for large batched changes and code migrations."

---

## Combining with Other Flags

```bash
# Worktree + tmux (launch in its own tmux session)
claude --worktree feature-auth --tmux

# Useful for fire-and-forget parallel work
```

---

## Non-Git Version Control Support

For SVN, Perforce, Mercurial, etc., you can configure `WorktreeCreate` and `WorktreeRemove` hooks to provide custom worktree creation and cleanup logic. When configured, these hooks replace the default git behavior when you use `--worktree`.

---

## Limitations and Gotchas

### 1. Environment Setup Required Per Worktree
Each new worktree needs its own development environment initialized:
- `npm install` / `yarn` / `bun install`
- Virtual environment setup (Python)
- Any project-specific setup steps

This can add overhead, especially for projects with heavy dependencies.

### 2. No Automatic Merge
Changes from worktree branches are **not** automatically merged back. You must handle merge/rebase yourself. There is no built-in conflict resolution for worktree branches.

### 3. Subagents Cannot Spawn Sub-Subagents
Subagents cannot spawn other subagents. This limits nesting depth. If a worktree subagent needs to delegate, it cannot.

### 4. Disk Space
Each worktree is a separate copy of the working directory (though git shares objects internally). Many worktrees on large repos can consume significant disk space.

### 5. Gitignore Configuration
You should add `.claude/worktrees/` to `.gitignore` to prevent worktree contents from appearing as untracked files in the main repository.

### 6. Session Resumption
Sessions are stored per project directory. The `/resume` picker shows sessions from the same git repository, **including worktrees**, which helps find worktree sessions later.

### 7. Background Subagent Permission Model
Background subagents (which worktree agents often are) auto-deny any permissions not pre-approved before launch. If a background worktree agent hits a permission it needs, that tool call fails but the agent continues. You can resume it in the foreground to retry.

### 8. Context Window Limits
When subagents complete, their results return to the main conversation. Running many worktree subagents that each return detailed results can consume significant context. For tasks exceeding the context window, agent teams are recommended over subagents.

---

## Summary Table

| Feature | Support |
|---------|---------|
| CLI flag | `--worktree` / `-w` |
| Named worktrees | Yes (`claude -w my-name`) |
| Auto-named worktrees | Yes (`claude -w`) |
| Subagent isolation | Yes (`isolation: worktree` in frontmatter) |
| Runtime request | Yes ("use worktrees for your agents") |
| In-session creation | Yes ("start a worktree") |
| Auto-cleanup (no changes) | Yes |
| Auto-cleanup (with changes) | Prompted |
| Auto-merge | No (manual git merge required) |
| Non-git VCS | Via WorktreeCreate/WorktreeRemove hooks |
| tmux integration | Yes (`--tmux` flag) |
| Agent teams compatibility | Yes |

---

## References

- [Official Claude Code Worktree Docs](https://code.claude.com/docs/en/common-workflows) - "Run parallel Claude Code sessions with Git worktrees"
- [Official Subagent Docs](https://code.claude.com/docs/en/sub-agents) - `isolation` frontmatter field
- [Boris Cherny Announcement (X/Twitter)](https://x.com/bcherny/status/2025007393290272904) - Feb 21, 2026
- [Boris Cherny Thread (Threads)](https://www.threads.com/@boris_cherny/post/DVAAnexgRUj) - Usage details
- [AI Engineer Guide](https://aiengineerguide.com/blog/claude-code-git-worktree/) - Feature overview
- [Geeky Gadgets](https://www.geeky-gadgets.com/claude-code-worktree-support/) - Setup and limits
- [incident.io Blog](https://incident.io/blog/shipping-faster-with-claude-code-and-git-worktrees) - Real-world usage
- [Git Worktree Documentation](https://git-scm.com/docs/git-worktree)
