mod order;
mod tests;

use std::io;
use std::marker::PhantomData;
use std::mem;

/// Iterator that yields all errors from a supplied underlying iterator,
/// followed by the maximal line (if any) yielded by the underlying iterator,
/// as determined by the specified comparator (i.e., order policy).
struct MaxLine<I, C>
where
    I: IntoIterator<Item = io::Result<String>>,
    C: order::Policy,
{
    lines: I::IntoIter,
    max: Option<String>,
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

pub fn longest<I>(lines: I) -> impl Iterator<Item = io::Result<String>>
where
    I: IntoIterator<Item = io::Result<String>>,
{
    MaxLine::<I, order::Longest>::new(lines)
}

pub fn shortest<I>(lines: I) -> impl Iterator<Item = io::Result<String>>
where
    I: IntoIterator<Item = io::Result<String>>,
{
    MaxLine::<I, order::Shortest>::new(lines)
}
