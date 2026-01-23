# Task MCP

An MCP server to prevent agents from losing track of all the dev servers they
spawn, thus burning through port numbers and running `pkill` indiscriminately.

> [!WARNING]
> This server currently has a bug whereby it _also_ loses track of tasks.
> See `./tbd/connection-bug.md`.

## Sample

You may wish to paste something like the following into AGENTS.md or CLAUDE.md:

```markdown
## Process Management

Use the MCP task server for background processes, not raw bash backgrounding
or pkill. This includes dev servers, watch processes, and any long-running
commands.

- `task_ensure` to start (idempotent - safe to call if already running)
- `task_list` to see what's running
- `task_logs` to check output
- `task_stop` to stop cleanly

Never use `pkill`, `kill`, or `&` backgrounding for process management.
```
