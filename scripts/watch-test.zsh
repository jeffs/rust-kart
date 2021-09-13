#!/usr/bin/env zsh

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
    # https://github.com/rust-lang/cargo/issues/1983
    clear
    echo -e "\e[2m[$(date +%T)] cargo test $*\e[22m\n" \
        && cargo --color=always test "$@" -- --color=always |& head -"$((${WATCH_HEIGHT:-$(tput lines)} - 3))"
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
