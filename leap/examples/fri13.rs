//! Prints a visual representation of how Friday the 13th is distributed across years.

use leap::week::{Day, DAYS};
use leap::Date;

fn is_friday_the_13th(day: Day, date: Date) -> bool {
    day == Day::Fri && date.day() == 13
}

fn is_leap_year(year: u16) -> bool {
    Date::from_ymd(year, 2, 29).is_ok()
}

fn main() {
    // The first day of 1758 happened to be a Sunday, which is the first item of `days`.
    let mut date = Date::from_ymd(1757, 12, 31).expect("hard-coded start date");
    let mut days = DAYS.iter().cycle();

    let stop = Date::from_ymd(2158, 1, 1).expect("hard-coded stop date");

    while {
        date = date.plus_one_day();
        date != stop
    } {
        if (date.month(), date.day()) == (1, 1) {
            print!("{}: ", date.year())
        }

        let day = *days.next().expect("there's always tomorrow");

        // Sanity check.
        #[cfg(debug_assertions)]
        if date == Date::from_ymd(2025, 2, 10).expect("hard-coded date") {
            assert_eq!(day, Day::Mon);
        }

        let b = is_friday_the_13th(day, date);
        print!("{}", if b { 'X' } else { ' ' });

        if (date.month(), date.day()) == (12, 31) {
            let sep = match date.year() {
                year if year as usize % 2 == 0 => "\n",
                year if is_leap_year(year) => " ",
                _ => "  ",
            };
            print!("{sep}");
        }
    }
}
