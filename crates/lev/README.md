# lev

Levenshtein distance calculator.

## Usage

```sh
lev kitten sitting
# Output:    3 sitting

lev kitten sitting mitten bitten
# Output:    3 sitting
#            1 mitten
#            1 bitten
```

## Library

```rust
use lev::lev;

assert_eq!(lev("kitten".chars(), "sitting".chars()), 3);
```
