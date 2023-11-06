mod function;
pub use function::Monoid;
use function::{Add, Mul};

pub mod error;
use error::BadFunc;

pub fn parse(func: &str) -> Result<&'static dyn Monoid, BadFunc> {
    match func {
        "add" => Ok(&Add),
        "mul" => Ok(&Mul),
        _ => Err(BadFunc(func.to_owned())),
    }
}
