#![deny(warnings)]

use types::{DateTime, Range, Grain, TimeSequence};

// Alternates SeqA and SeqB depending on what happens first
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
pub struct Union<SeqA, SeqB>(SeqA, SeqB);

impl<SeqA, SeqB> Union<SeqA, SeqB>
    where for<'b> SeqA: TimeSequence<'b>,
          for<'b> SeqB: TimeSequence<'b>
{
    fn _base(&self, t0: &DateTime, future: bool) -> Box<Iterator<Item=Range>> {
        let (mut astream, mut bstream) = if future {
            (self.0._future_raw(t0).peekable(),
             self.1._future_raw(t0).peekable())
        } else {
            (self.0._past_raw(t0).peekable(), self.1._past_raw(t0).peekable())
        };
        Box::new((0..).map(move |_| {
            let anext = astream.peek().unwrap().clone();
            let bnext = bstream.peek().unwrap().clone();
            if (anext.start <= bnext.start && future) ||
               (anext.start > bnext.start && !future) {
                astream.next();
                anext
            } else {
                bstream.next();
                bnext
            }
        }))
    }
}

impl<'a, SeqA, SeqB> TimeSequence<'a> for Union<SeqA, SeqB>
    where for<'b> SeqA: TimeSequence<'b>,
          for<'b> SeqB: TimeSequence<'b>
{
    // NOTE: resolution of A and B don't need to be the same, check Range items
    // TODO: get rid of resolution? or return Resolution::Mixed
    fn resolution(&self) -> Grain { self.1.resolution() }

    fn _future_raw(&self, t0: &DateTime) -> Box<Iterator<Item=Range>> {
        self._base(t0, true)
    }

    fn _past_raw(&self, t0: &DateTime) -> Box<Iterator<Item=Range>> {
        self._base(t0, false)
    }
}


#[cfg(test)]
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

    // past-inclusive/raw
    let monwed = Union(Weekday(1), Weekday(3));
    let mut monwedfri = Union(monwed, Weekday(5))._past_raw(&dt(2015, 2, 27));
    assert_eq!(monwedfri.next().unwrap(),
        Range{start: dt(2015, 2, 27), end: dt(2015, 2, 28), grain: Grain::Day});
}
