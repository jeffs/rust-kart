use std::fmt;
use std::ops::{RangeFrom, RangeInclusive};

use crate::week::{Day, DAYS};
use crate::{Error, Result};

const MONTHS: RangeInclusive<u8> = 1..=12;
const YEARS: RangeFrom<u16> = 1..;

#[allow(clippy::cast_possible_truncation)]
const DAYS_PER_WEEK: u8 = DAYS.len() as u8;

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

/// Returns the number of days between the first days of the year 1 and the specified year.
fn days_before_year(year: u16) -> usize {
    debug_assert!(YEARS.contains(&year));
    let years = usize::from(year - 1);
    years * 365       // Number of years, times days per year,
        + years / 4   // plus leap years, which are divisible by 4,
        - years / 100 // but not by 100, 
        + years / 400 // unless also divisible by 400.  I don't make the rules.
}

fn days_before_month(year: u16, month: u8) -> usize {
    debug_assert!(YEARS.contains(&year));
    debug_assert!(MONTHS.contains(&month));
    (1..month).map(|m| month_days(year, m).count()).sum()
}

/// Proleptic Gregorian calendar date.
#[derive(Clone, Copy, PartialEq, Eq)]
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

    /// Returns the number of days between the first day of the year 1 and this date.
    fn count_days(self) -> usize {
        let days = usize::from(self.day - 1);
        days_before_year(self.year) + days_before_month(self.year, self.month) + days
    }

    #[must_use]
    pub fn day_of_week(self) -> Day {
        // +1 because January 1st of the year 1 CE would have been a Monday.
        DAYS[(self.count_days() + 1) % DAYS.len()]
    }

    fn day_of_next_month(self, day: u8) -> Date {
        let month = self.month + 1;
        let (year, month) = if MONTHS.contains(&month) {
            (self.year, month)
        } else {
            (self.year + 1, 1)
        };
        Date { year, month, day }
    }

    /// Returns the date [one day later](https://www.youtube.com/watch?v=Ph1M0F99Xv8) than self.
    ///
    /// # Panics
    ///
    /// Will panic in debug mode only if the resulting date would exceed [`Date::MAX`].
    #[must_use]
    pub fn plus_one_day(self) -> Date {
        let day = self.day + 1;
        if month_days(self.year, self.month).contains(&day) {
            Date { day, ..self }
        } else {
            self.day_of_next_month(1)
        }
    }

    /// Returns the date [one week later](https://www.youtube.com/watch?v=BKP3Qe_zZ18) than self.
    ///
    /// # Panics
    ///
    /// Will panic in debug mode only if the resulting date would exceed [`Date::MAX`].
    #[must_use]
    pub fn plus_one_week(self) -> Date {
        let month_days = month_days(self.year, self.month);
        let day = self.day + DAYS_PER_WEEK;
        if month_days.contains(&day) {
            Date { day, ..self }
        } else {
            self.day_of_next_month(day - month_days.end())
        }
    }

    /// Constructs a date in the specified year, month, and day.  All three fields are 1-based.
    ///
    /// # Errors
    ///
    /// Returns [`Err`] if the date is invalid.  For example, you can't construct a nonsensical date
    /// like February 40th.
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
    pub fn from_ymd(year: u16, month: u8, day: u8) -> Result<Date> {
        (YEARS.contains(&year) && MONTHS.contains(&month) && month_days(year, month).contains(&day))
            .then_some(Date { year, month, day })
            .ok_or(Error::Date { year, month, day })
    }

    #[must_use]
    pub fn day(self) -> u8 {
        self.day
    }

    #[must_use]
    pub fn month(self) -> u8 {
        self.month
    }

    #[must_use]
    pub fn year(self) -> u16 {
        self.year
    }

    /// Returns true iff this date is February 29.
    #[must_use]
    pub fn is_leap_day(self) -> bool {
        self.month == 2 && self.day == 29
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Date { year, month, day } = self;
        write!(f, "{year:04}-{month:02}-{day:02}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_are_one_based() {
        assert!(Date::from_ymd(1, 1, 1).is_ok());
        assert!(Date::from_ymd(1, 1, 0).is_err());
        assert!(Date::from_ymd(1, 0, 1).is_err());
        assert!(Date::from_ymd(0, 1, 1).is_err());
    }

    #[test]
    fn day_of_week_works() {
        for (year, day) in [
            (1, Day::Mon), // 0001-01-01 would have been a Monday.
            (2, Day::Tue), // 365 % 7 == 1, so each year starts one week day later than the last.
            (3, Day::Wed),
            (4, Day::Thu),
            (5, Day::Sat), // 0004 would have been a leap year, pushing 0005 back an extra day.
        ] {
            let got = Date::from_ymd(year, 1, 1).map(Date::day_of_week);
            assert_eq!(got, Ok(day), "{year}-01-01");
        }
    }
}
