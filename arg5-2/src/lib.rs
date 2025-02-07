use std::ffi::OsString;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum Error {
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
            Error::CharTautology(c) => write!(f, "flag -{c} would always be true"),
            Error::LongTautology(s) => write!(f, "flag --{s} would always be true"),
            Error::FlexTautology(c, s) => write!(f, "flag -{c}|--{s} would always be true"),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub trait FlagName: Copy {
    fn tautology(self) -> Error;
}

impl FlagName for char {
    fn tautology(self) -> Error {
        Error::CharTautology(self)
    }
}

impl FlagName for &'static str {
    fn tautology(self) -> Error {
        Error::LongTautology(self)
    }
}

impl FlagName for (char, &'static str) {
    fn tautology(self) -> Error {
        Error::FlexTautology(self.0, self.1)
    }
}

pub struct Parser<'a> {
    #[allow(dead_code)]
    char_flags: Vec<(char, &'a mut bool)>,
}

impl<'a> Parser<'a> {
    /// # Errors
    ///
    /// Will return an [`Error`] if the specified variable is already `true`.
    pub fn char_flag(&mut self, var: &'a mut bool, name: char, _description: &str) -> Result<()> {
        (!*var).then_some(()).ok_or(Error::CharTautology(name))?;
        self.char_flags.push((name, var));
        Ok(())
    }

    /// # Errors
    ///
    /// Will return an [`Error`] if the specified arguments cannot be parsed.
    pub fn parse(self, _args: impl IntoIterator<Item = OsString>) -> Result<()> {
        todo!()
    }

    pub fn new() -> Parser<'a> {
        Parser {
            char_flags: Vec::new(),
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
    fn char_flag_tautology() {
        let (mut f, mut t) = (false, true);

        let mut parser = Parser::new();
        assert_eq!(
            parser.char_flag(&mut f, 'f', "fake flag initialized false"),
            Ok(())
        );
        assert_eq!(
            parser.char_flag(&mut t, 't', "fake flag initialized true"),
            Err(Error::CharTautology('t')),
        );
    }

    #[test]
    fn parse_short_flag() {
        for (args, want) in [(fake_args([]), false), (fake_args(["-v"]), true)] {
            let mut got = false;

            let mut parser = Parser::new();
            parser.char_flag(&mut got, 'v', "fake flag").unwrap();

            assert_eq!(parser.parse(args), Ok(()));
            assert_eq!(got, want);
        }
    }
}
