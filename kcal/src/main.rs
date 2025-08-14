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
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Food(e) => e.fmt(f),
            Error::Portion(e) => e.fmt(f),
            Error::Arg1(arg) => write!(f, "{arg}: expected food or portion size"),
            Error::Usage => f.write_str(USAGE),
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

enum RawArgs {
    /// The user asked for help.
    Help,
    /// The user passed exactly one or two arguments.
    Args(String, Option<String>),
}

fn args() -> Result<RawArgs> {
    match &mut env::args().skip(1).collect::<Vec<_>>()[..] {
        [first, ..] if matches!(first.as_str(), "-h" | "--help") => Ok(RawArgs::Help),
        [first] => Ok(RawArgs::Args(take(first), None)),
        [first, second] => Ok(RawArgs::Args(take(first), Some(take(second)))),
        _ => Err(Error::Usage),
    }
}

enum Args {
    Help,
    Size(Portion),
    Food(Food),
    Both(Portion, Food),
}

impl Args {
    fn from_env() -> Result<Args> {
        let RawArgs::Args(arg1, arg2) = args()? else {
            return Ok(Args::Help);
        };
        if let Ok(size) = arg1.parse::<Portion>() {
            Ok(if let Some(food) = arg2 {
                Args::Both(size, food.parse()?)
            } else {
                Args::Size(size)
            })
        } else if let Ok(food) = arg1.parse::<Food>() {
            Ok(if let Some(size) = arg2 {
                Args::Both(size.parse()?, food)
            } else {
                Args::Food(food)
            })
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
    match Args::from_env()? {
        Args::Help => {
            println!("{USAGE}");
        }
        Args::Size(size) => {
            // Convert to the most common other unit in the same dimension.
            println!("{size} = {}", size.convert());
        }
        Args::Food(food) => {
            // Print kcal and protein per 100g of the specified food.
            println!("{} {}", food.kcal.round(), food.protein.round());
        }
        Args::Both(size, food) => {
            let (kcal, protein) = scale(size, &food);
            println!("{kcal} {protein}");
        }
    }
    Ok(())
}

fn main() {
    if let Err(err) = main_imp() {
        if !matches!(err, Error::Usage) {
            eprint!("Error: ");
        }
        eprintln!("{err}");
        exit(2);
    }
}
