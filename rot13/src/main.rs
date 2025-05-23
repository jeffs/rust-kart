use std::io::{self, Read, Write};

fn main() {
    for byte in io::BufReader::new(io::stdin()).bytes() {
        let Ok(byte) = byte else {
            return;
        };
        let byte = match byte {
            b'a'..=b'z' => b'a' + (byte - b'a' + 13) % 26,
            b'A'..=b'Z' => b'A' + (byte - b'A' + 13) % 26,
            b'0'..=b'9' => b'0' + (byte - b'0' + 5) % 10,
            _ => byte,
        };
        if io::stdout().write_all(&[byte]).is_err() {
            return;
        }
    }
}
