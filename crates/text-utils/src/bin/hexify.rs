//! Converts decimal digit sequences in arguments to hexadecimal.

use text_utils::hexify;

fn main() {
    std::env::args()
        .skip(1)
        .map(|arg| hexify(&arg))
        .for_each(|result| println!("{result}"));
}
