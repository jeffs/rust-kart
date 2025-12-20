//! Rust source file parsing using syn

use std::fs;
use std::path::Path;

use syn::{
    Attribute, Expr, ExprLit, Item, ItemMod, ItemUse, Lit, Meta, UseGroup, UseName, UsePath,
    UseRename, UseTree,
};

use crate::ParseError;

/// Parse a Rust source file
///
/// # Errors
/// Returns `ParseError::Io` if the file cannot be read, or `ParseError::Syntax`
/// if the file contains invalid Rust syntax.
pub fn parse_file(path: &Path) -> Result<syn::File, ParseError> {
    let content = fs::read_to_string(path).map_err(|error| ParseError::Io {
        path: path.to_path_buf(),
        error,
    })?;
    syn::parse_file(&content).map_err(|error| ParseError::Syntax {
        path: path.to_path_buf(),
        error,
    })
}

/// Extract all mod declarations from a parsed file
#[must_use]
pub fn extract_mod_declarations(file: &syn::File) -> Vec<&ItemMod> {
    file.items
        .iter()
        .filter_map(|item| {
            if let Item::Mod(item_mod) = item {
                Some(item_mod)
            } else {
                None
            }
        })
        .collect()
}

/// Extract all use statements from a parsed file
#[must_use]
pub fn extract_use_statements(file: &syn::File) -> Vec<&ItemUse> {
    file.items
        .iter()
        .filter_map(|item| {
            if let Item::Use(item_use) = item {
                Some(item_use)
            } else {
                None
            }
        })
        .collect()
}

/// Get the `#[path = "..."]` attribute value if present
#[must_use]
pub fn get_path_attribute(item_mod: &ItemMod) -> Option<String> {
    find_path_in_attrs(&item_mod.attrs)
}

/// Helper to find path attribute in a list of attributes
fn find_path_in_attrs(attrs: &[Attribute]) -> Option<String> {
    for attr in attrs {
        if attr.path().is_ident("path") {
            if let Meta::NameValue(nv) = &attr.meta {
                if let Expr::Lit(ExprLit {
                    lit: Lit::Str(s), ..
                }) = &nv.value
                {
                    return Some(s.value());
                }
            }
        }
    }
    None
}

/// Walk a `UseTree` and extract all import paths as vectors of segments
pub fn walk_use_tree(
    tree: &UseTree,
    current_path: &mut Vec<String>,
    results: &mut Vec<Vec<String>>,
) {
    match tree {
        UseTree::Path(UsePath { ident, tree, .. }) => {
            current_path.push(ident.to_string());
            walk_use_tree(tree, current_path, results);
            current_path.pop();
        }
        UseTree::Name(UseName { ident, .. }) | UseTree::Rename(UseRename { ident, .. }) => {
            current_path.push(ident.to_string());
            results.push(current_path.clone());
            current_path.pop();
        }
        UseTree::Glob(_) => {
            // For glob imports, we add edge to the module itself
            results.push(current_path.clone());
        }
        UseTree::Group(UseGroup { items, .. }) => {
            for item in items {
                walk_use_tree(item, current_path, results);
            }
        }
    }
}

/// Extract all import paths from a use statement
#[must_use]
pub fn extract_use_paths(item_use: &ItemUse) -> Vec<Vec<String>> {
    let mut results = Vec::new();
    let mut path = Vec::new();
    walk_use_tree(&item_use.tree, &mut path, &mut results);
    results
}
