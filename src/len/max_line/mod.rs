pub mod order;
mod tests;

use std::io;
use std::marker::PhantomData;
use std::mem;

/// Iterator that yields all errors from a supplied underlying iterator,
/// followed by the longest line (if any) yielded by the underlying iterator.
pub struct MaxLine<I, C>
where
    I: IntoIterator<Item = io::Result<String>>,
    C: order::Policy,
{
    lines: I::IntoIter,
    max: Option<String>, // the longest line seen so far, if any
    phantom: PhantomData<C>,
}

impl<I, C> MaxLine<I, C>
where
    I: IntoIterator<Item = io::Result<String>>,
    C: order::Policy,
{
    pub fn new(lines: I) -> MaxLine<I, C> {
        MaxLine {
            lines: lines.into_iter(),
            max: None,
            phantom: PhantomData,
        }
    }
}

impl<I, C> Iterator for MaxLine<I, C>
where
    I: IntoIterator<Item = io::Result<String>>,
    C: order::Policy,
{
    type Item = io::Result<String>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(res) = self.lines.next() {
            if let Err(_) = res {
                return Some(res);
            }
            self.max = C::max(mem::take(&mut self.max), res.ok());
        }
        self.max.take().map(Ok)
    }
}

pub fn longest<I>(lines: I) -> MaxLine<I, order::Longest>
where
    I: IntoIterator<Item = io::Result<String>>,
{
    MaxLine::<I, order::Longest>::new(lines)
}

pub fn shortest<I>(lines: I) -> MaxLine<I, order::Shortest>
where
    I: IntoIterator<Item = io::Result<String>>,
{
    MaxLine::<I, order::Shortest>::new(lines)
}
