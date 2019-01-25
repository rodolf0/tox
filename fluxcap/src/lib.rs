#![deny(warnings)]

mod constants;
mod time_parser;
pub use time_parser::time_grammar;

mod time_semantics;
pub use time_semantics::{TimeMachine, TimeEl};

#[cfg(test)]
mod time_test;
