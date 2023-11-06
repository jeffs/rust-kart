use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub struct BadFunc(pub String);

impl Display for BadFunc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let BadFunc(arg) = self;
        write!(f, "{arg}: bad function; expected 'add' or 'mul'")
    }
}

impl Error for BadFunc {}

#[derive(Debug)]
pub struct NoFunc;

impl Display for NoFunc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "expected function")
    }
}

impl Error for NoFunc {}
