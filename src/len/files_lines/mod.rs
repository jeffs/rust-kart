mod tests;

use std::collections::VecDeque;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Lines};
use std::path::{Path, PathBuf};

fn prefix_error(path: &Path, err: io::Error) -> io::Error {
    let what = format!("{}: {}", path.to_string_lossy(), err);
    io::Error::new(err.kind(), what)
}

/// Iterates over the lines of a sequence of files.
///
/// Reports all errors, but:
/// * After any encoding error, skips the remainder of the current file.
/// * After any other error, returns None.
pub struct FilesLines {
    paths: VecDeque<PathBuf>,
    lines: Option<Lines<BufReader<File>>>,
}

impl FilesLines {
    pub fn new<P, I>(paths: I) -> FilesLines
    where
        P: AsRef<Path>,
        I: IntoIterator<Item = P>,
    {
        FilesLines {
            paths: paths.into_iter().map(|p| p.as_ref().to_owned()).collect(),
            lines: None,
        }
    }

    /// Opens the next file if no file is already open, and if another path is
    /// available, assigning a line iterator to self.lines.  On failure to open
    /// a file, returns the error prefixed with the path, but does not pop
    /// the path from the queue.
    fn open_lines(&mut self) -> io::Result<()> {
        if self.lines.is_none() && !self.paths.is_empty() {
            match File::open(&self.paths[0]) {
                Ok(file) => {
                    self.lines = Some(io::BufReader::new(file).lines());
                }
                Err(err) => {
                    return Err(prefix_error(&self.paths[0], err));
                }
            }
        }
        Ok(())
    }

    fn skip_file(&mut self) {
        self.paths.pop_front();
        self.lines = None;
    }
}

impl Iterator for FilesLines {
    type Item = io::Result<String>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Err(err) = self.open_lines() {
                self.skip_file();
                return Some(Err(err));
            }

            if self.lines.is_none() {
                return None;
            }

            let opt = self.lines.as_mut().and_then(|lines| lines.next());
            if let Some(Ok(_)) = opt {
                return opt;
            }

            self.skip_file();

            if opt.is_some() {
                return opt;
            }
        }
    }
}
