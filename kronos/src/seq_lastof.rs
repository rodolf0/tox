#![deny(warnings)]

use std::collections::VecDeque;
use crate::types::{DateTime, Range, TimeSequence};

// Guard against impossible sequences, eg: 32nd day of the month
const INFINITE_FUSE: usize = 1000;

#[derive(Clone)]
pub struct LastOf<Frame, Win>(pub usize, pub Win, pub Frame)
    where Frame: TimeSequence,
          Win: TimeSequence + Clone;


impl<Frame, Win> LastOf<Frame, Win>
    where Frame: TimeSequence,
          Win: TimeSequence + Clone
{
    fn _base(&self, t0: &DateTime, future: bool)
        -> Box<dyn Iterator<Item=Range> + '_>
    {
        let win = self.1.clone();
        let nth = self.0;
        let frame = if future {
            self.2._future_raw(t0)
        } else {
            self.2._past_raw(t0)
        };
        Box::new(frame
            .map(move |outer| {
                let mut buf = VecDeque::new();
                for inner in win._future_raw(&outer.start) {
                    if inner.start >= outer.end {
                        return buf.remove(nth-1);
                    }
                    buf.push_front(inner);
                    buf.truncate(nth);
                }
                None
            })
            .enumerate()
            .filter_map(|(i, elem)| { assert!(i <= INFINITE_FUSE); elem })
        )
    }
}

impl<Frame, Win> TimeSequence for LastOf<Frame, Win>
    where Frame: TimeSequence,
          Win: TimeSequence + Clone
{
    fn _future_raw(&self, t0: &DateTime) -> Box<dyn Iterator<Item=Range> + '_> {
        self._base(t0, true)
    }

    fn _past_raw(&self, t0: &DateTime) -> Box<dyn Iterator<Item=Range> + '_> {
        self._base(t0, false)
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use crate::types::{Date, Grain};
    use crate::seq_grain::Grains;
    use crate::seq_named::{Weekend, Month};


    fn dt(year: i32, month: u32, day: u32) -> DateTime {
        Date::from_ymd(year, month, day).and_hms(0, 0, 0)
    }

    #[test]
    #[should_panic]
    fn lastof_fuse() {
        let badlastof = LastOf(32, Grains(Grain::Day), Grains(Grain::Month));
        badlastof.future(&dt(2015, 2, 25)).next();
    }

    #[test]
    fn lastof() {
        // last weekend of the year
        let weekendofyear = LastOf(1, Weekend, Grains(Grain::Year));
        let mut weekendofyear = weekendofyear.future(&dt(2015, 2, 25));
        assert_eq!(weekendofyear.next().unwrap(),
            Range{start: dt(2015, 12, 26), end: dt(2015, 12, 28), grain: Grain::Day});
        assert_eq!(weekendofyear.next().unwrap(),
            Range{start: dt(2016, 12, 31), end: dt(2017, 1, 2), grain: Grain::Day});

        // 2nd-to-last day of february
        let daybeforelastfeb = LastOf(2, Grains(Grain::Day), Month(2));
        let mut daybeforelastfeb = daybeforelastfeb.future(&dt(2015, 2, 25));
        assert_eq!(daybeforelastfeb.next().unwrap(),
            Range{start: dt(2015, 2, 27), end: dt(2015, 2, 28), grain: Grain::Day});
        assert_eq!(daybeforelastfeb.next().unwrap(),
            Range{start: dt(2016, 2, 28), end: dt(2016, 2, 29), grain: Grain::Day});

        // 29th-to-last day of feb
        let t29th_before_last = LastOf(29, Grains(Grain::Day), Month(2));
        let mut t29th_before_last = t29th_before_last.future(&dt(2015, 2, 25));
        assert_eq!(t29th_before_last.next().unwrap(),
            Range{start: dt(2016, 2, 1), end: dt(2016, 2, 2), grain: Grain::Day});
        assert_eq!(t29th_before_last.next().unwrap(),
            Range{start: dt(2020, 2, 1), end: dt(2020, 2, 2), grain: Grain::Day});

        // backward: 2nd-to-last day of february
        let daybeforelastfeb = LastOf(2, Grains(Grain::Day), Month(2));
        let mut daybeforelastfeb = daybeforelastfeb.past(&dt(2015, 2, 25));
        assert_eq!(daybeforelastfeb.next().unwrap(),
            Range{start: dt(2014, 2, 27), end: dt(2014, 2, 28), grain: Grain::Day});
        assert_eq!(daybeforelastfeb.next().unwrap(),
            Range{start: dt(2013, 2, 27), end: dt(2013, 2, 28), grain: Grain::Day});
        assert_eq!(daybeforelastfeb.next().unwrap(),
            Range{start: dt(2012, 2, 28), end: dt(2012, 2, 29), grain: Grain::Day});

        // backward: 5th-to-last day of february
        let fithbeforelastfeb = LastOf(5, Grains(Grain::Day), Month(2));
        let mut fithbeforelastfeb = fithbeforelastfeb.past(&dt(2015, 2, 26));
        assert_eq!(fithbeforelastfeb.next().unwrap(),
            Range{start: dt(2015, 2, 24), end: dt(2015, 2, 25), grain: Grain::Day});

        // backward: 5th-to-last day of february starting that day - inclusive/raw
        let fithbeforelastfeb = LastOf(5, Grains(Grain::Day), Month(2));
        let mut fithbeforelastfeb = fithbeforelastfeb._past_raw(&dt(2015, 2, 24));
        assert_eq!(fithbeforelastfeb.next().unwrap(),
            Range{start: dt(2015, 2, 24), end: dt(2015, 2, 25), grain: Grain::Day});
    }
}
