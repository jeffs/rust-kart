# Cargo-norm

Cargo-norm is a program to convert Rust source file paths to the format
expected by the cargo --bin flag.  This is useful when binding keys in a
text editor to run cargo commands on the currently open file; [e.g., in
Vim][vim].


[vim]: https://github.com/jeffs/geode-profile-home/blob/3b657a2f9b75916eef71202bf644ebce61022f2e/etc/nvim/after/ftplugin/rust.vim#L23-L29
