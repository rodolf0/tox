extern crate chrono;
extern crate kronos;
extern crate earlgrey;
extern crate lexers;

mod time;
pub use time::TimeMachine;

#[cfg(test)]
mod time_test;

//mod learn;
//pub use learn::{TrainData, load_training, learn, score_tree};
