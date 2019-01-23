#![deny(warnings)]

use crate::utils;
use crate::types::{DateTime, Grain, Range, TimeSequence, Duration};


#[derive(Clone)]
pub struct MGrain {
    duration: Duration,
    resolution: Grain,
}

impl MGrain {
    pub fn new(duration: Duration) -> MGrain {
        MGrain{duration, resolution: utils::grain_from_duration(duration)}
    }

    pub fn new2(duration: Duration, resolution: Grain) -> MGrain {
        MGrain{duration, resolution}
    }

    fn _base(&self, t0: &DateTime, future: bool) -> Box<Iterator<Item=Range>> {
        let base = utils::truncate(*t0, self.resolution);
        let hop = if future { self.duration } else { - self.duration };
        let duration = self.duration;
        let grain = self.resolution;
        Box::new((0..).map(move |x| Range{
            start: base + hop * x,
            end: base + hop * x + duration,
            grain
        }))
    }
}

impl TimeSequence for MGrain {
    fn _future_raw(&self, t0: &DateTime) -> Box<Iterator<Item=Range>> {
        self._base(t0, true)
    }

    fn _past_raw(&self, t0: &DateTime) -> Box<Iterator<Item=Range>> {
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

    fn dttm(year: i32, month: u32, day: u32, h: u32, m: u32, s: u32) -> DateTime {
        Date::from_ymd(year, month, day).and_hms(h, m, s)
    }

    #[test]
    fn grain_basic() {
        let twodays = MGrain::new(Duration::days(2));
        let mut iter = twodays.future(&dt(2015, 2, 28));
        assert_eq!(iter.next().unwrap(),
            Range{start: dt(2015, 2, 28), end: dt(2015, 3, 2), grain: Grain::Day});
        assert_eq!(iter.next().unwrap(),
            Range{start: dt(2015, 3, 2), end: dt(2015, 3, 4), grain: Grain::Day});

        let mut iter = twodays.past(&dt(2015, 2, 28));
        assert_eq!(iter.next().unwrap(),
            Range{start: dt(2015, 2, 26), end: dt(2015, 2, 28), grain: Grain::Day});
        assert_eq!(iter.next().unwrap(),
            Range{start: dt(2015, 2, 24), end: dt(2015, 2, 26), grain: Grain::Day});

        // past inclusive
        let mut iter = twodays._past_raw(&dt(2015, 2, 28));
        assert_eq!(iter.next().unwrap(),
            Range{start: dt(2015, 2, 28), end: dt(2015, 3, 2), grain: Grain::Day});
        assert_eq!(iter.next().unwrap(),
            Range{start: dt(2015, 2, 26), end: dt(2015, 2, 28), grain: Grain::Day});
    }

    #[test]
    fn smaller_grains() {
        let twohs30m = MGrain::new(Duration::minutes(2*60+30));

        let mut iter = twohs30m.future(&dt(2015, 2, 27));
        assert_eq!(iter.next().unwrap(),
            Range{start: dttm(2015, 2, 27, 0, 0, 0),
                  end: dttm(2015, 2, 27, 2, 30, 0), grain: Grain::Minute});
        assert_eq!(iter.next().unwrap(),
            Range{start: dttm(2015, 2, 27, 2, 30, 0),
                  end: dttm(2015, 2, 27, 5, 00, 0), grain: Grain::Minute});

        let mut iter = twohs30m.past(&dttm(2015, 2, 27, 3, 0, 0));
        assert_eq!(iter.next().unwrap(),
            Range{start: dttm(2015, 2, 27, 0, 30, 0),
                  end: dttm(2015, 2, 27, 3, 0, 0), grain: Grain::Minute});
        assert_eq!(iter.next().unwrap(),
            Range{start: dttm(2015, 2, 26, 22, 00, 0),
                  end: dttm(2015, 2, 27, 0, 30, 0), grain: Grain::Minute});
    }

    #[test]
    fn more_mgrain() {
        let twoweeks = MGrain::new(Duration::weeks(2));
        let mut twoweeks = twoweeks.future(&dt(2015, 2, 27));
        assert_eq!(twoweeks.next().unwrap(),
            Range{start: dt(2015, 2, 22), end: dt(2015, 3, 8), grain: Grain::Week});
        assert_eq!(twoweeks.next().unwrap(),
            Range{start: dt(2015, 3, 8), end: dt(2015, 3, 22), grain: Grain::Week});

        let threedays = MGrain::new(Duration::days(3));
        let mut iter = threedays.future(&dt(2015, 2, 27));
        assert_eq!(iter.next().unwrap(),
            Range{start: dt(2015, 2, 27), end: dt(2015, 3, 2), grain: Grain::Day});
        assert_eq!(iter.next().unwrap(),
            Range{start: dt(2015, 3, 2), end: dt(2015, 3, 5), grain: Grain::Day});
        assert_eq!(iter.next().unwrap(),
            Range{start: dt(2015, 3, 5), end: dt(2015, 3, 8), grain: Grain::Day});

        let mut iter = threedays.past(&dt(2015, 2, 17));
        assert_eq!(iter.next().unwrap(),
            Range{start: dt(2015, 2, 14), end: dt(2015, 2, 17), grain: Grain::Day});
    }
}
