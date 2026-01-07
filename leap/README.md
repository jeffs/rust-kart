# leap

Proleptic Gregorian calendar date library with leap year support.

## Features

- Date validation and construction
- Day of week calculation
- Leap year handling
- Date arithmetic (plus one day/week)

## Usage

```rust
use leap::Date;

let date = Date::from_ymd(2024, 2, 29).unwrap();
assert!(date.is_leap_day());
println!("{}", date.day_of_week()); // Thursday
```
