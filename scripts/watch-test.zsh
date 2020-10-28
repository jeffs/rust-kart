#!/usr/bin/env zsh

declare tmp=$(mktemp -d)
trap "rm -rf $tmp" EXIT

declare a=$tmp/a b=$tmp/b

# Print timestamps of regular files under the current directory.
ts() {
    fd --type=file --exec-batch stat -f %m
}

clear-test() {
    local run="cargo --color=always --quiet run"
    clear
    cargo --color=always test
}

ts >$a
clear-test

while true; do
    sleep 0.5
    ts >$b
    if ! cmp --silent $a $b; then
        mv $b $a
        clear-test
    fi
done
