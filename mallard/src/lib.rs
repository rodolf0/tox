extern crate chrono;
extern crate kronos;
extern crate earlgrey;
extern crate lexers;

mod grammar;
mod eval;

pub use eval::parse_time;
