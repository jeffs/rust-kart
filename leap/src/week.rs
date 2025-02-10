#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Day {
    Sun,
    Mon,
    Tue,
    Wed,
    Thu,
    Fri,
    Sat,
}

pub const DAYS: [Day; 7] = [
    Day::Sun,
    Day::Mon,
    Day::Tue,
    Day::Wed,
    Day::Thu,
    Day::Fri,
    Day::Sat,
];
