#!/usr/bin/env -S zsh -euo pipefail

cd "$(dirname "$(dirname "$0")")"

rm -rf target/doc
RUSTDOCFLAGS='--enable-index-page -Zunstable-options' \
    cargo +nightly doc --document-private-items --no-deps

rm -rf docs
mv target/doc docs
