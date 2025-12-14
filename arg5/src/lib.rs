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
    U32(&'a mut u32),
    OptU32(&'a mut Option<u32>),
}

impl Store<'_> {
    #[allow(dead_code)]
    fn capacity(&self) -> Capacity {
        match self {
            Store::I32(_) | Store::Str(_) | Store::U32(_) => Capacity::Mandatory,
            Store::OptI32(_) | Store::OptStr(_) | Store::OptU32(_) => Capacity::Optional,
        }
    }
}

#[derive(Debug)]
pub struct Binding<'a> {
    store: Store<'a>,
    appetite: Appetite,
}

impl Binding<'_> {
    #[allow(dead_code)]
    fn capacity(&self) -> Capacity {
        self.store.capacity()
    }

    fn parse(&mut self, arg: String) -> Result<(), ParseError> {
        assert!(self.appetite != Appetite::Full);
        match &mut self.store {
            Store::I32(target) => {
                **target = arg.parse().map_err(|err| ParseError {
                    what: format!("{err} '{arg}'"),
                })?;
                self.appetite = Appetite::Full;
            }
            Store::OptI32(target) => {
                **target = Some(arg.parse().map_err(|err| ParseError {
                    what: format!("{err} '{arg}'"),
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
            Store::U32(target) => {
                **target = arg.parse().map_err(|err| ParseError {
                    what: format!("{err} '{arg}'"),
                })?;
                self.appetite = Appetite::Full;
            }
            Store::OptU32(target) => {
                **target = Some(arg.parse().map_err(|err| ParseError {
                    what: format!("{err} '{arg}'"),
                })?);
                self.appetite = Appetite::Full;
            }
        }
        Ok(())
    }
}

pub trait Bind {
    fn bind(&mut self) -> Binding<'_>;
}

impl Bind for i32 {
    fn bind(&mut self) -> Binding<'_> {
        Binding {
            store: Store::I32(self),
            appetite: Appetite::Hungry,
        }
    }
}

impl Bind for Option<i32> {
    fn bind(&mut self) -> Binding<'_> {
        Binding {
            store: Store::OptI32(self),
            appetite: Appetite::Peckish,
        }
    }
}

impl Bind for String {
    fn bind(&mut self) -> Binding<'_> {
        Binding {
            store: Store::Str(self),
            appetite: Appetite::Hungry,
        }
    }
}

impl Bind for Option<String> {
    fn bind(&mut self) -> Binding<'_> {
        Binding {
            store: Store::OptStr(self),
            appetite: Appetite::Peckish,
        }
    }
}

impl Bind for u32 {
    fn bind(&mut self) -> Binding<'_> {
        Binding {
            store: Store::U32(self),
            appetite: Appetite::Hungry,
        }
    }
}

impl Bind for Option<u32> {
    fn bind(&mut self) -> Binding<'_> {
        Binding {
            store: Store::OptU32(self),
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

fn metavar(name: &str) -> String {
    name.replace('-', "_").to_uppercase()
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

    fn exit_help(&mut self, arg0: &str) -> ! {
        println!("Usage: {}", self.usage(arg0));
        exit(0)
    }

    #[must_use]
    pub fn new() -> Parser<'a> {
        Parser::default()
    }

    /// # Errors
    ///
    /// Will return `Err` if any mandatory argument is not provided, or if any
    /// supplied argument cannot be parsed.
    ///
    /// # Panics
    ///
    /// Will panic if the current program name cannot be determined.
    #[allow(clippy::similar_names)] // args, arg0
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
                    what: format!("expected {name}"),
                });
            }
        }
        Ok(())
    }

    // arg0 is the name of the current program, to appear in usage messages.
    fn parse_arg(&mut self, arg: String, arg0: &str) -> Result<(), ParseError> {
        if let Some(arg) = arg.strip_prefix("--") {
            for name in self.parameters.keys() {
                if arg == *name {
                    todo!("parse {name}");
                }
            }
            if arg == "--help" {
                self.exit_help(arg0); // Exit the program.
            } else {
                todo!("parse by key")
            }
        } else if arg.starts_with('-') {
            if arg == "-h" {
                self.exit_help(arg0); // Exit the program.
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
                what: format!("unexpected positional argument '{arg}'"),
            })
        }
    }

    #[must_use]
    pub fn usage(&self, arg0: &str) -> String {
        let mut text = arg0.to_string();
        // Print nonpositional parameters first, sorted for deterministic order.
        let mut nonpositionals: Vec<_> = self
            .parameters
            .keys()
            .filter(|name| !self.positionals.contains(name))
            .collect();
        nonpositionals.sort();
        for name in nonpositionals {
            let parameter = &self.parameters[name];
            let meta = metavar(name);
            text = match parameter.capacity() {
                Capacity::Mandatory => format!("{text} --{name} {meta}"),
                Capacity::Optional => format!("{text} [--{name} {meta}]"),
                Capacity::Variadic => format!("{text} [--{name} {meta}...]"),
            };
        }
        for name in &self.positionals {
            let parameter = &self.parameters[name];
            let meta = metavar(name);
            text = match parameter.capacity() {
                Capacity::Mandatory => format!("{text} {meta}"),
                Capacity::Optional => format!("{text} [{meta}]"),
                Capacity::Variadic => format!("{text} [{meta}...]"),
            };
        }
        text
    }
}
