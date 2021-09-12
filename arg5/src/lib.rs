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
    String { target: &'a mut String, seen: bool },
    I32 { target: &'a mut i32 , seen: bool },
}

impl<'a> Store<'a> {
    fn set_seen(seen: &mut bool, arg: &String) -> Result<(), ParseError> {
        if *seen {
            let what = format!("{}: redundant argument", arg);
            return Err(ParseError::UsageError(what));
        }
        *seen = true;
        Ok(())
    }

    fn parse(&mut self, arg: String) -> Result<(), ParseError> {
        match self {
            Store::String { target, seen } => {
                Self::set_seen(seen, &arg)?;
                **target = arg;
            }
            Store::I32 { target, seen } => {
                Self::set_seen(seen, &arg)?;
                **target = arg.parse()?;
            }
        }
        Ok(())
    }
}

pub trait Bind {
    fn store(target: &mut Self) -> Store;
}

impl Bind for String {
    fn store(target: &mut Self) -> Store {
        Store::String { target, seen: false }
    }
}

impl Bind for i32 {
    fn store(target: &mut Self) -> Store {
        Store::I32 { target, seen: false }
    }
}

pub struct Parameter<'store> {
    name: &'static str,
    #[allow(unused)]
    flag: Option<char>,
    store: Store<'store>,
}

impl<'store> Parameter<'store> {
    pub fn new<T: Bind>(name: &'static str, target: &'store mut T) -> Self {
        Parameter {
            name,
            flag: None,
            store: Bind::store(target),
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
                parameter.store.parse(arg)?;
            } else {
                let what = format!("{}: unexpected positional argument", arg);
                return Err(ParseError::UsageError(what));
            }
        }
        Ok(())
    }
}
