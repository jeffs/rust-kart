//! ROT13 cipher that also rotates digits by 5.

use std::{
    fs,
    io::{self, Read, Write},
    path::Path,
};
use text_utils::rot13_byte;

fn write_rot13_bytes(bytes: impl IntoIterator<Item = u8>) {
    for byte in bytes {
        io::stdout().write_all(&[rot13_byte(byte)]).unwrap();
    }
}

fn process_stdin() {
    write_rot13_bytes(io::BufReader::new(io::stdin()).bytes().map(Result::unwrap));
}

fn process_file(path: &Path) {
    let bytes = fs::read(path).unwrap();
    write_rot13_bytes(bytes);
}

fn main() {
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    if args.is_empty() {
        process_stdin();
    } else {
        for path in args {
            process_file(Path::new(&path));
        }
    }
}
