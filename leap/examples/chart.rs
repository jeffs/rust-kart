use leap::week::Day;
use leap::Date;

fn main() {
    for year in 1800..2200 {
        print!("{year}: ");

        let mut date = Date::from_ymd(year - 1, 12, 31).expect("last day before starting year");
        while {
            date = date.plus_one_day();
            date.year() == year
        } {
            if date.day_of_week() == Day::Fri && date.day() == 13 {
                print!("X");
            } else {
                print!(" ");
            }

            // Sanity check.
            #[cfg(debug_assertions)]
            if date == Date::from_ymd(2025, 2, 10).expect("hard-coded date") {
                assert_eq!(date.day_of_week(), Day::Mon);
            }
        }

        println!();
    }
}
