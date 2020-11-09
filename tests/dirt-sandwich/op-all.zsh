#!/usr/bin/env zsh
#
# This "Dirt Sandwich" test checks the output of the All operation
# (len::cli::Op::All) applied to some text files, a binary data (the eponymous
# dirt), and the text files again.  This test is also something of a Disaster
# Recovery Test (DiRT), as the program must recover gracefully from bad input:
# Issue a warning then continue as if nothing had gone wrong.

set -euo pipefail

cargo build --quiet "$@"

target/debug/len --color=always \
    tests/data/utf8/* \
    tests/data/bad \
    tests/data/utf8/* >& tests/dirt-sandwich/op-all-$$.out

diff tests/dirt-sandwich/op-all.gold tests/dirt-sandwich/op-all-$$.out

rm tests/dirt-sandwich/op-all-$$.out
