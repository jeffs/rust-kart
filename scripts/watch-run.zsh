#!/usr/bin/env zsh

declare tmp=$(mktemp -d)
trap "rm -rf $tmp" EXIT

declare a=$tmp/a b=$tmp/b

# Print timestamps of regular files under the current directory.
ts() {
    fd --type=file --exec-batch stat -f %m
}

clear-run() {
    local run="cargo --color=always --quiet run"
    clear
    cargo --color=always build \
        && clear \
        && echo "[$(date +%T)] $run" "$@" \
        && echo \
        && cat tests/data/* | ${=run} "$@"
}

ts >$a
clear-run "$@"

while true; do
    sleep 0.5
    ts >$b
    if ! cmp --silent $a $b; then
        mv $b $a
        clear-run "$@"
    fi
done
