#!/usr/bin/env -S zsh -euo pipefail

tmux new-window 'cargo check; read'
