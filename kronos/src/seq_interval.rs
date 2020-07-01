#![deny(warnings)]

use crate::utils;
use crate::types::{DateTime, Range, TimeSequence};

// example duckling intervals http://tinyurl.com/hk2vu34

#[derive(Clone)]
pub struct Interval<SeqA, SeqB>
    where SeqA: TimeSequence,
          SeqB: TimeSequence + Clone
{
    start: SeqA,
    end: SeqB,
    inclusive: bool,
}

impl<SeqA, SeqB> Interval<SeqA, SeqB>
    where SeqA: TimeSequence,
          SeqB: TimeSequence + Clone
{
    fn _base(&self, t0: &DateTime, future: bool) -> Box<dyn Iterator<Item=Range> + '_> {
        let endseq = self.end.clone();
        let inclusive = self.inclusive;

        // interval generator
        let interval = move |istart: Range| {
            use std::cmp;
            let iend = endseq._future_raw(&istart.start).next().unwrap();
            Range{
                start: istart.start,
                end: if inclusive { iend.end } else { iend.start },
                grain: cmp::min(istart.grain, iend.grain)
            }
        };

        // guesstimate resolution for framing/truncating reftime so that
        // initial interval can contain t0 even if end-of start element past
        let probe = interval(self.start._future_raw(t0).next().unwrap());
        // estimate grain from interval length
        let trunc_grain = utils::enclosing_grain_from_duration(probe.duration());
        // choose a time of reference aligned to interval on enclosing grain
        let t0 = utils::truncate(*t0, trunc_grain);
        let t0 = self.start._future_raw(&t0).next().unwrap().start;

        Box::new(if future {
            self.start._future_raw(&t0)
        } else {
            self.start._past_raw(&t0)
        }.map(interval))
    }
}

impl<SeqA, SeqB> TimeSequence for Interval<SeqA, SeqB>
    where SeqA: TimeSequence,
          SeqB: TimeSequence + Clone
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
    use crate::seq_named::{Weekday, Month};
    use crate::seq_nthof::NthOf;
    use crate::seq_grain::Grains;

    fn dt(year: i32, month: u32, day: u32) -> DateTime {
        Date::from_ymd(year, month, day).and_hms(0, 0, 0)
    }

    fn dttm(year: i32, month: u32, day: u32, h: u32, m: u32, s: u32) -> DateTime {
        Date::from_ymd(year, month, day).and_hms(h, m, s)
    }

    #[test]
    fn interval_basic() {
        // monday to friday
        let mon2fri = Interval{start: Weekday(1), end: Weekday(5), inclusive: true};

        let mut fut = mon2fri.future(&dt(2016, 2, 25));
        assert_eq!(fut.next().unwrap(),
            Range{start: dt(2016, 2, 22), end: dt(2016, 2, 27), grain: Grain::Day});
        assert_eq!(fut.next().unwrap(),
            Range{start: dt(2016, 2, 29), end: dt(2016, 3, 5), grain: Grain::Day});

        // past non-inclusive
        let mut past = mon2fri.past(&dt(2016, 2, 25));
        assert_eq!(past.next().unwrap(),
            Range{start: dt(2016, 2, 15), end: dt(2016, 2, 20), grain: Grain::Day});
        assert_eq!(past.next().unwrap(),
            Range{start: dt(2016, 2, 8), end: dt(2016, 2, 13), grain: Grain::Day});

        // past inclusive
        let mut past = mon2fri._past_raw(&dt(2016, 2, 25));
        assert_eq!(past.next().unwrap(),
            Range{start: dt(2016, 2, 22), end: dt(2016, 2, 27), grain: Grain::Day});
        assert_eq!(past.next().unwrap(),
            Range{start: dt(2016, 2, 15), end: dt(2016, 2, 20), grain: Grain::Day});
        assert_eq!(past.next().unwrap(),
            Range{start: dt(2016, 2, 8), end: dt(2016, 2, 13), grain: Grain::Day});
    }

    #[test]
    fn interval_afternoon() {
        let afternoon = Interval{
            start: NthOf(13, Grains(Grain::Hour), Grains(Grain::Day)),
            end: NthOf(19, Grains(Grain::Hour), Grains(Grain::Day)),
            inclusive: false};

        let mut iter = afternoon.future(&dt(2016, 2, 25));
        assert_eq!(iter.next().unwrap(),
            Range{start: dttm(2016, 2, 25, 12, 0, 0),
                  end: dttm(2016, 2, 25, 18, 0, 0), grain: Grain::Hour});
        assert_eq!(iter.next().unwrap(),
           Range{start: dttm(2016, 2, 26, 12, 0, 0),
                 end: dttm(2016, 2, 26, 18, 0, 0), grain: Grain::Hour});

        // past non-inclusive
        let mut iter = afternoon.past(&dttm(2016, 2, 25, 14, 0, 0));
        assert_eq!(iter.next().unwrap(),
            Range{start: dttm(2016, 2, 24, 12, 0, 0),
                  end: dttm(2016, 2, 24, 18, 0, 0), grain: Grain::Hour});
        assert_eq!(iter.next().unwrap(),
           Range{start: dttm(2016, 2, 23, 12, 0, 0),
                 end: dttm(2016, 2, 23, 18, 0, 0), grain: Grain::Hour});

        // past inclusive
        let mut iter = afternoon._past_raw(&dttm(2016, 2, 25, 14, 0, 0));
        assert_eq!(iter.next().unwrap(),
            Range{start: dttm(2016, 2, 25, 12, 0, 0),
                  end: dttm(2016, 2, 25, 18, 0, 0), grain: Grain::Hour});
        assert_eq!(iter.next().unwrap(),
            Range{start: dttm(2016, 2, 24, 12, 0, 0),
                  end: dttm(2016, 2, 24, 18, 0, 0), grain: Grain::Hour});
        assert_eq!(iter.next().unwrap(),
           Range{start: dttm(2016, 2, 23, 12, 0, 0),
                 end: dttm(2016, 2, 23, 18, 0, 0), grain: Grain::Hour});
    }

    #[test]
    fn interval_mixed() {
        let june2ndtileom = Interval{
            start: NthOf(2, Grains(Grain::Day), Month(6)),
            end: Month(6), inclusive: true};

        let mut iter = june2ndtileom.future(&dt(2016, 6, 25));
        assert_eq!(iter.next().unwrap(),
            Range{start: dt(2016, 6, 2), end: dt(2016, 7, 1), grain: Grain::Day});
        assert_eq!(iter.next().unwrap(),
            Range{start: dt(2017, 6, 2), end: dt(2017, 7, 1), grain: Grain::Day});

        // past non-inclusive
        let mut iter = june2ndtileom.past(&dt(2016, 6, 25));
        assert_eq!(iter.next().unwrap(),
            Range{start: dt(2015, 6, 2), end: dt(2015, 7, 1), grain: Grain::Day});
        assert_eq!(iter.next().unwrap(),
            Range{start: dt(2014, 6, 2), end: dt(2014, 7, 1), grain: Grain::Day});

        // past inclusive
        let mut iter = june2ndtileom._past_raw(&dt(2016, 6, 25));
        assert_eq!(iter.next().unwrap(),
            Range{start: dt(2016, 6, 2), end: dt(2016, 7, 1), grain: Grain::Day});
        assert_eq!(iter.next().unwrap(),
            Range{start: dt(2015, 6, 2), end: dt(2015, 7, 1), grain: Grain::Day});
        assert_eq!(iter.next().unwrap(),
            Range{start: dt(2014, 6, 2), end: dt(2014, 7, 1), grain: Grain::Day});
    }
}
