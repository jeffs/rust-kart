use self::{
    error::BadFunc,
    function::{Add, Mul},
};

pub use function::Function;

pub mod error;
mod function;

pub fn parse(func: &str) -> Result<&'static dyn Function, BadFunc> {
    match func {
        "add" => Ok(&Add),
        "mul" => Ok(&Mul),
        _ => Err(BadFunc(func.to_owned())),
    }
}
