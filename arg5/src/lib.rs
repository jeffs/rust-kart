// * All parameters have names.
// * One parameter may receive positional arguments.
// * Arity is determined by type.
//   - Option for 0, scalar for 1, Vec for any number

use std::collections::{HashMap, VecDeque};

#[derive(Debug, Eq, PartialEq)]
pub struct ParseError {
    pub what: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Capacity {
    Hungry,  // Requires more arguments.
    Peckish, // Could accept more arguments, but doesn't require them.
    Full,    // Cannot accept any more arguments.
}

#[derive(Debug)]
pub enum Store<'a> {
    I32(&'a mut i32),
    OptI32(&'a mut Option<i32>),
    Str(&'a mut String),
}

#[derive(Debug)]
pub struct Binding<'a> {
    store: Store<'a>,
    capacity: Capacity,
}

impl<'a> Binding<'a> {
    fn parse(&mut self, arg: String) -> Result<(), ParseError> {
        assert!(self.capacity != Capacity::Full);
        match &mut self.store {
            Store::I32(target) => {
                **target = arg.parse().map_err(|err| ParseError {
                    what: format!("{} '{}'", err, arg),
                })?;
                self.capacity = Capacity::Full;
            }
            Store::OptI32(target) => {
                **target = Some(arg.parse().map_err(|err| ParseError {
                    what: format!("{} '{}'", err, arg),
                })?);
                self.capacity = Capacity::Full;
            }
            Store::Str(target) => {
                **target = arg;
                self.capacity = Capacity::Full;
            }
        }
        Ok(())
    }
}

pub trait Bind {
    fn bind(&mut self) -> Binding;
}

impl Bind for i32 {
    fn bind(&mut self) -> Binding {
        Binding {
            store: Store::I32(self),
            capacity: Capacity::Hungry,
        }
    }
}

impl Bind for Option<i32> {
    fn bind(&mut self) -> Binding {
        Binding {
            store: Store::OptI32(self),
            capacity: Capacity::Peckish,
        }
    }
}

impl Bind for String {
    fn bind(&mut self) -> Binding {
        Binding {
            store: Store::Str(self),
            capacity: Capacity::Hungry,
        }
    }
}

#[derive(Debug)]
struct Parameter<'a> {
    name: &'static str,
    #[allow(unused)]
    flag: Option<char>,
    binding: Binding<'a>,
}

impl<'a> Parameter<'a> {
    fn capacity(&self) -> Capacity {
        self.binding.capacity
    }

    fn new<T: Bind>(name: &'static str, target: &'a mut T) -> Self {
        Parameter {
            name,
            flag: None,
            binding: target.bind(),
        }
    }

    fn parse(&mut self, arg: String) -> Result<(), ParseError> {
        self.binding.parse(arg).map_err(|err| ParseError {
            what: format!("{}: {}", self.name, err.what),
        })
    }
}

pub struct Parser<'a> {
    parameters: HashMap<&'static str, Parameter<'a>>,
    positionals: VecDeque<&'static str>, // Names of positional parameters.
}

impl<'a> Parser<'a> {
    pub fn declare<T: Bind>(&mut self, name: &'static str, target: &'a mut T) {
        self.parameters.insert(name, Parameter::new(name, target));
    }

    pub fn declare_positional<T: Bind>(&mut self, name: &'static str, target: &'a mut T) {
        self.declare(name, target);
        self.positionals.push_back(name);
    }

    pub fn new() -> Parser<'a> {
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
        if let Some(name) = self.positionals.pop_front() {
            if self.parameters[name].capacity() == Capacity::Hungry {
                return Err(ParseError {
                    what: format!("{}: expected argument", name),
                });
            }
        }
        Ok(())
    }

    fn parse_arg(&mut self, arg: String) -> Result<(), ParseError> {
        if arg.starts_with("-") {
            todo!("parse by key")
        } else if let Some(name) = self.positionals.pop_front() {
            let parameter = self.parameters.get_mut(name).unwrap();
            parameter.parse(arg)?;
            if parameter.capacity() != Capacity::Full {
                self.positionals.push_front(name);
            }
            Ok(())
        } else {
            Err(ParseError {
                what: format!("unexpected positional argument '{}'", arg),
            })
        }
    }
}
