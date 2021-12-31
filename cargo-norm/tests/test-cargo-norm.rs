mod cargo;

use std::process::{Command, ExitStatus};
use std::str;
use std::sync::Once;

const BINARY: &str = "../target/debug/cargo-norm";

static BUILD_FIXTURE: Once = Once::new();

fn pass(arg: &str) -> (ExitStatus, String) {
    BUILD_FIXTURE.call_once(|| cargo::build().expect("can't build"));
    let output = Command::new(BINARY)
        .arg(arg)
        .output()
        .expect("can't get output");
    let stdout = str::from_utf8(&output.stdout).expect("non-UTF-8 output");
    (output.status, stdout.trim_end().to_string())
}

mod cargo_norm {
    use super::*;

    // Every directory under src, except delta, contains a main.rs file.
    //
    //  tests/data/src
    //  └── bin
    //     ├── alpha.rs
    //     └── beta
    //        ├── gamma.rs
    //        └── delta
    //            └── epsilon.rs
    #[test]
    fn converts_files_to_bin_names() {
        for (want, arg) in [
            ("alpha", "tests/data/src/bin/alpha.rs"),
            ("beta", "tests/data/src/bin/beta/main.rs"),
            ("beta", "tests/data/src/bin/beta/gamma.rs"),
            ("beta", "tests/data/src/bin/beta/delta/epsilon.rs"),
            ("data", "tests/data/src/bin/main.rs"),
            ("data", "tests/data/src/main.rs"),
        ] {
            let (status, got) = pass(arg);
            assert!(status.success());
            assert_eq!(want, got);
        }
    }
}
