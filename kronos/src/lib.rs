extern crate chrono;

pub mod constants;

mod semantics;
pub use semantics::{Seq, Range, Granularity};
pub use semantics::{day, week, weekend, month, quarter, year};
pub use semantics::{day_of_week, month_of_year};
pub use semantics::{nthof, lastof, intersect, merge, interval, skip};
pub use semantics::{this, next, a_year, shift};

mod utils;
pub use utils::{date_add, date_sub, days_in_month, enclose};

#[cfg(test)]
mod semantics_test;
