pub struct StopOnce<I, P> {
    iter: Option<I>,
    is_last: P,
}

impl<I: Iterator, P: FnMut(&I::Item) -> bool> Iterator for StopOnce<I, P> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.iter.as_mut()?.next()?;
        if (self.is_last)(&item) {
            self.iter = None;
        }
        Some(item)
    }
}

pub trait Stopper: Iterator
where
    Self: Sized,
{
    /// Like `Iterator::take_while`, but additionally yields the terminal item.
    fn stop_once<P: FnMut(&Self::Item) -> bool>(self, predicate: P) -> StopOnce<Self, P>;
}

impl<I: Iterator> Stopper for I {
    fn stop_once<P: FnMut(&Self::Item) -> bool>(self, predicate: P) -> StopOnce<Self, P> {
        StopOnce {
            iter: Some(self),
            is_last: predicate,
        }
    }
}
