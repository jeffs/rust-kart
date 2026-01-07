use std::{env, ffi};

use crate::{
    error::{Error, Result},
    git,
};

#[derive(Clone, Copy)]
enum RemovalPolicy {
    /// Do not remove any branches.
    None,
    /// Remove local branches only.
    Local,
    /// Remove both local and remote branches.
    Remote,
}

/// # Errors
///
/// Returns an error if `git` cannot be found, or returns bad status.
async fn archive_imp(branch: &ffi::OsStr, policy: RemovalPolicy) -> Result<()> {
    let old = [ffi::OsStr::new("refs"), ffi::OsStr::new("heads"), branch]
        .iter()
        .collect::<std::path::PathBuf>();
    let new = [ffi::OsStr::new("refs"), ffi::OsStr::new("archive"), branch]
        .iter()
        .collect::<std::path::PathBuf>();

    println!("mk {}", new.display());
    git::git([
        ffi::OsStr::new("update-ref"),
        new.as_os_str(),
        old.as_os_str(),
    ])
    .await?;

    if let RemovalPolicy::None = policy {
        return Ok(());
    }

    // Remove the remote branch, if so ordered. This must be done _before_
    // removing the local branch, so that we can use the local branch to
    // identify the correct remote.
    if let RemovalPolicy::Remote = policy
        && let Some(upstream) = git::upstream(branch).await
    {
        let (remote, name) = upstream.split_once('/').unwrap();
        println!("rm {upstream}");
        git::git(["push", remote, &format!(":{name}")]).await?;
    }

    // Remove the local branch, if so ordered. Force the removal, even if the
    // branch is not fully merged, since we've just saved a ref to it.
    println!("rm {}", branch.display());
    git::git([ffi::OsStr::new("branch"), ffi::OsStr::new("-D"), branch]).await?;
    Ok(())
}

/// Archives branches by moving them to refs/archive/.
///
/// # Errors
///
/// Returns an error if no branch name is provided, or git fails.
pub async fn archive(args: env::ArgsOs) -> Result<()> {
    let mut policy = RemovalPolicy::None;

    let args = args
        .filter(|arg| match arg.to_str() {
            Some("-d") => {
                policy = RemovalPolicy::Local;
                false
            }
            Some("-D") => {
                policy = RemovalPolicy::Remote;
                false
            }
            _ => true,
        })
        .collect::<Vec<_>>();

    if args.is_empty() {
        return Err(Error::BranchName);
    }

    for arg in args {
        archive_imp(&arg, policy).await?;
    }

    Ok(())
}
