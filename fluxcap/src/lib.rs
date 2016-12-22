#[macro_use] extern crate lazy_static;
extern crate regex;
extern crate chrono;
extern crate kronos;
extern crate earlgrey;
extern crate lexers;

mod time;
mod learn;

pub use time::{build_grammar, TimeMachine, Time};
pub use learn::{TrainData, load_training, learn, score_tree};
