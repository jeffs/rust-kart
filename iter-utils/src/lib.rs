//! Iterator utilities: infinite sequences, lazy creation, and inclusive termination.

mod delay;
mod perpetuity;
mod stop_once;

pub use delay::{delay, Delay, Delayed};
pub use perpetuity::{assimilate, Perpetuity, Successors, Take};
pub use stop_once::{StopOnce, Stopper};
