#![deny(warnings)]

mod types;
pub use crate::types::{Grain, TimeSequence, Range, Season};

mod utils;

mod seq_named;
pub use crate::seq_named::{Weekday, Month, Weekend, Year};

mod seq_grain;
pub use crate::seq_grain::Grains;

mod seq_nthof;
pub use crate::seq_nthof::NthOf;

mod seq_lastof;
pub use crate::seq_lastof::LastOf;

mod seq_union;
pub use crate::seq_union::Union;

mod seq_intersect;
pub use crate::seq_intersect::Intersect;

mod seq_except;
pub use crate::seq_except::Except;

mod seq_interval;
pub use crate::seq_interval::Interval;

mod seq_seasons;
pub use crate::seq_seasons::Seasons;

mod seq_mgrain;
pub use crate::seq_mgrain::MGrain;

mod seq_func;
pub use crate::seq_func::{Map, shift};

#[cfg(test)]
mod mixed_tests;
