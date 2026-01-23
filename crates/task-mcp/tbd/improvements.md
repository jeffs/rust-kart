# Improve the MCP Task Server

We have a custom MCP task server for managing background processes (dev servers, watch commands, etc.). It works, but Claude doesn't naturally reach for it over familiar bash patterns like `&` backgrounding and `pkill`.

## Current Tools

- `task_ensure` - Start a background task (idempotent)
- `task_list` - List running tasks
- `task_logs` - View task output
- `task_stop` - Stop a task

## Goal

Make the task server "handier" for Claude - it should be the obvious choice over raw process management.

## Problems Observed

1. Claude defaulted to `pkill -f 'vite'` instead of `task_stop` to kill a dev server
2. Claude used `(cmd &) && sleep 3 && curl ... && pkill` for verification instead of starting a managed task
3. The MCP tool descriptions are minimal and don't compete with Claude's priors about bash

## Ideas to Explore

**Instruction-level fixes:**
- Stronger language in MCP server instructions (directive, not just descriptive)
- Tool descriptions that explicitly call out anti-patterns ("Use this instead of pkill")

**Feature improvements:**
- `task_ensure` could return the URL when it detects a web server started
- `task_restart` for quick iteration
- Health checks / readiness probes
- Better output when a task is already running ("already running on port X")

**Integration improvements:**
- Could the task server detect common patterns (vite, npm run dev, cargo watch) and provide richer feedback?
- Named task presets per-project (e.g., "dev" always means "npm run dev" in this repo)

**Hook/warning approach:**
- A Claude Code hook that detects `&` backgrounding or `pkill`/`kill` in bash commands
- Could warn or block, suggesting the task server instead
- This would train the behavior through friction rather than just documentation
- Tradeoff: might be annoying for legitimate one-off cases, but could have escape hatch

## Questions to Answer

1. Where does the task server code live? Review it first.
2. What's the minimal change to make Claude prefer it?
3. What features would make it genuinely better, not just better-documented?

## Constraint

Keep it simple. This is a dev tool, not a production process manager.
