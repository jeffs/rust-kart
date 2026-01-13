use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
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
