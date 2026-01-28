# ffbm - Firefox Bookmark Manager

CLI tool for managing Firefox bookmarks through a version-control-friendly workflow.

## Quick Start

```bash
# List available Firefox profiles
cargo run -p ffbm -- profiles

# Close Firefox first, then:
cargo run -p ffbm -- save --profile "Original profile"  # Copy DBs to ./saved/
cargo run -p ffbm -- export                             # Export to TOML files
# Edit files in ./exported/ as desired
cargo run -p ffbm -- import                             # Update ./saved/ DBs
cargo run -p ffbm -- restore --profile "Original profile"  # Copy back to Firefox
```

## Commands

- `profiles` - List all available Firefox profiles (from Profile Groups)
- `save --profile NAME` - Copy database files from Firefox profile to `./saved/`
- `export` - Export `./saved/` database to TOML files in `./exported/`
- `import` - Update `./saved/` database from modified files in `./exported/`
- `restore --profile NAME` - Copy `./saved/` database back to Firefox profile

Profile names are case-insensitive and must match exactly.

## Ordering with `__index.toml`

Each folder contains an `__index.toml` file that controls item ordering:

```toml
order = [
    "My Bookmark",          # bookmark (filename sans .toml)
    "Subfolder/",           # folder (trailing /)
    "---abc123def456",      # separator (--- prefix + guid)
    "Another Bookmark",
]
```

**Reordering items:** Edit the `order` array to change positions. On import,
items are assigned positions 0, 1, 2, ... based on their array index.

**Adding items:** New files not in the index appear after indexed items
(sorted alphabetically).

**Removing items:** Index entries without matching files are silently ignored.

## Architecture

```
src/
  main.rs      # CLI entry, commands
  lib.rs       # Public API
  error.rs     # Error types
  profile.rs   # Firefox profile discovery (Profile Groups/*.sqlite)
  types.rs     # Bookmark, Separator, Folder, FolderIndex, Profile types
  db.rs        # SQLite read/write operations
  export.rs    # DB -> TOML file tree (writes __index.toml for ordering)
  import.rs    # TOML file tree -> DB (reads __index.toml for ordering)
```

## Firefox Profile Groups

Firefox stores profile information in SQLite databases under
`~/Library/Application Support/Firefox/Profile Groups/`. Each database has a
`Profiles` table with columns including `name` and `path` (relative to the
Firefox directory).

The `profiles` command scans these databases and lists all profiles that have
a valid `places.sqlite` file.

## Firefox Database

- `places.sqlite`: bookmarks in `moz_bookmarks`, URLs in `moz_places`, keywords
  in `moz_keywords`
- Root folder IDs: 2=menu, 3=toolbar, 4=tags (internal), 5=other, 6=mobile
- Tags are folders under parent=4 with child bookmarks pointing to same place_id

## File Format

Bookmarks are stored as TOML files without embedded position data:

```toml
url = "https://example.com"
title = "Example Site"
tags = ["tag1", "tag2"]
keyword = "ex"
date_added = 1234567890000000
last_modified = 1234567890000000
guid = "abc123def456"
```

Separators use GUID-based filenames (`---{guid}.separator`):

```toml
date_added = 1234567890000000
last_modified = 1234567890000000
guid = "abc123def456"
```

## Known Issues

- Bookmark count mismatch: DB has ~1581 bookmarks, export yields ~1485 (some
  filtered)
- No validation that imported GUIDs are unique before insert
- Re-export skip logic uses base filename; same-title bookmarks share skip
  decision

## Testing

```bash
# List profiles to find a test target
cargo run -p ffbm -- profiles

# Test save/export/import cycle (close Firefox first)
cargo run -p ffbm -- save --profile "Your Profile Name"
cargo run -p ffbm -- export
cargo run -p ffbm -- import
```
