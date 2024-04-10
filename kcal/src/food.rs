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
    food!(carrot,  41,  0.8, 100, "Carrots, baby, raw",                           "https://fdc.nal.usda.gov/fdc-app.html#/food-details/2258587/nutrients"),
    food!(celery,  14,  0.7, 100, "Celery, raw",                                  "https://fdc.nal.usda.gov/fdc-app.html#/food-details/169988/nutrients"),
    food!(chicken, 60, 11.0,  56, "Blazing Buffalo Style Roasted Chicken Breast", "https://boarshead.com/products/detail/440-blazing-buffalo-style-roasted-chicken-breast"),
    food!(turkey,  64,  7.7,  57, "Deli Sliced Turkey",                           "https://www.nutritionix.com/food/deli-sliced-turkey"),
    food!(veg,     20,  1.5,  57, "mixed vegetables",                             "rough average of squash, mushrooms, and asparagus"),
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
