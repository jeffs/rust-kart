//! Graph data structures for module dependencies

use std::collections::HashMap;
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

    /// Check if this path represents a tests module
    ///
    /// Returns true if the path is "tests" or ends with `::tests`
    #[must_use]
    pub fn is_tests_module(&self) -> bool {
        self.0 == "tests" || self.0.ends_with("::tests")
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
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
    /// Deduplicated edges: (from, to) -> kind
    /// If both `ModDeclaration` and `UseImport` exist for the same edge,
    /// `ModDeclaration` takes precedence.
    edges: HashMap<(ModulePath, ModulePath), EdgeKind>,
}

impl ModuleGraph {
    /// Create a new empty graph for the given crate
    #[must_use]
    pub fn new(crate_name: String) -> Self {
        Self {
            crate_name,
            modules: HashMap::new(),
            edges: HashMap::new(),
        }
    }

    /// Add a module to the graph
    pub fn add_module(&mut self, module: Module) {
        self.modules.insert(module.path.clone(), module);
    }

    /// Add an edge between two modules (no self-edges allowed)
    ///
    /// If an edge already exists between the same modules, `ModDeclaration`
    /// takes precedence over `UseImport`.
    pub fn add_edge(&mut self, from: ModulePath, to: ModulePath, kind: EdgeKind) {
        if from == to {
            return;
        }
        let key = (from, to);
        self.edges
            .entry(key)
            .and_modify(|existing| {
                // ModDeclaration takes precedence over UseImport
                if kind == EdgeKind::ModDeclaration {
                    *existing = kind;
                }
            })
            .or_insert(kind);
    }

    /// Iterate over all modules
    pub fn modules(&self) -> impl Iterator<Item = &Module> {
        self.modules.values()
    }

    /// Iterate over all edges as (from, to, kind) tuples
    pub fn edges(&self) -> impl Iterator<Item = (&ModulePath, &ModulePath, EdgeKind)> {
        self.edges
            .iter()
            .map(|((from, to), kind)| (from, to, *kind))
    }

    /// Remove all `tests` modules and their associated edges from the graph
    ///
    /// This removes any module whose path is "tests" or ends with `::tests`,
    /// along with any edges to or from those modules.
    pub fn exclude_tests_modules(&mut self) {
        // Collect paths to remove
        let tests_paths: Vec<ModulePath> = self
            .modules
            .keys()
            .filter(|path| path.is_tests_module())
            .cloned()
            .collect();

        // Remove the modules
        for path in &tests_paths {
            self.modules.remove(path);
        }

        // Remove edges to/from tests modules
        self.edges
            .retain(|(from, to), _| !from.is_tests_module() && !to.is_tests_module());
    }
}
