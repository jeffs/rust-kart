use std::{fmt, str::FromStr};

use crate::unit::Unit;

#[derive(Debug)]
pub enum BadPortion {
    BadAmount(String),
    BadUnit(String),
    MissingUnit,
}

impl fmt::Display for BadPortion {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        match self {
            BadPortion::BadAmount(value) => write!(f, "{value}: bad amount"),
            BadPortion::BadUnit(value) => write!(f, "{value}: bad unit"),
            BadPortion::MissingUnit => write!(f, "missing unit"),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Portion {
    pub number: f64,
    pub unit: Unit,
}

impl Portion {
    #[must_use]
    pub fn convert(&self) -> Portion {
        self.convert_to(self.unit.dual())
    }

    #[must_use]
    pub fn convert_to(&self, unit: Unit) -> Portion {
        Portion {
            number: self.number * unit.per(self.unit),
            unit,
        }
    }
}

impl fmt::Display for Portion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.unit {
            // There's no point in showing fractions of a gram.
            Unit::Gram => write!(f, "{}", self.number.round())?,
            _ => write!(f, "{:.2}", self.number)?,
        }
        write!(f, "{}", self.unit)
    }
}

impl FromStr for Portion {
    type Err = BadPortion;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let unit_begin = s
            .bytes()
            .position(|c| c != b'.' && !c.is_ascii_digit())
            .unwrap_or(s.len());
        let (number, unit) = s.split_at(unit_begin);
        let number = number
            .parse()
            .map_err(|_| BadPortion::BadAmount(number.to_string()))?;
        let unit = unit
            .parse()
            .map_err(|_| BadPortion::BadUnit(unit.to_string()))?;
        Ok(Portion { number, unit })
    }
}
