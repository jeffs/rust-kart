mod a_parser {
    use arg5::{ParseError, Parser};

    // The first arg to a program is typically the program name.
    const ARG0: &str = "arg0";

    #[test]
    fn rejects_an_unexpected_positional_argument() {
        let mut parser = Parser::new();
        assert!(parser.parse([ARG0, "arg1"]).is_err());
    }

    mod given_a_positional_string_parameter {
        use super::*;

        fn try_parse(args: &[&'static str]) -> Result<String, ParseError> {
            let mut got = String::new();
            let mut parser = Parser::new();
            parser.declare_positional("par", &mut got);
            parser.parse(args)?;
            Ok(got)
        }

        #[test]
        fn can_assign_an_argument() {
            assert_eq!(try_parse(&[ARG0, "arg"]).unwrap(), "arg");
        }

        #[test]
        fn rejects_a_second_argument() {
            assert!(try_parse(&[ARG0, "arg1", "arg2"]).is_err());
        }

        #[test]
        fn rejects_an_empty_argument_list() {
            assert!(try_parse(&[ARG0]).is_err());
        }
    }

    mod given_a_positional_i32_parameter {
        use super::*;

        fn try_parse(args: &[&'static str]) -> Result<i32, ParseError> {
            let mut got = 0;
            let mut parser = Parser::new();
            parser.declare_positional("par", &mut got);
            parser.parse(args)?;
            Ok(got)
        }

        #[test]
        fn can_parse_and_assign_an_argument() {
            assert_eq!(try_parse(&[ARG0, "42"]).unwrap(), 42);
        }

        #[test]
        fn rejects_a_bad_integer() {
            assert!(try_parse(&[ARG0, "not-an-integer"]).is_err());
        }

        #[test]
        fn rejects_a_second_argument() {
            assert!(try_parse(&[ARG0, "42", "43"]).is_err());
        }

        #[test]
        fn rejects_an_empty_argument_list() {
            assert!(try_parse(&[ARG0]).is_err());
        }
    }

    mod given_two_positional_parameters {
        use super::*;

        fn try_parse(args: &[&'static str]) -> Result<(i32, String), ParseError> {
            let mut got_i32 = 0;
            let mut got_str = String::new();
            let mut parser = Parser::new();
            parser.declare_positional("par1", &mut got_i32);
            parser.declare_positional("par2", &mut got_str);
            parser.parse(args)?;
            Ok((got_i32, got_str))
        }

        #[test]
        fn can_parse_and_assign_them() {
            let (got_i32, got_str) = try_parse(&[ARG0, "42", "hello"]).unwrap();
            assert_eq!(got_i32, 42);
            assert_eq!(got_str, "hello");
        }

        #[test]
        fn rejects_a_third_argument() {
            assert!(try_parse(&[ARG0, "42", "43", "44"]).is_err());
        }

        #[test]
        fn rejects_an_empty_argument_list() {
            assert!(try_parse(&[ARG0]).is_err());
        }

        #[test]
        fn rejects_a_unary_argument_list() {
            assert!(try_parse(&[ARG0, "42"]).is_err());
        }
    }

    mod given_an_optional_parameter {
        use super::*;

        fn try_parse(args: &[&'static str]) -> Result<Option<i32>, ParseError> {
            let mut got: Option<i32> = None;
            let mut parser = Parser::new();
            parser.declare_positional("opt", &mut got);
            parser.parse(args).map(|_| got)
        }

        #[test]
        fn can_parse_and_assign_an_argument() {
            assert_eq!(try_parse(&[ARG0, "42"]).unwrap(), Some(42));
        }

        #[test]
        fn accepts_an_empty_argument_list() {
            assert_eq!(try_parse(&[ARG0]), Ok(None));
        }

        #[test]
        fn rejects_an_extra_argument() {
            let mut parser = Parser::new();
            assert!(parser.parse([ARG0, "42", "43"]).is_err());
        }

        #[test]
        fn can_describe_usage() {
            let mut got_i32 = 0;
            let mut got_str = String::new();
            let mut got_opt: Option<i32> = None;
            let mut parser = Parser::new();
            parser.declare_positional("count", &mut got_i32);
            parser.declare_positional("word", &mut got_str);
            parser.declare_positional("opt", &mut got_opt);
            assert_eq!(parser.usage("repeat"), "repeat <count> <word> [opt]");
        }
    }
}
