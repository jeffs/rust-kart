//! Firefox Bookmark Manager CLI.
//!
//! A tool for managing Firefox bookmarks through a version-control-friendly
//! workflow.

use std::fs;
use std::path::{Path, PathBuf};

use clap::{Args, Parser, Subcommand};
use ffbm::{
    check_firefox_not_running, export_bookmarks, find_profile, import_bookmarks, list_profiles,
    Error, Result,
};

/// Files to copy during save/restore operations.
const DATABASE_FILES: &[&str] = &[
    "places.sqlite",
    "places.sqlite-wal",
    "places.sqlite-shm",
    "favicons.sqlite",
    "favicons.sqlite-wal",
    "favicons.sqlite-shm",
];

/// Files that must exist for a valid backup.
const REQUIRED_FILES: &[&str] = &["places.sqlite"];

#[derive(Parser)]
#[command(name = "ffbm")]
#[command(about = "Firefox Bookmark Manager - manage bookmarks via file tree")]
#[command(arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

/// Arguments for commands that require a Firefox profile.
#[derive(Args)]
struct ProfileArgs {
    /// Name of the Firefox profile (from Profile Groups).
    #[arg(long)]
    profile: String,
}

#[derive(Subcommand)]
enum Command {
    /// List available Firefox profiles.
    Profiles,
    /// Copy Firefox bookmark databases to saved/ directory.
    Save(ProfileArgs),
    /// Export saved database to TOML file tree in exported/ directory.
    Export,
    /// Update saved database from modified file tree.
    Import,
    /// Copy saved database back to Firefox profile.
    Restore(ProfileArgs),
}

/// Copy database files from source to destination.
fn copy_database_files(src: &Path, dst: &Path, files: &[&str]) -> Result<usize> {
    fs::create_dir_all(dst)?;

    let mut copied = 0;
    for file in files {
        let src_path = src.join(file);
        let dst_path = dst.join(file);

        if src_path.exists() {
            fs::copy(&src_path, &dst_path)?;
            copied += 1;
        }
    }

    Ok(copied)
}

/// Verify required files exist in a directory.
fn verify_required_files(dir: &Path, files: &[&str]) -> Result<()> {
    for file in files {
        let path = dir.join(file);
        if !path.exists() {
            return Err(Error::DatabaseNotFound {
                name: (*file).to_string(),
                path: dir.to_path_buf(),
            });
        }
    }
    Ok(())
}

/// Create a backup of the profile before restore.
fn create_backup(profile: &Path) -> Result<PathBuf> {
    let backup_dir = profile.join(".ffbm-backup");
    fs::create_dir_all(&backup_dir)?;

    for file in DATABASE_FILES {
        let src = profile.join(file);
        let dst = backup_dir.join(file);
        if src.exists() {
            fs::copy(&src, &dst)?;
        }
    }

    Ok(backup_dir)
}

/// Remove WAL/SHM files to force database rebuild.
fn remove_wal_files(dir: &Path) -> Result<()> {
    for suffix in ["-wal", "-shm"] {
        for base in ["places.sqlite", "favicons.sqlite"] {
            let path = dir.join(format!("{base}{suffix}"));
            if path.exists() {
                fs::remove_file(&path)?;
            }
        }
    }
    Ok(())
}

/// Execute the profiles command.
fn cmd_profiles() -> Result<()> {
    let profiles = list_profiles()?;

    for profile in profiles {
        println!("{}", profile.name);
        println!("  {}", profile.path.display());
    }

    Ok(())
}

/// Execute the save command.
fn cmd_save(profile: &Path) -> Result<()> {
    check_firefox_not_running(profile)?;

    let saved_dir = PathBuf::from("saved");
    let copied = copy_database_files(profile, &saved_dir, DATABASE_FILES)?;

    println!("Saved {copied} database files to saved/");
    Ok(())
}

/// Execute the export command.
fn cmd_export() -> Result<()> {
    let saved_dir = PathBuf::from("saved");
    verify_required_files(&saved_dir, REQUIRED_FILES)?;

    let export_dir = PathBuf::from("exported");
    let db_path = saved_dir.join("places.sqlite");

    let stats = export_bookmarks(&db_path, &export_dir)?;

    println!(
        "Exported {} bookmarks, {} separators ({} folders, {} indexes)",
        stats.bookmarks_exported,
        stats.separators_exported,
        stats.folders_created,
        stats.indexes_written
    );
    if stats.bookmarks_skipped > 0 || stats.separators_skipped > 0 {
        println!(
            "Skipped {} existing bookmarks, {} existing separators",
            stats.bookmarks_skipped, stats.separators_skipped
        );
    }

    Ok(())
}

/// Execute the import command.
fn cmd_import() -> Result<()> {
    let saved_dir = PathBuf::from("saved");
    verify_required_files(&saved_dir, REQUIRED_FILES)?;

    let export_dir = PathBuf::from("exported");
    if !export_dir.exists() {
        return Err(Error::DatabaseNotFound {
            name: "exported directory".to_string(),
            path: export_dir,
        });
    }

    let db_path = saved_dir.join("places.sqlite");
    let stats = import_bookmarks(&db_path, &export_dir)?;

    println!(
        "Imported {} bookmarks, {} separators ({} folders created)",
        stats.bookmarks_imported, stats.separators_imported, stats.folders_created
    );

    Ok(())
}

/// Execute the restore command.
fn cmd_restore(profile: &Path) -> Result<()> {
    check_firefox_not_running(profile)?;

    let saved_dir = PathBuf::from("saved");
    verify_required_files(&saved_dir, REQUIRED_FILES)?;

    // Create backup
    let backup_dir = create_backup(profile)?;
    println!("Created backup in {}", backup_dir.display());

    // Remove WAL/SHM files from saved to force clean restore
    remove_wal_files(&saved_dir)?;

    // Copy files to profile
    let copied = copy_database_files(&saved_dir, profile, DATABASE_FILES)?;

    // Remove WAL/SHM files from profile to force Firefox to rebuild
    remove_wal_files(profile)?;

    println!("Restored {copied} database files to Firefox profile");
    Ok(())
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Command::Profiles => cmd_profiles(),
        Command::Save(args) => {
            let profile = match find_profile(&args.profile) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            };
            cmd_save(&profile.path)
        }
        Command::Export => cmd_export(),
        Command::Import => cmd_import(),
        Command::Restore(args) => {
            let profile = match find_profile(&args.profile) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            };
            cmd_restore(&profile.path)
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
