#![cfg(test)]

use std::io;

pub trait Expect {
    fn expect_err(&mut self) -> io::Error;
    fn expect_line(&mut self) -> String;
    fn expect_none(self);
}

impl<I> Expect for I
where
    I: Iterator<Item = io::Result<String>>,
{
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
