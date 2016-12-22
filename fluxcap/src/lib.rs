#[macro_use] extern crate lazy_static;
extern crate regex;
extern crate chrono;
extern crate kronos;
extern crate earlgrey;
extern crate lexers;

mod time;
pub use time::{TimeMachine, Time};

mod learn;
pub use learn::{TrainData, load_training, learn, score_tree};
