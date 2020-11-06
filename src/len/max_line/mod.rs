mod tests;

use std::io;

fn len(line: &str) -> usize {
    line.chars().count()
}

/// Iterator that yields all errors from a supplied underlying iterator,
/// followed by the longest line (if any) yielded by the underlying iterator.
pub struct MaxLine<I>
where
    I: IntoIterator<Item = io::Result<String>>,
{
    lines: I::IntoIter,
    max: Option<String>, // the longest line seen so far, if any
}

impl<I> MaxLine<I>
where
    I: IntoIterator<Item = io::Result<String>>,
{
    pub fn new(lines: I) -> MaxLine<I> {
        MaxLine {
            lines: lines.into_iter(),
            max: None,
        }
    }
}

impl<I> Iterator for MaxLine<I>
where
    I: IntoIterator<Item = io::Result<String>>,
{
    type Item = io::Result<String>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(res) = self.lines.next() {
            match res {
                Ok(line) => {
                    if self.max.is_none() || len(&line) > len(self.max.as_ref().unwrap()) {
                        self.max = Some(line.clone());
                    }
                }
                _ => {
                    return Some(res);
                }
            }
        }
        self.max.take().map(Ok)
    }
}
