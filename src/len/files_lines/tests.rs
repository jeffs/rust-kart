#![cfg(test)]

use super::*;
use crate::len::expect::Expect;
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

fn expect_single_error(path: &str, kind: io::ErrorKind) {
    let mut lines = FilesLines::new(&[path]);
    let err = lines.expect_err();
    assert_eq!(kind, err.kind());
    assert!(
        err.to_string().contains(path),
        format!(r#"expected path "{}" in "{}""#, path, err)
    );
    lines.expect_none();
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
    expect_single_error("tests/data/nonesuch", io::ErrorKind::NotFound);
}

#[test]
fn non_utf8_file() {
    expect_single_error("tests/data/bad", io::ErrorKind::InvalidData);
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
fn forwards_errors() -> io::Result<()> {
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
    Ok(got.expect_none())
}

#[test]
fn recurses_on_dirs() -> io::Result<()> {
    let paths = ["tests/data/dir"];
    let mut lines = FilesLines::new(&paths);
    assert_eq!("f", lines.expect_line());
    assert_eq!("h", lines.expect_line());
    assert_eq!("i", lines.expect_line());
    assert_eq!("j", lines.expect_line());
    Ok(lines.expect_none())
}
