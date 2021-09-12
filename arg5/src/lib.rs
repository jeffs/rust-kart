// * All parameters have names.
// * One parameter may receive positional arguments.
// * Arity is determined by type.
//   - Option for 0, scalar for 1, Vec for any number

use std::collections::HashMap;
use std::num::ParseIntError;

#[derive(Debug)]
pub enum ParseError {
    ParseIntError(ParseIntError),
    UsageError(String),
}

impl From<ParseIntError> for ParseError {
    fn from(error: ParseIntError) -> Self {
        ParseError::ParseIntError(error)
    }
}

pub enum Store<'a> {
    String(&'a mut String),
    I32(&'a mut i32),
}

pub trait Bind {
    fn store(r: &mut Self) -> Store;
}

impl Bind for String {
    fn store(r: &mut Self) -> Store {
        Store::String(r)
    }
}

impl Bind for i32 {
    fn store(r: &mut Self) -> Store {
        Store::I32(r)
    }
}

pub struct Parameter<'store> {
    name: &'static str,
    #[allow(unused)]
    flag: Option<char>,
    store: Store<'store>,
}

impl<'store> Parameter<'store> {
    pub fn new<T: Bind>(name: &'static str, r: &'store mut T) -> Self {
        Parameter {
            name,
            flag: None,
            store: Bind::store(r),
        }
    }
}

pub struct Parser<'stores> {
    parameters: HashMap<&'static str, Parameter<'stores>>,
    positional: Option<&'static str>,
}

impl<'stores> Parser<'stores> {
    pub fn new() -> Parser<'stores> {
        Parser {
            parameters: HashMap::new(),
            positional: None,
        }
    }

    pub fn declare_positional<'store: 'stores>(&mut self, parameter: Parameter<'store>) {
        let name = parameter.name;
        self.declare(parameter);
        self.positional = Some(name);
    }

    fn declare<'store: 'stores>(&mut self, parameter: Parameter<'store>) {
        self.parameters.insert(parameter.name, parameter);
    }

    pub fn parse<I: IntoIterator<Item = String>>(&mut self, args: I) -> Result<(), ParseError> {
        for arg in args {
            if arg.starts_with("-") {
                // TODO: Parse by key.
            } else if let Some(parameter) = self
                .positional
                .and_then(|name| self.parameters.get_mut(name))
            {
                match &mut parameter.store {
                    Store::String(s) => **s = arg,
                    Store::I32(i) => **i = arg.parse()?,
                }
            } else {
                let what = format!("{}: unexpected positional argument", arg);
                return Err(ParseError::UsageError(what));
            }
        }
        Ok(())
    }
}
