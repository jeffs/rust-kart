use std::{fmt, str::FromStr};

use crate::{Portion, Unit};

#[derive(Debug)]
pub struct BadFood(String);

impl fmt::Display for BadFood {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: bad food", self.0)
    }
}

#[derive(Clone)]
pub struct Food {
    /// kilocalories per 100g of this food
    pub kcal: f64,
    /// grams of protein per 100g of this food
    pub protein: f64,
}

macro_rules! food {
    ($name: ident, $kcal: expr, $protein: expr, $per: expr) => {
        (
            stringify!($name),
            Food {
                kcal: $kcal as f64 * 100.0 / $per as f64,
                protein: $protein as f64 * 100.0 / $per as f64,
            },
        )
    };
}

// TODO: Move to food.csv.
#[rustfmt::skip]
const FOODS: &[(&str, Food)] = &[
    food!(asparagus,       20,  2.2, 100),
    food!(avocado,         80,  1.0,  50),
    food!(banana,          89,  1.1, 100),
    food!(broccoli,        34,  2.8, 100),
    food!(brussels,        43,  3.4, 100),
    food!(butter,         717,  0.9, 100),
    food!(cheese,         110,  7.0,  28),
    food!(cabbage,         25,  1.3, 100),
    food!(carrot,          41,  0.8, 100),
    food!(cauliflower,     25,  1.9, 100),
    food!(celery,          14,  0.7, 100),
    food!(chicken,         60, 11.0,  56),
    food!(cucumber,        15,  0.6, 100),
    food!(eggwhite,        25,  5.0,  46),
    food!(endive,          17,  1.3, 100),
    food!(enoki,           44,  2.4, 100),
    food!(lettuce,         17,  1.0, 100),
    food!(mushroom,        22,  3.1, 100),
    food!(oil,            884,  0.0, 100),
    food!(onion,           41,  1.3, 100),
    food!(shallot,         72,  2.5, 100),
    food!(pepper,          20,  0.9, 100),
    food!(popcorn,        130,  4.0,  40),
    food!(potato,          79,  2.1, 100),
    food!(spinach,         23,  2.9, 100),
    food!(strawberry,      32,  0.7, 100),
    food!(sugar,          385,  0.0, 100),
    food!(thigh,          149, 18.6, 100),
    food!(tomato,          22,  0.7, 100),
    food!(turkey,          64,  7.7,  57),
    food!(veg,             20,  1.5,  57),
    food!(whiskey,        250,  0.0, 100),
];

/// Parses foods in the format C,P/Z.
/// * C is the number of kilocalories per serving
/// * P is the number of grams of protein per serving
/// * Z is the serving size
fn parse_custom(s: &str) -> Option<Food> {
    let (cp, z) = s.split_once('/')?;
    let (c, p) = cp.split_once(',')?;

    let kcal: f64 = c.parse().ok()?;
    let protein: f64 = p.parse().ok()?;
    let hundreds: f64 = z.parse::<Portion>().ok()?.convert_to(Unit::Gram).number / 100.0;

    Some(Food {
        kcal: kcal / hundreds,
        protein: protein / hundreds,
    })
}

impl FromStr for Food {
    type Err = BadFood;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(food) = FOODS
            .iter()
            .find_map(|(slug, food)| (*slug == s).then_some(food))
        {
            Ok(food.clone())
        } else if let Some(food) = parse_custom(s) {
            Ok(food)
        } else {
            Err(BadFood(s.to_string()))
        }
    }
}
