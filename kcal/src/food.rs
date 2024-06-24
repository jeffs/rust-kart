use std::{fmt::Display, str::FromStr};

use crate::{Portion, Unit};

pub struct BadFood(String);

impl Display for BadFood {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: bad food", self.0)
    }
}

#[derive(Clone)]
pub struct Food {
    pub description: &'static str,
    pub source: &'static str,
    /// kilocalories per 100g of this food
    pub kcal: f64,
    /// grams of protein per 100g of this food
    pub protein: f64,
}

macro_rules! food {
    ($name: ident, $kcal: expr, $protein: expr, $per: expr, $description: expr, $source: expr) => {
        (
            stringify!($name),
            Food {
                description: $description,
                kcal: $kcal as f64 * 100.0 / $per as f64,
                protein: $protein as f64 * 100.0 / $per as f64,
                source: $source,
            },
        )
    };
}

#[rustfmt::skip]
const FOODS: &[(&str, Food)] = &[
    food!(broccoli,    34,  2.8, 100, "Broccoli, raw",                                "https://fdc.nal.usda.gov/fdc-app.html#/food-details/170379/nutrients"),
    food!(cabbage,     25,  1.3, 100, "Cabbage, raw",                                 "https://fdc.nal.usda.gov/fdc-app.html#/food-details/169975/nutrients"),
    food!(carrot,      41,  0.8, 100, "Carrots, baby, raw",                           "https://fdc.nal.usda.gov/fdc-app.html#/food-details/2258587/nutrients"),
    food!(cauliflower, 25,  1.9, 100, "Cauliflower, raw",                             "https://fdc.nal.usda.gov/fdc-app.html#/food-details/169986/nutrients"),
    food!(celery,      14,  0.7, 100, "Celery, raw",                                  "https://fdc.nal.usda.gov/fdc-app.html#/food-details/169988/nutrients"),
    food!(chicken,     60, 11.0,  56, "Blazing Buffalo Style Roasted Chicken Breast", "https://boarshead.com/products/detail/440-blazing-buffalo-style-roasted-chicken-breast"),
    food!(cucumber,    15,  0.6, 100, "Cucumber, with peel, raw",                     "https://fdc.nal.usda.gov/fdc-app.html#/food-details/168409/nutrients"),
    food!(eggwhite,    25,  5.0,  46, "Egg whites",                                   "label"),
    food!(endive,      17,  1.3, 100, "Endive",                                       "https://fdc.nal.usda.gov/fdc-app.html#/food-details/168412/nutrients"),
    food!(enoki,       44,  2.4, 100, "Mushroom, enoki",                              "https://fdc.nal.usda.gov/fdc-app.html#/food-details/2003600/nutrients"),
    food!(mushroom,    22,  3.1, 100, "Mushrooms, white, raw",                        "https://fdc.nal.usda.gov/fdc-app.html#/food-details/169251/nutrients"),
    food!(oil,        884,  0.0, 100, "Oil, olive, salad or cooking",                 "https://fdc.nal.usda.gov/fdc-app.html#/food-details/171413/nutrients"),
    food!(onion,       41,  1.3, 100, "Red Onion",                                    "https://www.nutritionix.com/food/red-onion"),
    food!(pepper,      20,  0.9, 100, "Peppers, sweet, green, raw",                   "https://fdc.nal.usda.gov/fdc-app.html#/food-details/170427/nutrients"),
    food!(popcorn,    100,  4.0,  27, "Popcorn",                                      "label"),
    food!(popcorn_usda, 387, 12.9, 100, "Snacks, popcorn, air-popped",                  "https://fdc.nal.usda.gov/fdc-app.html#/food-details/167959/nutrients"),
    food!(spinach,     23,  2.9, 100, "Spinach, raw",                                 "https://fdc.nal.usda.gov/fdc-app.html#/food-details/168462/nutrients"),
    food!(thigh,      149, 18.6, 100, "Chicken, thigh, boneless, skinless, raw",      "https://fdc.nal.usda.gov/fdc-app.html#/food-details/2646171/nutrients"),
    food!(tomato,      22,  0.7, 100, "Tomato, roma",                                 "https://fdc.nal.usda.gov/fdc-app.html#/food-details/1999634/nutrients"),
    food!(turkey,      64,  7.7,  57, "Deli Sliced Turkey",                           "https://www.nutritionix.com/food/deli-sliced-turkey"),
    food!(veg,         20,  1.5,  57, "Mixed vegetables",                             "rough average of squash, mushrooms, and asparagus"),
    food!(whiskey,    250,  0.0, 100, "Whisky",                                       "Google"),
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
    let hundreds: f64 = z
        .parse::<Portion>()
        .ok()?
        .convert_to(Unit::Gram)
        .ok()?
        .number
        / 100.0;

    Some(Food {
        description: "ad hoc",
        source: "user",
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
