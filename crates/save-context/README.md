# save-context

## What It Does

Manages Claude Code context overhead by enabling/disabling:

- **Skills**: Moves files between `.claude/commands/` and `.claude/commands.disabled/`
- **MCP servers**: Modifies `disabledMcpjsonServers` in `.claude/settings.local.json`

## Commands

```sh
save-context status                    # Show skills + MCP with token estimates
save-context skill list                # List all skills
save-context skill disable <name>...   # Disable skills
save-context skill enable <name>...    # Enable skills
save-context skill disable-all         # Disable all skills
save-context skill enable-all          # Re-enable all disabled skills
save-context mcp list                  # List MCP servers
save-context mcp disable <name>        # Disable MCP server
save-context mcp enable <name>         # Enable MCP server
```

## MCP Server Discovery

Reads from all these locations:
- `.claude/settings.local.json`
- `.claude/settings.json`
- `.mcp.json`
- `~/.claude.json` (user-level, per-project MCP servers)
