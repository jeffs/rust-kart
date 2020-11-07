#![allow(dead_code, unused_imports)]

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

struct InitState {
    paths: VecDeque<PathBuf>,
}

struct OpenState {
    paths: VecDeque<PathBuf>,
    open: Opening,
}

enum State {
    Init(InitState),
    Open(OpenState),
    Done,
}

impl State {
    fn init(paths: VecDeque<PathBuf>) -> State {
        State::Init(InitState { paths })
    }

    fn open(paths: VecDeque<PathBuf>, open: Opening) -> State {
        State::Open(OpenState { paths, open })
    }

    fn from_init(mut value: InitState) -> (State, Option<io::Error>) {
        if let Some(path) = value.paths.pop_front() {
            match open_file(&path) {
                Ok(open) => (State::open(value.paths, open), None),
                Err(err) => {
                    let err = prefix_error(&path, err);
                    (State::init(value.paths), Some(err))
                }
            }
        } else {
            (State::Done, None)
        }
    }

    fn from_open(mut value: OpenState) -> (State, Option<io::Result<String>>) {
        match value.open.lines.next() {
            next@Some(Ok(_)) => (State::Open(value), next),

            Some(Err(err)) => {
                let err = prefix_error(&value.open.path, err);
                (State::init(value.paths), Some(Err(err)))
            }

            None => {
                let (state, err) = State::from_init(InitState { paths: value.paths });
                match err {
                    Some(err) => (state, Some(Err(err))),
                    None => State::next(state),
                }
            }
        }
    }

    fn next(state: State) -> (State, Option<io::Result<String>>) {
        match state {
            State::Init(value) => {
                let (state, err) = State::from_init(value);
                match err {
                    Some(err) => (state, Some(Err(err))),
                    None => State::next(state),
                }
            }
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
        let paths = paths.map(|p| p.as_ref().to_owned());
        let state = State::init(paths.collect());
        FilesLines { state }
    }
}

impl Iterator for FilesLines {
    type Item = io::Result<String>;

    fn next(&mut self) -> Option<Self::Item> {
        let (state, next) = State::next(mem::take(&mut self.state));
        self.state = state;
        next
    }
}
