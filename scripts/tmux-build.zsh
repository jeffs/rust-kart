
#!/usr/bin/env zsh

set -euo pipefail

tmux send-keys -t right 'clear; cargo --color=always build |& head -20' C-m
