#![doc = include_str!("../README.md")]

mod delay;
mod perpetuity;
mod stop_once;

pub use delay::{delay, Delay, Delayed};
pub use perpetuity::{assimilate, Perpetuity, Successors, Take};
pub use stop_once::{StopOnce, Stopper};
