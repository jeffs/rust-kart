trait Infinerator {
    type Item;

    fn next(&mut self) -> Self::Item;

    fn into_iter() -> impl Iterator<Item = Self::Item>;
}

#[cfg(test)]
mod test {
    use super::*;

    struct Fibs(u32, u32);

    impl Fibs {
        pub fn new() -> Fibs {
            Fibs(0, 1)
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            (usize::MAX, None)
        }
    }

    impl Infinerator for Fibs {
        type Item = u32;

        fn next(&mut self) -> Self::Item {
            *self = Fibs(self.1, self.0 + self.1);
            self.0
        }
    }

    #[test]
    fn size_hint() {
        let fibs = Fibs::new();
        assert_eq!(fibs.size_hint(), (usize::MAX, None));
    }
}
