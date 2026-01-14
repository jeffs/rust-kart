use crate::branch::{Node, Topology};

/// Render the branch topology as ASCII art.
#[must_use]
pub fn render(topology: &Topology) -> String {
    let mut lines = Vec::new();

    // Render root node.
    render_node(&topology.root, &mut lines, "", true);

    lines.join("\n") + "\n"
}

fn render_node(node: &Node, lines: &mut Vec<String>, prefix: &str, is_root: bool) {
    // Format this node.
    let node_text = format_node(node);

    if is_root {
        lines.push(node_text);
    } else {
        // This is handled by the parent's render_children call.
        unreachable!()
    }

    // Render children.
    render_children(&node.children, lines, prefix);
}

fn render_children(children: &[Node], lines: &mut Vec<String>, prefix: &str) {
    let count = children.len();
    for (i, node) in children.iter().enumerate() {
        let is_last = i == count - 1;
        let connector = if is_last { "└─ " } else { "├─ " };
        let child_prefix = if is_last { "   " } else { "│  " };

        lines.push(format!("{prefix}{connector}{}", format_node(node)));

        if !node.children.is_empty() {
            render_children(&node.children, lines, &format!("{prefix}{child_prefix}"));
        }
    }
}

/// Format a node for display.
fn format_node(node: &Node) -> String {
    if node.branches.is_empty() {
        // Pure commit node: show hash in brackets.
        format!("[{}]", node.commit)
    } else if node.branches.len() == 1 {
        // Single branch: show name and ahead count.
        let info = &node.branches[0];
        if info.ahead > 0 {
            format!("{} (+{})", info.name, info.ahead)
        } else {
            info.name.clone()
        }
    } else {
        // Multiple branches at same commit: show all names, ahead count once at end.
        let names: Vec<&str> = node.branches.iter().map(|info| info.name.as_str()).collect();
        let ahead = node.branches[0].ahead;
        if ahead > 0 {
            format!("{} (+{})", names.join(", "), ahead)
        } else {
            names.join(", ")
        }
    }
}
