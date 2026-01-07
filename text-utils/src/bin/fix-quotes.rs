//! Replaces smart quotes with ASCII equivalents.

use std::io;
use text_utils::replace_quotes;

fn main() {
    let input = io::read_to_string(io::stdin()).expect("text from stdin");
    let output = replace_quotes(&input);
    print!("{output}");
}
