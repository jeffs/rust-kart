use leap::week::{Day, DAYS};
use leap::Date;

fn is_friday_the_13th(day: Day, date: Date) -> bool {
    day == Day::Fri && date.day() == 13
}

fn main() {
    // The first day of 1764 happened to be a Sunday, which is the first item of `days`.
    let mut date = Date::from_ymd(1764, 1, 1).expect("hard-coded start date");
    let mut days = DAYS.iter().cycle();

    let stop = Date::from_ymd(2123, 1, 1).expect("hard-coded stop date");

    while date != stop {
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
            println!()
        }

        date = date.plus_one_day();
    }
}
