use std::fmt;
use std::{env, process::exit};

use kcal::{BadFood, BadPortion, Food, Portion, Unit};

const USAGE: &str = "Usage:

    kcal {FOOD SIZE | SIZE FOOD}    # show calories and protein
    kcal FOOD                       # show calories and protein per 100g
    kcal SIZE                       # convert to common alternative unit
";

#[derive(Debug)]
enum Error {
    Food(BadFood),
    Portion(BadPortion),
    /// The first argument could not be parsed.
    Arg1(String),
    /// A third argument was received, but we support only one or two.
    Arg3(String),
    Usage,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Food(e) => e.fmt(f),
            Error::Portion(e) => e.fmt(f),
            Error::Arg1(arg) => write!(f, "{arg}: expected food or portion size"),
            Error::Arg3(arg) => write!(f, "{arg}: unexpected argument"),
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

#[allow(clippy::similar_names)] // args, arg1, arg2, arg3
fn args() -> Result<(String, Option<String>)> {
    let mut args = env::args().skip(1);
    let arg1 = args.next().ok_or(Error::Usage)?;
    let arg2 = args.next();
    if let Some(arg3) = args.next() {
        return Err(Error::Arg3(arg3));
    }
    Ok((arg1, arg2))
}

// TODO: Support creation and storage of new foods from the CLI.
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
        if !matches!(err, Error::Usage) {
            eprint!("error: ");
        }
        eprintln!("{err}");
        exit(2);
    }
}
