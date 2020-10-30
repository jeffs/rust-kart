use std::collections::VecDeque;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Lines};
use std::path::PathBuf;

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
    pub fn new<I: Iterator<Item = PathBuf>>(paths: I) -> FilesLines {
        FilesLines {
            paths: paths.collect(),
            lines: None,
        }
    }

    fn next_file(&self) -> Option<io::Result<File>> {
        self.paths.front().map(|path| File::open(path))
    }
}

impl Iterator for FilesLines {
    type Item = io::Result<String>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.lines.is_none() {
            match self.next_file() {
                Some(Ok(file)) => {
                    // Start reading the next file.
                    self.lines = Some(io::BufReader::new(file).lines());
                }
                Some(Err(err)) => {
                    // Skip the next file, and report the error.
                    self.paths.pop_front();
                    return Some(Err(err));
                }
                None => {
                    // There are no more files to read.
                }
            }
        }

        let opt = self.lines.as_mut().and_then(|lines| lines.next());
        if opt.is_none() {
            // We've completed the current file.  Advance to the next, if any.
            self.lines = None;
            self.paths.pop_front();
            if self.paths.is_empty() {
                None
            } else {
                self.next()
            }
        } else if let Some(Err(err)) = opt {
            // Skip the rest of the file.
            self.lines = None;
            self.paths.pop_front().map(|path| {
                // Prefix error with file name.
                let path = path.to_string_lossy();
                Err(io::Error::new(err.kind(), format!("{}: {}", path, err)))
            })
        } else {
            opt
        }
    }
}
