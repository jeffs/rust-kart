mod a_parser {
    use arg5::{Parameter, Parser};

    #[test]
    fn rejects_an_unexpected_positional_argument() {
        let mut parser = Parser::new();
        assert!(parser.parse([String::from("arg1")]).is_err());
    }

    mod given_a_positional_string_parameter {
        use super::*;

        #[test]
        fn can_assign_an_argument() {
            let want = "hello";
            let mut got = String::new();
            let mut parser = Parser::new();
            parser.declare_positional(Parameter::new("arg1", &mut got));
            parser.parse([String::from(want)]).unwrap();
            assert_eq!(got, want);
        }
    }

    mod given_a_positional_i32_parameter {
        use super::*;

        #[test]
        fn can_parse_and_assign_an_argument() {
            let want = 42;
            let mut got = 0;
            let mut parser = Parser::new();
            parser.declare_positional(Parameter::new("arg1", &mut got));
            parser.parse([String::from("42")]).unwrap();
            assert_eq!(got, want);
        }

        #[test]
        fn rejects_a_bad_integer() {
            let mut got = 0;
            let mut parser = Parser::new();
            parser.declare_positional(Parameter::new("arg1", &mut got));
            assert!(parser.parse([String::from("not-an-integer")]).is_err());
        }
    }
}
