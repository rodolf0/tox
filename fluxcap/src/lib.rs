#![deny(warnings)]

mod time_machine;
pub use time_machine::{TimeMachine, TimeEl};

#[cfg(test)]
mod time_test;

//mod time_training;
//mod learn;
//pub use learn::{TrainData, load_training, learn, score_tree};
