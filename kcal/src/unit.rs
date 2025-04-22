use std::{fmt::Display, str::FromStr};

const GRAMS_PER_POUND: f64 = 453.592;
const OUNCES_PER_POUND: f64 = 16.0;

const GRAMS_PER_OUNCE: f64 = GRAMS_PER_POUND / OUNCES_PER_POUND;
const OUNCES_PER_GRAM: f64 = OUNCES_PER_POUND / GRAMS_PER_POUND;

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
