# Rust Kart

A workspace of small utilities written in Rust.  Aside from the Len package, no
part of Rust Kart has any third-party build dependencies.  Some commands, like
git-prune and tmux-send, launch third-party subcommands (such as git and tmux,
respectively) at runtime.  Once the Arg5 library (./arg5) is sufficiently
mature, the dependency on Clap can be removed.

    arg5        -- argument parsing library
    b2c         -- decodes ASCII encoded binary
    cargo-norm  -- converts Rust source file paths to cargo --bin arguments
    git-prune   -- deletes obsolete Git branches
    len         -- sorts lines by length
    pangram     -- makes as many words as possible from specified letters
    tmux-send   -- sends text to the bottom-right tmux pane

The scripts directory holds shell scripts useful in the development of
rust-kart itself.
