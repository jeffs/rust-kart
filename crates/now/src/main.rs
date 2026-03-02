#![doc = include_str!("../README.md")]

fn main() {
    let time = jiff::Zoned::now();
    let period = if time.hour() < 12 { 'a' } else { 'p' };
    println!("{} {period}.m.", time.strftime("%B %-d, %Y at %-I:%M:%S"));
}
