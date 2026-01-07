# iter-utils

Iterator utilities for infinite sequences, lazy creation, and termination.

## Features

- **Perpetuity**: A trait for infinite sequences where `next_item()` returns `T` directly (no `Option`).
- **delay**: Lazy iterator creation for recursive patterns (e.g., Fibonacci).
- **stop_once**: Like `take_while`, but includes the terminal item.

## Usage

```rust
use iter_utils::{Perpetuity, delay, Stopper};

// Infinite sequences without .unwrap()
let mut ids = 1..;
let id = ids.next_item(); // Returns i32, not Option<i32>

// Lazy recursive iterators
fn fibs() -> Box<dyn Iterator<Item = u32>> {
    Box::new([0, 1].into_iter()
        .chain(delay(fibs).zip(delay(fibs).skip(1)).map(|(a, b)| a + b)))
}

// Inclusive termination
let items: Vec<_> = (1..10).stop_once(|&x| x == 5).collect();
// [1, 2, 3, 4, 5] - includes the 5
```
