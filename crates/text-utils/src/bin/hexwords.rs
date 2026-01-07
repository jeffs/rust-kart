//! Filters stdin for words representable in hexadecimal.
//!
//! Prints lines that contain only hex digits, where 'o' and 'i' become '0' and '1'.

use std::io;
use std::process::exit;
use text_utils::to_hex_char;

fn main() {
    let mut buf = String::new();
    for line in io::stdin().lines() {
        let line = match line {
            Ok(line) => line,
            Err(err) if err.kind() == io::ErrorKind::InvalidData => continue,
            Err(_) => exit(1),
        };

        buf.clear();
        buf.extend(line.chars().filter_map(to_hex_char));
        if buf.chars().count() != line.chars().count() {
            continue;
        }

        println!("{buf}");
    }
}
