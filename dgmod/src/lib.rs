#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

pub mod graph;
pub mod mermaid;
pub mod parser;
pub mod resolver;
pub mod workspace;

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use graph::{EdgeKind, Module, ModuleGraph, ModuleKind, ModulePath};
use parser::{extract_mod_declarations, extract_use_paths, extract_use_statements, parse_file};
use resolver::{
    find_crate_root, is_inline_module, is_internal_path, resolve_module_file, resolve_use_target,
};

/// Errors that can occur during parsing
#[derive(Debug)]
pub enum ParseError {
    /// Failed to read a source file
    Io {
        path: PathBuf,
        error: std::io::Error,
    },
    /// Failed to parse Rust syntax
    Syntax { path: PathBuf, error: syn::Error },
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io { path, error } => {
                write!(f, "Failed to read {}: {error}", path.display())
            }
            Self::Syntax { path, error } => {
                write!(f, "Parse error in {}: {error}", path.display())
            }
        }
    }
}

impl std::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { error, .. } => Some(error),
            Self::Syntax { error, .. } => Some(error),
        }
    }
}

/// Errors that can occur during module resolution
#[derive(Debug)]
pub enum ResolveError {
    /// Module file not found
    ModuleNotFound {
        module_name: String,
        expected_paths: Vec<PathBuf>,
    },
}

impl std::fmt::Display for ResolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ModuleNotFound {
                module_name,
                expected_paths,
            } => {
                write!(f, "Module '{module_name}' not found. Expected:")?;
                for path in expected_paths {
                    write!(f, "\n  - {}", path.display())?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for ResolveError {}

/// Errors that can occur during crate analysis
#[derive(Debug)]
pub enum AnalyzeError {
    /// No crate root found
    NoCrateRoot { path: PathBuf },
    /// Parse error
    Parse(ParseError),
    /// Resolution error
    Resolve(ResolveError),
}

impl std::fmt::Display for AnalyzeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoCrateRoot { path } => {
                write!(
                    f,
                    "No crate root found at {}. Expected src/lib.rs or src/main.rs",
                    path.display()
                )
            }
            Self::Parse(e) => write!(f, "{e}"),
            Self::Resolve(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for AnalyzeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::NoCrateRoot { .. } => None,
            Self::Parse(e) => Some(e),
            Self::Resolve(e) => Some(e),
        }
    }
}

impl From<ParseError> for AnalyzeError {
    fn from(e: ParseError) -> Self {
        Self::Parse(e)
    }
}

impl From<ResolveError> for AnalyzeError {
    fn from(e: ResolveError) -> Self {
        Self::Resolve(e)
    }
}

/// Parsed module info for use statement resolution
struct ParsedModule {
    path: ModulePath,
    file: syn::File,
}

/// Analyze a crate and build its module dependency graph
///
/// # Errors
/// Returns `AnalyzeError` if the crate root is not found, if any source file
/// cannot be parsed, or if a module file cannot be resolved.
pub fn analyze_crate(crate_dir: &Path, crate_name: &str) -> Result<ModuleGraph, AnalyzeError> {
    let root_file = find_crate_root(crate_dir).ok_or_else(|| AnalyzeError::NoCrateRoot {
        path: crate_dir.to_path_buf(),
    })?;

    let mut graph = ModuleGraph::new(crate_name.to_string());
    let mut parsed_modules = Vec::new();

    // Add root module
    let root_path = ModulePath::crate_root();
    graph.add_module(Module {
        path: root_path.clone(),
        source_file: root_file.clone(),
        kind: ModuleKind::Root,
    });

    // Parse root and discover all modules
    let root_parsed = parse_file(&root_file)?;
    discover_modules(
        &root_parsed,
        &root_path,
        &root_file,
        &mut graph,
        &mut parsed_modules,
    )?;
    parsed_modules.push(ParsedModule {
        path: root_path,
        file: root_parsed,
    });

    // Build known modules set for use resolution
    let known_modules: HashSet<ModulePath> = graph.modules().map(|m| m.path.clone()).collect();

    // Process use statements in all parsed modules
    for parsed in &parsed_modules {
        add_use_edges(&parsed.path, &parsed.file, &known_modules, &mut graph);
    }

    Ok(graph)
}

/// Recursively discover modules from mod declarations
fn discover_modules(
    file: &syn::File,
    parent_path: &ModulePath,
    parent_file: &Path,
    graph: &mut ModuleGraph,
    parsed_modules: &mut Vec<ParsedModule>,
) -> Result<(), AnalyzeError> {
    let parent_dir = parent_file.parent().unwrap_or(Path::new("."));

    for item_mod in extract_mod_declarations(file) {
        let mod_name = item_mod.ident.to_string();
        let child_path = parent_path.child(&mod_name);

        // Add mod declaration edge
        graph.add_edge(
            parent_path.clone(),
            child_path.clone(),
            EdgeKind::ModDeclaration,
        );

        if is_inline_module(item_mod) {
            // Inline module - content is in the same file
            graph.add_module(Module {
                path: child_path.clone(),
                source_file: parent_file.to_path_buf(),
                kind: ModuleKind::Inline,
            });

            // Process inline module's items
            if let Some((_, items)) = &item_mod.content {
                // Create a temporary syn::File for the inline module
                let inline_file = syn::File {
                    shebang: None,
                    attrs: vec![],
                    items: items.clone(),
                };
                discover_modules(
                    &inline_file,
                    &child_path,
                    parent_file,
                    graph,
                    parsed_modules,
                )?;
                parsed_modules.push(ParsedModule {
                    path: child_path,
                    file: inline_file,
                });
            }
        } else {
            // External module - resolve file path
            let child_file = resolve_module_file(parent_dir, &mod_name, item_mod)?;

            graph.add_module(Module {
                path: child_path.clone(),
                source_file: child_file.clone(),
                kind: ModuleKind::External,
            });

            // Parse and recursively discover
            let child_parsed = parse_file(&child_file)?;

            discover_modules(
                &child_parsed,
                &child_path,
                &child_file,
                graph,
                parsed_modules,
            )?;
            parsed_modules.push(ParsedModule {
                path: child_path,
                file: child_parsed,
            });
        }
    }

    Ok(())
}

/// Add edges for use statements in a module
fn add_use_edges(
    module_path: &ModulePath,
    file: &syn::File,
    known_modules: &HashSet<ModulePath>,
    graph: &mut ModuleGraph,
) {
    for item_use in extract_use_statements(file) {
        for segments in extract_use_paths(item_use) {
            if !is_internal_path(&segments) {
                continue;
            }

            if let Some(target) = resolve_use_target(&segments, module_path, known_modules) {
                graph.add_edge(module_path.clone(), target, EdgeKind::UseImport);
            }
        }
    }
}
