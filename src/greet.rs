#[allow(dead_code)]
pub fn greeting<S: AsRef<str>>(name: S) -> String {
    format!("Hello, {}.", name.as_ref())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn greeting_works() {
        assert_eq!(greeting("world"), "Hello, world.");
    }
}
