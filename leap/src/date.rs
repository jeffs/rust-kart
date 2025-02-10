use std::ops::RangeInclusive;

use crate::{Error, Result};

fn month_days(year: u16, month: u8) -> RangeInclusive<u8> {
    let month = usize::from(month);
    let is_leap_year = year % 4 == 0 && year % 100 != 0 || year % 400 == 0;
    1..=[
        31,                                 // January
        if is_leap_year { 29 } else { 28 }, // February
        31,                                 // March
        30,                                 // April
        31,                                 // May
        30,                                 // June
        31,                                 // July
        31,                                 // August
        30,                                 // September
        31,                                 // October
        30,                                 // November
        31,                                 // December
    ][month - 1]
}

#[derive(Clone, Copy)]
pub struct Date {
    year: u16,
    month: u8,
    day: u8,
}

impl Date {
    pub fn day(self) -> u8 {
        self.day
    }

    pub fn from_ymd(year: u16, month: u8, day: u8) -> Result<Date> {
        (0 < year && (1..=12).contains(&month) && month_days(year, month).contains(&day))
            .then_some(Date { year, month, day })
            .ok_or(Error::Date { year, month, day })
    }

    pub fn month(self) -> u8 {
        self.month
    }

    pub fn year(self) -> u16 {
        self.year
    }
}
