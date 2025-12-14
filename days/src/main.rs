//! This program prints the time remaining until a specified date.  Run it with
//! watch(1) to see a countdown.

use chrono::prelude::*;
use std::env;
use std::process;

fn format_part((count, unit): (i64, &str)) -> Option<String> {
    match count {
        0 => None,
        1 => Some(format!("1 {unit}")),
        _ => Some(format!("{count} {unit}s")),
    }
}

fn print_parts(parts: &[String]) {
    match parts {
        [] => println!("now"),
        [only] => println!("{only}"),
        [first, second] => println!("{first} and {second}"),
        [init @ .., last] => {
            for part in init {
                print!("{part}, ");
            }
            println!("and {last}");
        }
    }
}

fn parse_args() -> Result<NaiveDate, arg5::ParseError> {
    let mut parameters = arg5::Parser::new();
    let mut year = 0;
    let mut month = 0;
    let mut day = 0;
    parameters.declare_positional("year", &mut year);
    parameters.declare_positional("month", &mut month);
    parameters.declare_positional("day", &mut day);
    parameters.parse(env::args())?;
    NaiveDate::from_ymd_opt(year, month, day).ok_or_else(|| arg5::ParseError {
        what: String::from("bad date"),
    })
}

fn main() {
    let date = parse_args().unwrap_or_else(|error| {
        eprintln!("Error: {}", error.what);
        process::exit(1);
    });
    let duration = date.and_hms_opt(0, 0, 0).unwrap() - Local::now().naive_local();
    let parts: Vec<_> = [
        (duration.num_weeks(), "week"),
        (duration.num_days() % 7, "day"),
        (duration.num_hours() % 24, "hour"),
        (duration.num_minutes() % 60, "minute"),
        (duration.num_seconds() % 60, "second"),
    ]
    .into_iter()
    .filter_map(format_part)
    .collect();
    print_parts(&parts);
}

#[cfg(test)]
mod tests {
    use super::format_part;

    #[test]
    fn test_format_part() {
        assert_eq!(format_part((1, "week")), Some(String::from("1 week")));
        assert_eq!(format_part((0, "week")), None);
        assert_eq!(format_part((2, "week")), Some(String::from("2 weeks")));
    }
}
