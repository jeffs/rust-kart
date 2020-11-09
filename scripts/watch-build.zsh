#!/usr/bin/env zsh

declare tmp=$(mktemp -d)
trap "rm -rf $tmp" EXIT

declare a=$tmp/a b=$tmp/b

# Print timestamps of regular files under the current directory.
ts() {
    fd --type=file --exec-batch stat -f %m
}

clear-build() {
    local run="cargo --color=always build"
    clear
    echo -e "\e[2m[$(date +%T)] $run" "$@" "\e[22m\n" \
        && ${=run} "$@" |& head -20
}

ts >$a
clear-build "$@"

while true; do
    sleep 0.5
    ts >$b
    if ! cmp --silent $a $b; then
        mv $b $a
        clear-build "$@"
    fi
done
