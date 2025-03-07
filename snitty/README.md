# Snitty

This is WIP on a Rust linter extension.

* [x] Redundant empty comment lines
* [ ] Comments without space after leader (`//foo`)
* [ ] Comments that aren't wrapped at the correct width, or that break URLs
* [ ] Variables called `result` that aren't `Result`
* [ ] Lines that break `rustfmt`, such as very long string literals
* [ ] Nested blocks that ought to be separate functions
