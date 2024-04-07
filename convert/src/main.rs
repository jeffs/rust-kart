use std::{env, process::exit};

use convert::{BadPortion, PortionSize};

const USAGE: &str = "usage: convert SIZE [FOOD]";

struct Args {
    size: PortionSize,
    // food: Option<Food>,
}

impl Args {
    fn from_env() -> Result<Args, String> {
        let mut args = env::args().skip(1);
        let size = args.next().ok_or(USAGE)?;
        // let _food = args.next().ok_or(USAGE)?;
        if let Some(arg) = args.next() {
            return Err(format!("{arg}: unexpected argument"));
        }
        Ok(Args {
            size: size.parse().map_err(|err: BadPortion| err.to_string())?,
        })
    }
}

fn main_imp() -> Result<(), String> {
    let args = Args::from_env()?;
    let size = args.size.convert().map_err(|err| err.to_string())?;
    println!("{} = {size}", args.size);
    Ok(())
}

fn main() {
    if let Err(err) = main_imp() {
        eprintln!("error: {err}");
        exit(2);
    }
}
