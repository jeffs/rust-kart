#![allow(dead_code, unused_imports)]

use std::ffi::OsString;
use std::fmt;

#[derive(Debug, PartialEq)]
enum Error {
    /// The target variable for a Boolean flag was already true when the variable was registered.
    /// The variable should have been initialized to false, or else there is no way to tell whether
    /// the flag was specified in arguments.
    CharTautology(char),
    LongTautology(&'static str),
    FlexTautology(char, &'static str),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::CharTautology(c) => write!(f, "flag {c} would always be true"),
            Error::LongTautology(s) => write!(f, "flag {s} would always be true"),
            Error::FlexTautology(c, s) => write!(f, "flag {c}|{s} would always be true"),
        }
    }
}

type Result<T> = std::result::Result<T, Error>;

struct Parser {}

impl Parser {
    /// # Errors
    ///
    /// Will return an [`Error`] if the specified variable is already `true`.
    fn flag(&mut self, variable: &mut bool, c: char, _description: &str) -> Result<()> {
        (!*variable)
            .then_some(())
            .ok_or_else(|| Error::CharTautology(c))?;

        // TODO: save arg for use in parsing

        Ok(())
    }

    /// # Errors
    ///
    /// Will return an [`Error`] if the specified arguments cannot be parsed.
    fn parse(self, _args: impl IntoIterator<Item = OsString>) -> Result<()> {
        todo!()
    }

    fn new() -> Parser {
        Parser {}
    }
}

mod tests {
    use std::ffi::OsStr;
    use std::iter;

    use super::*;

    fn fake_args(args: impl IntoIterator<Item = &'static str>) -> Vec<OsString> {
        let mut vec = vec![OsStr::new("fake-arg0").to_owned()];
        vec.extend(args.into_iter().map(OsStr::new).map(OsStr::to_owned));
        vec
    }

    #[test]
    fn short_flag_tautology() {
        let (mut f, mut t) = (false, true);

        let mut parser = Parser::new();
        assert_eq!(
            parser.flag(&mut f, 'f', "fake flag initialized false"),
            Ok(())
        );
        assert_eq!(
            parser.flag(&mut t, 't', "fake flag initialized true"),
            Err(Error::CharTautology('t')),
        );
    }

    #[test]
    fn parse_short_flag() {
        for (args, want) in [(fake_args([]), false), (fake_args(["-v"]), true)] {
            let mut got = false;

            let mut parser = Parser::new();
            parser.flag(&mut got, 'v', "fake flag").unwrap();

            assert_eq!(parser.parse(args), Ok(()));
            assert_eq!(got, want);
        }
    }
}
