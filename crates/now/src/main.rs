#![doc = include_str!("../README.md")]

fn main() {
    let now = jiff::Zoned::now();
    let period = if now.hour() < 12 { 'a' } else { 'p' };
    let time = now.strftime("%-I:%M:%S");
    let date = now.strftime("%B %-d, %Y");
    println!("{time} {period}.m. on {date}");
}
