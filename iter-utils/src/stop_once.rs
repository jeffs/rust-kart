//! Iterator adapter that yields items until a predicate matches, inclusive.

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stop_once_includes_terminal() {
        let items: Vec<_> = (1..10).stop_once(|&x| x == 5).collect();
        assert_eq!(items, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn stop_once_empty_if_first_matches() {
        let items: Vec<_> = (1..10).stop_once(|&x| x == 1).collect();
        assert_eq!(items, vec![1]);
    }

    #[test]
    fn stop_once_all_if_none_match() {
        let items: Vec<_> = (1..5).stop_once(|&x| x == 10).collect();
        assert_eq!(items, vec![1, 2, 3, 4]);
    }
}
