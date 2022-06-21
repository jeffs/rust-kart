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
    match parts.len() {
        0 => println!("now"),
        1 => println!("{}", parts[0]),
        2 => println!("{} and {}", parts[0], parts[1]),
        _ => {
            let m = parts.len() - 1;
            for part in &parts[0..m] {
                print!("{}, ", part);
            }
            println!("and {}", parts[m]);
        }
    };
}

fn parse_args() -> Result<Date<Local>, arg5::ParseError> {
    let mut parameters = arg5::Parser::new();
    let mut year = 0;
    let mut month = 0;
    let mut day = 0;
    parameters.declare_positional("year", &mut year);
    parameters.declare_positional("month", &mut month);
    parameters.declare_positional("day", &mut day);
    parameters.parse(env::args())?;
    Ok(Local.ymd(year, month, day))
}

#[cfg(test)]
mod tests {
    #[test]
    fn format() {
        assert_eq!("1 week", super::format(1, "week"));
        assert_eq!("0 weeks", super::format(0, "week"));
        assert_eq!("2 weeks", super::format(2, "week"));
    }
}

fn main() {
    let date = parse_args().unwrap_or_else(|error| {
        eprintln!("Error: {}", error.what);
        process::exit(1);
    });
    let duration = date.and_hms(0, 0, 0) - Local::now();
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
