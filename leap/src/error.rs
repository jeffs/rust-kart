use std::fmt;

#[derive(Debug)]
pub enum Error {
    Date { year: u16, month: u8, day: u8 },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Date { .. } => write!(f, "bad date: {self:?}"),
        }
    }
}
