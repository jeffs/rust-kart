use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io;

#[derive(Debug)]
pub struct CargoError {
    what: String,
}

impl CargoError {
    pub fn new(what: &str) -> CargoError {
        CargoError {
            what: what.to_string(),
        }
    }
}

impl Display for CargoError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.what)
    }
}

impl Error for CargoError {}

impl From<io::Error> for CargoError {
    fn from(error: io::Error) -> Self {
        CargoError {
            what: error.to_string(),
        }
    }
}
