//  counter mandatory positional  | example
//  ------- --------- ----------  | -------
//        f         f          f  |  ignore
//        f         f          t  |    port
//        f         t          f  |  action (like tar)
//        f         t          t  | pattern (like grep)
//        t         f          f  | verbose
//        t         f          t  |       †
//        t         t          f  |       †
//        t         t          t  |       †
//
// † Counters are rarely mandatory or positional.

#![allow(dead_code, unused_imports)]

use std::env::Args;
use std::error::Error;

/// Indicates a problem parsing a command-line argument.
pub struct ArgError {}

/// Refers to the variable where an argument value should be stored.
pub trait Target {
    /// The result is true if the arg was consumed.
    fn offer(&mut self, arg: &String) -> Result<bool, ArgError>;
}

/// Counts the number of times a named argument was specified.
///
/// For example, a parser having a 'v'/"verbose" Parameter whose target is a
/// Count would handle the argument "-vvv" by incrementing the Count three
/// times.
pub struct Count(u32);

impl Default for Count {
    fn default() -> Self {
        Count(0)
    }
}

impl Target for Count {
    /// Increments this Count.
    fn offer(&mut self, _arg: &String) -> Result<bool, ArgError> {
        self.0 += 1;
        Ok(false)
    }
}

/// Specifies one or more command-line arguments.
pub struct Parameter<'a> {
    /// A single character for use in flag sets.  For example, to support "-i"
    /// (possibly as part of a flag set like "-ijk"), a parameter's short name
    /// should be 'i'.
    pub flag: Option<char>,

    /// A stand-alone option.  For example, to support "--input-file" (either
    /// alone or, with a value as in "--input-file=data.csv"), a parameter's
    /// long name should be "input-file".  A stylized version of the name
    /// (upper cased, with underscores instad of hyphens) represents the value
    /// in help messages; for example, "INPUT_FILE".
    pub name: &'static str,

    /// Whether this parameter may be bound to an argument implicitly, rather
    /// than by name.  For example, if the program "main" has one positional
    /// parameter whose long name is "input-file", the following command lines
    /// are equivalent:
    /// ```sh
    /// main --input-file data.csv
    /// main data.csv
    /// ```
    pub is_positional: bool,

    /// Refers to the variable whose value is updated when parameter is bound.
    /// If the target variable is not an Option, the parameter is mandatory:
    /// Failure to supply it (either by name or positionally) is an error.
    pub target: &'a dyn Target,
}

struct Parser<'a> {
    parameters: Vec<Parameter<'a>>,
}

impl<'a> Parser<'a> {
    fn new() -> Parser<'a> {
        Parser {
            parameters: Vec::new(),
        }
    }

    fn add(&mut self, parameter: Parameter<'a>) {
        self.parameters.push(parameter);
    }

    fn parse(&mut self, _args: Args) {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut flag_verbose = Count::default();
        let mut parser = Parser::new();
        parser.add(Parameter {
            flag: Some('v'),
            name: "verbose",
            is_positional: false,
            target: &mut flag_verbose,
        })
    }
}
