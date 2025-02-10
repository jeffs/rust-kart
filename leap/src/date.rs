use std::ops::{RangeFrom, RangeInclusive};
use std::{fmt, u16};

use crate::{Error, Result};

const DAYS_PER_WEEK: u8 = 7;
const MONTHS: RangeInclusive<u8> = 1..=12;
const YEARS: RangeFrom<u16> = 1..;

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
    /// December 31st, tens of thousands of years from the day you're reading this.
    pub const MAX: Date = Date {
        year: u16::MAX,
        month: 12,
        day: 31,
    };

    /// Returns the date [one week later](https://www.youtube.com/watch?v=BKP3Qe_zZ18).
    ///
    /// # Panics
    ///
    /// Will panic in debug mode only if the resulting date would exceed [`Date::MAX`].
    pub fn plus_one_week(self) -> Date {
        let month_days = month_days(self.year, self.month);
        let day = self.day + DAYS_PER_WEEK;
        if month_days.contains(&day) {
            return Date { day, ..self };
        }

        let day = day - month_days.end();
        let month = self.month + 1;
        if MONTHS.contains(&month) {
            return Date { month, day, ..self };
        }

        Date {
            year: self.year + 1,
            month: 1,
            day,
        }
    }

    pub fn day(self) -> u8 {
        self.day
    }

    /// Constructs a date in the specified year CE, on the specified 1-based month and day indexes.
    ///
    /// # Examples
    ///
    /// ```
    /// use leap::Date;
    ///
    /// assert_eq!(
    ///     Date::from_ymd(2000, 1, 1)
    ///         .expect("January 1st, 2000")
    ///         .to_string(),
    ///     "2000-01-01"
    /// );
    /// ```
    #[must_use]
    pub fn from_ymd(year: u16, month: u8, day: u8) -> Result<Date> {
        (YEARS.contains(&year) && MONTHS.contains(&month) && month_days(year, month).contains(&day))
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

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Date { year, month, day } = self;
        write!(f, "{year:04}-{month:02}-{day:02}")
    }
}
