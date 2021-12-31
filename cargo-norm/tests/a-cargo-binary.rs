mod cargo;

use std::process::Command;

const BINARY: &str = "../target/debug/cargo-norm";

#[test]
fn runs() {
    cargo::build().expect("can't build");
    Command::new(BINARY).status().expect("can't run");
}
