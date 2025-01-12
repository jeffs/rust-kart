//! Converts each sequence of decimal digits in each arg to hexadecimal.

fn hexify(mut text: &str) -> String {
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
        result += &format!("{}{value:x}", &text[..begin]);
        text = &text[end..];
    }
    result + text
}

fn main() {
    std::env::args()
        .skip(1)
        .map(|arg| hexify(&arg))
        .for_each(|result| println!("{result}"));
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_hexify() {
        assert_eq!(hexify(""), "");
        assert_eq!(hexify("hello"), "hello");
        assert_eq!(hexify("42"), "2a");
        assert_eq!(hexify("172.31.64.0/20"), "ac.1f.40.0/14");
        assert_eq!(hexify("a172.31.64.0/20b"), "aac.1f.40.0/14b");
    }
}
