// * All parameters have names.
// * One parameter may receive positional arguments.
// * Arity is determined by type.
//   - Option for 0, scalar for 1, Vec for any number
// * Automatically prints usage and exits the process on -h|--help

use std::collections::HashMap;
use std::process::exit;

#[derive(Debug, Eq, PartialEq)]
pub struct ParseError {
    pub what: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Appetite {
    Hungry,  // Requires more arguments.
    Peckish, // Could accept more arguments, but doesn't require them.
    Full,    // Cannot accept any more arguments.
}

#[allow(dead_code)]
enum Capacity {
    Mandatory, // exactly one
    Optional,  // zero or one
    Variadic,  // zero or more
}

#[derive(Debug)]
pub enum Store<'a> {
    I32(&'a mut i32),
    OptI32(&'a mut Option<i32>),
    Str(&'a mut String),
    OptStr(&'a mut Option<String>),
}

impl<'a> Store<'a> {
    #[allow(dead_code)]
    fn capacity(&self) -> Capacity {
        match self {
            Store::I32(_) => Capacity::Mandatory,
            Store::OptI32(_) => Capacity::Optional,
            Store::Str(_) => Capacity::Mandatory,
            Store::OptStr(_) => Capacity::Optional,
        }
    }
}

#[derive(Debug)]
pub struct Binding<'a> {
    store: Store<'a>,
    appetite: Appetite,
}

impl<'a> Binding<'a> {
    #[allow(dead_code)]
    fn capacity(&self) -> Capacity {
        self.store.capacity()
    }

    fn parse(&mut self, arg: String) -> Result<(), ParseError> {
        assert!(self.appetite != Appetite::Full);
        match &mut self.store {
            Store::I32(target) => {
                **target = arg.parse().map_err(|err| ParseError {
                    what: format!("{} '{}'", err, arg),
                })?;
                self.appetite = Appetite::Full;
            }
            Store::OptI32(target) => {
                **target = Some(arg.parse().map_err(|err| ParseError {
                    what: format!("{} '{}'", err, arg),
                })?);
                self.appetite = Appetite::Full;
            }
            Store::Str(target) => {
                **target = arg;
                self.appetite = Appetite::Full;
            }
            Store::OptStr(target) => {
                **target = Some(arg);
                self.appetite = Appetite::Full;
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
            appetite: Appetite::Hungry,
        }
    }
}

impl Bind for Option<i32> {
    fn bind(&mut self) -> Binding {
        Binding {
            store: Store::OptI32(self),
            appetite: Appetite::Peckish,
        }
    }
}

impl Bind for String {
    fn bind(&mut self) -> Binding {
        Binding {
            store: Store::Str(self),
            appetite: Appetite::Hungry,
        }
    }
}

impl Bind for Option<String> {
    fn bind(&mut self) -> Binding {
        Binding {
            store: Store::OptStr(self),
            appetite: Appetite::Peckish,
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
    fn appetite(&self) -> Appetite {
        self.binding.appetite
    }

    #[allow(dead_code)]
    fn capacity(&self) -> Capacity {
        self.binding.capacity()
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

#[derive(Default)]
pub struct Parser<'a> {
    parameters: HashMap<&'static str, Parameter<'a>>,
    positionals: Vec<&'static str>, // Names of positional parameters.
    positional_index: usize,        // Parameter for next positional argument.
}

impl<'a> Parser<'a> {
    pub fn declare<T: Bind>(&mut self, name: &'static str, target: &'a mut T) {
        self.parameters.insert(name, Parameter::new(name, target));
    }

    pub fn declare_positional<T: Bind>(&mut self, name: &'static str, target: &'a mut T) {
        self.declare(name, target);
        self.positionals.push(name);
    }

    pub fn new() -> Parser<'a> {
        Parser::default()
    }

    pub fn parse<S, I>(&mut self, args: I) -> Result<(), ParseError>
    where
        S: ToString,
        I: IntoIterator<Item = S>,
    {
        let mut args = args.into_iter();
        let arg0 = match args.next() {
            Some(arg) => arg.to_string(),
            None => panic!("empty args; expected (at least) program name"),
        };
        for arg in args {
            self.parse_arg(arg.to_string(), &arg0)?;
        }
        // Return Err if any parameter is still hungry.
        for name in &self.positionals {
            if self.parameters[name].appetite() == Appetite::Hungry {
                return Err(ParseError {
                    what: format!("{}: expected argument", name),
                });
            }
        }
        Ok(())
    }

    // arg0 is the name of the current program, to appear in usage messages.
    fn parse_arg(&mut self, arg: String, arg0: &str) -> Result<(), ParseError> {
        if arg.starts_with('-') {
            if arg == "-h" || arg == "--help" {
                println!("Usage: {}", self.usage(arg0));
                exit(0);
            } else {
                todo!("parse by key")
            }
        } else if let Some(name) = self.positionals.get(self.positional_index) {
            let parameter = self.parameters.get_mut(name).unwrap();
            parameter.parse(arg)?;
            if parameter.appetite() == Appetite::Full {
                self.positional_index += 1;
            }
            Ok(())
        } else {
            Err(ParseError {
                what: format!("unexpected positional argument '{}'", arg),
            })
        }
    }

    pub fn usage(&self, arg0: &str) -> String {
        let mut text = arg0.to_string();
        // TODO: Print nonpositional parameters.
        for name in &self.positionals {
            let parameter = &self.parameters[name];
            text = match parameter.capacity() {
                Capacity::Mandatory => format!("{} <{}>", text, name),
                Capacity::Optional => format!("{} [{}]", text, name),
                Capacity::Variadic => format!("{} [{}...]", text, name),
            };
        }
        text
    }
}
