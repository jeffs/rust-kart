//! Error types for ffbm.

use std::fmt;
use std::path::PathBuf;

/// Errors that can occur during bookmark management.
#[derive(Debug)]
pub enum Error {
    /// Firefox is currently running; close it first.
    FirefoxRunning,
    /// Firefox application directory not found.
    FirefoxDirNotFound,
    /// No Profile Groups databases found.
    NoProfileGroups,
    /// Profile name not found.
    ProfileNotFound { name: String },
    /// Multiple profiles match the given name.
    AmbiguousProfile { name: String, matches: Vec<String> },
    /// Profile path specified does not exist.
    ProfilePathInvalid { path: PathBuf },
    /// Database file not found in profile.
    DatabaseNotFound { name: String, path: PathBuf },
    /// Database error.
    Sqlite(rusqlite::Error),
    /// I/O error.
    Io(std::io::Error),
    /// TOML serialization error.
    TomlSerialize(toml::ser::Error),
    /// TOML deserialization error.
    TomlDeserialize(toml::de::Error),
    /// Invalid bookmark data.
    InvalidBookmark { reason: String },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FirefoxRunning => write!(
                f,
                "Firefox is running; close it before modifying bookmarks"
            ),
            Self::FirefoxDirNotFound => write!(
                f,
                "Firefox directory not found (expected ~/Library/Application Support/Firefox)"
            ),
            Self::NoProfileGroups => write!(
                f,
                "no Profile Groups databases found in Firefox directory"
            ),
            Self::ProfileNotFound { name } => {
                write!(f, "profile not found: {name}")
            }
            Self::AmbiguousProfile { name, matches } => {
                write!(
                    f,
                    "multiple profiles match \"{name}\": {}",
                    matches.join(", ")
                )
            }
            Self::ProfilePathInvalid { path } => {
                write!(f, "profile path does not exist: {}", path.display())
            }
            Self::DatabaseNotFound { name, path } => {
                write!(f, "database {} not found in {}", name, path.display())
            }
            Self::Sqlite(e) => write!(f, "SQLite error: {e}"),
            Self::Io(e) => write!(f, "I/O error: {e}"),
            Self::TomlSerialize(e) => write!(f, "TOML serialization error: {e}"),
            Self::TomlDeserialize(e) => write!(f, "TOML deserialization error: {e}"),
            Self::InvalidBookmark { reason } => write!(f, "invalid bookmark: {reason}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Sqlite(e) => Some(e),
            Self::Io(e) => Some(e),
            Self::TomlSerialize(e) => Some(e),
            Self::TomlDeserialize(e) => Some(e),
            Self::FirefoxRunning
            | Self::FirefoxDirNotFound
            | Self::NoProfileGroups
            | Self::ProfileNotFound { .. }
            | Self::AmbiguousProfile { .. }
            | Self::ProfilePathInvalid { .. }
            | Self::DatabaseNotFound { .. }
            | Self::InvalidBookmark { .. } => None,
        }
    }
}

impl From<rusqlite::Error> for Error {
    fn from(e: rusqlite::Error) -> Self {
        Self::Sqlite(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<toml::ser::Error> for Error {
    fn from(e: toml::ser::Error) -> Self {
        Self::TomlSerialize(e)
    }
}

impl From<toml::de::Error> for Error {
    fn from(e: toml::de::Error) -> Self {
        Self::TomlDeserialize(e)
    }
}

/// A specialized Result type for ffbm operations.
pub type Result<T> = std::result::Result<T, Error>;
