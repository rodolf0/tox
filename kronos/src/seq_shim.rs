#![deny(warnings)]

use std::rc::Rc;
use crate::types::{DateTime, Range, TimeSequence};

// seq_*.rs hold many different types that implement TimeSequence, Shim
// is a helper to allow different sequence types to be used as if they were one

#[derive(Clone)]
pub struct Shim(pub Rc<dyn TimeSequence>);

impl TimeSequence for Shim {
    fn _future_raw(&self, t0: &DateTime) -> Box<dyn Iterator<Item=Range> + '_> {
        self.0._future_raw(t0)
    }

    fn _past_raw(&self, t0: &DateTime) -> Box<dyn Iterator<Item=Range> + '_> {
        self.0._past_raw(t0)
    }

    fn future(&self, t0: &DateTime) -> Box<dyn Iterator<Item=Range> + '_> {
        self.0.future(t0)
    }

    fn past(&self, t0: &DateTime) -> Box<dyn Iterator<Item=Range> + '_> {
        self.0.past(t0)
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use crate::types::{Date, Grain};
    use crate::seq_named::Weekday;
    use crate::seq_grain::Grains;
    use crate::seq_nthof::NthOf;

    fn dt(year: i32, month: u32, day: u32) -> DateTime {
        Date::from_ymd(year, month, day).and_hms(0, 0, 0)
    }

    #[test]
    fn try_shim() {
        let weekday2 = Rc::new(Weekday(2));
        let tue3mo = NthOf(3, Shim(weekday2), Grains(Grain::Month));
        let mut tue3mo = tue3mo.future(&dt(2019, 1, 12));
        assert_eq!(tue3mo.next().unwrap(),
            Range{start: dt(2019, 1, 15), end: dt(2019, 1, 16), grain: Grain::Day});

    }
}
