#![deny(warnings)]

use utils;
use types::{Date, DateTime, Duration, Range, Grain, TimeSequence};


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
    fn _future_raw(&self, t0: &DateTime) -> Box<Iterator<Item=Range>> {
        self._base(t0, true)
    }

    fn _past_raw(&self, t0: &DateTime) -> Box<Iterator<Item=Range>> {
        self._base(t0, false)
    }
}


#[derive(Clone)]
pub struct Weekend;

impl Weekend {
    fn _base(&self, t0: &DateTime, future: bool) -> Box<Iterator<Item=Range>> {
        let base = utils::find_weekend(t0.date(), future).and_hms(0, 0, 0);
        let sign = if future { 1 } else { -1 };
        Box::new((0..).map(move |x| Range{
            start: base + Duration::days(sign * x * 7),
            end: base + Duration::days(sign * x * 7 + 2),
            grain: Grain::Day
        }))
    }
}

impl<'a> TimeSequence<'a> for Weekend {
    fn _future_raw(&self, t0: &DateTime) -> Box<Iterator<Item=Range>> {
        self._base(t0, true)
    }

    fn _past_raw(&self, t0: &DateTime) -> Box<Iterator<Item=Range>> {
        self._base(t0, false)
    }
}


#[derive(Clone)]
pub struct Year(pub i32);

impl<'a> TimeSequence<'a> for Year {
    fn _future_raw(&self, _: &DateTime) -> Box<Iterator<Item=Range>> {
        use std::iter;
        Box::new(iter::once(Range{
            start: Date::from_ymd(self.0, 1, 1).and_hms(0, 0, 0),
            end: Date::from_ymd(self.0 + 1, 1, 1).and_hms(0, 0, 0),
            grain: Grain::Year
        }))
    }

    fn _past_raw(&self, t0: &DateTime) -> Box<Iterator<Item=Range>> {
        self._future_raw(t0)
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use types::{Date, Grain};

    fn dt(year: i32, month: u32, day: u32) -> DateTime {
        Date::from_ymd(year, month, day).and_hms(0, 0, 0)
    }

    #[test]
    fn weekday() {
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
    fn month() {
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

    #[test]
    fn weekend() {
        // start from a Wednesday
        let mut weekend = Weekend.future(&dt(2016, 3, 23));
        assert_eq!(weekend.next().unwrap(),
            Range{start: dt(2016, 3, 26), end: dt(2016, 3, 28), grain: Grain::Day});
        assert_eq!(weekend.next().unwrap(),
            Range{start: dt(2016, 4, 2), end: dt(2016, 4, 4), grain: Grain::Day});

        // start from Saturday
        let mut weekend = Weekend.future(&dt(2016, 3, 12));
        assert_eq!(weekend.next().unwrap(),
            Range{start: dt(2016, 3, 12), end: dt(2016, 3, 14), grain: Grain::Day});

        // start from Sunday
        let mut weekend = Weekend.future(&dt(2016, 3, 20));
        assert_eq!(weekend.next().unwrap(),
            Range{start: dt(2016, 3, 19), end: dt(2016, 3, 21), grain: Grain::Day});

        // from Sunday going backward
        let mut weekend = Weekend.past(&dt(2016, 3, 20));
        assert_eq!(weekend.next().unwrap(),
            Range{start: dt(2016, 3, 12), end: dt(2016, 3, 14), grain: Grain::Day});

        // from Sunday going backward raw
        let mut weekend = Weekend._past_raw(&dt(2016, 3, 20));
        assert_eq!(weekend.next().unwrap(),
            Range{start: dt(2016, 3, 19), end: dt(2016, 3, 21), grain: Grain::Day});
    }
}
