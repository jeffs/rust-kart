// * All parameters have names.
// * One parameter may receive positional arguments.
// * Arity is determined by type.
//   - Option for 0, scalar for 1, Vec for any number

use std::collections::HashMap;
use std::num::ParseIntError;

#[derive(Debug)]
pub struct ParseError {
    pub what: String,
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
            return Err(ParseError {
                what: format!("{}: redundant argument", arg),
            });
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
                    return Err(ParseError {
                        what: String::from("expected argument"),
                    });
                }
            }
            Store::I32 { seen, .. } => {
                if !seen {
                    return Err(ParseError {
                        what: String::from("expected argument"),
                    });
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
struct Parameter<'store> {
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

    pub fn declare_positional<T: Bind>(&mut self, name: &'static str, target: &'stores mut T) {
        self.declare(name, target);
        self.positional = Some(name);
    }

    pub fn declare<T: Bind>(&mut self, name: &'static str, target: &'stores mut T) {
        self.parameters.insert(name, Parameter::new(name, target));
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

    pub fn parse<S, I>(&mut self, args: I) -> Result<(), ParseError>
    where
        S: ToString,
        I: IntoIterator<Item = S>,
    {
        for arg in args.into_iter().skip(1) {
            self.parse_arg(arg.to_string())?;
        }
        for parameter in self.parameters.values() {
            parameter.validate()?;
        }
        Ok(())
    }
}
