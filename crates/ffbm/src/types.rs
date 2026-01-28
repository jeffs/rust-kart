//! Core types for bookmark representation.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// A Firefox profile discovered from Profile Groups.
#[derive(Debug, Clone)]
pub struct Profile {
    /// The display name of the profile.
    pub name: String,
    /// Absolute path to the profile directory.
    pub path: PathBuf,
}

/// A bookmark entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    /// The URL of the bookmark.
    pub url: String,
    /// The display title.
    pub title: String,
    /// Tags associated with this bookmark.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Keyword shortcut for the bookmark.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub keyword: String,
    /// Date added as microseconds since Unix epoch.
    pub date_added: i64,
    /// Last modified as microseconds since Unix epoch.
    pub last_modified: i64,
    /// Firefox sync GUID.
    pub guid: String,
}

/// A separator entry in a bookmark folder.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Separator {
    /// Date added as microseconds since Unix epoch.
    pub date_added: i64,
    /// Last modified as microseconds since Unix epoch.
    pub last_modified: i64,
    /// Firefox sync GUID.
    pub guid: String,
}

/// Index file for explicit ordering of items in a folder.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderIndex {
    /// Ordered list of item names.
    /// - Bookmarks: filename without `.toml` extension
    /// - Separators: `---{guid}`
    /// - Subfolders: `name/` (trailing slash)
    pub order: Vec<String>,
}

/// Internal representation of a folder from the database.
#[derive(Debug, Clone)]
pub struct Folder {
    /// Database ID.
    pub id: i64,
    /// Parent folder ID.
    pub parent: i64,
    /// Folder title.
    pub title: String,
}

/// Internal representation of a bookmark from the database.
#[derive(Debug, Clone)]
pub struct DbBookmark {
    /// Parent folder ID.
    pub parent: i64,
    /// Place ID (foreign key to `moz_places`).
    pub place_id: Option<i64>,
    /// Bookmark title.
    pub title: String,
    /// Position within parent.
    pub position: i32,
    /// Firefox sync GUID.
    pub guid: String,
    /// Date added as microseconds since Unix epoch.
    pub date_added: i64,
    /// Last modified as microseconds since Unix epoch.
    pub last_modified: i64,
}

/// Internal representation of a separator from the database.
#[derive(Debug, Clone)]
pub struct DbSeparator {
    /// Parent folder ID.
    pub parent: i64,
    /// Position within parent.
    pub position: i32,
    /// Firefox sync GUID.
    pub guid: String,
    /// Date added as microseconds since Unix epoch.
    pub date_added: i64,
    /// Last modified as microseconds since Unix epoch.
    pub last_modified: i64,
}

/// Root folder IDs in Firefox bookmarks.
pub mod roots {
    /// The root of all bookmark folders.
    pub const ROOT: i64 = 1;
    /// Bookmarks Menu folder.
    pub const MENU: i64 = 2;
    /// Bookmarks Toolbar folder.
    pub const TOOLBAR: i64 = 3;
    /// Tags folder (internal, not exported).
    pub const TAGS: i64 = 4;
    /// Other Bookmarks folder.
    pub const OTHER: i64 = 5;
    /// Mobile Bookmarks folder.
    pub const MOBILE: i64 = 6;
}

/// Bookmark item types in `moz_bookmarks`.
pub mod item_types {
    /// A bookmark (URL).
    pub const BOOKMARK: i32 = 1;
    /// A folder.
    pub const FOLDER: i32 = 2;
    /// A separator line.
    pub const SEPARATOR: i32 = 3;
}

/// Map a root folder ID to its export directory name.
#[must_use]
pub fn root_dir_name(root_id: i64) -> Option<&'static str> {
    match root_id {
        roots::MENU => Some("menu"),
        roots::TOOLBAR => Some("toolbar"),
        roots::OTHER => Some("other"),
        roots::MOBILE => Some("mobile"),
        _ => None,
    }
}

/// Map a directory name back to a root folder ID.
#[must_use]
pub fn dir_name_to_root(name: &str) -> Option<i64> {
    match name {
        "menu" => Some(roots::MENU),
        "toolbar" => Some(roots::TOOLBAR),
        "other" => Some(roots::OTHER),
        "mobile" => Some(roots::MOBILE),
        _ => None,
    }
}
