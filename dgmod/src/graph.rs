//! Graph data structures for module dependencies

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/// Newtype wrapper for module paths (e.g., "crate", `alpha::delta`)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModulePath(String);

impl ModulePath {
    /// Create the crate root path
    #[must_use]
    pub fn crate_root() -> Self {
        Self("crate".to_string())
    }

    /// Create a child module path
    #[must_use]
    pub fn child(&self, name: &str) -> Self {
        if self.0 == "crate" {
            Self(name.to_string())
        } else {
            Self(format!("{}::{name}", self.0))
        }
    }

    /// Get the string representation
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The kind of module
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleKind {
    /// Crate root (lib.rs or main.rs)
    Root,
    /// Inline module: `mod foo { ... }`
    Inline,
    /// External file: `mod foo;` → foo.rs or foo/mod.rs
    External,
}

/// How a dependency edge was established
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeKind {
    /// Parent declares child: `mod foo;`
    ModDeclaration,
    /// Import via use statement: `use foo::Bar;`
    UseImport,
}

/// A module within a crate
#[derive(Debug, Clone)]
pub struct Module {
    /// Fully qualified path (e.g., "crate", `alpha::delta`)
    pub path: ModulePath,
    /// Absolute path to the source file
    pub source_file: PathBuf,
    /// Whether this is root, inline, or external
    pub kind: ModuleKind,
}

/// The complete dependency graph for a crate
#[derive(Debug)]
pub struct ModuleGraph {
    /// Name of the crate
    pub crate_name: String,
    /// All modules indexed by path
    modules: HashMap<ModulePath, Module>,
    /// Deduplicated edges (from, to)
    edges: HashSet<(ModulePath, ModulePath)>,
}

impl ModuleGraph {
    /// Create a new empty graph for the given crate
    #[must_use]
    pub fn new(crate_name: String) -> Self {
        Self {
            crate_name,
            modules: HashMap::new(),
            edges: HashSet::new(),
        }
    }

    /// Add a module to the graph
    pub fn add_module(&mut self, module: Module) {
        self.modules.insert(module.path.clone(), module);
    }

    /// Add an edge between two modules (no self-edges allowed)
    pub fn add_edge(&mut self, from: ModulePath, to: ModulePath) {
        if from != to {
            self.edges.insert((from, to));
        }
    }

    /// Iterate over all modules
    pub fn modules(&self) -> impl Iterator<Item = &Module> {
        self.modules.values()
    }

    /// Iterate over all edges as (from, to) pairs
    pub fn edges(&self) -> impl Iterator<Item = (&ModulePath, &ModulePath)> {
        self.edges.iter().map(|(from, to)| (from, to))
    }
}
