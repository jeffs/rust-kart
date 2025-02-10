mod date;
mod error;

pub use date::Date;
pub use error::Error;

pub type Result<T> = std::result::Result<T, Error>;
