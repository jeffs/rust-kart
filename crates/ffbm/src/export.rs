//! Export bookmarks from database to file tree.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::db::{
    self, build_folder_path, build_folder_titles, build_folder_tree, db_bookmark_to_export,
    db_separator_to_export,
};
use crate::error::Result;
use crate::types::{root_dir_name, roots, Folder, FolderIndex};

/// Sanitize a title for use as a filename.
///
/// Replaces problematic characters and limits length.
fn sanitize_filename(title: &str) -> String {
    let sanitized: String = title
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' | '\0'..='\x1f' => '_',
            _ => c,
        })
        .collect();

    // Trim whitespace and limit length (respecting char boundaries)
    let trimmed = sanitized.trim();
    if trimmed.len() > 100 {
        trimmed
            .char_indices()
            .take_while(|(i, _)| *i < 100)
            .map(|(_, c)| c)
            .collect()
    } else if trimmed.is_empty() {
        "untitled".to_string()
    } else {
        trimmed.to_string()
    }
}

/// Determine the root folder for a given folder ID.
fn find_root_folder(folder_id: i64, folder_tree: &HashMap<i64, i64>) -> i64 {
    let mut current = folder_id;
    while let Some(&parent) = folder_tree.get(&current) {
        if parent == roots::ROOT {
            return current;
        }
        current = parent;
    }
    folder_id
}

/// Calculate folder depth for sorting.
fn folder_depth(f: &Folder, tree: &HashMap<i64, i64>) -> usize {
    let mut depth = 0;
    let mut current = f.parent;
    while let Some(&parent) = tree.get(&current) {
        depth += 1;
        if current <= roots::MOBILE {
            break;
        }
        current = parent;
    }
    depth
}

/// An item to be recorded in the folder index.
struct IndexItem {
    /// The name to write in the index (sans extension for bookmarks, with / for
    /// folders, ---{guid} for separators).
    name: String,
    /// Original position from database for sorting.
    position: i32,
}

/// Export bookmarks from database to file tree.
///
/// Skips existing TOML files to preserve user comments.
///
/// # Errors
///
/// Returns an error if the database cannot be read or files cannot be written.
#[allow(clippy::too_many_lines)]
pub fn export_bookmarks(db_path: &Path, export_dir: &Path) -> Result<ExportStats> {
    let conn = db::open_readonly(db_path)?;

    let folders = db::read_folders(&conn)?;
    let bookmarks = db::read_bookmarks(&conn)?;
    let separators = db::read_separators(&conn)?;

    let folder_tree = build_folder_tree(&folders);
    let folder_titles = build_folder_titles(&folders);

    // Build folder ID to Folder map
    let folder_map: HashMap<i64, &Folder> = folders.iter().map(|f| (f.id, f)).collect();

    let mut stats = ExportStats::default();

    // Track created directories for separator and bookmark naming
    let mut created_dirs: HashSet<PathBuf> = HashSet::new();

    // Collect pre-existing files to skip (preserve user comments)
    let pre_existing: HashSet<PathBuf> = if export_dir.exists() {
        walkdir::WalkDir::new(export_dir)
            .into_iter()
            .filter_map(std::result::Result::ok)
            .filter(|e| e.file_type().is_file())
            .map(|e| e.path().to_path_buf())
            .collect()
    } else {
        HashSet::new()
    };

    // Track files written in this session (for unique_filename)
    let mut written_this_session: HashSet<PathBuf> = HashSet::new();

    // Track index items per directory: dir_path -> Vec<IndexItem>
    let mut folder_indices: HashMap<PathBuf, Vec<IndexItem>> = HashMap::new();

    // Create root directories
    for root_id in [roots::MENU, roots::TOOLBAR, roots::OTHER, roots::MOBILE] {
        if let Some(dir_name) = root_dir_name(root_id) {
            let root_path = export_dir.join(dir_name);
            if !root_path.exists() {
                fs::create_dir_all(&root_path)?;
            }
            created_dirs.insert(root_path);
        }
    }

    // Create folder structure
    // Sort folders by depth to ensure parents are created first
    let mut sorted_folders: Vec<&Folder> = folders
        .iter()
        .filter(|f| f.id > roots::MOBILE && f.parent != roots::TAGS)
        .collect();

    sorted_folders.sort_by_key(|f| folder_depth(f, &folder_tree));

    // Build map from folder ID to its position within parent (from DB)
    // We need to query positions for folders
    let folder_positions: HashMap<i64, i32> = {
        let mut stmt = conn.prepare(
            "SELECT id, position FROM moz_bookmarks WHERE type = 2",
        )?;
        stmt.query_map([], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i32>(1)?)))?
            .filter_map(std::result::Result::ok)
            .collect()
    };

    // Create directories for folders and track them in parent's index
    for folder in &sorted_folders {
        let root = find_root_folder(folder.id, &folder_tree);
        if let Some(root_name) = root_dir_name(root) {
            let path = build_folder_path(folder.id, &folder_tree, &folder_titles);
            let mut dir_path = export_dir.join(root_name);
            for segment in &path {
                dir_path = dir_path.join(sanitize_filename(segment));
            }
            if !dir_path.exists() {
                fs::create_dir_all(&dir_path)?;
                stats.folders_created += 1;
            }

            // Add folder to parent's index
            if let Some(parent_dir) = dir_path.parent() {
                let folder_name = sanitize_filename(&folder.title);
                let position = folder_positions.get(&folder.id).copied().unwrap_or(0);
                folder_indices
                    .entry(parent_dir.to_path_buf())
                    .or_default()
                    .push(IndexItem {
                        name: format!("{folder_name}/"),
                        position,
                    });
            }

            created_dirs.insert(dir_path);
        }
    }

    // Export bookmarks
    for db_bm in &bookmarks {
        // Skip tag entries (bookmarks under tags root)
        if db_bm.parent == roots::TAGS {
            continue;
        }
        if let Some(parent_folder) = folder_map.get(&db_bm.parent)
            && parent_folder.parent == roots::TAGS
        {
            continue;
        }

        let root = find_root_folder(db_bm.parent, &folder_tree);
        let Some(root_name) = root_dir_name(root) else {
            continue;
        };

        // Build directory path
        let mut dir_path = export_dir.join(root_name);
        if db_bm.parent != root {
            let path = build_folder_path(db_bm.parent, &folder_tree, &folder_titles);
            for segment in &path {
                dir_path = dir_path.join(sanitize_filename(segment));
            }
        }

        // Ensure directory exists
        if !dir_path.exists() {
            fs::create_dir_all(&dir_path)?;
        }

        // Convert to export format
        let Some(bookmark) = db_bookmark_to_export(&conn, db_bm)? else {
            continue;
        };

        // Check if base file already exists from previous export (preserve user
        // comments)
        let base_name = sanitize_filename(&bookmark.title);
        let base_path = dir_path.join(format!("{base_name}.toml"));
        if pre_existing.contains(&base_path) {
            stats.bookmarks_skipped += 1;
            // Still add to index with original filename
            folder_indices
                .entry(dir_path.clone())
                .or_default()
                .push(IndexItem {
                    name: base_name,
                    position: db_bm.position,
                });
            continue;
        }

        // Generate unique filename for same-title collisions within this export
        let (file_path, index_name) = {
            let mut path = base_path;
            let mut name = base_name.clone();
            let mut counter = 1;
            while path.exists() || written_this_session.contains(&path) {
                name = format!("{base_name}-{counter}");
                path = dir_path.join(format!("{name}.toml"));
                counter += 1;
            }
            (path, name)
        };

        // Write TOML file
        let toml_content = toml::to_string_pretty(&bookmark)?;
        fs::write(&file_path, toml_content)?;
        written_this_session.insert(file_path);
        stats.bookmarks_exported += 1;

        // Add to folder index
        folder_indices
            .entry(dir_path)
            .or_default()
            .push(IndexItem {
                name: index_name,
                position: db_bm.position,
            });
    }

    // Export separators
    for db_sep in &separators {
        let root = find_root_folder(db_sep.parent, &folder_tree);
        let Some(root_name) = root_dir_name(root) else {
            continue;
        };

        // Build directory path
        let mut dir_path = export_dir.join(root_name);
        if db_sep.parent != root {
            let path = build_folder_path(db_sep.parent, &folder_tree, &folder_titles);
            for segment in &path {
                dir_path = dir_path.join(sanitize_filename(segment));
            }
        }

        // Ensure directory exists
        if !dir_path.exists() {
            fs::create_dir_all(&dir_path)?;
        }

        let separator = db_separator_to_export(db_sep);
        // Use GUID instead of position for filename
        let base_name = format!("---{}", separator.guid);
        let base_path = dir_path.join(format!("{base_name}.separator"));
        if pre_existing.contains(&base_path) {
            stats.separators_skipped += 1;
            // Still add to index
            folder_indices
                .entry(dir_path.clone())
                .or_default()
                .push(IndexItem {
                    name: base_name,
                    position: db_sep.position,
                });
            continue;
        }

        let (file_path, index_name) = {
            let mut path = base_path;
            let mut name = base_name.clone();
            let mut counter = 1;
            while path.exists() || written_this_session.contains(&path) {
                name = format!("{base_name}-{counter}");
                path = dir_path.join(format!("{name}.separator"));
                counter += 1;
            }
            (path, name)
        };

        let toml_content = toml::to_string_pretty(&separator)?;
        fs::write(&file_path, toml_content)?;
        written_this_session.insert(file_path);
        stats.separators_exported += 1;

        // Add to folder index
        folder_indices
            .entry(dir_path)
            .or_default()
            .push(IndexItem {
                name: index_name,
                position: db_sep.position,
            });
    }

    // Write __index.toml for each folder with content
    for (dir_path, mut items) in folder_indices {
        // Sort by original DB position
        items.sort_by_key(|item| item.position);

        let index = FolderIndex {
            order: items.into_iter().map(|item| item.name).collect(),
        };

        let index_path = dir_path.join("__index.toml");
        let toml_content = toml::to_string_pretty(&index)?;
        fs::write(index_path, toml_content)?;
        stats.indexes_written += 1;
    }

    Ok(stats)
}

/// Statistics from an export operation.
#[derive(Debug, Default)]
pub struct ExportStats {
    /// Number of folders created.
    pub folders_created: usize,
    /// Number of bookmarks exported.
    pub bookmarks_exported: usize,
    /// Number of bookmarks skipped (already exist).
    pub bookmarks_skipped: usize,
    /// Number of separators exported.
    pub separators_exported: usize,
    /// Number of separators skipped (already exist).
    pub separators_skipped: usize,
    /// Number of __index.toml files written.
    pub indexes_written: usize,
}
