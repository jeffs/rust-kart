#!/usr/bin/env zsh

set -uo pipefail

declare tmp=$(mktemp -d)
trap "rm -rf $tmp" EXIT

declare a=$tmp/a b=$tmp/b

if [ $(uname) = Linux ]; then
    # Print timestamps of regular files under the current directory.
    ts() { fdfind --type=file --exec-batch stat --format=%Y; }
else
    ts() { fd --type=file --exec-batch stat -f %m; }
fi

clear-run() {
    local run='cargo --color=always --quiet run'
    clear
    cargo --color=always build || return 1
    clear
    echo -e "\e[2m[$(date +%T)] $run" "$@" "\e[22m\n"
    if [ -r stdin ]; then
        ${=run} "$@" < stdin | head -20
    else
        ${=run} "$@" | head -20
    fi
    local st=$?
    echo -ne "\n\e[2m[$(date +%T)] $st\e[22m"
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
