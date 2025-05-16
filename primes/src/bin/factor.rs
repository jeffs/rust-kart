//! Prints the prime factorization of small natural numbers.

fn factor(mut n: u32) -> Vec<(u32, u32)> {
    let mut pairs = Vec::new();

    // 0 in particular needs a special case, because it's divisible by anything.
    if n < 2 {
        return pairs;
    }

    for p in rk_primes::Sieve::default().primes() {
        let mut e = 0;
        while n % p == 0 {
            n /= p;
            e += 1;
        }
        if e != 0 {
            pairs.push((p, e));
        }
        if n == 1 {
            return pairs;
        }
    }

    pairs
}

fn main_imp() -> Result<(), String> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty() {
        println!("Usage: factor N [N...]");
        return Ok(());
    }
    for arg in args {
        let n = arg.parse().map_err(|_| "args should be natural numbers")?;
        println!("{:?}", factor(n));
    }
    Ok(())
}

fn main() {
    if let Err(err) = main_imp() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
