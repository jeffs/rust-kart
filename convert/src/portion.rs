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

pub struct PortionSize {
    amount: f64,
    unit: Unit,
}

impl PortionSize {
    pub fn convert(&self) -> Result<PortionSize, BadConversion> {
        self.convert_to(self.unit.dual())
    }

    fn convert_to(&self, unit: Unit) -> Result<PortionSize, BadConversion> {
        let amount = self.amount * unit.per(self.unit)?;
        Ok(PortionSize { amount, unit })
    }
}

impl Display for PortionSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.unit {
            Unit::Gram => write!(f, "{}", self.amount.round())?,
            _ => write!(f, "{:.2}", self.amount)?,
        }
        write!(f, "{}", self.unit)
    }
}

impl FromStr for PortionSize {
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
        Ok(PortionSize { amount, unit })
    }
}
