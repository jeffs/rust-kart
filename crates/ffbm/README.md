# ffbm - Firefox Bookmark Manager

CLI tool for managing Firefox bookmarks through a version-control-friendly workflow.

> [!caution]
> This is a brand new thing that was mostly AI-coded. It might have any number of bugs, limitations, or weird edge cases. For example:
>
> - It relies on a particuar, macOS-specific file layout.
> - I am woefully unfamiliar with how Firefox actually works, so I probably got stuff wrong.
> - I'm not sure how this interacts with Firefox syncing bookmarks across machines.
> - There's no real UI. Using this requires running commands and editing TOML files.

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

## Testing

If you're going to test this manually (as I did), do it using a throwaway profile. It's fairly easy to duplicate an existing profile through the Firefox UI, then use the duplicate for testing.

```bash
# List profiles to find a test target
cargo run -p ffbm -- profiles

# Test save/export/import cycle (close Firefox first)
cargo run -p ffbm -- save --profile "Your Profile Name"
cargo run -p ffbm -- export
cargo run -p ffbm -- import
```
