#![deny(warnings)]

mod constants;

mod time_grammar;

mod time_semantics;
pub use time_semantics::{TimeMachine, TimeResult};

#[cfg(test)]
mod tests;
mod tests_json;
