//! Text transformation utilities: ROT13, quote normalization, hex conversion.

use std::fmt::Write;

/// Applies ROT13 to a byte, rotating letters by 13 and digits by 5.
#[must_use]
pub fn rot13_byte(byte: u8) -> u8 {
    match byte {
        b'a'..=b'z' => b'a' + (byte - b'a' + 13) % 26,
        b'A'..=b'Z' => b'A' + (byte - b'A' + 13) % 26,
        b'0'..=b'9' => b'0' + (byte - b'0' + 5) % 10,
        _ => byte,
    }
}

/// Replaces smart quotes with ASCII equivalents.
#[must_use]
pub fn replace_quotes(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            '\u{201c}' | '\u{201d}' => '"',
            '\u{2018}' | '\u{2019}' => '\'',
            _ => c,
        })
        .collect()
}

/// Converts decimal digit sequences in text to hexadecimal.
///
/// # Panics
///
/// Panics if a decimal number in the text is too large to fit in a `u64`.
#[must_use]
pub fn hexify(mut text: &str) -> String {
    let mut result = String::new();
    while let Some(begin) = text.find(|c: char| c.is_ascii_digit()) {
        let next = begin + 1;
        let rest = &text[next..];
        let end = next
            + rest
                .find(|c: char| !c.is_ascii_digit())
                .unwrap_or(rest.len());
        let value: u64 = text[begin..end]
            .parse()
            .expect("Can't parse ASCII digits as u64; value too big?");
        write!(&mut result, "{}{value:x}", &text[..begin]).expect("write to string should succeed");
        text = &text[end..];
    }
    result + text
}

/// Converts a character to its hex representation if possible.
/// Returns `None` for non-hex-representable alphabetic characters.
#[must_use]
pub fn to_hex_char(c: char) -> Option<char> {
    match c {
        'a' | 'b' | 'c' | 'd' | 'e' | 'f' | 'A' | 'B' | 'C' | 'D' | 'E' | 'F' => Some(c),
        'i' | 'I' => Some('1'),
        'o' | 'O' => Some('0'),
        _ if c.is_alphabetic() => None,
        _ => Some(c),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rot13_byte() {
        assert_eq!(rot13_byte(b'a'), b'n');
        assert_eq!(rot13_byte(b'n'), b'a');
        assert_eq!(rot13_byte(b'A'), b'N');
        assert_eq!(rot13_byte(b'0'), b'5');
        assert_eq!(rot13_byte(b' '), b' ');
    }

    #[test]
    fn test_replace_quotes() {
        assert_eq!(replace_quotes("\u{201c}hello\u{201d}"), "\"hello\"");
        assert_eq!(replace_quotes("\u{2018}world\u{2019}"), "'world'");
    }

    #[test]
    fn test_hexify() {
        assert_eq!(hexify(""), "");
        assert_eq!(hexify("hello"), "hello");
        assert_eq!(hexify("42"), "2a");
        assert_eq!(hexify("172.31.64.0/20"), "ac.1f.40.0/14");
    }

    #[test]
    fn test_to_hex_char() {
        assert_eq!(to_hex_char('a'), Some('a'));
        assert_eq!(to_hex_char('i'), Some('1'));
        assert_eq!(to_hex_char('o'), Some('0'));
        assert_eq!(to_hex_char('g'), None);
        assert_eq!(to_hex_char(' '), Some(' '));
    }
}
