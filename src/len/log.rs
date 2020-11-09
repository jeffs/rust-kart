use std::fmt::Display;
use std::io::Write;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

pub struct Log {
    color: termcolor::ColorChoice,
}

impl Log {
    pub fn new(color: ColorChoice) -> Log {
        Log { color }
    }

    fn prefix(&self, prefix: &str) {
        let mut stderr = StandardStream::stderr(self.color);
        let mut res = stderr.set_color(ColorSpec::new().set_fg(Some(Color::Red)));
        if res.is_ok() {
            res = write!(&mut stderr, "{}", prefix);
            let _ = stderr.reset();
        }
        if res.is_err() {
            eprint!("{}", prefix);
        }
    }

    pub fn print<T: Display>(&self, prefix: &str, what: T) {
        self.prefix(prefix);
        eprintln!(": {}", what);
    }

    pub fn error<T: Display>(&self, what: T) {
        self.print("error", what);
    }

    // TODO: Move Log into its own crate.
    #[allow(dead_code)]
    pub fn fatal<T: Display>(&self, what: T) -> ! {
        self.error(what);
        std::process::exit(1);
    }

    pub fn warning<T: Display>(&self, what: T) {
        self.print("warning", what);
    }
}
