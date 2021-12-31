use std::env::current_dir;
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io;
use std::iter::successors;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug)]
struct CargoError {
    what: String,
}

impl Display for CargoError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.what)
    }
}

impl Error for CargoError {}

impl From<io::Error> for CargoError {
    fn from(error: io::Error) -> Self {
        CargoError {
            what: error.to_string(),
        }
    }
}

// fn is_root(dir: &Path) -> bool {
//     let mut parts = dir.components();
//     parts.next() == Some(Component::RootDir) && parts.next().is_none()
// }

fn parent(path: &PathBuf) -> Option<PathBuf> {
    path.parent().map(|p| p.to_owned())
}

fn pwd() -> PathBuf {
    current_dir()
        .expect("can't get working directory")
        .canonicalize()
        .expect("can't canonicalize working directory")
}

fn find_binary() -> Option<PathBuf> {
    for dir in successors(Some(pwd()), parent) {
        let file = dir.join("target/debug/cargo-norm");
        if file.exists() {
            return Some(file);
        }
    }
    None
}

fn build() -> Result<(), CargoError> {
    if !Command::new("cargo").arg("build").status()?.success() {
        let what = "cargo build returned bad status".to_string();
        return Err(CargoError { what });
    }
    Ok(())
}

#[test]
fn runs() {
    build().expect("can't build");
    let binary = find_binary().expect("can't find binary");
    Command::new(binary).status().expect("can't run");
}
