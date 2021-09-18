use std::fmt;
use std::io;

#[derive(Debug)]
pub struct MainError {
    pub what: String,
}

impl fmt::Display for MainError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.what)
    }
}

impl From<arg5::ParseError> for MainError {
    fn from(err: arg5::ParseError) -> Self {
        Self { what: err.what }
    }
}

impl From<String> for MainError {
    fn from(what: String) -> Self {
        Self { what }
    }
}

impl From<io::Error> for MainError {
    fn from(err: io::Error) -> Self {
        Self {
            what: format!("{}", err),
        }
    }
}
