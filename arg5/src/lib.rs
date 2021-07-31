#![allow(dead_code, unused_imports)]

use std::collections::VecDeque;
use std::error::Error;
use std::num::ParseIntError;

#[derive(Debug)]
struct ArgError {
    what: String,
}

impl From<ParseIntError> for ArgError {
    fn from(_: ParseIntError) -> Self {
        ArgError {
            what: String::from("bad argument: expected integer"),
        }
    }
}

trait Arg {
    fn take(args: &mut VecDeque<String>) -> Result<Self, ArgError>
    where
        Self: Sized;
}

impl Arg for i32 {
    fn take(args: &mut VecDeque<String>) -> Result<Self, ArgError> {
        match args.pop_front() {
            Some(arg) => Ok(arg.parse()?),
            None => Err(ArgError {
                what: String::from("missing argument: expected integer"),
            }),
        }
    }
}

impl Arg for String {
    fn take(args: &mut VecDeque<String>) -> Result<Self, ArgError> {
        match args.pop_front() {
            Some(arg) => Ok(arg),
            None => Err(ArgError {
                what: String::from("missing argument: expected string"),
            }),
        }
    }
}

struct Parser {
    args: VecDeque<String>,
}

impl Parser {
    fn from<I: Iterator<Item = String>>(args: I) -> Parser {
        Parser {
            args: args.collect(),
        }
    }

    fn arg<T: Arg>(&mut self) -> Result<T, ArgError> {
        T::take(&mut self.args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() -> Result<(), ArgError> {
        let args = [String::from("hello"), String::from("42")];

        let mut parse = Parser::from(args.iter().cloned());

        let arg: String = parse.arg()?;
        assert_eq!(String::from("hello"), arg);

        let arg: i32 = parse.arg()?;
        assert_eq!(42, arg);

        Ok(())
    }
}
