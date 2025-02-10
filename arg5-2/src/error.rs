use std::ffi::OsString;
use std::fmt::{self, Debug};

/// Something was wrong with the initialization of a [`Parser`].  In principle, this should be a
/// compile-time error.  The appropriate response may be to [`panic`].
///
/// [`Parser`]: super::Parser
#[derive(Debug, PartialEq)]
pub enum Init {
    /// The supplied flag or option name is not supported.  Future versions of this library may be
    /// extended to support non-ASCII and/or non-alphanumeric flag/option names, but the current
    /// version remains conservative in the name of portability.  Note that the current limitations
    /// apply only to argument names, not values; e.g., args like `--date Mañana` are fine.
    CharName(char),
    LongName(&'static str),
    /// The supplied flag or option name was bound to multiple target variables.
    CharDup(char),
    /// The target variable for a Boolean flag was already true when the variable was registered.
    /// The variable should have been initialized to false, or else there is no way to tell whether
    /// the flag was specified in arguments.
    CharTautology(char),
    LongTautology(&'static str),
    FlexTautology(char, &'static str),
}

impl fmt::Display for Init {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Init::CharTautology(c) => write!(f, "flag -{c} would always be true"),
            Init::LongTautology(s) => write!(f, "flag --{s} would always be true"),
            Init::FlexTautology(c, s) => write!(f, "flag -{c}|--{s} would always be true"),
            Init::CharName(c) => write!(f, "non-ASCII flag name -{c} is unsupported"),
            Init::LongName(s) => write!(f, "non-ASCII flag name --{s} is unsupported"),
            Init::CharDup(c) => write!(f, "flag -{c} cannot be bound to multiple variables"),
        }
    }
}

/// A correctly initialized [`Parser`] has rejected rutime arguments.  This is normal, and typically
/// indicates end user error (a bad command line), rather than incorrect usage of this crate.
///
/// [`Parser`]: super::Parser
#[derive(PartialEq)]
pub enum Parse {
    /// A flag/option name was not recognized.
    CharName(u8),
    LongName(OsString),
}

impl Debug for Parse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            &Parse::CharName(b) => write!(f, "CharName({})", char::from(b)),
            Parse::LongName(s) => (),
        }
    }
}
