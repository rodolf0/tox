#![deny(warnings)]

use types::{DateTime, Range, Grain, TimeSequence};

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
// - Everyday except Fridays
// - Mondays except March
// - March except mondays (never happens: a march without mondays)

#[derive(Clone)]
pub struct Except<SeqA, SeqB>(pub SeqA, pub SeqB);

impl<SeqA, SeqB> Except<SeqA, SeqB>
    where for<'b> SeqA: TimeSequence<'b>,
          for<'b> SeqB: TimeSequence<'b>
{
    fn _base(&self, t0: &DateTime, future: bool) -> Box<Iterator<Item=Range>> {
        let (stream, mut except) = if future {
            (self.0._future_raw(t0), self.1._future_raw(t0))
        } else {
            (self.0._past_raw(t0), self.1._past_raw(t0))
        };
        let mut nexcept = except.next().unwrap();
        Box::new(stream.filter(move |range| {
            // advance exception filter up to current range
            while (nexcept.end < range.start && future) ||
                  (nexcept.start >= range.end && !future) {
                nexcept = except.next().unwrap();
            }
            range.intersect(&nexcept).is_none()
        }))
    }
}

impl<'a, SeqA, SeqB> TimeSequence<'a> for Except<SeqA, SeqB>
    where for<'b> SeqA: TimeSequence<'b>,
          for<'b> SeqB: TimeSequence<'b>
{
    fn resolution(&self) -> Grain { self.0.resolution() }

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
fn test_except() {
    use seq_grain::Grains;
    use seq_named::Weekday;

    // days except Friday and thursdays
    let except = Except(Except(Grains(Grain::Day), Weekday(5)), Weekday(4));
    let mut fut = except.future(&dt(2018, 8, 22));
    assert_eq!(fut.next().unwrap(),
        Range{start: dt(2018, 8, 22), end: dt(2018, 8, 23), grain: Grain::Day});
    assert_eq!(fut.next().unwrap(),
        Range{start: dt(2018, 8, 25), end: dt(2018, 8, 26), grain: Grain::Day});
    assert_eq!(fut.next().unwrap(),
        Range{start: dt(2018, 8, 26), end: dt(2018, 8, 27), grain: Grain::Day});

    let mut past = except.past(&dt(2018, 8, 19));
    assert_eq!(past.next().unwrap(),
        Range{start: dt(2018, 8, 18), end: dt(2018, 8, 19), grain: Grain::Day});
    assert_eq!(past.next().unwrap(),
        Range{start: dt(2018, 8, 15), end: dt(2018, 8, 16), grain: Grain::Day});

    let mut past = except.past(&dt(2018, 8, 17));
    assert_eq!(past.next().unwrap(),
        Range{start: dt(2018, 8, 15), end: dt(2018, 8, 16), grain: Grain::Day});
}


#[test]
fn test_except_diff_grains() {
    use seq_named::{Weekday, Month};

    // mondays except september
    let except = Except(Weekday(1), Month(9));
    let mut fut = except.future(&dt(2018, 8, 22));
    assert_eq!(fut.next().unwrap(),
        Range{start: dt(2018, 8, 27), end: dt(2018, 8, 28), grain: Grain::Day});
    assert_eq!(fut.next().unwrap(),
        Range{start: dt(2018, 10, 1), end: dt(2018, 10, 2), grain: Grain::Day});

    // mondays except August - past
    let except = Except(Weekday(1), Month(8));
    let mut past = except.past(&dt(2018, 8, 22));
    assert_eq!(past.next().unwrap(),
        Range{start: dt(2018, 7, 30), end: dt(2018, 7, 31), grain: Grain::Day});
}
