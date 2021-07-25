mod tests;

use std::collections::VecDeque;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Lines};
use std::mem;
use std::path::{Path, PathBuf};

fn prefix_error<P>(path: P, err: io::Error) -> io::Error
where
    P: AsRef<Path>,
{
    let what = format!("{}: {}", path.as_ref().to_string_lossy(), err);
    io::Error::new(err.kind(), what)
}

struct Opening {
    path: PathBuf,
    lines: Lines<BufReader<File>>,
}

fn open_file<P>(path: P) -> io::Result<Opening>
where
    P: AsRef<Path>,
{
    let path = path.as_ref().to_owned();
    let file = File::open(&path)?;
    let lines = io::BufReader::new(file).lines();
    Ok(Opening { path, lines })
}

// Replace any directory at the front of paths with its direct child paths.
// Repeat until the front of paths is not a directory.
fn flatten_front(paths: &mut VecDeque<PathBuf>) -> io::Result<()> {
    while let Some(path) = paths.pop_front() {
        if !path.is_dir() {
            paths.push_front(path);
            break;
        }

        let dir = match path.read_dir() {
            Ok(dir) => dir,
            Err(err) => return Err(prefix_error(path, err)),
        };

        let mut subs = Vec::new();
        for res in dir {
            match res {
                Ok(entry) => subs.push(entry.path()),
                Err(err) => return Err(prefix_error(path, err)),
            }
        }

        subs.sort();
        for path in subs.into_iter().rev() {
            paths.push_front(path);
        }
    }
    Ok(())
}

struct OpenState {
    paths: VecDeque<PathBuf>,
    open: Opening,
}

enum State {
    Init { paths: VecDeque<PathBuf> },
    Open(OpenState),
    Done,
}

impl State {
    fn from_init(mut paths: VecDeque<PathBuf>) -> (State, Option<io::Result<String>>) {
        if let Err(err) = flatten_front(&mut paths) {
            return (State::Init { paths }, Some(Err(err)));
        }
        match paths.pop_front() {
            Some(path) => match open_file(&path) {
                Ok(open) => (State::Open(OpenState { paths, open }), None),
                Err(err) => {
                    let err = prefix_error(path, err);
                    (State::Init { paths }, Some(Err(err)))
                }
            },
            None => (State::Done, None),
        }
    }

    fn from_open(mut value: OpenState) -> (State, Option<io::Result<String>>) {
        match value.open.lines.next() {
            next @ Some(Ok(_)) => (State::Open(value), next),
            Some(Err(err)) => {
                let err = prefix_error(&value.open.path, err);
                (State::Init { paths: value.paths }, Some(Err(err)))
            }
            None => (State::Init { paths: value.paths }, None),
        }
    }

    fn is_done(&self) -> bool {
        match self {
            State::Done => true,
            _ => false,
        }
    }

    /// Transition state, spinning out (the result of) an action.
    fn next(state: State) -> (State, Option<io::Result<String>>) {
        match state {
            State::Init { paths } => State::from_init(paths),
            State::Open(value) => State::from_open(value),
            State::Done => (State::Done, None),
        }
    }
}

impl Default for State {
    fn default() -> Self {
        State::Done
    }
}

/// Iterates over the lines of a sequence of files.
///
/// Reports all errors, but:
/// * After any encoding error, skips the remainder of the current file.
/// * After any other error, returns None.
pub struct FilesLines {
    state: State,
}

impl FilesLines {
    pub fn new<P, I>(paths: I) -> FilesLines
    where
        P: AsRef<Path>,
        I: IntoIterator<Item = P>,
    {
        let paths = paths.into_iter();
        let paths = paths.map(|p| p.as_ref().to_owned()).collect();
        let state = State::Init { paths };
        FilesLines { state }
    }
}

impl Iterator for FilesLines {
    type Item = io::Result<String>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut state_next = State::next(mem::take(&mut self.state));
        while !(state_next.0.is_done() || state_next.1.is_some()) {
            state_next = State::next(state_next.0);
        }
        self.state = state_next.0;
        state_next.1
    }
}
