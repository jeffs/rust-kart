//! Converts a Rust source file path to a binary name.  For example:




//!     $ 
//!     project/
//!     |- src/
//!     |  `- bin/
//!     |     `- foo/
//!     |        |- bar/
//!     |        |  `- baz.rs
//!     |        `- main.rs
//!     `- Cargo.toml
//!
//!     $ cd project/src/bin
//!     $ cargo-norm foo/bar/baz.rs
//!     foo

use std::env::args;
use std::path::Path;
use std::process::exit;

fn main() {
    let mut parameters = arg5::Parser::new();

    let mut command = String::new();
    parameters.declare_positional("command", &mut command);

    let mut file = String::new();
    parameters.declare_positional("file", &mut file);

    if let Err(err) = parameters.parse(args()) {
        eprintln!("Error: {}", err.what);
        exit(2);
    }

    let file: &Path = file.as_ref();

    // Find the project root.
    // If the file is 

    println!("{}", file.display());
}
