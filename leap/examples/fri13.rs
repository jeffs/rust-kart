use std::iter;

use leap::week::Day;
use leap::Date;

fn main() {
    for year in 1800..2200 {
        let marks = iter::successors(Date::from_ymd(year, 1, 1).ok(), |today| {
            let tomorrow = today.plus_one_day();
            (tomorrow.year() == year).then_some(tomorrow)
        })
        .map(|date| {
            if (date.day_of_week(), date.day()) == (Day::Fri, 13) {
                'X'
            } else {
                ' '
            }
        })
        .collect::<String>();
        println!("{year}: {marks}");
    }
}
