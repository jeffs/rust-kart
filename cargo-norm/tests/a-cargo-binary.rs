use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io;
use std::process::Command;

const BINARY: &str = "../target/debug/cargo-norm";

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
    Command::new(BINARY).status().expect("can't run");
}
