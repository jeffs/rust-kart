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

impl From<String> for MainError {
    fn from(what: String) -> Self {
        Self { what }
    }
}

impl From<&str> for MainError {
    fn from(what: &str) -> Self {
        Self {
            what: what.to_string(),
        }
    }
}

impl From<io::Error> for MainError {
    fn from(err: io::Error) -> Self {
        Self {
            what: err.to_string(),
        }
    }
}
