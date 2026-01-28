//! Import bookmarks from file tree to database.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::db::{self, get_or_create_place_id, set_keyword};
use crate::error::Result;
use crate::types::{dir_name_to_root, Bookmark, FolderIndex, Separator};

/// Position assigned to items not listed in the index.
/// They appear after all indexed items.
const UNLISTED_POSITION_BASE: i32 = 100_000;

/// Characters used for GUID generation.
const GUID_CHARS: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_-";

/// Generate a Firefox-style GUID.
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

/// A file tree entry that can be either a bookmark or separator.
#[derive(Debug)]
enum Entry {
    Bookmark(Bookmark),
    Separator(Separator),
}

/// A folder with its path and entries.
type FolderData = Vec<(String, Vec<(String, Entry)>)>;

/// Read a bookmark from a TOML file.
fn read_bookmark(path: &Path) -> Result<Bookmark> {
    let content = fs::read_to_string(path)?;
    let bookmark: Bookmark = toml::from_str(&content)?;
    Ok(bookmark)
}

/// Read a separator from a TOML file.
fn read_separator(path: &Path) -> Result<Separator> {
    let content = fs::read_to_string(path)?;
    let separator: Separator = toml::from_str(&content)?;
    Ok(separator)
}

/// Read the folder index if present.
fn read_folder_index(dir: &Path) -> Option<FolderIndex> {
    let index_path = dir.join("__index.toml");
    if index_path.is_file() {
        let content = fs::read_to_string(&index_path).ok()?;
        toml::from_str(&content).ok()
    } else {
        None
    }
}

/// Build a position map from a folder index.
///
/// Returns a map from item name to position. Folder names have their trailing
/// `/` stripped. Items not in the index get positions starting at
/// `UNLISTED_POSITION_BASE`.
fn build_position_map(index: Option<&FolderIndex>) -> HashMap<String, i32> {
    let mut map = HashMap::new();
    if let Some(idx) = index {
        for (i, name) in idx.order.iter().enumerate() {
            // Strip trailing `/` from folder names
            let key = name.strip_suffix('/').unwrap_or(name).to_string();
            #[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
            map.insert(key, i as i32);
        }
    }
    map
}

/// Get position for an item, using the index map or assigning an unlisted
/// position.
fn get_position(
    name: &str,
    position_map: &HashMap<String, i32>,
    unlisted_counter: &mut i32,
) -> i32 {
    if let Some(&pos) = position_map.get(name) {
        pos
    } else {
        let pos = UNLISTED_POSITION_BASE + *unlisted_counter;
        *unlisted_counter += 1;
        pos
    }
}

/// Recursively read all entries from a directory.
///
/// Uses `__index.toml` if present for ordering, otherwise sorts alphabetically.
fn read_directory_entries(dir: &Path) -> Result<Vec<(String, Entry)>> {
    let mut entries = Vec::new();

    if !dir.is_dir() {
        return Ok(entries);
    }

    // Read all files first
    let mut file_entries: Vec<(String, Entry)> = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name().to_string_lossy().to_string();

        if path.is_file() {
            // Skip index file
            if file_name == "__index.toml" {
                continue;
            }

            let ext = path.extension().and_then(|e| e.to_str());
            if ext.is_some_and(|e| e.eq_ignore_ascii_case("toml")) {
                let bookmark = read_bookmark(&path)?;
                let name = file_name.strip_suffix(".toml").unwrap_or(&file_name);
                file_entries.push((name.to_string(), Entry::Bookmark(bookmark)));
            } else if ext.is_some_and(|e| e.eq_ignore_ascii_case("separator")) {
                let separator = read_separator(&path)?;
                let name = file_name.strip_suffix(".separator").unwrap_or(&file_name);
                file_entries.push((name.to_string(), Entry::Separator(separator)));
            }
        }
    }

    // Read index if present
    let index = read_folder_index(dir);

    if let Some(idx) = index {
        // Build name-to-position map from index order
        let name_to_pos: HashMap<&str, usize> = idx
            .order
            .iter()
            .enumerate()
            .map(|(i, name)| (name.as_str(), i))
            .collect();

        // Separate indexed and unlisted entries
        let mut indexed: Vec<(usize, (String, Entry))> = Vec::new();
        let mut unlisted: Vec<(String, Entry)> = Vec::new();

        for (name, entry) in file_entries {
            if let Some(&pos) = name_to_pos.get(name.as_str()) {
                indexed.push((pos, (name, entry)));
            } else {
                unlisted.push((name, entry));
            }
        }

        // Sort indexed by position
        indexed.sort_by_key(|(pos, _)| *pos);

        // Sort unlisted alphabetically
        unlisted.sort_by(|(a, _), (b, _)| a.cmp(b));

        // Indexed items first, then unlisted
        entries = indexed.into_iter().map(|(_, e)| e).collect();
        entries.extend(unlisted);
    } else {
        // No index: sort alphabetically as fallback
        file_entries.sort_by(|(a, _), (b, _)| a.cmp(b));
        entries = file_entries;
    }

    Ok(entries)
}

/// Recursively read folder structure from a directory.
///
/// Returns folders ordered by index if present. Includes all directories,
/// even empty ones, to maintain the folder hierarchy.
fn read_folder_structure(dir: &Path, parent_path: &str) -> Result<FolderData> {
    let mut folders = Vec::new();

    if !dir.is_dir() {
        return Ok(folders);
    }

    // Read entries in this directory
    let entries = read_directory_entries(dir)?;
    // Always include folders to maintain hierarchy (empty folders become
    // parents for nested content)
    folders.push((parent_path.to_string(), entries));

    // Collect subdirectories
    let mut subdirs: Vec<_> = fs::read_dir(dir)?
        .filter_map(std::result::Result::ok)
        .filter(|e| e.path().is_dir())
        .collect();

    // Read index for ordering subdirectories
    let index = read_folder_index(dir);

    if let Some(idx) = index {
        // Build name-to-position map for folders (entries ending with /)
        let name_to_pos: HashMap<&str, usize> = idx
            .order
            .iter()
            .enumerate()
            .filter_map(|(i, name)| {
                name.strip_suffix('/').map(|n| (n, i))
            })
            .collect();

        // Separate indexed and unlisted subdirs
        let mut indexed: Vec<(usize, std::fs::DirEntry)> = Vec::new();
        let mut unlisted: Vec<std::fs::DirEntry> = Vec::new();

        for subdir in subdirs {
            let name = subdir.file_name().to_string_lossy().to_string();
            if let Some(&pos) = name_to_pos.get(name.as_str()) {
                indexed.push((pos, subdir));
            } else {
                unlisted.push(subdir);
            }
        }

        // Sort indexed by position
        indexed.sort_by_key(|(pos, _)| *pos);

        // Sort unlisted alphabetically
        unlisted.sort_by_key(std::fs::DirEntry::file_name);

        // Recombine
        subdirs = indexed.into_iter().map(|(_, e)| e).collect();
        subdirs.extend(unlisted);
    } else {
        // No index: sort subdirectories by name for consistent ordering
        subdirs.sort_by_key(std::fs::DirEntry::file_name);
    }

    for subdir in subdirs {
        let subdir_name = subdir.file_name().to_string_lossy().to_string();
        let subdir_path = if parent_path.is_empty() {
            subdir_name.clone()
        } else {
            format!("{parent_path}/{subdir_name}")
        };
        let sub_folders = read_folder_structure(&subdir.path(), &subdir_path)?;
        folders.extend(sub_folders);
    }

    Ok(folders)
}

/// Import bookmarks from file tree to database.
///
/// This performs a full rebuild: deletes all existing bookmarks and re-imports.
///
/// # Errors
///
/// Returns an error if the database cannot be modified or files cannot be read.
#[allow(clippy::too_many_lines)]
pub fn import_bookmarks(db_path: &Path, import_dir: &Path) -> Result<ImportStats> {
    let mut conn = db::open_readwrite(db_path)?;
    let tx = conn.transaction()?;

    let mut stats = ImportStats::default();

    // Delete all existing bookmarks except roots
    db::delete_all_bookmarks(&tx)?;

    // Map from folder path to database ID
    let mut folder_ids: HashMap<String, i64> = HashMap::new();

    // Cache of position maps per directory path
    let mut position_maps: HashMap<String, HashMap<String, i32>> = HashMap::new();

    // Process each root directory
    for root_name in ["menu", "toolbar", "other", "mobile"] {
        let root_dir = import_dir.join(root_name);
        if !root_dir.exists() {
            continue;
        }

        let Some(root_id) = dir_name_to_root(root_name) else {
            continue;
        };

        // Read folder structure
        let folder_data = read_folder_structure(&root_dir, "")?;

        // Build position maps for all directories
        for (folder_path, _) in &folder_data {
            let dir_path = if folder_path.is_empty() {
                root_dir.clone()
            } else {
                root_dir.join(folder_path)
            };
            let index = read_folder_index(&dir_path);
            let key = if folder_path.is_empty() {
                root_name.to_string()
            } else {
                format!("{root_name}/{folder_path}")
            };
            position_maps.insert(key, build_position_map(index.as_ref()));
        }

        // Track unlisted item counters per parent
        let mut unlisted_counters: HashMap<i64, i32> = HashMap::new();

        // First pass: create all folders with positions from parent's index
        for (folder_path, _) in &folder_data {
            if folder_path.is_empty() {
                // Root level, use root_id directly
                folder_ids.insert(root_name.to_string(), root_id);
                continue;
            }

            let parts: Vec<&str> = folder_path.split('/').collect();
            let folder_name = parts.last().copied().unwrap_or("");

            // Find parent folder ID and its position map
            let (parent_id, parent_key) = if parts.len() == 1 {
                (root_id, root_name.to_string())
            } else {
                let parent_path =
                    format!("{root_name}/{}", parts[..parts.len() - 1].join("/"));
                let pid = *folder_ids.get(&parent_path).unwrap_or(&root_id);
                (pid, parent_path)
            };

            // Get position from parent's index
            let position_map = position_maps.get(&parent_key).cloned().unwrap_or_default();
            let unlisted = unlisted_counters.entry(parent_id).or_insert(0);
            let position = get_position(folder_name, &position_map, unlisted);

            #[allow(clippy::cast_possible_truncation)]
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_micros() as i64)
                .unwrap_or(0);

            // Generate a GUID for the folder
            let guid = generate_guid();

            let folder_id = db::insert_folder(
                &tx,
                parent_id,
                folder_name,
                position,
                &guid,
                now,
                now,
            )?;

            let full_path = format!("{root_name}/{folder_path}");
            folder_ids.insert(full_path, folder_id);
            stats.folders_created += 1;
        }

        // Second pass: create bookmarks and separators with positions from index
        for (folder_path, entries) in &folder_data {
            let (parent_id, parent_key) = if folder_path.is_empty() {
                (root_id, root_name.to_string())
            } else {
                let full_path = format!("{root_name}/{folder_path}");
                let pid = *folder_ids.get(&full_path).unwrap_or(&root_id);
                (pid, full_path)
            };

            // Get position map for this folder
            let position_map = position_maps.get(&parent_key).cloned().unwrap_or_default();
            let unlisted = unlisted_counters.entry(parent_id).or_insert(0);

            for (name, entry) in entries {
                let position = get_position(name, &position_map, unlisted);

                match entry {
                    Entry::Bookmark(bm) => {
                        // Get or create place_id for the URL
                        let place_id = get_or_create_place_id(&tx, &bm.url)?;

                        // Insert bookmark with position from index
                        db::insert_bookmark(
                            &tx,
                            parent_id,
                            place_id,
                            &bm.title,
                            position,
                            &bm.guid,
                            bm.date_added,
                            bm.last_modified,
                        )?;

                        // Set keyword if present
                        if !bm.keyword.is_empty() {
                            set_keyword(&tx, place_id, &bm.keyword)?;
                        }

                        // Add tags
                        for tag in &bm.tags {
                            db::add_tag(&tx, place_id, tag)?;
                        }

                        stats.bookmarks_imported += 1;
                    }
                    Entry::Separator(sep) => {
                        db::insert_separator(
                            &tx,
                            parent_id,
                            position,
                            &sep.guid,
                            sep.date_added,
                            sep.last_modified,
                        )?;
                        stats.separators_imported += 1;
                    }
                }
            }
        }
    }

    tx.commit()?;

    // Vacuum the database to reclaim space
    conn.execute("VACUUM", [])?;

    Ok(stats)
}

/// Statistics from an import operation.
#[derive(Debug, Default)]
pub struct ImportStats {
    /// Number of folders created.
    pub folders_created: usize,
    /// Number of bookmarks imported.
    pub bookmarks_imported: usize,
    /// Number of separators imported.
    pub separators_imported: usize,
}
