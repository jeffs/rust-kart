use std::{fmt::Display, str::FromStr};

pub struct BadFood(String);

impl Display for BadFood {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: bad food", self.0)
    }
}

pub struct FoodDescriptor {
    pub description: &'static str,
    pub source: &'static str,
    /// kilocalories per 100g of this food
    pub kcal: f64,
    /// grams of protein per 100g of this food
    pub protein: f64,
}

pub type Food = &'static FoodDescriptor;

macro_rules! food {
    ($name: ident, $kcal: expr, $protein: expr, $per: expr, $description: expr, $source: expr) => {
        (
            stringify!($name),
            FoodDescriptor {
                description: $description,
                kcal: $kcal as f64 * 100.0 / $per as f64,
                protein: $protein as f64 * 100.0 / $per as f64,
                source: $source,
            },
        )
    };
}

#[rustfmt::skip]
const FOODS: &[(&str, FoodDescriptor)] = &[
    food!(turkey, 64, 7.7, 57, "Deli Sliced Turkey", "https://www.nutritionix.com/food/deli-sliced-turkey"),
];

impl FromStr for Food {
    type Err = BadFood;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        FOODS
            .iter()
            .find_map(|(slug, food)| (*slug == s).then_some(food))
            .ok_or_else(|| BadFood(s.to_string()))
    }
}
