use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{env, fmt, fs, io};

#[derive(Debug)]
struct DbErrorLocation {
    file: PathBuf,
    line: Option<usize>,
}

impl fmt::Display for DbErrorLocation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.file.display().fmt(f)?;
        if let Some(line) = self.line {
            write!(f, ":{line}")?;
        }
        Ok(())
    }
}

#[derive(Debug)]
enum DbErrorKind {
    Io(io::Error),
    Syntax,
    Duplicate(String),
    Arg(String),
}

impl fmt::Display for DbErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Io(e) => e.fmt(f),
            Self::Syntax => write!(f, "syntax error"),
            Self::Duplicate(s) => write!(f, "duplicate entry for {s}"),
            Self::Arg(s) => write!(f, "no such target: {s}"),
        }
    }
}

#[derive(Debug)]
struct DbError {
    location: DbErrorLocation,
    kind: DbErrorKind,
}

impl DbError {
    fn new(location: DbErrorLocation, kind: DbErrorKind) -> Self {
        Self { location, kind }
    }

    fn io(file: PathBuf, cause: io::Error) -> Self {
        let location = DbErrorLocation { file, line: None };
        Self::new(location, DbErrorKind::Io(cause))
    }

    fn syntax(location: DbErrorLocation) -> Self {
        Self::new(location, DbErrorKind::Syntax)
    }

    fn duplicate(location: DbErrorLocation, name: String) -> Self {
        Self::new(location, DbErrorKind::Duplicate(name))
    }

    fn arg(file: PathBuf, arg: String) -> Self {
        let location = DbErrorLocation { file, line: None };
        Self::new(location, DbErrorKind::Arg(arg))
    }
}

impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.location, self.kind)
    }
}

#[derive(Debug)]
enum Error {
    Db(DbError),
}

impl From<DbError> for Error {
    fn from(e: DbError) -> Self {
        Self::Db(e)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Db(e) => e.fmt(f),
        }
    }
}

#[allow(dead_code)]
pub struct Database(
    /// Maps jump target names to directory paths.
    HashMap<String, PathBuf>,
);

// struct

impl Database {
    fn read_file(path: impl AsRef<Path>) -> Result<Self, DbError> {
        let path = path.as_ref();
        let file = fs::read_to_string(path).map_err(|e| DbError::io(path.into(), e))?;

        let mut jumps = HashMap::new();

        for (index, line) in file.lines().enumerate() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let location = || DbErrorLocation {
                file: path.to_path_buf(),
                line: Some(index + 1),
            };

            let (dir, names) = line
                .split_once(',')
                .ok_or_else(|| DbError::syntax(location()))?;

            for name in names.split(',') {
                if jumps.insert(name.into(), dir.into()).is_some() {
                    return Err(DbError::duplicate(location(), name.into()));
                }
            }
        }

        Ok(Database(jumps))
    }
}

/// # Notes
///
/// Reads config from `~/.config/jump/targets.csv`, where `~` is returned by [`std::env::home_dir`].
/// That function is deprecated because it behaved inconsistently on Windows before Rust 1.85, but
/// it does what we want here.
///
/// The `targets.csv` file supports blank lines, comment lines (beginning with `#`), and jagged
/// lines.  The first column in each row is a directory path, and all subsequent columns are short
/// names for that path.
///
/// # TODO
///
/// Support database file path specfication via environment variables.
fn main_imp() -> Result<(), DbError> {
    #[allow(deprecated)]
    let home = env::home_dir().expect("user should have a home directory");
    let db_path = home.join(".config/jump/targets.csv");

    let db = Database::read_file(&db_path)?;
    for arg in env::args().skip(1) {
        let Some(path) = db.0.get(&arg) else {
            return Err(DbError::arg(db_path, arg));
        };

        if let Ok(tail) = path.strip_prefix("~") {
            // Expand leading `~`.
            println!("{}", home.join(tail).display())
        } else {
            println!("{}", path.display());
        }
    }
    Ok(())
}

fn main() {
    if let Err(e) = main_imp() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
