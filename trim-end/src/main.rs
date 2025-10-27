//! Removes trailing whitespace from each line in the specified files, in place;
//! or from stdin if no files are specified, writing the results to stdout.

use std::{
    env, fmt,
    path::{Path, PathBuf},
};

use tokio::{
    fs,
    io::{self, AsyncWriteExt},
};

#[derive(Debug)]
enum Error {
    Stdin(io::Error),
    File(PathBuf, io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Stdin(e) => e.fmt(f),
            Self::File(p, e) => write!(f, "{}: {e}", p.display()),
        }
    }
}

/// Copies text from stdin to stdout, one line at a time, removing trailing
/// whitespace from each, and terminating each with a single newline character.
///
/// # Notes
///
/// Unlike [`process_file`], this function does not remove trailing blank lines.
fn process_stdin() -> std::io::Result<()> {
    for line in std::io::stdin().lines() {
        println!("{}", line?.trim_end());
    }
    Ok(())
}

/// Removes trailing whitespace from each line in the file at the specified
/// path, overwriting the file. Removes trailing blank lines entirely, and adds
/// a trailing newline if the file lacked one.
///
/// # Notes
///
/// This function buffers the entire file in RAM. It's probably not worth
/// reading one line at a time, because local, sequential file I/O is so fast
/// these days that string processing might actually be the bottleneck.
async fn process_file(path: &Path) -> io::Result<()> {
    let text = fs::read_to_string(&path).await?;
    let mut lines = text.lines().map(str::trim_end).collect::<Vec<_>>();
    while lines.last().is_some_and(|s| s.is_empty()) {
        lines.pop();
    }
    let mut output = fs::File::create(&path).await?;
    for line in &lines {
        output.write_all(line.as_bytes()).await?;
        output.write_u8(b'\n').await?;
    }
    Ok(())
}

async fn main_imp() -> Vec<Error> {
    let args = env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();

    if args.is_empty() {
        return match process_stdin() {
            Ok(()) => vec![],
            Err(e) => vec![Error::Stdin(e)],
        };
    }

    let mut set = tokio::task::JoinSet::new();
    for path in args {
        set.spawn(async { process_file(&path).await.map_err(|e| Error::File(path, e)) });
    }

    set.join_all()
        .await
        .into_iter()
        .filter_map(Result::err)
        .collect::<Vec<_>>()
}

#[tokio::main]
async fn main() {
    let errors = main_imp().await;
    if !errors.is_empty() {
        for error in errors {
            eprintln!("error: {error}");
        }
        std::process::exit(1);
    }
}
