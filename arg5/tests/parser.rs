mod when_no_positional_parameter_is_declared {
    use arg5::Parser;

    #[test]
    fn rejects_an_unexpected_positional_argument() {
        let mut parser = Parser::new();
        assert!(parser.parse([String::from("arg1")]).is_err());
    }
}

mod when_a_positional_parameter_is_declared {
    use arg5::{Parameter, Parser};

    #[test]
    fn can_assign_a_positional_string() {
        let want = "hello";
        let mut got = String::new();
        let mut parser = Parser::new();
        parser.declare_positional(Parameter::new("arg1", &mut got));
        parser.parse([String::from(want)]).unwrap();
        assert_eq!(got, want);
    }

    #[test]
    fn can_assign_a_positional_i32() {
        let want = 42;
        let mut got = 0;
        let mut parser = Parser::new();
        parser.declare_positional(Parameter::new("arg1", &mut got));
        parser.parse([String::from("42")]).unwrap();
        assert_eq!(got, want);
    }
}
