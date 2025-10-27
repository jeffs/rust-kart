use std::env;

use crate::error::{Error, Result};

/// Prints the name of the local trunk branch, if any is identified.
///
/// # Errors
///
/// Returns an error if `args` is nonempty, or no local trunk can be identified.
pub async fn trunk(mut args: env::ArgsOs) -> Result<()> {
    if let Some(arg) = args.next() {
        return Err(Error::Arg(arg));
    }
    println!("{}", crate::trunk::local().await?);
    Ok(())
}
