// * All parameters have names.
// * One parameter may receive positional arguments.
// * Arity is determined by type.
//   - Option for 0, scalar for 1, Vec for any number

use std::collections::{HashMap, VecDeque};

#[derive(Debug)]
pub struct ParseError {
    pub what: String,
}

enum Capacity {
    #[allow(dead_code)]
    Hungry,
    #[allow(dead_code)]
    Peckish,
    Full,
}

impl Capacity {
    fn is_full(&self) -> bool {
        if let Capacity::Full = self {
            true
        } else {
            false
        }
    }
}

#[derive(Debug)]
pub enum Store<'a> {
    String { target: &'a mut String },
    I32 { target: &'a mut i32 },
}

impl<'a> Store<'a> {
    fn parse(&mut self, arg: String) -> Result<Capacity, ParseError> {
        match self {
            Store::String { target } => {
                **target = arg;
                Ok(Capacity::Full)
            }
            Store::I32 { target } => match arg.parse() {
                Ok(value) => {
                    **target = value;
                    Ok(Capacity::Full)
                }
                Err(err) => Err(ParseError {
                    what: format!("{}: {}", arg, err),
                }),
            },
        }
    }
}

pub trait Bind {
    fn store(target: &mut Self) -> Store;
}

impl Bind for String {
    fn store(target: &mut Self) -> Store {
        Store::String { target }
    }
}

impl Bind for i32 {
    fn store(target: &mut Self) -> Store {
        Store::I32 { target }
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

    fn parse(&mut self, arg: String) -> Result<Capacity, ParseError> {
        self.store.parse(arg).map_err(|err| self.decorate(err))
    }

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
    positionals: VecDeque<&'static str>, // Names of positional parameters.
}

impl<'stores> Parser<'stores> {
    pub fn declare<T: Bind>(&mut self, name: &'static str, target: &'stores mut T) {
        self.parameters.insert(name, Parameter::new(name, target));
    }

    pub fn declare_positional<T: Bind>(&mut self, name: &'static str, target: &'stores mut T) {
        self.declare(name, target);
        self.positionals.push_back(name);
    }

    pub fn new() -> Parser<'stores> {
        Parser {
            parameters: HashMap::new(),
            positionals: VecDeque::new(),
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
        if !self.positionals.is_empty() {
            return Err(ParseError {
                what: "not enough arguments".to_string(),
            });
        }
        Ok(())
    }

    fn parse_arg(&mut self, arg: String) -> Result<(), ParseError> {
        if arg.starts_with("-") {
            todo!("parse by key")
        } else if let Some(name) = self.positionals.pop_front() {
            if !self.parameters.get_mut(name).unwrap().parse(arg)?.is_full() {
                self.positionals.push_front(name);
            }
            Ok(())
        } else {
            Err(ParseError {
                what: format!("{}: unexpected positional argument", arg),
            })
        }
    }
}
