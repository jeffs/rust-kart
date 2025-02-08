use std::ffi::OsStr;

use super::*;

fn fake_args(args: impl IntoIterator<Item = &'static str>) -> Vec<OsString> {
    let mut vec = vec![OsStr::new("fake-arg0").to_owned()];
    vec.extend(args.into_iter().map(OsStr::new).map(OsStr::to_owned));
    vec
}

#[test]
fn char_flag_non_ascii() {
    const NAME: char = 'ñ';
    let mut var = false;
    let mut parser = Parser::new();
    assert_eq!(
        parser.char_flag(&mut var, NAME, "fake char flag with non-ASCII name"),
        Err(InitError::CharName(NAME))
    );
}

#[test]
fn char_flag_tautology() {
    let (mut f, mut t) = (false, true);
    let mut parser = Parser::new();
    assert_eq!(
        parser.char_flag(&mut f, 'f', "fake flag initialized false"),
        Ok(())
    );
    assert_eq!(
        parser.char_flag(&mut t, 't', "fake flag initialized true"),
        Err(InitError::CharTautology('t')),
    );
}

#[test]
fn char_flag_parse() {
    for (args, want) in [(fake_args([]), false), (fake_args(["-v"]), true)] {
        let mut got = false;
        let mut parser = Parser::new();
        parser.char_flag(&mut got, 'v', "fake flag").unwrap();
        assert_eq!(parser.parse(args), Ok(()));
        assert_eq!(got, want);
    }
}

#[test]
fn long_flag_bad_name() {
    for name in ["año", "has a space", "-leading-hyphen", "camelCase"] {
        let mut var = false;
        let mut parser = Parser::new();
        assert_eq!(
            parser.long_flag(&mut var, name, "fake long flag with bad name"),
            Err(InitError::LongName(name)),
        );
    }
}

#[test]
fn long_flag_tautology() {
    let (mut f, mut t) = (false, true);
    let mut parser = Parser::new();
    assert_eq!(
        parser.long_flag(&mut f, "this-is-fine", "fake flag initialized false"),
        Ok(())
    );
    assert_eq!(
        parser.long_flag(&mut t, "this-is-bad", "fake flag initialized true"),
        Err(InitError::LongTautology("this-is-bad")),
    );
}

#[test]
fn long_flag_parse() {
    for (args, want) in [(fake_args([]), false), (fake_args(["--fake"]), true)] {
        let mut got = false;
        let mut parser = Parser::new();
        parser.long_flag(&mut got, "fake", "fake flag").unwrap();
        assert_eq!(parser.parse(args.clone()), Ok(()), "{args:?}");
        assert_eq!(got, want, "{args:?}");
    }
}
