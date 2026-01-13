use crate::branch::{Branch, Topology};

/// Render the branch topology as ASCII art.
#[must_use]
pub fn render(topology: &Topology) -> String {
    if topology.branches.is_empty() {
        return format!("{} (no other branches)\n", topology.trunk);
    }

    let mut lines = Vec::new();
    lines.push(topology.trunk.clone());

    render_branches(&topology.branches, &mut lines, "");

    lines.join("\n") + "\n"
}

fn render_branches(branches: &[Branch], lines: &mut Vec<String>, prefix: &str) {
    let count = branches.len();
    for (i, branch) in branches.iter().enumerate() {
        let is_last = i == count - 1;
        let connector = if is_last { "└─ " } else { "├─ " };
        let child_prefix = if is_last { "   " } else { "│  " };

        let stats = format_stats(branch.ahead, branch.behind);
        lines.push(format!("{prefix}{connector}{}{stats}", branch.name));

        if !branch.children.is_empty() {
            render_branches(&branch.children, lines, &format!("{prefix}{child_prefix}"));
        }
    }
}

fn format_stats(ahead: usize, behind: usize) -> String {
    match (ahead, behind) {
        (0, 0) => String::new(),
        (a, 0) => format!(" (+{a})"),
        (0, b) => format!(" (-{b})"),
        (a, b) => format!(" (+{a}, -{b})"),
    }
}
