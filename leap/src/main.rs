use leap::Date;

fn main() {
    for year in 2000..=2025 {
        let mut date = Date::from_ymd(year, 1, 1).expect("hard-coded date should be valid");
        while date.year() == year {
            print!("{}", if date.is_leap_day() { 'L' } else { '_' });
            date = date.plus_one_day();
        }
        println!();
    }
}
