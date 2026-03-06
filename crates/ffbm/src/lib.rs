#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

mod db;
mod error;
mod export;
mod import;
mod profile;
mod types;

pub use error::{Error, Result};
pub use export::{ExportStats, export_bookmarks};
pub use import::{ImportStats, import_bookmarks};
pub use profile::{check_firefox_not_running, find_profile, list_profiles};
pub use types::{Bookmark, Profile, Separator};
