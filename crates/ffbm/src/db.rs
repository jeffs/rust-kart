//! Database operations for Firefox bookmarks.

use std::collections::HashMap;
use std::path::Path;

use rusqlite::{params, Connection, Transaction};

use crate::error::Result;
use crate::types::{item_types, roots, Bookmark, DbBookmark, DbSeparator, Folder, Separator};

/// Characters used for GUID generation.
const GUID_CHARS: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_-";

/// Generate a Firefox-style GUID (12 characters, base64-ish).
fn generate_guid() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);

    let mut guid = String::with_capacity(12);
    let mut val = timestamp;
    for _ in 0..12 {
        guid.push(GUID_CHARS[(val % 64) as usize] as char);
        val /= 64;
    }
    guid
}

/// Generate a reverse host string for a URL.
fn reverse_host(url: &str) -> String {
    // Extract host from URL and reverse it with trailing dot
    if let Some(host) = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
    {
        let host = host.split('/').next().unwrap_or("");
        let host = host.split(':').next().unwrap_or(host);
        let reversed: String = host.chars().rev().collect();
        format!("{reversed}.")
    } else {
        String::new()
    }
}

/// Open a read-only connection to places.sqlite.
///
/// # Errors
///
/// Returns an error if the database cannot be opened.
pub fn open_readonly(path: &Path) -> Result<Connection> {
    let conn = Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    Ok(conn)
}

/// Open a read-write connection to places.sqlite.
///
/// # Errors
///
/// Returns an error if the database cannot be opened.
pub fn open_readwrite(path: &Path) -> Result<Connection> {
    let conn = Connection::open_with_flags(
        path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;
    Ok(conn)
}

/// Get the URL for a `place_id`.
///
/// # Errors
///
/// Returns an error if the database query fails.
pub fn get_url(conn: &Connection, place_id: i64) -> Result<Option<String>> {
    let mut stmt = conn.prepare_cached("SELECT url FROM moz_places WHERE id = ?")?;
    let result: Option<String> = stmt.query_row(params![place_id], |row| row.get(0)).ok();
    Ok(result)
}

/// Get or create a `place_id` for a URL.
///
/// # Errors
///
/// Returns an error if the database query or insert fails.
pub fn get_or_create_place_id(conn: &Connection, url: &str) -> Result<i64> {
    // Try to find existing
    let mut stmt = conn.prepare_cached("SELECT id FROM moz_places WHERE url = ?")?;
    if let Ok(id) = stmt.query_row(params![url], |row| row.get::<_, i64>(0)) {
        return Ok(id);
    }

    // Create new place entry
    conn.execute(
        "INSERT INTO moz_places (url, url_hash, rev_host, frecency, guid)
         VALUES (?, hash(?), ?, 0, ?)",
        params![url, url, reverse_host(url), generate_guid()],
    )?;

    Ok(conn.last_insert_rowid())
}

/// Get the keyword for a `place_id`.
///
/// # Errors
///
/// Returns an error if the database query fails.
pub fn get_keyword(conn: &Connection, place_id: i64) -> Result<Option<String>> {
    let mut stmt = conn.prepare_cached("SELECT keyword FROM moz_keywords WHERE place_id = ?")?;
    let result: Option<String> = stmt.query_row(params![place_id], |row| row.get(0)).ok();
    Ok(result)
}

/// Set the keyword for a `place_id`.
///
/// # Errors
///
/// Returns an error if the database operations fail.
pub fn set_keyword(conn: &Connection, place_id: i64, keyword: &str) -> Result<()> {
    if keyword.is_empty() {
        conn.execute("DELETE FROM moz_keywords WHERE place_id = ?", params![place_id])?;
    } else {
        // Delete any existing keyword for this place
        conn.execute("DELETE FROM moz_keywords WHERE place_id = ?", params![place_id])?;
        // Delete any existing entry with this keyword
        conn.execute(
            "DELETE FROM moz_keywords WHERE keyword = ?",
            params![keyword],
        )?;
        // Insert new keyword
        conn.execute(
            "INSERT INTO moz_keywords (keyword, place_id) VALUES (?, ?)",
            params![keyword, place_id],
        )?;
    }
    Ok(())
}

/// Get tags for a `place_id` (bookmarks under parent=4 pointing to this place).
///
/// # Errors
///
/// Returns an error if the database query fails.
pub fn get_tags(conn: &Connection, place_id: i64) -> Result<Vec<String>> {
    let mut stmt = conn.prepare_cached(
        "SELECT f.title FROM moz_bookmarks b
         JOIN moz_bookmarks f ON b.parent = f.id
         WHERE b.fk = ? AND f.parent = ?",
    )?;

    let tags: Vec<String> = stmt
        .query_map(params![place_id, roots::TAGS], |row| row.get(0))?
        .filter_map(std::result::Result::ok)
        .collect();

    Ok(tags)
}

/// Read all folders from the database.
///
/// # Errors
///
/// Returns an error if the database query fails.
pub fn read_folders(conn: &Connection) -> Result<Vec<Folder>> {
    let mut stmt = conn.prepare(
        "SELECT id, parent, title
         FROM moz_bookmarks
         WHERE type = ?",
    )?;

    let folders: Vec<Folder> = stmt
        .query_map(params![item_types::FOLDER], |row| {
            Ok(Folder {
                id: row.get(0)?,
                parent: row.get(1)?,
                title: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
            })
        })?
        .filter_map(std::result::Result::ok)
        .collect();

    Ok(folders)
}

/// Read all bookmarks from the database.
///
/// # Errors
///
/// Returns an error if the database query fails.
pub fn read_bookmarks(conn: &Connection) -> Result<Vec<DbBookmark>> {
    let mut stmt = conn.prepare(
        "SELECT parent, fk, title, position, guid, dateAdded, lastModified
         FROM moz_bookmarks
         WHERE type = ?",
    )?;

    let bookmarks: Vec<DbBookmark> = stmt
        .query_map(params![item_types::BOOKMARK], |row| {
            Ok(DbBookmark {
                parent: row.get(0)?,
                place_id: row.get(1)?,
                title: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
                position: row.get(3)?,
                guid: row.get(4)?,
                date_added: row.get(5)?,
                last_modified: row.get(6)?,
            })
        })?
        .filter_map(std::result::Result::ok)
        .collect();

    Ok(bookmarks)
}

/// Read all separators from the database.
///
/// # Errors
///
/// Returns an error if the database query fails.
pub fn read_separators(conn: &Connection) -> Result<Vec<DbSeparator>> {
    let mut stmt = conn.prepare(
        "SELECT parent, position, guid, dateAdded, lastModified
         FROM moz_bookmarks
         WHERE type = ?",
    )?;

    let separators: Vec<DbSeparator> = stmt
        .query_map(params![item_types::SEPARATOR], |row| {
            Ok(DbSeparator {
                parent: row.get(0)?,
                position: row.get(1)?,
                guid: row.get(2)?,
                date_added: row.get(3)?,
                last_modified: row.get(4)?,
            })
        })?
        .filter_map(std::result::Result::ok)
        .collect();

    Ok(separators)
}

/// Delete all non-root bookmarks and tags.
///
/// # Errors
///
/// Returns an error if the database operations fail.
pub fn delete_all_bookmarks(tx: &Transaction<'_>) -> Result<()> {
    // Delete all bookmark entries except root folders (ids 1-6)
    tx.execute(
        "DELETE FROM moz_bookmarks WHERE id NOT IN (1, 2, 3, 4, 5, 6)",
        [],
    )?;

    // Clear keywords
    tx.execute("DELETE FROM moz_keywords", [])?;

    Ok(())
}

/// Insert a folder into the database.
///
/// # Errors
///
/// Returns an error if the insert fails.
pub fn insert_folder(
    tx: &Transaction<'_>,
    parent: i64,
    title: &str,
    position: i32,
    guid: &str,
    date_added: i64,
    last_modified: i64,
) -> Result<i64> {
    tx.execute(
        "INSERT INTO moz_bookmarks (type, parent, title, position, guid, dateAdded, lastModified)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
        params![
            item_types::FOLDER,
            parent,
            title,
            position,
            guid,
            date_added,
            last_modified
        ],
    )?;
    Ok(tx.last_insert_rowid())
}

/// Insert a bookmark into the database.
///
/// # Errors
///
/// Returns an error if the insert fails.
#[allow(clippy::too_many_arguments)]
pub fn insert_bookmark(
    tx: &Transaction<'_>,
    parent: i64,
    place_id: i64,
    title: &str,
    position: i32,
    guid: &str,
    date_added: i64,
    last_modified: i64,
) -> Result<i64> {
    tx.execute(
        "INSERT INTO moz_bookmarks (type, parent, fk, title, position, guid, dateAdded, lastModified)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        params![
            item_types::BOOKMARK,
            parent,
            place_id,
            title,
            position,
            guid,
            date_added,
            last_modified
        ],
    )?;
    Ok(tx.last_insert_rowid())
}

/// Insert a separator into the database.
///
/// # Errors
///
/// Returns an error if the insert fails.
pub fn insert_separator(
    tx: &Transaction<'_>,
    parent: i64,
    position: i32,
    guid: &str,
    date_added: i64,
    last_modified: i64,
) -> Result<i64> {
    tx.execute(
        "INSERT INTO moz_bookmarks (type, parent, position, guid, dateAdded, lastModified)
         VALUES (?, ?, ?, ?, ?, ?)",
        params![
            item_types::SEPARATOR,
            parent,
            position,
            guid,
            date_added,
            last_modified
        ],
    )?;
    Ok(tx.last_insert_rowid())
}

/// Get or create a tag folder under the tags root.
///
/// # Errors
///
/// Returns an error if the database operations fail.
pub fn get_or_create_tag_folder(tx: &Transaction<'_>, tag_name: &str) -> Result<i64> {
    // Look for existing tag folder
    let mut stmt = tx.prepare_cached(
        "SELECT id FROM moz_bookmarks WHERE parent = ? AND title = ? AND type = ?",
    )?;

    if let Ok(id) = stmt.query_row(params![roots::TAGS, tag_name, item_types::FOLDER], |row| {
        row.get::<_, i64>(0)
    }) {
        return Ok(id);
    }

    // Get next position
    let position: i32 = tx
        .query_row(
            "SELECT COALESCE(MAX(position), -1) + 1 FROM moz_bookmarks WHERE parent = ?",
            params![roots::TAGS],
            |row| row.get(0),
        )
        .unwrap_or(0);

    // Create new tag folder
    #[allow(clippy::cast_possible_truncation)]
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_micros() as i64)
        .unwrap_or(0);

    insert_folder(tx, roots::TAGS, tag_name, position, &generate_guid(), now, now)
}

/// Add a tag to a bookmark (via `place_id`).
///
/// # Errors
///
/// Returns an error if the database operations fail.
pub fn add_tag(tx: &Transaction<'_>, place_id: i64, tag_name: &str) -> Result<()> {
    let tag_folder_id = get_or_create_tag_folder(tx, tag_name)?;

    // Check if already tagged
    let mut stmt = tx.prepare_cached(
        "SELECT 1 FROM moz_bookmarks WHERE parent = ? AND fk = ? AND type = ?",
    )?;

    if stmt
        .query_row(params![tag_folder_id, place_id, item_types::BOOKMARK], |_| Ok(()))
        .is_ok()
    {
        return Ok(());
    }

    // Get next position
    let position: i32 = tx
        .query_row(
            "SELECT COALESCE(MAX(position), -1) + 1 FROM moz_bookmarks WHERE parent = ?",
            params![tag_folder_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    #[allow(clippy::cast_possible_truncation)]
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_micros() as i64)
        .unwrap_or(0);

    // Create tag entry (a bookmark under the tag folder pointing to the place)
    tx.execute(
        "INSERT INTO moz_bookmarks (type, parent, fk, position, guid, dateAdded, lastModified)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
        params![
            item_types::BOOKMARK,
            tag_folder_id,
            place_id,
            position,
            generate_guid(),
            now,
            now
        ],
    )?;

    Ok(())
}

/// Convert a database bookmark to the export Bookmark type.
///
/// # Errors
///
/// Returns an error if the URL or tags cannot be retrieved.
pub fn db_bookmark_to_export(
    conn: &Connection,
    db_bm: &DbBookmark,
) -> Result<Option<Bookmark>> {
    let Some(place_id) = db_bm.place_id else {
        return Ok(None);
    };

    let Some(url) = get_url(conn, place_id)? else {
        return Ok(None);
    };

    let tags = get_tags(conn, place_id)?;
    let keyword = get_keyword(conn, place_id)?.unwrap_or_default();

    Ok(Some(Bookmark {
        url,
        title: db_bm.title.clone(),
        tags,
        keyword,
        date_added: db_bm.date_added,
        last_modified: db_bm.last_modified,
        guid: db_bm.guid.clone(),
    }))
}

/// Convert a database separator to the export Separator type.
#[must_use]
pub fn db_separator_to_export(db_sep: &DbSeparator) -> Separator {
    Separator {
        date_added: db_sep.date_added,
        last_modified: db_sep.last_modified,
        guid: db_sep.guid.clone(),
    }
}

/// Build a folder ID to parent ID map.
#[must_use]
pub fn build_folder_tree(folders: &[Folder]) -> HashMap<i64, i64> {
    folders.iter().map(|f| (f.id, f.parent)).collect()
}

/// Build a folder ID to title map.
#[must_use]
pub fn build_folder_titles(folders: &[Folder]) -> HashMap<i64, String> {
    folders.iter().map(|f| (f.id, f.title.clone())).collect()
}

/// Build the path from a folder to a root folder.
#[must_use]
pub fn build_folder_path(
    folder_id: i64,
    folder_tree: &HashMap<i64, i64>,
    folder_titles: &HashMap<i64, String>,
) -> Vec<String> {
    let mut path = Vec::new();
    let mut current = folder_id;

    while let Some(&parent) = folder_tree.get(&current) {
        if current <= roots::MOBILE {
            // Stop at root folders
            break;
        }
        if let Some(title) = folder_titles.get(&current) {
            path.push(title.clone());
        }
        current = parent;
    }

    path.reverse();
    path
}
