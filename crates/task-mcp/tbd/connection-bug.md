# Bug: MCP Task Server - Tasks Disappear After Start

## Summary

When using `task_ensure` to start a background task, the task starts successfully (returns PID, `alive: true`), but then immediately disappears from tracking. Subsequent `task_list` returns empty, and `task_logs` fails with "Connection closed".

## Steps to Reproduce

1. Start a task:
```
mcp__task__task_ensure(name: "vite-dev", command: "npm run dev", cwd: "/path/to/project")
```

2. Response shows success:
```json
{
  "status": "started",
  "task": {
    "name": "vite-dev",
    "pid": 85168,
    "command": "npm run dev",
    "cwd": "/Users/jeff/git/journey/web",
    "alive": true,
    "uptime_secs": 0
  }
}
```

3. Immediately call `task_list`:
```
mcp__task__task_list()
```

4. Returns empty:
```json
{
  "tasks": []
}
```

5. Call `task_logs`:
```
mcp__task__task_logs(name: "vite-dev")
```

6. Fails with:
```
MCP error -32000: Connection closed
```

## Observed Behavior

- The underlying process DOES start and run (verified via `curl http://localhost:5173`)
- The task server loses track of it immediately after spawning
- The "Connection closed" error suggests the MCP connection is dropping

## Expected Behavior

- Task should remain in `task_list` as long as the process is running
- `task_logs` should return the stdout/stderr from the process
- `task_stop` should be able to terminate it by name

## Environment

- macOS (Darwin 25.2.0)
- Claude Code CLI
- Task: Vite dev server (`npm run dev`)

## Possible Causes

1. MCP server connection instability
2. Task tracking state not persisting across MCP calls
3. Race condition between task spawn and state registration
4. The spawned process might be detaching in a way that breaks tracking

## Workaround

The process runs fine - it's just untracked. Currently no way to stop it via task server; requires manual intervention or finding the PID.
