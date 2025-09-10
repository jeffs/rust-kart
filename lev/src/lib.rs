/// Returns the [Levenshtein distance][1] between `a` and `b`.
///
/// [1]: https://en.wikipedia.org/wiki/Levenshtein_distance#Definition
pub fn lev<T: PartialEq>(
    a: impl Clone + Iterator<Item = T>,
    b: impl Clone + Iterator<Item = T>,
) -> usize {
    let (mut tail_a, mut tail_b) = (a.clone(), b.clone());
    let Some(x) = tail_a.next() else {
        return b.count();
    };
    let Some(y) = tail_b.next() else {
        return a.count();
    };
    if x == y {
        lev(tail_a, tail_b)
    } else {
        1 + lev(tail_a.clone(), b)
            .min(lev(a, tail_b.clone()))
            .min(lev(tail_a, tail_b))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lev_works_on_wikipedia_examples() {
        for (a, b, n) in [
            ("kitten", "sitting", 3),       // sub, sub, insert
            ("uninformed", "uniformed", 1), // delete
        ] {
            assert_eq!(lev(a.bytes(), b.bytes()), n);
            assert_eq!(lev(a.chars(), b.chars()), n);
        }
    }
}
