#![cfg(test)]

use super::*;
use std::path::Path;
use std::{fs, iter};

fn expect_err(lines: &mut FilesLines) -> Result<(), String> {
    match lines.next() {
        None => return Err("want Some(err); got None".to_owned()),
        Some(Ok(line)) => return Err(format!("want Some(err); got line: {}", line)),
        Some(Err(_)) => Ok(()),
    }
}

fn expect_none(mut lines: FilesLines) -> Result<(), String> {
    match lines.next() {
        None => Ok(()),
        Some(Ok(line)) => Err(format!("unexpected line: {}", line)),
        Some(Err(err)) => Err(format!("unexpected error: {}", err)),
    }
}

fn read_lines<P: AsRef<Path>, I: Iterator<Item = P>>(paths: I) -> io::Result<Vec<String>> {
    let mut lines = vec![];
    for path in paths {
        for line in io::BufReader::new(File::open(path)?).lines() {
            lines.push(line?);
        }
    }
    Ok(lines)
}

#[test]
fn no_files() -> Result<(), String> {
    expect_none(FilesLines::new(iter::empty()))
}

#[test]
fn empty_file() -> Result<(), String> {
    let paths = vec![PathBuf::from("tests/data/utf8/empty")];
    expect_none(FilesLines::new(paths.into_iter()))
}

#[test]
fn no_such_file() -> Result<(), String> {
    let paths = vec![PathBuf::from("tests/data/nonesuch")];
    let mut lines = FilesLines::new(paths.into_iter());
    expect_err(&mut lines)?;
    expect_none(lines)
}

#[test]
fn utf8_files() -> io::Result<()> {
    let paths = fs::read_dir("tests/data/utf8")?
        .collect::<io::Result<Vec<_>>>()?
        .iter()
        .map(|entry| entry.path())
        .collect::<Vec<_>>();
    let lines = FilesLines::new(paths.clone().into_iter()).collect::<io::Result<Vec<_>>>()?;
    assert_eq!(read_lines(paths.into_iter())?, lines);
    Ok(())
}
