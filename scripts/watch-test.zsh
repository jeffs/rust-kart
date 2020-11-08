#!/usr/bin/env zsh

declare tmp=$(mktemp -d)
trap "rm -rf $tmp" EXIT

declare a=$tmp/a b=$tmp/b

# Print timestamps of regular files under the current directory.
ts() {
    fd --type=file --exec-batch stat -f %m
}

clear-run() {
    # https://github.com/rust-lang/cargo/issues/1983
    local run="cargo --color=always test --color=always -- --color=always"
    clear
    echo -e "\e[2m[$(date +%T)] $run" "$@" "\e[22m\n" \
        && ${=run} "$@" |& head -32
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
