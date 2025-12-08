//! Extracts front matter (such as Obsidian YAML) from Markdown files.

fn main() {
    let stdin = std::io::stdin();
    let mut lines = stdin.lines();
    let Some("---") = lines.next().and_then(Result::ok).as_deref() else {
        return;
    };
    while let Some(Ok(line)) = lines.next()
        && line != "---"
    {
        println!("{line}");
    }
}
