// * All parameters have names.
// * One parameter may receive positional arguments.
// * Arity is determined by type.
//   - Option for 0, scalar for 1, Vec for any number

use std::collections::HashMap;
use std::num::ParseIntError;

#[derive(Debug)]
pub struct ParseError {
    what: String,
}

impl From<ParseIntError> for ParseError {
    fn from(error: ParseIntError) -> Self {
        ParseError {
            what: format!("{}", error),
        }
    }
}

#[derive(Debug)]
pub enum Store<'a> {
    String { target: &'a mut String, seen: bool },
    I32 { target: &'a mut i32, seen: bool },
}

impl<'a> Store<'a> {
    fn set_seen(seen: &mut bool, arg: &String) -> Result<(), ParseError> {
        if *seen {
            let what = format!("{}: redundant argument", arg);
            return Err(ParseError { what });
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

    fn validate(&self) -> Result<(), ParseError> {
        match self {
            Store::String { seen, .. } => {
                if !seen {
                    let what = String::from("expected argument");
                    return Err(ParseError { what });
                }
            }
            Store::I32 { seen, .. } => {
                if !seen {
                    let what = String::from("expected argument");
                    return Err(ParseError { what });
                }
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
        Store::String {
            target,
            seen: false,
        }
    }
}

impl Bind for i32 {
    fn store(target: &mut Self) -> Store {
        Store::I32 {
            target,
            seen: false,
        }
    }
}

#[derive(Debug)]
pub struct Parameter<'store> {
    name: &'static str,
    #[allow(unused)]
    flag: Option<char>,
    store: Store<'store>,
}

impl<'store> Parameter<'store> {
    fn decorate(&self, err: ParseError) -> ParseError {
        ParseError {
            what: format!("{}: {}", self.name, err.what),
        }
    }

    pub fn new<T: Bind>(name: &'static str, target: &'store mut T) -> Self {
        Parameter {
            name,
            flag: None,
            store: Bind::store(target),
        }
    }

    fn parse(&mut self, arg: String) -> Result<(), ParseError> {
        self.store.parse(arg).map_err(|err| self.decorate(err))
    }

    fn validate(&self) -> Result<(), ParseError> {
        self.store.validate().map_err(|err| self.decorate(err))
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

    fn parse_arg(&mut self, arg: String) -> Result<(), ParseError> {
        if arg.starts_with("-") {
            todo!() // TODO: Parse by key.
        } else if let Some(name) = self.positional {
            self.parameters.get_mut(name).unwrap().parse(arg)
        } else {
            Err(ParseError {
                what: format!("{}: unexpected positional argument", arg),
            })
        }
    }

    pub fn parse<I: IntoIterator<Item = String>>(&mut self, args: I) -> Result<(), ParseError> {
        for arg in args {
            self.parse_arg(arg)?;
        }
        for parameter in self.parameters.values() {
            parameter.validate()?;
        }
        Ok(())
    }
}
