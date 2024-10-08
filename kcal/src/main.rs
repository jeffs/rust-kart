use std::{env, process::exit};

use kcal::{BadFood, BadPortion, Food, Portion, Unit};

const USAGE: &str = "usage: convert SIZE [FOOD]";

#[allow(clippy::similar_names)] // args, arg1, arg2, arg3
fn args() -> Result<(String, Option<String>), String> {
    let mut args = env::args().skip(1);
    let arg1 = args.next().ok_or(USAGE)?;
    let arg2 = args.next();
    if let Some(arg3) = args.next() {
        return Err(format!("{arg3}: unexpected argument"));
    }
    Ok((arg1, arg2))
}

// TODO: Support SIZE/SIZExKCAL,G for one-off foods.
// TODO: Support creation and storage of new foods from the CLI.
struct Args {
    size: Option<Portion>,
    food: Option<Food>,
}

impl Args {
    fn from_env() -> Result<Args, String> {
        let (arg1, arg2) = args()?;
        if let Ok(size) = arg1.parse::<Portion>() {
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

fn scale(size: Portion, food: &Food) -> (f64, f64) {
    let grams = size.convert_to(Unit::Gram);
    let [kcal, protein] = [food.kcal, food.protein].map(|f| (f * grams.number / 100.0).round());
    (kcal, protein)
}

fn main_imp() -> Result<(), String> {
    let args = Args::from_env()?;
    match (args.size, args.food) {
        (Some(size), Some(food)) => {
            let (kcal, protein) = scale(size, &food);
            println!("{kcal} {protein}");
        }
        (Some(size), None) => {
            let converted = size.convert();
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
