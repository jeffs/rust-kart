mod basic;
mod cache;
mod sieve;
mod under1000;
mod under100000;

pub use basic::{is_prime, primes, Primes};
pub use cache::Cache;
pub use sieve::Sieve;
pub use under1000::UNDER_1000;
pub use under100000::UNDER_100000;
