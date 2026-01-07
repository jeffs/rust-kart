//! Provides a trait similar to the standard [`Iterator`], but for sequences
//! that never terminate. In particular, [`Perpetuity::next_item`] returns
//! `Self::Item`, not `Option<Self::Item>`.  The interface to functions like
//! `successors` can also be much simpler, as neither the initial value
//! nor the callback need deal with options.  For example, compare the following
//! two expressions:
//!
//! ```ignore
//! use std::iter;
//! use iter_utils::Perpetuity;
//!
//! let powers_of_two = iter::successors(Some(1), |a| Some(a * 2));
//! let powers_of_two = iter_utils::successors(1, |a| a * 2);
//! ```
//!
//! One common form of infinite range is sequences of IDs.  For example:
//!
//! ```ignore
//! let mut ids = 1..;
//! let thing1 = Thing { id: ids.next().unwrap() };
//! let thing2 = Thing { id: ids.next().unwrap() };
//! // ...
//! ```
//!
//! Using Perpetuity, we no longer need to unwrap anything:
//!
//! ```ignore
//! use iter_utils::Perpetuity;
//!
//! let mut ids = 1..;
//! let thing1 = Thing { id: ids.next_item() };
//! let thing2 = Thing { id: ids.next_item() };
//! ```

use std::{mem, ops::RangeFrom};

/// Like [`Iterator`], but returns items directly, rather than [`Option`]s.
pub trait Perpetuity: Sized {
    type Item;

    fn next_item(&mut self) -> Self::Item;

    fn into_iter(self) -> IntoIter<Self> {
        IntoIter(self)
    }

    fn take(self, count: usize) -> Take<Self> {
        Take { items: self, count }
    }
}

/// Returns the specified [`Perpetuity`], erasing any further type information.
/// ```
/// let _: Vec<i32> = (0..).take(4).collect();
/// ```
/// ```compile_fail
/// let _: Vec<i32> = iter_utils::assimilate(0..).take(4).collect();
/// ```
pub fn assimilate<T>(items: impl Perpetuity<Item = T>) -> impl Perpetuity<Item = T> {
    items
}

pub struct IntoIter<I: Perpetuity>(I);

impl<I: Perpetuity> Iterator for IntoIter<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.0.next_item())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (usize::MAX, None)
    }
}

pub struct Take<I: Perpetuity> {
    items: I,
    count: usize,
}

impl<I: Perpetuity> Iterator for Take<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        (self.count > 0).then(|| {
            self.count -= 1;
            self.items.next_item()
        })
    }
}

pub struct Successors<T, F: FnMut(&T) -> T> {
    next: T,
    succ: F,
}

impl<T, F: FnMut(&T) -> T> Perpetuity for Successors<T, F> {
    type Item = T;
    fn next_item(&mut self) -> Self::Item {
        let next = (self.succ)(&self.next);
        mem::replace(&mut self.next, next)
    }
}

macro_rules! range_from {
    ($t:ty) => {
        impl Perpetuity for RangeFrom<$t> {
            type Item = $t;
            fn next_item(&mut self) -> Self::Item {
                <Self as Iterator>::next(self).unwrap()
            }
        }
    };
}

range_from!(i8);
range_from!(i16);
range_from!(i32);
range_from!(i64);
range_from!(i128);
range_from!(u8);
range_from!(u16);
range_from!(u32);
range_from!(u64);
range_from!(u128);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_item() {
        let mut got = Vec::new();
        let mut count = 0..;
        for _ in 0..4 {
            got.push(count.next_item());
        }
        assert_eq!(got, [0, 1, 2, 3]);
    }

    #[test]
    fn into_iter() {
        let got: Vec<i32> = assimilate(0..).into_iter().take(4).collect();
        assert_eq!(got, [0, 1, 2, 3]);
    }

    #[test]
    fn take() {
        let got: Vec<i32> = assimilate(0..).take(4).collect();
        assert_eq!(got, [0, 1, 2, 3]);
    }
}
