use std::{env, process::exit};

use kcal::{BadConversion, BadFood, BadPortion, Food, PortionSize, Unit};

const USAGE: &str = "usage: convert SIZE [FOOD]";

struct Args {
    size: Option<PortionSize>,
    food: Option<Food>,
}

impl Args {
    fn from_env() -> Result<Args, String> {
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
}

fn scale(size: PortionSize, food: Food) -> Result<(f64, f64), BadConversion> {
    let grams = size.convert_to(Unit::Gram)?;
    let [kcal, protein] = [food.kcal, food.protein].map(|f| (f * grams.number / 100.0).round());
    Ok((kcal, protein))
}

fn main_imp() -> Result<(), String> {
    let args = Args::from_env()?;
    match (args.size, args.food) {
        (Some(size), Some(food)) => {
            let (kcal, protein) = scale(size, food).map_err(|err| err.to_string())?;
            println!("{kcal} {protein}");
        }
        (Some(size), None) => {
            let converted = size.convert().map_err(|err| err.to_string())?;
            println!("{size} = {converted}",);
        }
        (None, Some(food)) => {
            // Print kcal and protein per 100g of the specified food.
            println!("{} {}", food.kcal.round(), food.protein.round());
        }
        (None, None) => {
            // Args::from_env returns an error if no args were supplied.
            unreachable!();
        }
    }
    Ok(())
}

fn main() {
    if let Err(err) = main_imp() {
        eprintln!("error: {err}");
        exit(2);
    }
}
