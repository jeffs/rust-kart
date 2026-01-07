//! Module path resolution

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use syn::ItemMod;

use crate::graph::ModulePath;
use crate::parser::get_path_attribute;
use crate::ResolveError;

/// Find the crate root file (lib.rs or main.rs)
#[must_use]
pub fn find_crate_root(crate_dir: &Path) -> Option<PathBuf> {
    let src_dir = crate_dir.join("src");

    let lib_rs = src_dir.join("lib.rs");
    if lib_rs.exists() {
        return Some(lib_rs);
    }

    let main_rs = src_dir.join("main.rs");
    if main_rs.exists() {
        return Some(main_rs);
    }

    None
}

/// Resolve the file path for a module declaration
///
/// # Errors
/// Returns `ResolveError::ModuleNotFound` if the module file cannot be found.
pub fn resolve_module_file(
    parent_dir: &Path,
    mod_name: &str,
    item_mod: &ItemMod,
) -> Result<PathBuf, ResolveError> {
    // Check for #[path = "..."] attribute
    if let Some(custom_path) = get_path_attribute(item_mod) {
        return Ok(parent_dir.join(custom_path));
    }

    // Standard module resolution: try mod_name.rs first, then mod_name/mod.rs
    let direct = parent_dir.join(format!("{mod_name}.rs"));
    if direct.exists() {
        return Ok(direct);
    }

    let nested = parent_dir.join(mod_name).join("mod.rs");
    if nested.exists() {
        return Ok(nested);
    }

    Err(ResolveError::ModuleNotFound {
        module_name: mod_name.to_string(),
        expected_paths: vec![direct, nested],
    })
}

/// Check if a mod declaration is inline (has body) vs external (just declaration)
#[must_use]
pub fn is_inline_module(item_mod: &ItemMod) -> bool {
    item_mod.content.is_some()
}

/// Check if a use path is internal (`crate::`, `self::`, `super::`)
#[must_use]
pub fn is_internal_path(segments: &[String]) -> bool {
    if segments.is_empty() {
        return false;
    }
    matches!(segments[0].as_str(), "crate" | "self" | "super")
}

/// Resolve a use path to its target module path
/// Returns None if the path refers to an external crate or cannot be resolved
#[must_use]
#[allow(clippy::implicit_hasher)]
pub fn resolve_use_target(
    segments: &[String],
    current_module: &ModulePath,
    known_modules: &HashSet<ModulePath>,
) -> Option<ModulePath> {
    if segments.is_empty() {
        return None;
    }

    match segments[0].as_str() {
        "crate" => {
            // crate::foo::bar -> resolve from root
            resolve_from_crate_root(&segments[1..], known_modules)
        }
        "self" => {
            // self::foo -> resolve from current module
            resolve_relative(current_module, &segments[1..], known_modules)
        }
        "super" => {
            // super::foo -> resolve from parent module
            let parent = get_parent_module(current_module)?;
            // Handle multiple super:: prefixes
            let mut current = parent;
            let mut remaining = &segments[1..];
            while !remaining.is_empty() && remaining[0] == "super" {
                current = get_parent_module(&current)?;
                remaining = &remaining[1..];
            }
            resolve_relative(&current, remaining, known_modules)
        }
        _ => {
            // External crate - not internal
            None
        }
    }
}

/// Resolve path starting from crate root
fn resolve_from_crate_root(
    segments: &[String],
    known_modules: &HashSet<ModulePath>,
) -> Option<ModulePath> {
    resolve_relative(&ModulePath::crate_root(), segments, known_modules)
}

/// Resolve path relative to a module, finding the deepest matching known module
fn resolve_relative(
    base: &ModulePath,
    segments: &[String],
    known_modules: &HashSet<ModulePath>,
) -> Option<ModulePath> {
    if segments.is_empty() {
        // Importing from the base module itself
        if known_modules.contains(base) {
            return Some(base.clone());
        }
        return None;
    }

    // Try to find the deepest module that matches
    let mut current = base.clone();
    let mut last_known = if known_modules.contains(&current) {
        Some(current.clone())
    } else {
        None
    };

    for segment in segments {
        current = current.child(segment);
        if known_modules.contains(&current) {
            last_known = Some(current.clone());
        }
    }

    last_known
}

/// Get parent module path
fn get_parent_module(module: &ModulePath) -> Option<ModulePath> {
    let s = module.as_str();
    if s == "crate" {
        return None;
    }
    if let Some(pos) = s.rfind("::") {
        Some(ModulePath::crate_root().child(&s[..pos]))
    } else {
        // Top-level module, parent is crate
        Some(ModulePath::crate_root())
    }
}
