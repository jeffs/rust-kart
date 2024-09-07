// TODO: Report CLI errors; supprot --help.

use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::{env, io};

use regex::Regex;
use walkdir::{DirEntry, WalkDir};

struct Extractor {
    re: Regex,
}

impl Extractor {
    pub fn new() -> Extractor {
        Extractor {
            re: Regex::new(r"^\t<string>(.*)</string>$").expect("hard-coded regex should be valid"),
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

fn find_weblocs(root: &Path) -> walkdir::Result<Vec<PathBuf>> {
    Ok(if root.is_dir() {
        let paths: Result<Vec<_>, _> = WalkDir::new(root).into_iter().collect();
        paths?
            .into_iter()
            .filter(is_webloc)
            .map(|entry| entry.path().to_path_buf())
            .collect()
    } else {
        vec![root.to_path_buf()]
    })
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

fn main() {
    for root in env::args().skip(1) {
        let paths = find_weblocs(root.as_ref()).unwrap_or_else(|err| {
            eprintln!("error: {err}");
            exit(1);
        });
        for path in paths.into_iter() {
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("file lacking UTF-8 stem");
            let links = read_links(&path).unwrap_or_else(|err| {
                eprintln!("error: {}: {err}", path.display());
                exit(1);
            });
            for link in links {
                println!("* [{stem}]({link})");
            }
        }
    }
}
