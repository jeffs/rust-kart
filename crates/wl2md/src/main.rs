// TODO: Report CLI errors; support --help.

use core::fmt;
use std::fmt::Display;
use std::fs::File;
use std::io::prelude::*;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::str::FromStr;
use std::sync::LazyLock;
use std::{env, io};

use regex::Regex;
use walkdir::WalkDir;

#[derive(Debug)]
struct Error(String);

impl Error {
    fn for_path<E: Display>(path: &Path, err: E) -> Error {
        Error(format!("{}: {err}", path.display()))
    }

    /// Callback for `std::io::Result::map_err`. `io::Error` messages often omit
    /// the path.
    fn add_path(path: &Path) -> impl Fn(io::Error) -> Error + Copy + '_ {
        |err| Error::for_path(path, err)
    }

    /// Prints an error message to `stderr`, and exits the current process with
    /// non-zero status. The return type is generic so that this function
    /// can be used in contexts where any specific return type `T` is
    /// required.  Semantically, this function returns [the never type][1]
    /// `!`--meaning it never actually returns at all--but in practice,
    /// returning `!` would often require callers to coerce the result to
    /// some specific type.
    ///
    /// [1]: https://doc.rust-lang.org/reference/types/never.html
    fn die<T>(self) -> T {
        eprintln!("error: {self}");
        exit(1)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// `walkdir::Error`, unlike `io::Error`, always includes the path where the
/// error occurred.
impl From<walkdir::Error> for Error {
    fn from(value: walkdir::Error) -> Self {
        Error(format!("{value}"))
    }
}

type Result<T> = std::result::Result<T, Error>;

struct Link {
    name: String,
    link: String,
}

impl fmt::Display for Link {
    /// Returns a Markdown inline link.  See also:
    /// <https://daringfireball.net/projects/markdown/syntax#link>
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]({})", self.name, self.link)
    }
}

struct LinkPattern(Regex);

impl LinkPattern {
    pub fn compile() -> LinkPattern {
        // I'd love to offer you a link to the webloc format docs, but there don't seem
        // to be any.
        LinkPattern(Regex::new(r"^\t<string>(.*)</string>$").expect("hard-coded regex"))
    }

    pub fn match_line(&self, line: &str) -> Option<String> {
        self.0
            .captures(line)
            .map(|captures| captures[1].to_string())
    }
}

static LINK_PATTERN: LazyLock<LinkPattern> = LazyLock::new(LinkPattern::compile);

struct WeblocFile(PathBuf);

impl WeblocFile {
    fn from_path(path: &Path) -> Option<WeblocFile> {
        let is_webloc_file =
            path.is_file() && path.extension().is_some_and(|s| s.as_bytes() == b"webloc");
        is_webloc_file.then(|| WeblocFile(path.to_owned()))
    }

    /// Returns this file's stem, or the full path if it has no stem.
    fn name(&self) -> String {
        self.0
            .file_stem()
            .unwrap_or(self.0.as_os_str())
            .to_string_lossy()
            .into_owned()
    }
}

impl TryFrom<WeblocFile> for Link {
    type Error = Error;
    fn try_from(value: WeblocFile) -> Result<Self> {
        let path = &value.0;
        let add_path = Error::add_path(path);
        let file = File::open(path).map_err(add_path)?;
        let mut links = io::BufReader::new(file)
            .lines()
            .collect::<io::Result<Vec<String>>>()
            .map_err(add_path)?
            .into_iter()
            .filter_map(|line| LINK_PATTERN.match_line(&line));
        match (links.next(), links.next()) {
            (Some(link), None) => Ok(Link {
                name: value.name(),
                link,
            }),
            _ => Err(Error::for_path(path, "expected exactly one link")),
        }
    }
}

enum SearchablePath {
    File(WeblocFile),
    Directory(PathBuf),
}

impl FromStr for SearchablePath {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        let path = Path::new(s);
        if let Some(file) = WeblocFile::from_path(path) {
            Ok(SearchablePath::File(file))
        } else if path.is_dir() {
            Ok(SearchablePath::Directory(path.to_owned()))
        } else {
            let what = format!("{s}: expected directory or .webloc file");
            Err(Error(what))
        }
    }
}

impl TryFrom<SearchablePath> for Vec<Link> {
    type Error = Error;
    fn try_from(value: SearchablePath) -> Result<Self> {
        match value {
            SearchablePath::File(file) => Ok(vec![file.try_into()?]),
            SearchablePath::Directory(root) => WalkDir::new(&root)
                .into_iter()
                .collect::<walkdir::Result<Vec<_>>>()?
                .into_iter()
                .filter_map(|dirent| WeblocFile::from_path(dirent.path()))
                .map(TryInto::try_into)
                .collect(),
        }
    }
}

fn main_imp<S: AsRef<str>>(args: impl IntoIterator<Item = S>) {
    args.into_iter()
        .map(|arg| arg.as_ref().parse())
        .collect::<Result<Vec<SearchablePath>>>()
        .unwrap_or_else(Error::die)
        .into_iter()
        .map(SearchablePath::try_into) // path -> links; may be >1 link if path is a directory
        .collect::<Result<Vec<Vec<Link>>>>()
        .unwrap_or_else(Error::die)
        .into_iter()
        .flatten()
        .for_each(|link| {
            // Markdown list item.  See also:
            // <https://daringfireball.net/projects/markdown/syntax#list>
            println!("* {link}");
        });
}

fn main() {
    if env::args().nth(1).is_some() {
        main_imp(env::args().skip(1));
    } else {
        main_imp(["."]);
    }
}
