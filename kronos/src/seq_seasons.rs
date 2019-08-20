#![deny(warnings)]

use crate::utils;
use crate::types::{Grain, DateTime, Range, TimeSequence, Season};


#[derive(Clone)]
pub struct Seasons(pub Season, pub bool); // north hemisphere

impl Seasons {
    fn _base(&self, t0: &DateTime, future: bool) -> Box<dyn Iterator<Item=Range>> {
        let (s0, s1) = utils::find_season(t0.date(), self.0, future, self.1);
        let s0 = s0.and_hms(0, 0, 0);
        let s1 = s1.and_hms(0, 0, 0);
        let sign = if future { 1 } else { -1 };
        Box::new((0..).map(move |x| Range{
            start: utils::shift_datetime(s0, Grain::Year, sign * x),
            end: utils::shift_datetime(s1, Grain::Year, sign * x),
            grain: Grain::Day
        }))
    }
}

impl TimeSequence for Seasons {
    fn _future_raw(&self, t0: &DateTime) -> Box<dyn Iterator<Item=Range>> {
        self._base(t0, true)
    }

    fn _past_raw(&self, t0: &DateTime) -> Box<dyn Iterator<Item=Range>> {
        self._base(t0, false)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::types::{Date, Grain};

    fn dt(year: i32, month: u32, day: u32) -> DateTime {
        Date::from_ymd(year, month, day).and_hms(0, 0, 0)
    }

    #[test]
    fn summer() {
        let summer = Seasons(Season::Summer, true);

        let mut iter = summer.future(&dt(2015, 9, 22));
        assert_eq!(iter.next().unwrap(),
            Range{start: dt(2016, 6, 21), end: dt(2016, 9, 21), grain: Grain::Day});
        assert_eq!(iter.next().unwrap(),
            Range{start: dt(2017, 6, 21), end: dt(2017, 9, 21), grain: Grain::Day});

        // past non-inclusive
        let mut iter = summer.past(&dt(2015, 9, 22));
        assert_eq!(iter.next().unwrap(),
            Range{start: dt(2015, 6, 21), end: dt(2015, 9, 21), grain: Grain::Day});
        assert_eq!(iter.next().unwrap(),
            Range{start: dt(2014, 6, 21), end: dt(2014, 9, 21), grain: Grain::Day});
    }

    #[test]
    fn winter() {
        let winter = Seasons(Season::Winter, true);
        // future
        let mut iter = winter.future(&dt(2015, 1, 22));
        assert_eq!(iter.next().unwrap(),
            Range{start: dt(2014, 12, 21), end: dt(2015, 3, 21), grain: Grain::Day});
        assert_eq!(iter.next().unwrap(),
            Range{start: dt(2015, 12, 21), end: dt(2016, 3, 21), grain: Grain::Day});

        // past non-inclusive
        let mut iter = winter.past(&dt(2015, 1, 22));
        assert_eq!(iter.next().unwrap(),
            Range{start: dt(2013, 12, 21), end: dt(2014, 3, 21), grain: Grain::Day});
        assert_eq!(iter.next().unwrap(),
            Range{start: dt(2012, 12, 21), end: dt(2013, 3, 21), grain: Grain::Day});

        // past inclusive
        let mut iter = winter._past_raw(&dt(2015, 1, 22));
        assert_eq!(iter.next().unwrap(),
            Range{start: dt(2014, 12, 21), end: dt(2015, 3, 21), grain: Grain::Day});
        assert_eq!(iter.next().unwrap(),
            Range{start: dt(2013, 12, 21), end: dt(2014, 3, 21), grain: Grain::Day});
    }
}
