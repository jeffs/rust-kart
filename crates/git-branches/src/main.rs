use std::{env, ffi, process::ExitCode};

const USAGE: &str = "usage: git-branches [rebase BRANCH]";

#[tokio::main]
async fn main() -> ExitCode {
    let mut args = env::args_os();
    args.next(); // Skip program name.

    match args.next().as_deref().and_then(ffi::OsStr::to_str) {
        None => show().await,
        Some("rebase") => rebase(args).await,
        Some(_) => {
            eprintln!("{USAGE}");
            ExitCode::from(2)
        }
    }
}

async fn show() -> ExitCode {
    match git_branches::topology().await {
        Ok(topology) => {
            print!("{}", git_branches::render(&topology));
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

async fn rebase(mut args: env::ArgsOs) -> ExitCode {
    let Some(branch) = args.next() else {
        eprintln!("{USAGE}");
        return ExitCode::from(2);
    };
    let branch = branch.to_string_lossy();

    match git_branches::rebase(&branch).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
