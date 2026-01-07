#![doc = include_str!("../README.md")]

mod date;
mod error;
pub mod week;

pub use date::Date;
pub use error::Error;

pub type Result<T> = std::result::Result<T, Error>;
