use std::fmt;

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
