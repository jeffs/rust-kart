#![doc = include_str!("../README.md")]

mod delay;
mod perpetuity;
mod stop_once;

pub use delay::{Delay, Delayed, delay};
pub use perpetuity::{Perpetuity, Successors, Take, assimilate};
pub use stop_once::{StopOnce, Stopper};
