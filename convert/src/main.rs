use std::{env, fmt::Display, process::exit, str::FromStr};

const GRAMS_PER_POUND: f64 = 453.592;
const OUNCES_PER_POUND: f64 = 16.0;

const GRAMS_PER_OUNCE: f64 = GRAMS_PER_POUND / OUNCES_PER_POUND;
const OUNCES_PER_GRAM: f64 = OUNCES_PER_POUND / GRAMS_PER_POUND;

struct BadConversion;

impl Display for BadConversion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "incompatible units")
    }
}

struct BadUnit;

#[derive(Clone, Copy, Eq, PartialEq)]
enum Unit {
    Gram,
    Ounce,
    Pound,
}

impl Unit {
    fn dual(self) -> Unit {
        match self {
            Unit::Gram => Unit::Ounce,
            Unit::Ounce => Unit::Gram,
            Unit::Pound => Unit::Gram,
        }
    }

    fn per(self, unit: Unit) -> Result<f64, BadConversion> {
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

// struct BadAmount;

#[derive(Debug)]
enum BadPortion {
    BadAmount,
    BadUnit,
}

impl Display for BadPortion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            BadPortion::BadAmount => "bad amount",
            BadPortion::BadUnit => "bad unit",
        };
        write!(f, "{s}")
    }
}

struct Portion {
    amount: f64,
    unit: Unit,
}

impl Portion {
    fn convert_to(self, unit: Unit) -> Result<Portion, BadConversion> {
        let amount = self.amount * self.unit.per(unit)?;
        Ok(Portion { amount, unit })
    }
}

impl Display for Portion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.2}{}", self.amount, self.unit)
    }
}

impl FromStr for Portion {
    type Err = BadPortion;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some(unit_begin) = s.find(|c: char| !c.is_ascii_digit()) else {
            //  Missing unit entirely.
            return Err(BadPortion::BadUnit);
        };
        let (amount, unit) = s.split_at(unit_begin);
        let Ok(amount) = amount.parse() else {
            return Err(BadPortion::BadAmount);
        };
        let Ok(unit) = unit.parse() else {
            return Err(BadPortion::BadUnit);
        };
        Ok(Portion { amount, unit })
    }
}

fn main() {
    for arg in env::args().skip(1) {
        let input = arg.parse::<Portion>().unwrap_or_else(|err| {
            eprintln!("error: {arg}: {err}");
            exit(2);
        });

        let dual = input.unit.dual();
        let output = input.convert_to(dual).unwrap_or_else(|err| {
            eprintln!("error: {arg}: can't convert to {dual}: {err}");
            exit(2);
        });
        println!("{output}");
    }
}
