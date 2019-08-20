#![deny(warnings)]

use crate::types::{DateTime, Range, TimeSequence};

// Guard against impossible intersections
const INFINITE_FUSE: usize = 1000;

// Return intersections/overlaps of SeqA with SeqB
//
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
// - Mondays of February
// - Monday 28th

#[derive(Clone)]
pub struct Intersect<SeqA, SeqB>(pub SeqA, pub SeqB)
    where SeqA: TimeSequence,
          SeqB: TimeSequence;

impl<SeqA, SeqB> Intersect<SeqA, SeqB>
    where SeqA: TimeSequence,
          SeqB: TimeSequence
{
    fn _base(&self, t0: &DateTime, future: bool) -> Box<dyn Iterator<Item=Range> + '_> {
        let (mut astream, mut bstream) = if future {
            (self.0._future_raw(t0), self.1._future_raw(t0))
        } else {
            (self.0._past_raw(t0), self.1._past_raw(t0))
        };
        let mut anext = astream.next().unwrap();
        let mut bnext = bstream.next().unwrap();
        Box::new((0..).map(move |_| {
            for _ in 0..INFINITE_FUSE {
                let overlap = anext.intersect(&bnext);
                if (anext.end <= bnext.end && future) ||
                   (anext.start >= bnext.start && !future) {
                    anext = astream.next().unwrap();
                } else {
                    bnext = bstream.next().unwrap();
                }
                if let Some(overlap) = overlap {
                    return overlap;
                }
            }
            panic!("Intersect INFINITE_FUSE blown");
        }))
    }
}

impl<SeqA, SeqB> TimeSequence for Intersect<SeqA, SeqB>
    where SeqA: TimeSequence,
          SeqB: TimeSequence
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
    use crate::seq_union::Union;

    fn dt(year: i32, month: u32, day: u32) -> DateTime {
        Date::from_ymd(year, month, day).and_hms(0, 0, 0)
    }

    fn dttm(year: i32, month: u32, day: u32, h: u32, m: u32, s: u32) -> DateTime {
        Date::from_ymd(year, month, day).and_hms(h, m, s)
    }

    #[test]
    fn intersect_basic() {
        // monday 28th
        let twenty8th = NthOf(28, Grains(Grain::Day), Grains(Grain::Month));
        let mon28th = Intersect(Weekday(1), twenty8th);

        let mut it_mon28th = mon28th.future(&dt(2016, 2, 25));
        assert_eq!(it_mon28th.next().unwrap(),
            Range{start: dt(2016, 3, 28), end: dt(2016, 3, 29), grain: Grain::Day});
        assert_eq!(it_mon28th.next().unwrap(),
            Range{start: dt(2016, 11, 28), end: dt(2016, 11, 29), grain: Grain::Day});
        assert_eq!(it_mon28th.next().unwrap(),
            Range{start: dt(2017, 8, 28), end: dt(2017, 8, 29), grain: Grain::Day});

        // backward: monday 28th
        let mut it_mon28th = mon28th.past(&dt(2016, 2, 25));
        assert_eq!(it_mon28th.next().unwrap(),
            Range{start: dt(2015, 12, 28), end: dt(2015, 12, 29), grain: Grain::Day});
        assert_eq!(it_mon28th.next().unwrap(),
            Range{start: dt(2015, 9, 28), end: dt(2015, 9, 29), grain: Grain::Day});

        // past-non-inclusive and range-end <= t0 .. so can't be 2015-12-28
        let mut it_mon28th = mon28th.past(&dttm(2015, 12, 28, 1, 0, 0));
        assert_eq!(it_mon28th.next().unwrap(),
            Range{start: dt(2015, 9, 28), end: dt(2015, 9, 29), grain: Grain::Day});
        // past-inclusive, should include 2015-12-28 cause range-start <= t0 < end
        let mut it_mon28th = mon28th._past_raw(&dttm(2015, 12, 28, 1, 0, 0));
        assert_eq!(it_mon28th.next().unwrap(),
            Range{start: dt(2015, 12, 28), end: dt(2015, 12, 29), grain: Grain::Day});
    }

    #[test]
    fn intersect2() {
        // tuesdays 3pm
        let mut tue3pm = Intersect(Weekday(2),
            NthOf(16, Grains(Grain::Hour), Grains(Grain::Day)))
                .future(&dt(2016, 2, 25));
        assert_eq!(tue3pm.next().unwrap(),
            Range{start: dttm(2016, 3, 1, 15, 0, 0),
                  end: dttm(2016, 3, 1, 16, 0, 0), grain: Grain::Hour});
        assert_eq!(tue3pm.next().unwrap(),
           Range{start: dttm(2016, 3, 8, 15, 0, 0),
                 end: dttm(2016, 3, 8, 16, 0, 0), grain: Grain::Hour});
        assert_eq!(tue3pm.next().unwrap(),
           Range{start: dttm(2016, 3, 15, 15, 0, 0),
                 end: dttm(2016, 3, 15, 16, 0, 0), grain: Grain::Hour});

        // thursdays of june
        let mut junthurs = Intersect(Weekday(4), Month(6)).future(&dt(2016, 2, 25));
        assert_eq!(junthurs.next().unwrap(),
            Range{start: dt(2016, 6, 2), end: dt(2016, 6, 3), grain: Grain::Day});
        assert_eq!(junthurs.next().unwrap(),
            Range{start: dt(2016, 6, 9), end: dt(2016, 6, 10), grain: Grain::Day});
        assert_eq!(junthurs.next().unwrap(),
            Range{start: dt(2016, 6, 16), end: dt(2016, 6, 17), grain: Grain::Day});
        assert_eq!(junthurs.next().unwrap(),
            Range{start: dt(2016, 6, 23), end: dt(2016, 6, 24), grain: Grain::Day});
        assert_eq!(junthurs.next().unwrap(),
            Range{start: dt(2016, 6, 30), end: dt(2016, 7, 1), grain: Grain::Day});
        assert_eq!(junthurs.next().unwrap(),
            Range{start: dt(2017, 6, 1), end: dt(2017, 6, 2), grain: Grain::Day});
    }


    #[test]
    fn intersect_union() {
        // mondays + wednesdays of June
        let monwedjune = Intersect(Union(Weekday(1), Weekday(3)), Month(6));
        let mut fut = monwedjune.future(&dt(2016, 2, 25));
        assert_eq!(fut.next().unwrap(),
            Range{start: dt(2016, 6, 1), end: dt(2016, 6, 2), grain: Grain::Day});
        assert_eq!(fut.next().unwrap(),
            Range{start: dt(2016, 6, 6), end: dt(2016, 6, 7), grain: Grain::Day});
        assert_eq!(fut.next().unwrap(),
            Range{start: dt(2016, 6, 8), end: dt(2016, 6, 9), grain: Grain::Day});
        let mut fut = fut.skip(6);
        assert_eq!(fut.next().unwrap(),
            Range{start: dt(2017, 6, 5), end: dt(2017, 6, 6), grain: Grain::Day});
    }
}
