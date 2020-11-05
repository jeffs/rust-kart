use std::io;

pub trait LineIterator: Iterator<Item = io::Result<String>> {}

impl<I: Iterator<Item = io::Result<String>>> LineIterator for I {}
