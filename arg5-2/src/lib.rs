// Each parameter has a name.
//
//  mandatory   variadic    |   symbol
//  ---------   --------    |   ------
//  no          no          |   ?
//  no          yes         |   *
//  yes         no          |   1
//  yes         yes         |   +

use std::ops::RangeInclusive;
use std::usize;

pub enum Error {
    /// The [`char`] could not be converted to an [`Arity`].
    ArityChar(char),
}

pub struct Arity(pub RangeInclusive<usize>);

impl TryFrom<char> for Arity {
    type Error = Error;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '?' => Ok(Arity(0..=1)),
            '*' => Ok(Arity(0..=usize::MAX)),
            '+' => Ok(Arity(1..=usize::MAX)),
            '1' => Ok(Arity(1..=1)),
            _ => Err(Error::ArityChar(value)),
        }
    }
}

pub struct Parser {}
