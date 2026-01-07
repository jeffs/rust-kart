//! Converts a Rust source file path to a binary name for use with cargo.

use std::env;
use std::path::Path;
use std::process::exit;

fn find_src_bin_dir_name(mut parts: Vec<&str>) -> Option<&str> {
    while parts.len() > 3 {
        parts.pop();
        if let [.., "src", "bin", dir] = &parts[..] {
            return Some(dir);
        }
    }
    None
}

fn main() {
    // Parse arguments.  If cargo-norm is invoked directly, there should be
    // exactly one argument: a file path.  However, when invoked indirectly as
    // a subcommand (as in "cargo norm"), cargo-norm receives both the
    // subcommand ("norm") and the file path.
    let args: Vec<_> = env::args().collect();
    let args: Vec<_> = args.iter().map(AsRef::as_ref).collect();
    let ([_, file] | [_, "norm", file]) = &args[..] else {
        eprintln!("Usage: cargo-norm FILE");
        exit(2);
    };

    // Convert the argument to an absolute path.
    let file: &Path = file.as_ref();
    let file = match file.canonicalize() {
        Ok(file) => file,
        Err(error) => {
            eprintln!("Error: {}: {}", file.display(), error);
            exit(1);
        }
    };

    // Extract bin name from paths that support simple pattern matching.
    let parts: Vec<_> = file
        .components()
        .filter_map(|s| s.as_os_str().to_str())
        .collect();
    let name = match &parts[..] {
        [.., name, "src", "main.rs"] | [.., name, "src", "bin", "main.rs"] => Some(*name),
        [.., "src", "bin", file] => {
            let path: &Path = file.as_ref();
            path.file_stem().and_then(|s| s.to_str())
        }
        // The following hypothetical pattern match would be lovely, but isn't
        // supported by Rust, probably because it requires a (potentially slow)
        // linear search.  Instead, we perform the search explicitly in
        // find_src_bin_dir_name.
        //
        // [.., "src", "bin", name, ..] => Some(name),
        _ => None,
    };

    if let Some(name) = name {
        println!("{name}");
    } else if let Some(name) = find_src_bin_dir_name(parts) {
        println!("{name}");
    } else {
        eprintln!("Error: {}: can't find bin name", file.display());
        exit(3);
    }
}
