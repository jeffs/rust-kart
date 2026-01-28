//! Firefox profile discovery and validation.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::Connection;

use crate::error::{Error, Result};
use crate::types::Profile;

/// Returns the Firefox application directory.
///
/// On macOS, this is `~/Library/Application Support/Firefox`.
///
/// # Errors
///
/// Returns `FirefoxDirNotFound` if the home directory cannot be determined or
/// the Firefox directory does not exist.
fn firefox_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or(Error::FirefoxDirNotFound)?;
    let firefox = home.join("Library/Application Support/Firefox");
    if firefox.is_dir() {
        Ok(firefox)
    } else {
        Err(Error::FirefoxDirNotFound)
    }
}

/// List all Firefox profiles from Profile Groups databases.
///
/// Scans `~/Library/Application Support/Firefox/Profile Groups/*.sqlite` for
/// profile information. Each profile must have a valid `places.sqlite` to be
/// included.
///
/// # Errors
///
/// Returns an error if the Firefox directory is not found or no Profile Groups
/// databases exist.
pub fn list_profiles() -> Result<Vec<Profile>> {
    let firefox = firefox_dir()?;
    let groups_dir = firefox.join("Profile Groups");

    if !groups_dir.is_dir() {
        return Err(Error::NoProfileGroups);
    }

    // Find all .sqlite files in Profile Groups directory
    let entries = fs::read_dir(&groups_dir)?;
    let mut sqlite_files: Vec<PathBuf> = Vec::new();
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "sqlite") {
            sqlite_files.push(path);
        }
    }

    if sqlite_files.is_empty() {
        return Err(Error::NoProfileGroups);
    }

    // Collect profiles from all databases, deduplicating by path
    let mut profiles_by_path: HashMap<PathBuf, Profile> = HashMap::new();

    for db_path in &sqlite_files {
        if let Ok(conn) = Connection::open(db_path)
            && let Ok(mut stmt) = conn.prepare("SELECT name, path FROM Profiles")
        {
            let rows = stmt.query_map([], |row| {
                let name: String = row.get(0)?;
                let rel_path: String = row.get(1)?;
                Ok((name, rel_path))
            });

            if let Ok(rows) = rows {
                for row in rows.flatten() {
                    let (name, rel_path) = row;
                    // Convert relative path to absolute
                    let abs_path = firefox.join(&rel_path);

                    // Only include profiles with places.sqlite
                    if abs_path.join("places.sqlite").exists() {
                        profiles_by_path.insert(
                            abs_path.clone(),
                            Profile {
                                name,
                                path: abs_path,
                            },
                        );
                    }
                }
            }
        }
    }

    if profiles_by_path.is_empty() {
        return Err(Error::NoProfileGroups);
    }

    // Sort by name for consistent output
    let mut profiles: Vec<Profile> = profiles_by_path.into_values().collect();
    profiles.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(profiles)
}

/// Find a profile by name (case-insensitive exact match).
///
/// # Errors
///
/// Returns `ProfileNotFound` if no profile matches, or `AmbiguousProfile` if
/// multiple profiles have the same name (case-insensitively).
pub fn find_profile(name: &str) -> Result<Profile> {
    let profiles = list_profiles()?;
    let name_lower = name.to_lowercase();

    let matches: Vec<&Profile> = profiles
        .iter()
        .filter(|p| p.name.to_lowercase() == name_lower)
        .collect();

    match matches.len() {
        0 => Err(Error::ProfileNotFound {
            name: name.to_string(),
        }),
        1 => Ok(matches[0].clone()),
        _ => Err(Error::AmbiguousProfile {
            name: name.to_string(),
            matches: matches.iter().map(|p| p.name.clone()).collect(),
        }),
    }
}

/// Check if Firefox is currently running by examining the profile.
///
/// Returns `Ok(())` if Firefox is not running, `Err(FirefoxRunning)` if it is.
///
/// # Errors
///
/// Returns `FirefoxRunning` if Firefox is running, or a database error if the
/// check fails for other reasons.
pub fn check_firefox_not_running(profile_path: &Path) -> Result<()> {
    // Check for .parentlock file with non-zero size or locked
    let parentlock = profile_path.join(".parentlock");
    if parentlock.exists() {
        // On macOS, the file exists but may be empty when Firefox is closed.
        // When Firefox is running, it holds a lock on the file.
        // We can try to open the places.sqlite and check for SQLITE_BUSY.
        let places = profile_path.join("places.sqlite");
        if places.exists() {
            // Try to open with exclusive access
            match Connection::open_with_flags(
                &places,
                rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
            ) {
                Ok(conn) => {
                    // Try a simple query to see if we get SQLITE_BUSY
                    match conn.execute_batch("SELECT 1 FROM moz_bookmarks LIMIT 1") {
                        Ok(()) => Ok(()),
                        Err(e) => {
                            if is_busy_error(&e) {
                                Err(Error::FirefoxRunning)
                            } else {
                                Err(Error::Sqlite(e))
                            }
                        }
                    }
                }
                Err(e) => {
                    if is_busy_error(&e) {
                        Err(Error::FirefoxRunning)
                    } else {
                        Err(Error::Sqlite(e))
                    }
                }
            }
        } else {
            Ok(())
        }
    } else {
        Ok(())
    }
}

/// Check if a rusqlite error indicates the database is locked/busy.
fn is_busy_error(e: &rusqlite::Error) -> bool {
    matches!(
        e,
        rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error {
                code: rusqlite::ffi::ErrorCode::DatabaseBusy
                    | rusqlite::ffi::ErrorCode::DatabaseLocked,
                ..
            },
            _
        )
    )
}
