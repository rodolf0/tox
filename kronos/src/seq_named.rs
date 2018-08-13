#![deny(warnings)]

use utils;
use types::{DateTime, Duration, Range, Grain, TimeSequence};


#[derive(Clone)]
pub struct Weekday(pub u32);

impl Weekday {
    fn _base(&self, t0: &DateTime, future: bool) -> Box<Iterator<Item=Range>> {
        let base = utils::find_dow(t0.date(), self.0, future).and_hms(0, 0, 0);
        let sign = if future { 1 } else { -1 };
        Box::new((0..).map(move |x| Range{
            start: base + Duration::days(sign * x * 7),
            end: base + Duration::days(sign * x * 7 + 1),
            grain: Grain::Day
        }))
    }
}

impl<'a> TimeSequence<'a> for Weekday {
    fn grain(&self) -> Grain { Grain::Day }

    fn _future_raw(&self, t0: &DateTime) -> Box<Iterator<Item=Range>> {
        self._base(t0, true)
    }

    fn _past_raw(&self, t0: &DateTime) -> Box<Iterator<Item=Range>> {
        self._base(t0, false)
    }
}


#[derive(Clone)]
pub struct Month(pub u32);

impl Month {
    fn _base(&self, t0: &DateTime, future: bool) -> Box<Iterator<Item=Range>> {
        let base = utils::truncate(*t0, Grain::Month).date();
        let base = utils::find_month(base, self.0, future).and_hms(0, 0, 0);
        let sign = if future { 1 } else { -1 };
        Box::new((0..).map(move |x| Range{
            start: utils::shift_datetime(base, Grain::Month, sign * 12 * x),
            end: utils::shift_datetime(base, Grain::Month, sign * 12 * x + 1),
            grain: Grain::Month
        }))
    }
}

impl<'a> TimeSequence<'a> for Month {
    fn grain(&self) -> Grain { Grain::Month }

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
fn test_weekday() {
    let monday = Weekday(1);
    let t0_sunday = dt(2018, 8, 5);
    let t0_monday = dt(2018, 8, 6);
    let t0_tuesday = dt(2018, 8, 7);

    // Standing on monday, next monday is same day
    let it = monday.future(&t0_monday).next().unwrap();
    assert_eq!(it, Range{
        start: dt(2018, 8, 6), end: dt(2018, 8, 7), grain: Grain::Day});

    // Standing on monday, prev monday is same day
    let it = monday._past_raw(&t0_monday).next().unwrap();
    assert_eq!(it, Range{
        start: dt(2018, 8, 6), end: dt(2018, 8, 7), grain: Grain::Day});
    // if non-inclusive then prev monday
    let it = monday.past(&t0_monday).next().unwrap();
    assert_eq!(it, Range{
        start: dt(2018, 7, 30), end: dt(2018, 7, 31), grain: Grain::Day});

    // Standing on tuesday, prev monday is a day before
    let it = monday._past_raw(&t0_tuesday).next().unwrap();
    assert_eq!(it, Range{
        start: dt(2018, 8, 6), end: dt(2018, 8, 7), grain: Grain::Day});
    let it = monday.past(&t0_tuesday).next().unwrap();
    assert_eq!(it, Range{
        start: dt(2018, 8, 6), end: dt(2018, 8, 7), grain: Grain::Day});

    // Standing on sunday, prev monday is a week before
    let it = monday._past_raw(&t0_sunday).next().unwrap();
    assert_eq!(it, Range{
        start: dt(2018, 7, 30), end: dt(2018, 7, 31), grain: Grain::Day});
    let it = monday.past(&t0_sunday).next().unwrap();
    assert_eq!(it, Range{
        start: dt(2018, 7, 30), end: dt(2018, 7, 31), grain: Grain::Day});
}

#[test]
fn test_month() {
    let t0_april = dt(2018, 4, 3);
    let t0_may = dt(2018, 5, 1);
    let t0_march = dt(2018, 3, 12);

    // Standing on april, next april is same month
    let it = Month(4).future(&t0_april).next().unwrap();
    assert_eq!(it, Range{
        start: dt(2018, 4, 1), end: dt(2018, 5, 1), grain: Grain::Month});

    // Standing on april, prev april is same month
    let it = Month(4)._past_raw(&t0_april).next().unwrap();
    assert_eq!(it, Range{
        start: dt(2018, 4, 1), end: dt(2018, 5, 1), grain: Grain::Month});
    // if non-inclusive, standing on april, past apr is in 2017
    let it = Month(4).past(&t0_april).next().unwrap();
    assert_eq!(it, Range{
        start: dt(2017, 4, 1), end: dt(2017, 5, 1), grain: Grain::Month});

    // Standing on may, prev april is prev month
    let it = Month(4)._past_raw(&t0_may).next().unwrap();
    assert_eq!(it, Range{
        start: dt(2018, 4, 1), end: dt(2018, 5, 1), grain: Grain::Month});
    let it = Month(4).past(&t0_may).next().unwrap();
    assert_eq!(it, Range{
        start: dt(2018, 4, 1), end: dt(2018, 5, 1), grain: Grain::Month});

    // Standing on march, prev april is prev year
    let it = Month(4)._past_raw(&t0_march).next().unwrap();
    assert_eq!(it, Range{
        start: dt(2017, 4, 1), end: dt(2017, 5, 1), grain: Grain::Month});
    let it = Month(4).past(&t0_march).next().unwrap();
    assert_eq!(it, Range{
        start: dt(2017, 4, 1), end: dt(2017, 5, 1), grain: Grain::Month});
}
