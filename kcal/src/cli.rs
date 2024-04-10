use std::env;

use crate::{BadConversion, BadFood, BadPortion, Food, PortionSize, Unit};

const USAGE: &str = "usage: convert SIZE [FOOD]";

pub struct Args {
    pub size: Option<PortionSize>,
    pub food: Option<Food>,
}

impl Args {
    pub fn from_env() -> Result<Args, String> {
        let mut args = env::args().skip(1);
        let arg1 = args.next().ok_or(USAGE)?;
        let arg2 = args.next();
        if let Some(extra) = args.next() {
            return Err(format!("{extra}: unexpected argument"));
        }

        if let Ok(size) = arg1.parse::<PortionSize>() {
            if let Some(food) = arg2 {
                let food = food.parse().map_err(|err: BadFood| err.to_string())?;
                Ok(Args {
                    size: Some(size),
                    food: Some(food),
                })
            } else {
                Ok(Args {
                    size: Some(size),
                    food: None,
                })
            }
        } else if let Ok(food) = arg1.parse::<Food>() {
            if let Some(size) = arg2 {
                let size = size.parse().map_err(|err: BadPortion| err.to_string())?;
                Ok(Args {
                    size: Some(size),
                    food: Some(food),
                })
            } else {
                Ok(Args {
                    size: None,
                    food: Some(food),
                })
            }
        } else {
            Err(format!("{arg1}: expected food or portion size"))
        }
    }

    pub fn main(self) -> Result<String, String> {
        Ok(match (self.size, self.food) {
            (Some(size), Some(food)) => {
                let (kcal, protein) = scale(size, food).map_err(|err| err.to_string())?;
                format!("{kcal} {protein}")
            }
            (Some(size), None) => {
                let converted = size.convert().map_err(|err| err.to_string())?;
                format!("{size} = {converted}")
            }
            (None, Some(food)) => {
                // Show kcal and protein per 100g of the specified food.
                format!("{} {}", food.kcal.round(), food.protein.round())
            }
            (None, None) => {
                // Args::from_env returns an error if no args were supplied.
                unreachable!()
            }
        })
    }
}

fn scale(size: PortionSize, food: Food) -> Result<(f64, f64), BadConversion> {
    let grams = size.convert_to(Unit::Gram)?;
    let [kcal, protein] = [food.kcal, food.protein].map(|f| (f * grams.number / 100.0).round());
    Ok((kcal, protein))
}
