#![cfg(test)]

use std::iter;
use super::*;

#[test]
fn empty() {
    assert!(MaxLine::new(iter::empty()).next().is_none());
}
