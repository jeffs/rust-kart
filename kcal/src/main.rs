//! Prints kilocalories and grams of protein in specified food portions.
//!
//! # TODO
//!
//! * Support refinement within each food
//!   - e.g., "cheese/brie" or "apple/red-delicious"
//! * Support help on specific topics; e.g., `kcal --help units`
//! * Support creation and storage of new foods from the CLI.

use std::{env, process::exit};
use std::{fmt, mem::take};

use kcal::{BadFood, BadPortion, Food, Portion, Unit};

const USAGE: &str = "Usage:

    kcal {FOOD SIZE | SIZE FOOD}    # show calories and protein
    kcal FOOD                       # show calories and protein per 100g
    kcal SIZE                       # convert to common alternative unit
";

#[derive(Debug)]
enum Error {
    /// The food was not recognized.
    Food(BadFood),
    /// The portion size could not be parsed.
    Portion(BadPortion),
    /// The first argument could not be parsed.
    Arg1(String),
    /// The command was called incorrectly.
    Usage,
    /// The user passed a flag requesting help, so any other args are moot.
    Help,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Food(e) => e.fmt(f),
            Error::Portion(e) => e.fmt(f),
            Error::Arg1(arg) => write!(f, "{arg}: expected food or portion size"),
            Error::Help | Error::Usage => f.write_str(USAGE),
        }
    }
}

impl From<BadFood> for Error {
    fn from(value: BadFood) -> Self {
        Error::Food(value)
    }
}

impl From<BadPortion> for Error {
    fn from(value: BadPortion) -> Self {
        Error::Portion(value)
    }
}

type Result<T> = std::result::Result<T, Error>;

fn args() -> Result<(String, Option<String>)> {
    match &mut env::args().skip(1).collect::<Vec<_>>()[..] {
        [first, ..] if matches!(first.as_str(), "-h" | "--help") => Err(Error::Help),
        [first] => Ok((take(first), None)),
        [first, second] => Ok((take(first), Some(take(second)))),
        _ => Err(Error::Usage),
    }
}

struct Args {
    size: Option<Portion>,
    food: Option<Food>,
}

impl Args {
    fn from_env() -> Result<Args> {
        let (arg1, arg2) = args()?;
        if let Ok(size) = arg1.parse::<Portion>() {
            if let Some(food) = arg2 {
                let food = food.parse()?;
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
                let size = size.parse()?;
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
            Err(Error::Arg1(arg1))
        }
    }
}

fn scale(size: Portion, food: &Food) -> (f64, f64) {
    let grams = size.convert_to(Unit::Gram);
    let [kcal, protein] = [food.kcal, food.protein].map(|f| (f * grams.number / 100.0).round());
    (kcal, protein)
}

fn main_imp() -> Result<()> {
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
        if let Error::Help = err {
            println!("{err}");
            exit(0);
        }
        if !matches!(err, Error::Usage) {
            eprint!("error: ");
        }
        eprintln!("{err}");
        exit(2);
    }
}
