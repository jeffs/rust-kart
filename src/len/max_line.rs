use std::io;

fn len(line: &str) -> usize {
    line.chars().count()
}

// Iterator that always returns the next error, if any, from a supplied sequence of line results,
// and the longest line seen, if any, otherwise.
pub struct MaxLine<I: Iterator<Item = io::Result<String>>> {
    lines: I,
    max: Option<String>, // the longest line seen so far, if any
}

impl<I: Iterator<Item = io::Result<String>>> MaxLine<I> {
    pub fn new(lines: I) -> MaxLine<I> {
        MaxLine { lines, max: None }
    }
}

impl<I: Iterator<Item = io::Result<String>>> Iterator for MaxLine<I> {
    type Item = io::Result<String>;

    // Return the next error, or the longest line seen so far if there are no more errors.
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
