//! Mermaid diagram output formatting

use std::fmt::Write;

use crate::graph::{EdgeKind, ModuleGraph};

/// Sanitize a module path for use as a Mermaid node ID
/// Replaces :: with _ since Mermaid doesn't allow :: in IDs
fn sanitize_id(path: &str) -> String {
    path.replace("::", "_")
}

/// Emit node declarations for all modules
fn emit_nodes(graph: &ModuleGraph, output: &mut String) {
    let mut paths: Vec<_> = graph.modules().map(|m| m.path.as_str()).collect();
    paths.sort_unstable();

    for path in paths {
        let id = sanitize_id(path);
        let _ = writeln!(output, "    {id}[\"{path}\"]");
    }
}

/// Emit edge declarations
fn emit_edges(graph: &ModuleGraph, output: &mut String) {
    let mut edges: Vec<_> = graph
        .edges()
        .map(|(from, to, kind)| (from.as_str(), to.as_str(), kind))
        .collect();
    edges.sort_unstable();

    for (from, to, kind) in edges {
        let from_id = sanitize_id(from);
        let to_id = sanitize_id(to);
        // Use solid arrow for mod declarations, dashed for use imports
        let arrow = match kind {
            EdgeKind::ModDeclaration => "-->",
            EdgeKind::UseImport => "-.->",
        };
        let _ = writeln!(output, "    {from_id} {arrow} {to_id}");
    }
}

impl ModuleGraph {
    /// Convert the graph to Mermaid flowchart syntax
    #[must_use]
    pub fn to_mermaid(&self) -> String {
        let mut output = String::from("flowchart TD\n");

        emit_nodes(self, &mut output);
        output.push('\n');
        emit_edges(self, &mut output);

        output
    }
}
