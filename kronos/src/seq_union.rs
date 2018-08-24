#![deny(warnings)]

use types::{DateTime, Range, TimeSequence};

// Alternates SeqA and SeqB depending on what happens first
// Union skips over Ranges totally contained by other sequence
//   |------a------|
//            |------b------|
//
//   |----a----|
//                |-----b-----|
//
//   |---------a---------|
//        |-----b-----|
//
// Exapmles:
// - Mondays and Fridays
// - Weekends and Tuesdays
// - overlapping (2pm to 3pm) and (1pm to 5pm)

#[derive(Clone)]
pub struct Union<SeqA, SeqB>(pub SeqA, pub SeqB);

impl<SeqA, SeqB> Union<SeqA, SeqB>
    where for<'b> SeqA: TimeSequence<'b>,
          for<'b> SeqB: TimeSequence<'b>
{
    fn _base(&self, t0: &DateTime, future: bool) -> Box<Iterator<Item=Range>> {
        let (mut astream, mut bstream) = if future {
            (self.0._future_raw(t0), self.1._future_raw(t0))
        } else {
            (self.0._past_raw(t0), self.1._past_raw(t0))
        };
        let mut anext = astream.next().unwrap();
        let mut bnext = bstream.next().unwrap();
        Box::new((0..).map(move |_| {
            if (anext.start <= bnext.start && future) ||
               (anext.start > bnext.start && !future) {
                // advance included bstream until out of shadow of astream
                while (bnext.end <= anext.end && future) ||
                      (bnext.start >= anext.start && !future) {
                    bnext = bstream.next().unwrap();
                }
                let unionret = anext.clone();
                anext = astream.next().unwrap();
                unionret
            } else {
                // advance included astream until out of shadow of bstream
                while (anext.end <= bnext.end && future) ||
                      (anext.start >= bnext.start && !future) {
                    anext = astream.next().unwrap();
                }
                let unionret = bnext.clone();
                bnext = bstream.next().unwrap();
                unionret
            }
        }))
    }
}

impl<'a, SeqA, SeqB> TimeSequence<'a> for Union<SeqA, SeqB>
    where for<'b> SeqA: TimeSequence<'b>,
          for<'b> SeqB: TimeSequence<'b>
{
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
    use types::Grain;

    fn dt(year: i32, month: u32, day: u32) -> DateTime {
        use types::Date;
        Date::from_ymd(year, month, day).and_hms(0, 0, 0)
    }

    #[test]
    fn test_union() {
        use seq_named::Weekday;

        let mut monwed = Union(Weekday(1), Weekday(3)).future(&dt(2015, 2, 27));
        assert_eq!(monwed.next().unwrap(),
            Range{start: dt(2015, 3, 2), end: dt(2015, 3, 3), grain: Grain::Day});
        assert_eq!(monwed.next().unwrap(),
            Range{start: dt(2015, 3, 4), end: dt(2015, 3, 5), grain: Grain::Day});
        assert_eq!(monwed.next().unwrap(),
            Range{start: dt(2015, 3, 9), end: dt(2015, 3, 10), grain: Grain::Day});

        let monwed = Union(Weekday(1), Weekday(3));
        let mut monwedfri = Union(monwed, Weekday(5)).future(&dt(2015, 2, 27));
        assert_eq!(monwedfri.next().unwrap(),
            Range{start: dt(2015, 2, 27), end: dt(2015, 2, 28), grain: Grain::Day});
        assert_eq!(monwedfri.next().unwrap(),
            Range{start: dt(2015, 3, 2), end: dt(2015, 3, 3), grain: Grain::Day});
        assert_eq!(monwedfri.next().unwrap(),
            Range{start: dt(2015, 3, 4), end: dt(2015, 3, 5), grain: Grain::Day});
    }

    #[test]
    fn test_union_past() {
        use seq_named::Weekday;

        let mut monwed = Union(Weekday(1), Weekday(3)).past(&dt(2015, 2, 27));
        assert_eq!(monwed.next().unwrap(),
            Range{start: dt(2015, 2, 25), end: dt(2015, 2, 26), grain: Grain::Day});
        assert_eq!(monwed.next().unwrap(),
            Range{start: dt(2015, 2, 23), end: dt(2015, 2, 24), grain: Grain::Day});

        let monwed = Union(Weekday(1), Weekday(3));
        let mut monwedfri = Union(monwed, Weekday(5)).past(&dt(2015, 2, 27));
        assert_eq!(monwedfri.next().unwrap(),
            Range{start: dt(2015, 2, 25), end: dt(2015, 2, 26), grain: Grain::Day});
        assert_eq!(monwedfri.next().unwrap(),
            Range{start: dt(2015, 2, 23), end: dt(2015, 2, 24), grain: Grain::Day});
        assert_eq!(monwedfri.next().unwrap(),
            Range{start: dt(2015, 2, 20), end: dt(2015, 2, 21), grain: Grain::Day});

        // past-inclusive/raw
        let monwed = Union(Weekday(1), Weekday(3));
        let mut monwedfri = Union(monwed, Weekday(5))._past_raw(&dt(2015, 2, 27));
        assert_eq!(monwedfri.next().unwrap(),
            Range{start: dt(2015, 2, 27), end: dt(2015, 2, 28), grain: Grain::Day});
    }

    #[test]
    fn test_diff_resolution() {
        use seq_named::{Month, Weekday};

        let mut mon_or_march = Union(Weekday(1), Month(3)).future(&dt(2015, 2, 27));
        assert_eq!(mon_or_march.next().unwrap(),
            Range{start: dt(2015, 3, 1), end: dt(2015, 4, 1), grain: Grain::Month});
        assert_eq!(mon_or_march.next().unwrap(),
            Range{start: dt(2015, 4, 6), end: dt(2015, 4, 7), grain: Grain::Day});
        assert_eq!(mon_or_march.next().unwrap(),
            Range{start: dt(2015, 4, 13), end: dt(2015, 4, 14), grain: Grain::Day});
        let mut mon_or_march = mon_or_march.skip(46);
        assert_eq!(mon_or_march.next().unwrap(),
            Range{start: dt(2016, 3, 1), end: dt(2016, 4, 1), grain: Grain::Month});
    }
}
