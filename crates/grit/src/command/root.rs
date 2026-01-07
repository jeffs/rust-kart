use std::ffi;

use crate::{
    error::{Error, Result},
    git::git,
};

/// # Errors
///
/// Returns an error if Git fails, or if `args` is nonempty.
pub async fn root(mut args: impl Iterator<Item = ffi::OsString>) -> Result<()> {
    if let Some(arg) = args.next() {
        return Err(Error::Arg(arg));
    }
    print!("{}", git(["rev-parse", "--show-toplevel"]).await?);
    Ok(())
}
