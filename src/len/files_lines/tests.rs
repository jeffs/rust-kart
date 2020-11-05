#![cfg(test)]

use std::iter;
use super::*;

#[test]
fn test_empty() -> Result<(), String> {
    match FilesLines::new(iter::empty()).next() {
        None => Ok(()),
        Some(Ok(line)) => Err(format!("unexpected line: {}", line)),
        Some(Err(err)) => Err(format!("unexpected error: {}", err)),
    }
}
