//#![deny(warnings)]

mod constants;
mod time_parser;

mod time_semantics;
pub use crate::time_semantics::{TimeMachine, TimeEl};

#[cfg(test)]
mod time_test;
