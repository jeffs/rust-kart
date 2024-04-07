use std::{fmt::Display, str::FromStr};

const GRAMS_PER_POUND: f64 = 453.592;
const OUNCES_PER_POUND: f64 = 16.0;

const GRAMS_PER_OUNCE: f64 = GRAMS_PER_POUND / OUNCES_PER_POUND;
const OUNCES_PER_GRAM: f64 = OUNCES_PER_POUND / GRAMS_PER_POUND;

pub struct BadConversion;

impl Display for BadConversion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "incompatible units")
    }
}

pub struct BadUnit;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Unit {
    Gram,
    Ounce,
    Pound,
}

impl Unit {
    pub fn dual(self) -> Unit {
        match self {
            Unit::Gram => Unit::Ounce,
            Unit::Ounce => Unit::Gram,
            Unit::Pound => Unit::Gram,
        }
    }

    pub fn per(self, unit: Unit) -> Result<f64, BadConversion> {
        match (self, unit) {
            _ if self == unit => Ok(1.0),
            (Unit::Gram, Unit::Ounce) => Ok(GRAMS_PER_OUNCE),
            (Unit::Gram, Unit::Pound) => Ok(GRAMS_PER_POUND),
            (Unit::Ounce, Unit::Gram) => Ok(OUNCES_PER_GRAM),
            (Unit::Ounce, Unit::Pound) => Ok(OUNCES_PER_POUND),
            (Unit::Pound, Unit::Gram) => Ok(1.0 / GRAMS_PER_POUND),
            (Unit::Pound, Unit::Ounce) => Ok(1.0 / OUNCES_PER_POUND),

            _ => Err(BadConversion),
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
            "g" => Ok(Unit::Gram),
            "lb" | "lbs" | "#" => Ok(Unit::Pound),
            "oz" => Ok(Unit::Ounce),
            _ => Err(BadUnit),
        }
    }
}
