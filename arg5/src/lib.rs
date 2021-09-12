// * All parameters have names.
// * One parameter may receive positional arguments.
// * Arity is determined by type.
//   - Option for 0, scalar for 1, Vec for any number

use std::collections::HashMap;

trait Parse {}

struct Parameter<'store> {
    name: &'static str,
    #[allow(unused)]
    flag: Option<char>,
    store: &'store mut String,
}

pub struct Parser<'stores> {
    parameters: HashMap<&'static str, Parameter<'stores>>,
    positional: Option<&'static str>,
}

impl<'stores> Parser<'stores> {
    fn new() -> Parser<'stores> {
        Parser {
            parameters: HashMap::new(),
            positional: None,
        }
    }

    fn declare_positional<'store: 'stores>(&mut self, parameter: Parameter<'store>) {
        let name = parameter.name;
        self.declare(parameter);
        self.positional = Some(name);
    }

    fn declare<'store: 'stores>(&mut self, parameter: Parameter<'store>) {
        self.parameters.insert(parameter.name, parameter);
    }

    fn parse<I: IntoIterator<Item = String>>(&mut self, args: I) {
        for arg in args {
            if arg.starts_with("-") {
                // TODO: Parse by key.
            } else if let Some(parameter) = self
                .positional
                .and_then(|name| self.parameters.get_mut(name))
            {
                // TODO: Parse positional argument.
                *parameter.store = arg;
            } else {
                // TODO: Report error: Unexpected positional argument.
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_can_assign_a_positional_string() {
        let want = "hello";
        let mut got = String::new();
        let mut parser = Parser::new();
        parser.declare_positional(Parameter {
            name: "arg1",
            flag: None,
            store: &mut got,
        });
        parser.parse([String::from(want)]);
        assert_eq!(got, want);
    }
}
