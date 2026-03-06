# kart

A Swiss Army knife driver that dispatches to external subcommands written in
any language, with dynamic shell completion support.

## Architecture

```
Tab press → shell → kart --complete -- subcmd args... → subcmd --complete -- args... → stdout
```

The driver sits between the shell's completion system and the individual
subcommand programs. Shells only talk to the driver; subcommands only speak a
simple line-based protocol.

### Subcommand discovery

Subcommands are executable files found in:

1. `~/.kart/commands/` (user-local)
2. `<install-prefix>/libexec/kart/` (alongside the binary)

Files may optionally have a `kart-` prefix, which is stripped to form the
command name (e.g., `kart-deploy` becomes `kart deploy`).

### Completion protocol

When the shell requests completions, the driver calls the subcommand with:

```
subcmd --complete -- arg1 arg2 partial
```

The subcommand prints completions to stdout, one per line. An optional
tab-separated description can follow the value:

```
staging	Staging environment
production	Production environment
dev
```

If the subcommand doesn't support `--complete` (exits non-zero or produces no
output), the driver returns no completions. This means subcommands without
completion support degrade gracefully.

### Shell integration

Generate a completion script and source it:

```sh
# Zsh — add to .zshrc
eval "$(kart completions zsh)"

# Bash — add to .bashrc
eval "$(kart completions bash)"

# Xonsh — add to .xonshrc
execx($(kart completions xonsh))
```

The generated scripts delegate all completion logic back to `kart --complete`,
so the shell never talks to subcommands directly.

## Writing a subcommand

Any executable that handles `--complete -- <args...>` can provide dynamic
completions. Examples:

### Python

```python
#!/usr/bin/env python3
import sys

ENVS = ["staging", "production", "dev"]

if "--complete" in sys.argv:
    sep = sys.argv.index("--")
    args = sys.argv[sep + 1:]
    partial = args[-1] if args else ""
    for env in ENVS:
        if env.startswith(partial):
            print(f"{env}\tDeploy target")
    sys.exit(0)

print(f"deploying to {sys.argv[1]}...")
```

### Shell

```bash
#!/bin/sh
if [ "$1" = "--complete" ]; then
    shift; shift  # skip --complete --
    echo "brief"
    echo "verbose"
    exit 0
fi
echo "everything is fine"
```

## Design notes

- **One protocol**: subcommands return lines with optional `\t` descriptions,
  regardless of which shell is active.
- **Dynamic by default**: completions are computed on each Tab press, so they
  can reflect live state (running containers, git branches, config entries).
- **No framework required**: subcommands don't need clap, argparse, or any
  particular library. The protocol is just argv in, lines out.
- **Graceful degradation**: subcommands that don't implement `--complete`
  simply produce no completions.
