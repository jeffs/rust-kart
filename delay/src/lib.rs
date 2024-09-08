/// Wraps an iterator of type I by creating it on the first call to `next()`.
/// Creates the wrapped iterator by calling F.
enum DelayState<I: Iterator, F: FnOnce() -> I> {
    // New wraps an Option only so that we can take F out of it during the
    // transition from New to Old.  It feels like this shouldn't be necessary,
    // but we're not allowed to move out of self in the body of Iterator::next,
    // which merely borrows (rather than owning) self. See also StackOverflow:
    // <https://stackoverflow.com/questions/36557412/how-can-i-change-enum-variant-while-moving-the-field-to-the-new-variant>
    New(Option<F>),
    Old(I),
}

pub struct Delayed<I: Iterator, F: FnOnce() -> I>(DelayState<I, F>);

impl<I: Iterator, F: FnOnce() -> I> Iterator for Delayed<I, F> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.0 {
            DelayState::New(new) => {
                // Every Delay begins life as New(Some(_)).  On the first call
                // to .next(), it is replaced by Old(_).  The only region in
                // which a Delay is New(None) is the following few lines, after
                // we take the function out of the option.
                let new = new
                    .take()
                    .expect("Delay to have either a function or an iterator");
                let mut old = new();
                let next = old.next();
                self.0 = DelayState::Old(old);
                next
            }
            DelayState::Old(old) => old.next(),
        }
    }
}

/// Wraps an iterator created by `new` on the first  call to
/// [`next()`][Iterator::next()].
///
/// For example, consider Haskell's [canonical zipWith implementation][hs]:
/// ```hs
/// fibs = 0 : 1 : zipWith (+) fibs (tail fibs)
/// ```
///
/// A naive translation to Rust would cause infinite recursion. We can emulate
/// Haskell's lazy evaluation by delaying the recursive calls:
/// ```
/// use delay::delay;
///
/// /// Returns an inefficient iterator over the Fibonacci numbers.
/// fn fibs() -> Box<dyn Iterator<Item = u32>> {
///     Box::new(
///         [0, 1]
///             .into_iter()
///             .chain(delay(fibs).zip(delay(fibs).skip(1)).map(|(x, y)| x + y)),
///     )
/// }
///
/// for (got, want) in fibs().zip([0, 1, 1, 2, 3, 5, 8, 13]) {
///     assert_eq!(got, want);
/// }
/// ```
///
/// Equivalently, but perhaps a bit more clearly:
/// ```
/// # use delay::delay;
/// fn fibs() -> Box<dyn Iterator<Item = u32>> {
///     let xs = delay(fibs);                     // 0 1 1 2 3 ...
///     let ys = delay(fibs).skip(1);             // 1 1 2 3 5 ...
///     let zs = xs.zip(ys).map(|(x, y)| x + y);  // 1 2 3 5 8 ...
///     Box::new([0, 1].into_iter().chain(zs))
/// }
///
/// # for (got, want) in fibs().zip([0, 1, 1, 2, 3, 5, 8, 13]) {
/// #     assert_eq!(got, want);
/// # }
/// ```
///
/// [hs]: https://wiki.haskell.org/The_Fibonacci_sequence#Canonical_zipWith_implementation
pub fn delay<I: Iterator, F: FnOnce() -> I>(new: F) -> Delayed<I, F> {
    Delayed(DelayState::New(Some(new)))
}

/// Lets you call `fibs.delay()` instead of `delay(fibs)`, if such is your wont.
pub trait Delay<I: Iterator, F: FnOnce() -> I> {
    fn delay(self) -> Delayed<I, F>;
}

impl<I: Iterator, F: FnOnce() -> I> Delay<I, F> for F {
    fn delay(self) -> Delayed<I, F> {
        delay(self)
    }
}
