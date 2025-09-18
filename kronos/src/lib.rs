#![deny(warnings)]

mod sequence;
pub use crate::sequence::{Grain, TimeSeq, TimeSpan};

#[cfg(test)]
mod tests;
