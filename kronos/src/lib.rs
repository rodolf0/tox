extern crate chrono;

pub mod constants;

mod semantics;
pub use semantics::{Seq, Range};
pub use semantics::{day, week, weekend, month, quarter, year};
pub use semantics::{day_of_week, month_of_year};
pub use semantics::{nthof, intersect, merge, interval};
pub use semantics::{this, next, a_year};

mod utils;

#[cfg(test)]
mod semantics_test;
