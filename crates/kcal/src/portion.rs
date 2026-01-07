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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_portion_parsing() {
        let p: Portion = "100g".parse().unwrap();
        assert_eq!(p.number, 100.0);
        assert_eq!(p.unit, Unit::Gram);

        let p: Portion = "8oz".parse().unwrap();
        assert_eq!(p.number, 8.0);
        assert_eq!(p.unit, Unit::Ounce);

        let p: Portion = "1.5lb".parse().unwrap();
        assert_eq!(p.number, 1.5);
        assert_eq!(p.unit, Unit::Pound);
    }

    #[test]
    fn test_portion_conversion() {
        let p: Portion = "1oz".parse().unwrap();
        let grams = p.convert_to(Unit::Gram);
        assert!((grams.number - 28.35).abs() < 0.1);
        assert_eq!(grams.unit, Unit::Gram);
    }

    #[test]
    fn test_portion_display() {
        let p = Portion { number: 100.0, unit: Unit::Gram };
        assert_eq!(p.to_string(), "100g");
    }
}
