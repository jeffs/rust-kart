use std::{env, process::exit};

use kcal::{BadFood, BadPortion, Food, PortionSize, Unit};

const USAGE: &str = "usage: convert SIZE [FOOD]";

struct Args {
    size: PortionSize,
    food: Option<Food>,
}

impl Args {
    fn from_env() -> Result<Args, String> {
        let mut args = env::args().skip(1);
        let size = args.next().ok_or(USAGE)?;
        let food = if let Some(food) = args.next() {
            if let Some(extra) = args.next() {
                return Err(format!("{extra}: unexpected argument"));
            }
            Some(food.parse().map_err(|err: BadFood| err.to_string())?)
        } else {
            None
        };

        Ok(Args {
            size: size.parse().map_err(|err: BadPortion| err.to_string())?,
            food,
        })
    }
}

fn main_imp() -> Result<(), String> {
    let args = Args::from_env()?;
    if let Some(food) = args.food {
        let amount = args
            .size
            .convert_to(Unit::Gram)
            .map_err(|err| err.to_string())?;
        let [kcal, protein] =
            [food.kcal, food.protein].map(|f| (f * amount.number / 100.0).round());
        println!("{kcal} {protein}")
    } else {
        let size = args.size.convert().map_err(|err| err.to_string())?;
        println!("{} = {size}", args.size);
    }
    Ok(())
}

fn main() {
    if let Err(err) = main_imp() {
        eprintln!("error: {err}");
        exit(2);
    }
}
