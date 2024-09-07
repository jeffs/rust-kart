// TODO: Report CLI errors; supprot --help.

use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::{env, io};

use regex::Regex;
use walkdir::{DirEntry, WalkDir};

struct Extractor {
    re: Regex,
}

impl Extractor {
    pub fn new() -> Extractor {
        Extractor {
            re: Regex::new(r"^\t<string>(.*)</string>$").unwrap(),
        }
    }

    pub fn link(&self, line: &str) -> Option<String> {
        self.re
            .captures(line)
            .and_then(|captures| captures.get(1).map(|m| m.as_str().to_string()))
    }
}

fn is_webloc(entry: &DirEntry) -> bool {
    entry
        .path()
        .extension()
        .filter(|&ext| ext == "webloc")
        .is_some()
}

fn find_weblocs(root: String) -> impl Iterator<Item = PathBuf> {
    WalkDir::new(root)
        .into_iter()
        .flatten()
        .filter(is_webloc)
        .map(|entry| entry.path().to_path_buf())
}

fn read_links(path: &Path) -> io::Result<Vec<String>> {
    let extract = Extractor::new();
    let file = File::open(path)?;
    let lines: io::Result<Vec<String>> = io::BufReader::new(file).lines().collect();
    Ok(lines?
        .iter()
        .filter_map(|line| extract.link(line))
        .collect())
}

fn get_stem(path: &Path) -> &str {
    path.file_stem()
        .and_then(|os_str| os_str.to_str())
        .unwrap_or("(non_utf8_file_name)")
}

fn main() -> io::Result<()> {
    for path in env::args().skip(1).flat_map(find_weblocs) {
        let path = &path;
        let stem = get_stem(path);
        for link in read_links(path)? {
            println!("* [{stem}]({link})");
        }
    }
    Ok(())
}
