use std::collections::VecDeque;
use std::fs::File;
use std::io;
use std::path::PathBuf;

/// Iterates over the lines of a sequence of files.
///
/// Reports all errors, but:
/// * After any encoding error, skips the remainder of the current file.
/// * After any other error, returns None.
struct FilesLines {
    paths: VecDeque<PathBuf>,
    file: Option<File>,
}

impl FilesLines {
    pub fn new<I: Iterator<Item = PathBuf>>(paths: I) -> FilesLines {
        FilesLines {
            paths: paths.collect(),
            file: None,
        }
    }

    fn next_file(&self) -> Option<io::Result<File>> {
        self.paths.front().map(|path| File::open(path))
    }
}

impl Iterator for FilesLines {
    type Item = io::Result<String>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.file.is_none() {
            let file = self.next_file();
            if let Some(Err(err)) = file {
                self.paths.pop_front();
                return Some(Err(err));
            }

            self.file = file.map(|res| res.ok()).flatten();
        }

        None
    }
}
