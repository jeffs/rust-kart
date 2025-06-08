type Word = u64;

const WORD_BITS: usize = Word::BITS as usize;

// The little-endian index of each 1 bit, times two and plus one, is prime.  For
// example, the first ten bits are:
//
//  bits:       1101101110
//  indexes:    9876543210
//
// Meaning:
//
//  index             number    prime?
//  -----             ------    ------
//    0     * 2 + 1 =    1        0
//    1     * 2 + 1 =    3        1
//    2     * 2 + 1 =    5        1
//    3     * 2 + 1 =    7        1
//    4     * 2 + 1 =    9        0
//    5     * 2 + 1 =   11        1
//    6     * 2 + 1 =   13        1
//    7     * 2 + 1 =   15        0
//    8     * 2 + 1 =   17        1
//    9     * 2 + 1 =   19        1
const FIRST_WORD: Word = 0x816d_129a_64b4_cb6e;

pub struct Primes<'a> {
    sieve: &'a mut Sieve,
    known: u32,
}

impl Iterator for Primes<'_> {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        self.known = match self.known {
            0 => 2,
            2 => 3,
            _ => ((self.known + 2)..)
                .step_by(2)
                .find(|&n| self.sieve.is_prime(n))
                .expect("another prime"),
        };
        Some(self.known)
    }
}

pub struct Factors<'a> {
    primes: Primes<'a>,
    value: u32,
}

impl Iterator for Factors<'_> {
    type Item = (u32, u32); // (factor, exponent)

    fn next(&mut self) -> Option<Self::Item> {
        let mut value = self.value;

        // The remaining value has no prime factors.
        if value < 2 {
            return None;
        }

        for prime in self.primes.by_ref() {
            // If the next prime is greater than the square root of the value,
            // then it won't divide the value, because we already factored out
            // whatever the quotient would be (since it's smaller than the
            // prime). The remaining value (after all the divisions we've
            // already applied to it) is either 1 (because we've divided it by
            // all its factors), in which case we already returned None; or is
            // prime itself, which is the only way it could still have a factor
            // greater than its square root.
            if prime * prime > value {
                self.value = 1;
                return Some((value, 1));
            }

            // Skip any prime that isn't a factor of our value.
            if value % prime != 0 {
                continue;
            }

            // Reduce our value, counting how many times this prime divides it.
            let mut power = 1;
            while {
                value /= prime;
                value % prime == 0
            } {
                power += 1;
            }
            self.value = value;
            return Some((prime, power));
        }
        unreachable!()
    }
}

pub struct Sieve {
    words: Vec<Word>,
}

impl Sieve {
    #[must_use]
    pub fn new() -> Self {
        Sieve { words: Vec::new() }
    }

    fn mark_nonprime(&mut self, value: u32) {
        if value % 2 == 0 {
            return; // We don't store bits for even numbers, anyway.
        }
        let index = value as usize / 2;
        self.words[index / WORD_BITS] &= !(1 << (index % WORD_BITS));
    }

    fn is_known_prime(&self, value: u32) -> bool {
        // We get twice as many bits per word by skipping even-indexed bits,
        // since no even numbers past 2 are prime.  We special-case 2.
        if value % 2 == 0 {
            return value == 2;
        }
        let index = (value / 2) as usize;
        self.words[index / WORD_BITS] & (1 << (index % WORD_BITS)) != 0
    }

    // TODO: Why is this a u32 rather than a usize?  Why not support 1<<32 primes?
    fn num_values(&self) -> u32 {
        u32::try_from(self.words.len() * WORD_BITS * 2).expect("sieve max size exceeded")
    }

    pub fn grow(&mut self) {
        if self.words.is_empty() {
            self.words.push(FIRST_WORD);
            return;
        }

        // Append a bunch of ones, then iterate through known primes and mark
        // all their multiples composite by clearing bits.  We always have at
        // least enough known primes to fill out twice the size of our table,
        // but we cap the number of new bits added at any time to avoid
        // overflowing our value type (since we use indexes as values).  The
        // cap value is arbitrary.
        //
        // TODO: What was I thinking when I wrote about "having at least enough primes?"
        let num_old_values = self.num_values();

        // TODO: This caps the total number of words, not the number of _new_ words.
        let new_len = 1_000_000.min(self.words.len() * 2);
        self.words.resize(new_len, !0);

        let num_new_values = self.num_values();
        for value in (3..num_old_values).step_by(2) {
            if !self.is_known_prime(value) {
                continue; // Skip non-prime.
            }
            let start = num_old_values - num_old_values % value + value;
            for new_index in (start..num_new_values).step_by(value as usize) {
                self.mark_nonprime(new_index); // New index is divisible by value.
            }
        }
    }

    pub fn is_prime(&mut self, value: u32) -> bool {
        while self.num_values() <= value {
            self.grow();
        }
        self.is_known_prime(value)
    }

    pub fn primes(&mut self) -> Primes {
        Primes {
            sieve: self,
            known: 0,
        }
    }

    pub fn factors(&mut self, value: u32) -> Factors {
        Factors {
            primes: self.primes(),
            value,
        }
    }
}

impl Default for Sieve {
    fn default() -> Self {
        Sieve::new()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::{UNDER_1000, UNDER_100000};

    use super::*;

    #[test]
    fn test_sieve_is_prime() {
        let mut sieve = Sieve::default();
        let known: HashSet<_> = UNDER_100000.into_iter().collect();
        for n in 0..100_000 {
            assert_eq!(
                sieve.is_prime(n),
                known.contains(&n),
                "whether {n} is prime"
            );
        }
    }

    #[test]
    fn test_sieve_primes() {
        let want = UNDER_1000;
        let mut sieve = Sieve::default();
        let got: Vec<_> = sieve.primes().take(want.len()).collect();
        assert_eq!(got, want);
    }

    #[test]
    fn test_sieve_factors() {
        let mut sieve = Sieve::default();
        let table: &[(u32, &[(u32, u32)])] = &[
            (0, &[]),
            (1, &[]),
            (2, &[(2, 1)]),
            (3, &[(3, 1)]),
            (4, &[(2, 2)]),
            (5, &[(5, 1)]),
            (6, &[(2, 1), (3, 1)]),
            (7, &[(7, 1)]),
            (8, &[(2, 3)]),
            (18, &[(2, 1), (3, 2)]),
        ];
        for &(arg, want) in table {
            let got: Vec<_> = sieve.factors(arg).collect();
            assert_eq!(got, want, "factorization of {arg}");
        }
    }
}
