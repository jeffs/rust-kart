//! Firefox Bookmark Manager library.
//!
//! Provides functionality for managing Firefox bookmarks through a
//! version-control-friendly workflow.

#![forbid(unsafe_code)]

mod db;
mod error;
mod export;
mod import;
mod profile;
mod types;

pub use error::{Error, Result};
pub use export::{export_bookmarks, ExportStats};
pub use import::{import_bookmarks, ImportStats};
pub use profile::{check_firefox_not_running, find_profile, list_profiles};
pub use types::{Bookmark, Profile, Separator};
