use std::ops::RangeFrom;

// Like std::iter::Iterator, but returns items directly, rather than options.
trait Infiniterator: Sized {
    type Item;

    fn next_item(&mut self) -> Self::Item;

    // We can't automatically impl IntoIterator for all Infiniterators, because
    // IntoIterator isn't defined here, and that's a rule.
    fn into_iter(self) -> IntoIter<Self> {
        IntoIter(self)
    }
}

struct IntoIter<I: Infiniterator>(I);

impl<I: Infiniterator> Iterator for IntoIter<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.0.next_item())
    }
}

// We can't implement traits for RangeFrom<A> as Iterator<Item=A> without
// declaring A: std::iter::Step, but Step is unstable (even though it's used in
// the implementation of RangeFrom, which is stable).  So, we can't impl traits
// for RangeFrom as Iterator in stable Rust.  See also:
// https://stackoverflow.com/a/56986698/3116635
//
// If we could use Step in stable Rust, the impl might look like this:
//
// ```
// impl<A: Step> Infiniterator for RangeFrom<A> {
//     type Item = A;
//     fn next_item(&mut self) -> Self::Item {
//         self.next().unwrap()
//     }
// }
// ```
//
// So instead, we crank out implementations for specific types of RangeFrom.
macro_rules! range_from {
    ($t:ty) => {
        impl Infiniterator for RangeFrom<$t> {
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
        let mut counts = Vec::new();
        let mut count = 0..;
        for _ in 0..4 {
            counts.push(count.next_item()); // Look ma, no .unwrap()!
        }
        assert_eq!(counts, [0, 1, 2, 3]);
    }

    #[test]
    fn into_iter() {
        fn take4<I: Infiniterator>(items: I) -> Vec<I::Item> {
            items.into_iter().take(4).collect()
        }
        assert_eq!(take4(0..), [0, 1, 2, 3]);
    }
}
