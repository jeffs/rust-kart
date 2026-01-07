//! Cargo workspace detection

use std::path::{Path, PathBuf};

use cargo_metadata::MetadataCommand;

/// Error type for workspace operations
#[derive(Debug)]
pub enum WorkspaceError {
    /// Failed to run cargo metadata
    Metadata(cargo_metadata::Error),
}

impl std::fmt::Display for WorkspaceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Metadata(e) => write!(f, "Failed to read workspace metadata: {e}"),
        }
    }
}

impl std::error::Error for WorkspaceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Metadata(e) => Some(e),
        }
    }
}

/// Information about a workspace member
#[derive(Debug)]
pub struct CrateMember {
    /// Name of the crate
    pub name: String,
    /// Root directory of the crate
    pub path: PathBuf,
}

/// Detect if a path is a workspace and get its members
///
/// # Errors
/// Returns `WorkspaceError::Metadata` if cargo metadata cannot be read.
pub fn detect_workspace(path: &Path) -> Result<Vec<CrateMember>, WorkspaceError> {
    let manifest = path.join("Cargo.toml");

    let metadata = MetadataCommand::new()
        .manifest_path(&manifest)
        .no_deps()
        .exec()
        .map_err(WorkspaceError::Metadata)?;

    let members: Vec<CrateMember> = metadata
        .workspace_members
        .iter()
        .filter_map(|id| {
            metadata
                .packages
                .iter()
                .find(|p| &p.id == id)
                .map(|pkg| CrateMember {
                    name: pkg.name.clone(),
                    path: pkg
                        .manifest_path
                        .parent()
                        .map(|p| p.to_path_buf().into_std_path_buf())
                        .unwrap_or_default(),
                })
        })
        .collect();

    Ok(members)
}

/// Check if a path is a workspace (has multiple members)
///
/// # Errors
/// Returns `WorkspaceError::Metadata` if cargo metadata cannot be read.
pub fn is_workspace(path: &Path) -> Result<bool, WorkspaceError> {
    let members = detect_workspace(path)?;
    Ok(members.len() > 1)
}
