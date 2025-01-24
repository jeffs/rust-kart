//! Prints lines from standard input that contain only hexadecimal digits, where 'a' through 'f' are
//! hexadecimal digits, as well as 'o' and 'i' (rendered as '0' and '1').  For example, here are
//! forty [frequent English words][2] of at least length 4 that may be thus represented:
//!
//! ```sh
//! $ sed 's/,.*//' var/unigram_freq.csv | cargo -q run | rg '^.{4}' | head -40 | column
//! 0ff1ce  face    c0ffee  ad0be   cafe    faced   0ecd    bead    c0ca    dec0
//! c0de    1dea    d1ed    d1ff    1eee    b00b    d1ce    debb1e  ab1de   acad
//! f00d    feed    dec1ded babe    beef    c1a0    deaf    c0ded   fade    f1fa
//! added   dead    ac1d    dec1de  decade  edd1e   fe0f    deed    c0c0a   a1ded
//! ```
//!
//! Inspired by [The Complete List of Hex Words][1], though I haven't seen its source code.
//!
//! [1]: https://stoney.sb.org/wordpress/the-complete-list-of-hex-words/
//! [2]: https://www.kaggle.com/datasets/rtatman/english-word-frequency

use std::io;
use std::process::exit;

fn to_hex(c: char) -> Option<char> {
    match c {
        'a' | 'b' | 'c' | 'd' | 'e' | 'f' | 'A' | 'B' | 'C' | 'D' | 'E' | 'F' => Some(c),
        'i' | 'I' => Some('1'),
        'o' | 'O' => Some('0'),
        _ if c.is_alphabetic() => None,
        _ => Some(c), // Apostrophes, hyphens, etc.
    }
}

fn main() {
    let mut buf = String::new();
    for line in io::stdin().lines() {
        let line = match line {
            Ok(line) => line,
            Err(err) if err.kind() == io::ErrorKind::InvalidData => continue,
            Err(_) => exit(1),
        };

        buf.clear();
        buf.extend(line.chars().filter_map(to_hex));
        if buf.chars().count() != line.chars().count() {
            continue;
        }

        println!("{buf}");
    }
}
