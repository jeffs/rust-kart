use std::{fmt::Display, str::FromStr};

use crate::unit::{BadConversion, Unit};

#[derive(Debug)]
pub enum BadPortion {
    BadAmount(String),
    BadUnit(String),
    MissingUnit,
}

impl Display for BadPortion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BadPortion::BadAmount(value) => write!(f, "{value}: bad amount"),
            BadPortion::BadUnit(value) => write!(f, "{value}: bad unit"),
            BadPortion::MissingUnit => write!(f, "missing unit"),
        }
    }
}

pub struct Portion {
    pub number: f64,
    pub unit: Unit,
}

impl Portion {
    pub fn convert(&self) -> Result<Portion, BadConversion> {
        self.convert_to(self.unit.dual())
    }

    pub fn convert_to(&self, unit: Unit) -> Result<Portion, BadConversion> {
        let amount = self.number * unit.per(self.unit)?;
        Ok(Portion {
            number: amount,
            unit,
        })
    }
}

impl Display for Portion {
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
            .find(|c: char| c != '.' && !c.is_ascii_digit())
            .ok_or(BadPortion::MissingUnit)?;
        let (amount, unit) = s.split_at(unit_begin);
        let amount = amount
            .parse()
            .map_err(|_| BadPortion::BadAmount(amount.to_string()))?;
        let unit = unit
            .parse()
            .map_err(|_| BadPortion::BadUnit(unit.to_string()))?;
        Ok(Portion {
            number: amount,
            unit,
        })
    }
}
