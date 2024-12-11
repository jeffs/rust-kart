use clap::Parser;
use std::env::current_dir;
use std::path::{Path, PathBuf};
use std::process::exit;

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Parser)]
struct Args {
    path: PathBuf,

    #[arg(default_value_t = String::from("::"))]
    sep: String,
}

fn normalize(path: &Path) -> Result<PathBuf> {
    let path = path.with_extension("");
    Ok(if path.is_absolute() {
        path.strip_prefix(current_dir()?)?.to_owned()
    } else {
        path
    })
}

fn main_imp() -> Result<()> {
    let args = Args::parse();
    let path = normalize(&args.path)?;
    let parts: Vec<_> = path
        .components()
        .filter_map(|p| p.as_os_str().to_str())
        .collect();
    let parts = match parts.first() {
        Some(&"src") => &parts[1..],
        _ => &parts.as_slice(),
    };
    let module = parts.join(&args.sep);
    println!("{module}");
    Ok(())
}

fn main() {
    if let Err(err) = main_imp() {
        eprintln!("Error: {err}");
        exit(1);
    }
}
