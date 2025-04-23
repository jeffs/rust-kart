use std::process::exit;

use rk_primes::{Sieve, UNDER_100000};

fn print_n(items: impl IntoIterator<Item = u32>, n: usize) {
    items
        .into_iter()
        .take(n)
        .for_each(|item| println!("{item}"));
}

fn main() {
    let count = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("error: expected count");
        exit(2);
    });
    let Ok(count) = count.parse::<usize>() else {
        eprintln!("error: {count}: expected natural number");
        exit(2);
    };
    if count <= UNDER_100000.len() {
        print_n(UNDER_100000, count);
    } else {
        print_n(Sieve::default().primes(), count);
    }
}
