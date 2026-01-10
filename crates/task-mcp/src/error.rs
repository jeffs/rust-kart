//! Error types for task-mcp.

use std::fmt;

/// Errors that can occur during task management.
#[derive(Debug)]
pub enum Error {
    /// Task with the given name was not found.
    TaskNotFound { name: String },
    /// Failed to spawn the process.
    SpawnFailed { name: String, reason: String },
    /// I/O error during file operations.
    Io(std::io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TaskNotFound { name } => write!(f, "task not found: {name}"),
            Self::SpawnFailed { name, reason } => {
                write!(f, "failed to spawn task '{name}': {reason}")
            }
            Self::Io(e) => write!(f, "I/O error: {e}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

/// A specialized Result type for task-mcp operations.
pub type Result<T> = std::result::Result<T, Error>;
