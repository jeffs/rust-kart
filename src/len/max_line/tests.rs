#![cfg(test)]

use super::*;
use crate::len::expect::Expect;
use std::{io, iter};

type ErrorBrief = (io::ErrorKind, &'static str);

fn as_errors<'a, I>(errors: I) -> impl 'a + Iterator<Item = io::Result<String>>
where
    I: 'a + IntoIterator<Item = &'a ErrorBrief>,
{
    errors
        .into_iter()
        .map(|&(kind, what)| Err(io::Error::new(kind, what)))
}

fn as_lines<'a, I>(lines: I) -> impl 'a + Iterator<Item = io::Result<String>>
where
    I: 'a + IntoIterator<Item = &'a &'static str>,
{
    lines.into_iter().map(|&line| Ok(line.to_owned()))
}

trait Assert {
    fn assert_err(&mut self, want: ErrorBrief);
    fn assert_line(&mut self, want: &str);
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
}

#[test]
fn empty() {
    assert!(MaxLine::new(iter::empty()).next().is_none());
}

#[test]
fn errors_only() {
    let errors = [
        (io::ErrorKind::InvalidData, "bad data 1"),
        (io::ErrorKind::NotFound, "not found"),
        (io::ErrorKind::InvalidData, "bad data 2"),
    ];
    let mut subject = MaxLine::new(as_errors(&errors));
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
    let mut subject = MaxLine::new(as_lines(&lines));
    subject.assert_line(lines[1]);
    subject.expect_none();
}
