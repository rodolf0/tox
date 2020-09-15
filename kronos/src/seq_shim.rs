#![deny(warnings)]

use crate::types::{DateTime, Range, TimeSequence};
use std::rc::Rc;

// seq_*.rs hold many different types that implement TimeSequence, Shim
// is a helper to allow different sequence types to be used as if they were one

#[derive(Clone)]
pub struct Shim<'a>(pub Rc<dyn TimeSequence + 'a>);

impl<'a> Shim<'a> {
    pub fn new(seq: impl TimeSequence + 'a) -> Shim<'a> {
        Shim(Rc::new(seq))
    }
}

impl<'a> TimeSequence for Shim<'a> {
    fn _future_raw(&self, t0: &DateTime) -> Box<dyn Iterator<Item = Range> + '_> {
        self.0._future_raw(t0)
    }

    fn _past_raw(&self, t0: &DateTime) -> Box<dyn Iterator<Item = Range> + '_> {
        self.0._past_raw(t0)
    }

    fn future(&self, t0: &DateTime) -> Box<dyn Iterator<Item = Range> + '_> {
        self.0.future(t0)
    }

    fn past(&self, t0: &DateTime) -> Box<dyn Iterator<Item = Range> + '_> {
        self.0.past(t0)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::seq_grain::Grains;
    use crate::seq_named::Weekday;
    use crate::seq_nthof::NthOf;
    use crate::types::{Date, Grain};

    fn dt(year: i32, month: u32, day: u32) -> DateTime {
        Date::from_ymd(year, month, day).and_hms(0, 0, 0)
    }

    #[test]
    fn try_shim() {
        let weekday2 = Rc::new(Weekday(2));
        let tue3mo = NthOf(3, Shim(weekday2), Grains(Grain::Month));
        let mut tue3mo = tue3mo.future(&dt(2019, 1, 12));
        assert_eq!(
            tue3mo.next().unwrap(),
            Range {
                start: dt(2019, 1, 15),
                end: dt(2019, 1, 16),
                grain: Grain::Day
            }
        );
    }

    #[test]
    fn any_container() {
        let abunch = vec![
            Shim::new(Weekday(3)),
            Shim::new(NthOf(3, Weekday(2), Grains(Grain::Month))),
        ];
        assert_eq!(abunch.len(), 2);
    }
}
