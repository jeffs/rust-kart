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

## File Format: Separators

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
