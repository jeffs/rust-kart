# Rust Kart Primes

A library and CLI for generating prime numbers.

Small primes (under 100,000) are hard-coded for fast generation. Larger primes
are generated using a memory-efficient sieve of Eratosthenes.

## Usage

```sh
$ primes 4
2
3
5
7 
```

To find the thousandth prime:

```sh
$ primes 1000 | tail -1
7919
```
