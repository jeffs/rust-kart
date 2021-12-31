mod error;

use std::process::Command;

pub use error::CargoError as Error;

pub fn build() -> Result<(), Error> {
    if !Command::new("cargo").arg("build").status()?.success() {
        return Err(Error::new("cargo build returned bad status"));
    }
    Ok(())
}
