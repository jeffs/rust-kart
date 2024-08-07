use std::io;

// TODO: Fix single quotes.
fn replace_quotes(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            '“' | '”' => '"',
            '‘' | '’' => '\'',
            _ => c,
        })
        .collect()
}

fn main() {
    let input = io::read_to_string(io::stdin()).expect("text from stdin");
    let output = replace_quotes(&input);
    println!("{output}");
}
