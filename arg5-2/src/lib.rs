mod error;

use std::ffi::OsString;
use std::os::unix::ffi::OsStrExt;

pub use error::{Init as InitError, Parse as ParseError};

/// The number of ASCII values.
const ASCII_COUNT: usize = 1 << 7;

/// Maps ASCII code points to variables.
type CharMap<'a, T> = [Option<&'a mut T>; ASCII_COUNT];
type LongMap<'a, T> = Vec<(&'static str, &'a mut T)>;

/// Returns true if the specified string is a valid flag or option name.
fn is_long_name(s: &str) -> bool {
    !s.is_empty()
        && !s.starts_with('-')
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() && !c.is_uppercase() || c == '-')
}

fn parse_char_flag(vars: &mut CharMap<bool>, name: u8) {
    if let Some(Some(var)) = vars.get_mut(usize::from(name)) {
        **var = true;
    }
}

/// The `&mut Vec` here is purely a weirdness of Rust.  We're not modifying the `Vec` at all.  We
/// are, however, potentially modifying the bools to which the `Vec` items refer.
fn parse_char_flags(vars: &mut CharMap<bool>, names: &[u8]) {
    for &name in names {
        parse_char_flag(vars, name);
    }
}

/// Returns true on success, and false if the map has no entry for the name.  Note that a single
/// name may be mapped to multiple target variables: This function does not stop at the first match.
#[must_use]
fn parse_long_flag(vars: &mut LongMap<bool>, name: &str) -> bool {
    let mut seen = false;
    for (key, var) in vars.iter_mut() {
        if *key == name {
            **var = true;
            seen = true;
        }
    }
    seen
}

pub struct Parser<'a> {
    char_flags: CharMap<'a, bool>,
    long_flags: LongMap<'a, bool>,
}

impl<'a> Parser<'a> {
    /// # Errors
    ///
    /// Will return an [`Error`] if the specified variable is already `true`.
    pub fn char_flag(
        &mut self,
        var: &'a mut bool,
        name: char,
        _description: &'static str,
    ) -> Result<(), InitError> {
        let byte: u8 = name
            .is_ascii_alphanumeric()
            .then_some(())
            .and_then(|()| name.try_into().ok())
            .ok_or(InitError::CharName(name))?;
        (!*var)
            .then_some(())
            .ok_or(InitError::CharTautology(name))?;
        self.char_flags[usize::from(byte)] = Some(var);
        Ok(())
    }

    /// # Errors
    ///
    /// Will return an [`Error`] if the specified variable is already `true`.
    pub fn long_flag(
        &mut self,
        var: &'a mut bool,
        name: &'static str,
        _description: &'static str,
    ) -> Result<(), InitError> {
        is_long_name(&name)
            .then_some(())
            .ok_or(InitError::LongName(name))?;
        (!*var)
            .then_some(())
            .ok_or(InitError::LongTautology(name))?;
        self.long_flags.push((name, var));
        Ok(())
    }

    /// # Errors
    ///
    /// Will return an [`Error`] if the specified arguments cannot be parsed.
    pub fn parse(mut self, args: impl IntoIterator<Item = OsString>) -> Result<(), ParseError> {
        for arg in args.into_iter().skip(1) {
            match arg.as_bytes() {
                b"--" => todo!("all remaining args are positional"),
                [b'-', b'-', bytes @ ..] => std::str::from_utf8(bytes)
                    .is_ok_and(|name| parse_long_flag(&mut self.long_flags, name))
                    .then_some(())
                    .ok_or(ParseError::LongName(arg))?,
                [b'-', bytes @ ..] => parse_char_flags(&mut self.char_flags, bytes),
                _ => todo!("positional"),
            }
        }
        Ok(())
    }

    pub const fn new() -> Parser<'a> {
        Parser {
            char_flags: [const { None }; ASCII_COUNT],
            long_flags: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;

    use super::*;

    fn fake_args(args: impl IntoIterator<Item = &'static str>) -> Vec<OsString> {
        let mut vec = vec![OsStr::new("fake-arg0").to_owned()];
        vec.extend(args.into_iter().map(OsStr::new).map(OsStr::to_owned));
        vec
    }

    #[test]
    fn char_flag_non_ascii() {
        const NAME: char = 'ñ';
        let mut var = false;
        let mut parser = Parser::new();
        assert_eq!(
            parser.char_flag(&mut var, NAME, "fake char flag with non-ASCII name"),
            Err(InitError::CharName(NAME))
        );
    }

    #[test]
    fn char_flag_tautology() {
        let (mut f, mut t) = (false, true);
        let mut parser = Parser::new();
        assert_eq!(
            parser.char_flag(&mut f, 'f', "fake flag initialized false"),
            Ok(())
        );
        assert_eq!(
            parser.char_flag(&mut t, 't', "fake flag initialized true"),
            Err(InitError::CharTautology('t')),
        );
    }

    #[test]
    fn char_flag_parse() {
        for (args, want) in [(fake_args([]), false), (fake_args(["-v"]), true)] {
            let mut got = false;
            let mut parser = Parser::new();
            parser.char_flag(&mut got, 'v', "fake flag").unwrap();
            assert_eq!(parser.parse(args), Ok(()));
            assert_eq!(got, want);
        }
    }

    #[test]
    fn long_flag_bad_name() {
        for name in ["año", "has a space", "-leading-hyphen", "camelCase"] {
            let mut var = false;
            let mut parser = Parser::new();
            assert_eq!(
                parser.long_flag(&mut var, name, "fake long flag with bad name"),
                Err(InitError::LongName(name)),
            );
        }
    }

    #[test]
    fn long_flag_tautology() {
        let (mut f, mut t) = (false, true);
        let mut parser = Parser::new();
        assert_eq!(
            parser.long_flag(&mut f, "this-is-fine", "fake flag initialized false"),
            Ok(())
        );
        assert_eq!(
            parser.long_flag(&mut t, "this-is-bad", "fake flag initialized true"),
            Err(InitError::LongTautology("this-is-bad")),
        );
    }

    #[test]
    fn long_flag_parse() {
        for (args, want) in [(fake_args([]), false), (fake_args(["--fake"]), true)] {
            let mut got = false;
            let mut parser = Parser::new();
            parser.long_flag(&mut got, "fake", "fake flag").unwrap();
            assert_eq!(parser.parse(args.clone()), Ok(()), "{args:?}");
            assert_eq!(got, want, "{args:?}");
        }
    }
}
