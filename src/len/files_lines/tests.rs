#![cfg(test)]

use super::*;
use std::path::Path;
use std::{fs, iter};

fn list_dir<P>(dir: P) -> io::Result<Vec<PathBuf>>
where
    P: AsRef<Path>,
{
    fs::read_dir(dir)?
        .map(|res| res.map(|entry| entry.path()))
        .collect()
}

fn read_lines<P>(path: P) -> io::Result<Vec<String>>
where
    P: AsRef<Path>,
{
    io::BufReader::new(File::open(path.as_ref())?)
        .lines()
        .collect()
}

fn read_lines_all<P, I>(paths: I) -> io::Result<Vec<String>>
where
    P: AsRef<Path>,
    I: IntoIterator<Item = P>,
{
    let mut lines = vec![];
    for path in paths {
        let file = File::open(path.as_ref())?;
        for line in io::BufReader::new(file).lines() {
            lines.push(line?);
        }
    }
    Ok(lines)
}

trait Expect {
    fn expect_err(&mut self) -> io::Error;
    fn expect_line(&mut self) -> String;
    fn expect_none(self);
}

impl Expect for FilesLines {
    fn expect_err(&mut self) -> io::Error {
        match self.next() {
            None => panic!("want Some(err); got None"),
            Some(Ok(line)) => panic!("want Some(err); got line: {}", line),
            Some(Err(err)) => err,
        }
    }

    fn expect_line(&mut self) -> String {
        match self.next() {
            None => panic!("want Some(Ok(line)); got None"),
            Some(Ok(line)) => line,
            Some(Err(err)) => panic!("unexpected error: {}", err),
        }
    }

    fn expect_none(mut self) {
        match self.next() {
            None => (),
            Some(Ok(line)) => panic!("unexpected line: {}", line),
            Some(Err(err)) => panic!("unexpected error: {}", err),
        }
    }
}

#[test]
fn no_files() {
    FilesLines::new(iter::empty::<&Path>()).expect_none();
}

#[test]
fn empty_file() {
    FilesLines::new(&["tests/data/utf8/empty"]).expect_none();
}

#[test]
fn no_such_file() {
    let mut lines = FilesLines::new(&["tests/data/nonesuch"]);
    lines.expect_err();
    lines.expect_none();
}

#[test]
fn utf8_files() -> io::Result<()> {
    let paths = list_dir("tests/data/utf8")?;
    let want = read_lines_all(&paths)?;
    let got = FilesLines::new(&paths).collect::<io::Result<Vec<_>>>()?;
    assert_eq!(want, got);
    Ok(())
}

#[test]
fn recoverable_errors() -> io::Result<()> {
    let paths = [
        "tests/data/utf8/fox",
        "tests/data/bad",
        "tests/data/utf8/men",
        "tests/data/nonesuch",
        "tests/data/bad",
        "tests/data/utf8/wide.py",
    ];
    let mut got = FilesLines::new(&paths);
    for want in read_lines(paths[0])? {
        assert_eq!(want, got.expect_line());
    }
    assert_eq!(io::ErrorKind::InvalidData, got.expect_err().kind());
    for want in read_lines(paths[2])? {
        assert_eq!(want, got.expect_line());
    }
    assert_eq!(io::ErrorKind::NotFound, got.expect_err().kind());
    assert_eq!(io::ErrorKind::InvalidData, got.expect_err().kind());
    for want in read_lines(paths[5])? {
        assert_eq!(want, got.expect_line());
    }
    got.expect_none();
    Ok(())
}
