use std::{
    fs,
    io::{self, Read, Write},
    path::Path,
};

fn rot13(byte: u8) -> u8 {
    match byte {
        b'a'..=b'z' => b'a' + (byte - b'a' + 13) % 26,
        b'A'..=b'Z' => b'A' + (byte - b'A' + 13) % 26,
        b'0'..=b'9' => b'0' + (byte - b'0' + 5) % 10,
        _ => byte,
    }
}

/// # Panics
///
/// Will panic if stdout cannot be written.
fn write_rot13_bytes(bytes: impl IntoIterator<Item = u8>) {
    for byte in bytes {
        io::stdout().write_all(&[rot13(byte)]).unwrap();
    }
}
/// # Panics
///
/// Will panic if stdin cannot be read.
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
