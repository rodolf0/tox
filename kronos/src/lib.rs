#![deny(warnings)]

mod sequence;
pub use crate::sequence::{Grain, TimeSeqSpec, TimeSpan, TimeSequence};

#[cfg(test)]
mod tests;
