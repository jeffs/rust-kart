use leap::week::{Day, DAYS};
use leap::Date;

fn is_friday_the_13th(day: Day, date: Date) -> bool {
    day == Day::Fri && date.day() == 13
}

fn main() {
    // The first day of 2023 happened to be a Sunday, which is the first item of `days`.
    let mut date = Date::from_ymd(2023, 1, 1).expect("hard-coded start date should be valid");
    let mut days = DAYS.iter().cycle();

    let stop = Date::from_ymd(2063, 1, 1).expect("hard-coded stop date should be valid");

    while date != stop {
        if (date.month(), date.day()) == (1, 1) {
            print!("{}: ", date.year())
        }

        let day = *days.next().expect("there's always tomorrow");
        let b = is_friday_the_13th(day, date);
        print!("{}", if b { 'F' } else { '_' });

        if (date.month(), date.day()) == (12, 31) {
            println!("\n")
        }

        date = date.plus_one_day();
    }
}
