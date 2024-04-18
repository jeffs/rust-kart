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
    food!(broccoli,    34,  2.8, 100, "Broccoli, raw",                                "https://fdc.nal.usda.gov/fdc-app.html#/food-details/170379/nutrients"),
    food!(carrot,      41,  0.8, 100, "Carrots, baby, raw",                           "https://fdc.nal.usda.gov/fdc-app.html#/food-details/2258587/nutrients"),
    food!(cauliflower, 25,  1.9, 100, "Cauliflower, raw",                             "https://fdc.nal.usda.gov/fdc-app.html#/food-details/169986/nutrients"),
    food!(celery,      14,  0.7, 100, "Celery, raw",                                  "https://fdc.nal.usda.gov/fdc-app.html#/food-details/169988/nutrients"),
    food!(chicken,     60, 11.0,  56, "Blazing Buffalo Style Roasted Chicken Breast", "https://boarshead.com/products/detail/440-blazing-buffalo-style-roasted-chicken-breast"),
    food!(endive,      17,  1.3, 100, "Endive",                                       "https://fdc.nal.usda.gov/fdc-app.html#/food-details/168412/nutrients"),
    food!(enoki,       44,  2.4, 100, "Mushroom, enoki",                              "https://fdc.nal.usda.gov/fdc-app.html#/food-details/2003600/nutrients"),
    food!(pepper,      20,  0.9, 100, "Peppers, sweet, green, raw",                   "https://fdc.nal.usda.gov/fdc-app.html#/food-details/170427/nutrients"),
    food!(popcorn,    387, 12.9, 100, "Snacks, popcorn, air-popped",                  "https://fdc.nal.usda.gov/fdc-app.html#/food-details/167959/nutrients"),
    food!(tomato,      22,  0.7, 100, "Tomato, roma",                                 "https://fdc.nal.usda.gov/fdc-app.html#/food-details/1999634/nutrients"),
    food!(turkey,      64,  7.7,  57, "Deli Sliced Turkey",                           "https://www.nutritionix.com/food/deli-sliced-turkey"),
    food!(veg,         20,  1.5,  57, "Mixed vegetables",                             "rough average of squash, mushrooms, and asparagus"),
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
