#![cfg(test)]

use super::*;
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

#[test]
fn test_no_files() -> Result<(), String> {
    expect_none(FilesLines::new(iter::empty()))
}

#[test]
fn test_empty_file() -> Result<(), String> {
    let paths = vec![PathBuf::from("tests/data/utf8/empty")].into_iter();
    expect_none(FilesLines::new(paths))
}

#[test]
fn test_no_such_file() -> Result<(), String> {
    let paths = vec![PathBuf::from("tests/data/nonesuch")].into_iter();
    let mut lines = FilesLines::new(paths);
    expect_err(&mut lines)?;
    expect_none(lines)
}

#[test]
fn test_good_files() -> io::Result<()> {
    let paths = fs::read_dir("tests/data/utf8")?
        .filter_map(|res| res.ok())
        .map(|entry| entry.path());
    let mut count = 0;
    for res in FilesLines::new(paths) {
        res?;
        count += 1;
    }
    assert_ne!(0, count);
    Ok(())
}
