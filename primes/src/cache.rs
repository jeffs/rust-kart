use std::{cell::RefCell, mem};

use super::UNDER_100000;

/// Returns true if none of the known primes divides n (other than n itself).
///
/// Prerequisite: The slice of known primes is sorted.
fn is_prime_known(n: u32, known: &[u32]) -> bool {
    !known
        .iter()
        .take_while(|&&p| p < n)
        .any(|p| n.is_multiple_of(*p))
}

pub struct Primes<'a> {
    cache: &'a Cache,
    next: u32,
}

impl Iterator for Primes<'_> {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        let next = if self.next == 2 {
            3
        } else {
            let mut next = self.next;
            while !self.cache.is_prime(next) {
                next += 2;
            }
            next
        };
        Some(mem::replace(&mut self.next, next))
    }
}

#[derive(Default)]
pub struct Cache {
    known: RefCell<Vec<u32>>,
}

impl Cache {
    fn extend_past(&self, n: u32) {
        let mut known = self.known.borrow_mut();
        if known.is_empty() {
            known.extend_from_slice(&UNDER_100000);
        }
        let last = known[known.len() - 1];
        if last < n {
            let limit = n * 2; // Arbitrary.
            for candidate in (last..limit).step_by(2) {
                if is_prime_known(candidate, &known) {
                    known.push(candidate);
                }
            }
        }
    }

    pub fn is_prime(&self, n: u32) -> bool {
        self.extend_past(n);
        self.known.borrow().binary_search(&n).is_ok()
    }

    #[must_use]
    pub fn new(known: &[u32]) -> Cache {
        Cache {
            known: RefCell::new(known.to_vec()),
        }
    }

    pub fn primes(&self) -> Primes<'_> {
        Primes {
            cache: self,
            next: 2,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::UNDER_1000;

    use super::*;

    #[ignore = "slow"]
    #[test]
    fn test_cache_is_prime() {
        let cache = Cache::new(&UNDER_1000);
        for n in 0..100_000 {
            assert_eq!(cache.is_prime(n), UNDER_100000.contains(&n));
        }
    }
}
