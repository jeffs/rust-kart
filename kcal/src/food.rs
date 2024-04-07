use std::{fmt::Display, str::FromStr};

pub struct BadFood(String);

impl Display for BadFood {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: bad food", self.0)
    }
}

pub struct Food {
    pub description: &'static str,
    pub source: &'static str,
    /// kilocalories per 100g of this food
    pub kcal: f64,
    /// grams of protein per 100g of this food
    pub protein: f64,
}

impl FromStr for Food {
    type Err = BadFood;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "turkey" => Ok(Food {
                description: "Deli Sliced Turkey",
                source: "https://www.nutritionix.com/food/deli-sliced-turkey",
                kcal: 64.0 * (100.0 / 57.0),
                protein: 7.7 * (100.0 / 57.0),
            }),
            _ => Err(BadFood(s.to_string())),
        }
    }
}
