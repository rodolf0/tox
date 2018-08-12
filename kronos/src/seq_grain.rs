#![deny(warnings)]

use utils;
use types::{DateTime, Range, Grain, TimeSequence};


#[derive(Clone)]
pub struct Grains(pub Grain);

impl Grains {
    fn _base(&self, t0: &DateTime, future: bool) -> Box<Iterator<Item=Range>> {
        let base = utils::truncate(*t0, self.0);
        let sign = if future { 1 } else { -1 };
        let grain = self.0;
        Box::new((0..).map(move |x| Range{
            start: utils::shift_datetime(base, grain, sign * x),
            end: utils::shift_datetime(base, grain, sign * x + 1),
            grain: grain
        }))
    }
}

impl<'a> TimeSequence<'a> for Grains {
    fn grain(&self) -> Grain { self.0 }

    fn future(&self, t0: &DateTime) -> Box<Iterator<Item=Range>> {
        self._base(t0, true)
    }

    fn past_inclusive(&self, t0: &DateTime) -> Box<Iterator<Item=Range>> {
        self._base(t0, false)
    }
}


#[cfg(test)]
fn dt(year: i32, month: u32, day: u32) -> DateTime {
    use types::Date;
    Date::from_ymd(year, month, day).and_hms(0, 0, 0)
}

#[test]
fn test_seq_grain() {
    let t0_27feb = dt(2015, 2, 27);
    let t0_1jan = dt(2016, 1, 1);

    let mut days = Grains(Grain::Day).future(&t0_27feb);
    assert_eq!(days.next().unwrap(),
        Range{start: dt(2015, 2, 27), end: dt(2015, 2, 28), grain: Grain::Day});
    assert_eq!(days.next().unwrap(),
        Range{start: dt(2015, 2, 28), end: dt(2015, 3, 1), grain: Grain::Day});

    // check "future" englobes date
    let mut weeks = Grains(Grain::Week).future(&t0_1jan);
    assert_eq!(weeks.next().unwrap(),
        Range{start: dt(2015, 12, 27), end: dt(2016, 1, 3), grain: Grain::Week});
    assert_eq!(weeks.next().unwrap(),
        Range{start: dt(2016, 1, 3), end: dt(2016, 1, 10), grain: Grain::Week});

    let mut months = Grains(Grain::Month).future(&t0_27feb);
    assert_eq!(months.next().unwrap(),
        Range{start: dt(2015, 2, 1), end: dt(2015, 3, 1), grain: Grain::Month});
    assert_eq!(months.next().unwrap(),
        Range{start: dt(2015, 3, 1), end: dt(2015, 4, 1), grain: Grain::Month});

    // backward iteration
    let mut years = Grains(Grain::Year).past(&t0_27feb);
    assert_eq!(years.next().unwrap(),
        Range{start: dt(2014, 1, 1), end: dt(2015, 1, 1), grain: Grain::Year});
    assert_eq!(years.next().unwrap(),
        Range{start: dt(2013, 1, 1), end: dt(2014, 1, 1), grain: Grain::Year});
    // if inclusive, past_inclusive renders same year
    let mut years = Grains(Grain::Year).past_inclusive(&t0_27feb);
    assert_eq!(years.next().unwrap(),
        Range{start: dt(2015, 1, 1), end: dt(2016, 1, 1), grain: Grain::Year});
}


#[cfg(test)]
fn dttm(year: i32, month: u32, day: u32, h: u32, m: u32, s: u32) -> DateTime {
    use types::Date;
    Date::from_ymd(year, month, day).and_hms(h, m, s)
}

#[test]
fn test_smaller_grains() {
    let mut minute = Grains(Grain::Minute).future(&dt(2015, 2, 27));
    assert_eq!(minute.next().unwrap(),
        Range{start: dttm(2015, 2, 27, 0, 0, 0),
              end: dttm(2015, 2, 27, 0, 1, 0), grain: Grain::Minute});

    let mut min = Grains(Grain::Minute).past(&dttm(2015, 2, 27, 23, 20, 0));
    assert_eq!(min.next().unwrap(),
        Range{start: dttm(2015, 2, 27, 23, 19, 0),
              end: dttm(2015, 2, 27, 23, 20, 0), grain: Grain::Minute});
    let mut min =
        Grains(Grain::Minute).past_inclusive(&dttm(2015, 2, 27, 23, 20, 0));
    assert_eq!(min.next().unwrap(),
        Range{start: dttm(2015, 2, 27, 23, 20, 0),
              end: dttm(2015, 2, 27, 23, 21, 0), grain: Grain::Minute});

    // non-inclusive past (default)
    let mut min = Grains(Grain::Minute).past(&dttm(2015, 2, 27, 23, 20, 25));
    assert_eq!(min.next().unwrap(),
        Range{start: dttm(2015, 2, 27, 23, 19, 0),
              end: dttm(2015, 2, 27, 23, 20, 0), grain: Grain::Minute});
    // inclusive past
    let mut min =
        Grains(Grain::Minute).past_inclusive(&dttm(2015, 2, 27, 23, 20, 25));
    assert_eq!(min.next().unwrap(),
        Range{start: dttm(2015, 2, 27, 23, 20, 0),
              end: dttm(2015, 2, 27, 23, 21, 0), grain: Grain::Minute});

    let mut minute = Grains(Grain::Minute).past(&dt(2015, 2, 27));
    assert_eq!(minute.next().unwrap(),
        Range{start: dttm(2015, 2, 26, 23, 59, 0),
              end: dttm(2015, 2, 27, 0, 0, 0), grain: Grain::Minute});
}

#[test]
fn test_virtual_grains() {
    let mut quarters = Grains(Grain::Quarter).future(&dt(2015, 2, 27));
    assert_eq!(quarters.next().unwrap(),
        Range{start: dt(2015, 1, 1), end: dt(2015, 4, 1), grain: Grain::Quarter});
    assert_eq!(quarters.next().unwrap(),
        Range{start: dt(2015, 4, 1), end: dt(2015, 7, 1), grain: Grain::Quarter});
}
