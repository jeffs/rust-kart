use std::{fmt::Display, str::FromStr};

const GRAMS_PER_POUND: f64 = 453.592;
const OUNCES_PER_POUND: f64 = 16.0;

const GRAMS_PER_OUNCE: f64 = GRAMS_PER_POUND / OUNCES_PER_POUND;
const OUNCES_PER_GRAM: f64 = OUNCES_PER_POUND / GRAMS_PER_POUND;

#[derive(Debug)]
pub struct BadUnit;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Unit {
    #[default]
    Gram,
    Ounce,
    Pound,
}

impl Unit {
    #[must_use]
    pub fn dual(self) -> Unit {
        match self {
            Unit::Gram => Unit::Ounce,
            Unit::Ounce | Unit::Pound => Unit::Gram,
        }
    }

    /// TODO: Set this per item, so you can define `Unit::Each`.  For example,
    /// the number of grams per egg is not necessarily the same as the number of
    /// grams per banana.
    #[must_use]
    pub fn per(self, unit: Unit) -> f64 {
        match (self, unit) {
            (Unit::Gram, Unit::Gram) | (Unit::Ounce, Unit::Ounce) | (Unit::Pound, Unit::Pound) => {
                1.0
            }
            (Unit::Gram, Unit::Ounce) => GRAMS_PER_OUNCE,
            (Unit::Gram, Unit::Pound) => GRAMS_PER_POUND,
            (Unit::Ounce, Unit::Gram) => OUNCES_PER_GRAM,
            (Unit::Ounce, Unit::Pound) => OUNCES_PER_POUND,
            (Unit::Pound, Unit::Gram) => 1.0 / GRAMS_PER_POUND,
            (Unit::Pound, Unit::Ounce) => 1.0 / OUNCES_PER_POUND,
        }
    }
}

impl Display for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Unit::Gram => "g",
            Unit::Ounce => "oz",
            Unit::Pound => "lb",
        };
        write!(f, "{s}")
    }
}

impl FromStr for Unit {
    type Err = BadUnit;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "" => Ok(Unit::default()),
            "g" => Ok(Unit::Gram),
            "lb" | "lbs" | "#" => Ok(Unit::Pound),
            "oz" => Ok(Unit::Ounce),
            _ => Err(BadUnit),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unit_parsing() {
        assert_eq!("g".parse::<Unit>().unwrap(), Unit::Gram);
        assert_eq!("oz".parse::<Unit>().unwrap(), Unit::Ounce);
        assert_eq!("lb".parse::<Unit>().unwrap(), Unit::Pound);
        assert_eq!("lbs".parse::<Unit>().unwrap(), Unit::Pound);
        assert!("invalid".parse::<Unit>().is_err());
    }

    #[test]
    fn test_unit_display() {
        assert_eq!(Unit::Gram.to_string(), "g");
        assert_eq!(Unit::Ounce.to_string(), "oz");
        assert_eq!(Unit::Pound.to_string(), "lb");
    }

    #[test]
    fn test_unit_conversion() {
        // 1 oz should be ~28.35g
        let grams_per_oz = Unit::Gram.per(Unit::Ounce);
        assert!((grams_per_oz - 28.35).abs() < 0.1);

        // 1 lb should be ~453.6g
        let grams_per_lb = Unit::Gram.per(Unit::Pound);
        assert!((grams_per_lb - 453.6).abs() < 0.1);

        // Identity conversions
        assert_eq!(Unit::Gram.per(Unit::Gram), 1.0);
    }
}
