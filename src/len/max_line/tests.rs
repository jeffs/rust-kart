#![cfg(test)]

use super::*;
use crate::len::expect::Expect;
use std::{io, iter};

type ErrorBrief = (io::ErrorKind, &'static str);

fn as_error((kind, what): ErrorBrief) -> io::Result<String> {
    Err(io::Error::new(kind, what))
}

fn as_line(line: &str) -> io::Result<String> {
    Ok(line.to_owned())
}

fn as_result(res: &Result<&'static str, ErrorBrief>) -> io::Result<String> {
    match *res {
        Ok(line) => as_line(&line),
        Err(brief) => as_error(brief),
    }
}

fn as_errors<'a, I>(briefs: I) -> impl 'a + Iterator<Item = io::Result<String>>
where
    I: 'a + IntoIterator<Item = &'a ErrorBrief>,
{
    briefs.into_iter().copied().map(as_error)
}

fn as_lines<'a, I>(lines: I) -> impl 'a + Iterator<Item = io::Result<String>>
where
    I: 'a + IntoIterator<Item = &'a &'static str>,
{
    lines.into_iter().copied().map(as_line)
}

fn as_results<'a, I>(results: I) -> impl Iterator<Item = io::Result<String>>
where
    I: IntoIterator<Item = &'a Result<&'static str, ErrorBrief>>,
{
    results.into_iter().map(as_result)
}

trait Assert {
    fn assert_err(&mut self, want: ErrorBrief);
    fn assert_line(&mut self, want: &str);
    fn assert_res(&mut self, want: Result<&'static str, ErrorBrief>);
}

impl<I> Assert for I
where
    I: Iterator<Item = io::Result<String>>,
{
    fn assert_err(&mut self, want: ErrorBrief) {
        let got = self.expect_err();
        assert_eq!(want, (got.kind(), got.to_string().as_ref()));
    }

    fn assert_line(&mut self, want: &str) {
        assert_eq!(want, self.expect_line());
    }

    fn assert_res(&mut self, want: Result<&'static str, ErrorBrief>) {
        match want {
            Ok(line) => self.assert_line(line),
            Err(brief) => self.assert_err(brief),
        }
    }
}

#[test]
fn empty() {
    assert!(longest(iter::empty()).next().is_none());
}

#[test]
fn errors_only() {
    let errors = [
        (io::ErrorKind::InvalidData, "bad data 1"),
        (io::ErrorKind::NotFound, "not found"),
        (io::ErrorKind::InvalidData, "bad data 2"),
    ];
    let mut subject = longest(as_errors(&errors));
    for &want in errors.iter() {
        subject.assert_err(want);
    }
    subject.expect_none();
}

#[test]
fn returns_first_line_of_max_length() {
    let lines = [
        "This is a short line.",
        "This, on the other hand, is a noticeably longer line.",
        "This is the exact same length as the long line above.",
        "Here is another of that length just for good measure.",
        "MaxLine should return the first one.",
    ];
    let mut subject = longest(as_lines(&lines));
    subject.assert_line(lines[1]);
    subject.expect_none();
}

#[test]
fn returns_first_line_of_min_length() {
    let lines = [
        "Our line is a very very very long line.",
        "This is a noticeably shorter line.",
        "This line is as short as that one.",
        "Here's another short line for you.",
        "MaxLine should return the first short line.",
    ];
    let mut subject = shortest(as_lines(&lines));
    subject.assert_line(lines[1]);
    subject.expect_none();
}

#[test]
fn recoverable_errors() {
    let results = [
        Err((io::ErrorKind::NotFound, "error before the lines")),
        Ok("MaxLine should return all the errors first."),
        Err((io::ErrorKind::InvalidData, "error between the lines")),
        Ok("After the errors, it should return the longest line."),
        Err((io::ErrorKind::NotFound, "error after the lines")),
    ];
    let mut subject = longest(as_results(results.iter()));
    for &i in &[0, 2, 4, 3] {
        subject.assert_res(results[i]);
    }
    subject.expect_none();
}

#[test]
fn wide_chars() {
    // MaxLine should compare length in code units, returning the latter of the
    // following two strings.  The first string has just as many grapheme
    // clusters as the second, and even more bytes because it uses four code
    // points that don't fit in UTF-8 code units, whereas the latter has only
    // one such code point: the COMBINING TILDE (U+0303).
    let lines = ["píñátá", "piñata"];
    let mut subject = longest(as_lines(&lines));
    subject.assert_line(lines[1]);
    subject.expect_none();
}
