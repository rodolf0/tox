#![deny(warnings)]

extern crate chrono;
type DateTime = chrono::NaiveDateTime;

use time_machine::{TimeMachine, TimeEl};
use kronos::Grain as g;
use kronos;


fn d(year: i32, month: u32, day: u32) -> DateTime {
    use chrono::naive::NaiveDate as Date;
    Date::from_ymd(year, month, day).and_hms(0, 0, 0)
}
fn r(s: DateTime, e: DateTime, gr: kronos::Grain) -> TimeEl {
    TimeEl::Time(kronos::Range{start: s, end: e, grain: gr})
}

#[test]
fn t_thisnext() {
    let tm = TimeMachine::new();
    let x = r(d(2016, 9, 12), d(2016, 9, 13), g::Day);
    assert_eq!(tm.eval1(d(2016, 9, 5), "next monday"), x);
    let x = r(d(2016, 9, 5), d(2016, 9, 6), g::Day);
    assert_eq!(tm.eval1(d(2016, 9, 5), "this monday"), x);
    let x = r(d(2017, 3, 1), d(2017, 4, 1), g::Month);
    assert_eq!(tm.eval1(d(2016, 9, 5), "next march"), x);
    assert_eq!(tm.eval1(d(2016, 9, 5), "this march"), x);
    let x = r(d(2016, 3, 1), d(2016, 4, 1), g::Month);
    assert_eq!(tm.eval1(d(2016, 3, 5), "this march"), x);
    let x = r(d(2017, 1, 1), d(2018, 1, 1), g::Year);
    assert_eq!(tm.eval1(d(2016, 3, 5), "next year"), x);
    let x = r(d(2016, 3, 6), d(2016, 3, 13), g::Week);
    assert_eq!(tm.eval1(d(2016, 3, 5), "next week"), x);
    let x = r(d(2016, 10, 1), d(2016, 11, 1), g::Month);
    assert_eq!(tm.eval1(d(2016, 9, 5), "next month"), x);
    let x = r(d(2016, 9, 13), d(2016, 9, 14), g::Day);
    assert_eq!(tm.eval1(d(2016, 9, 5), "tue after next"), x);
}

#[test]
fn t_direct() {
    let tm = TimeMachine::new();
    assert_eq!(tm.eval1(d(2016, 9, 5), "2002"),
               r(d(2002, 1, 1), d(2003, 1, 1), g::Year));
    assert_eq!(tm.eval1(d(2016, 10, 26), "monday"),
               r(d(2016, 10, 31), d(2016, 11, 1), g::Day));
    assert_eq!(tm.eval1(d(2016, 10, 26), "today"),
               r(d(2016, 10, 26), d(2016, 10, 27), g::Day));
    assert_eq!(tm.eval1(d(2016, 10, 25), "tomorrow"),
               r(d(2016, 10, 26), d(2016, 10, 27), g::Day));
    assert_eq!(tm.eval1(d(2016, 9, 5), "the 12th"),
               r(d(2016, 9, 12), d(2016, 9, 13), g::Day));
    assert_eq!(tm.eval1(d(2016, 9, 12), "the 12th"),
               r(d(2016, 9, 12), d(2016, 9, 13), g::Day));
}

#[test]
fn t_nthof() {
    let tm = TimeMachine::new();
    assert_eq!(tm.eval1(d(2016, 9, 5), "the 3rd mon of june"),
               r(d(2017, 6, 19), d(2017, 6, 20), g::Day));
    assert_eq!(tm.eval1(d(2016, 9, 5), "the 3rd day of the month"),
               r(d(2016, 9, 3), d(2016, 9, 4), g::Day));
    assert_eq!(tm.eval1(d(2016, 9, 5), "the 2nd week of august"),
               r(d(2017, 8, 6), d(2017, 8, 13), g::Week));
    assert_eq!(tm.eval1(d(2017, 1, 1), "the 8th fri of the year"),
               r(d(2017, 2, 24), d(2017, 2, 25), g::Day));
    assert_eq!(tm.eval1(d(2020, 1, 1), "the last day of feb"),
               r(d(2020, 2, 29), d(2020, 3, 1), g::Day));
    assert_eq!(tm.eval1(d(2016, 9, 5), "the 3rd day of the 2nd week of may"),
               r(d(2017, 5, 9), d(2017, 5, 10), g::Day));
    let x = tm.eval(d(2016, 9, 5), "2nd week of june after next");
    assert_eq!(x[0], r(d(2018, 6, 3), d(2018, 6, 10), g::Week));
    assert_eq!(x[1], r(d(2018, 6, 3), d(2018, 6, 10), g::Week));
    assert_eq!(tm.eval1(d(2016, 9, 5), "2nd day of june 2014"),
               r(d(2014, 6, 2), d(2014, 6, 3), g::Day));
    assert_eq!(tm.eval1(d(2016, 9, 5), "2nd thu of sep 2014"),
               r(d(2014, 9, 11), d(2014, 9, 12), g::Day));
    let x = tm.eval(d(2016, 9, 5), "third tuesday of the month after next");
    assert_eq!(x.len(), 2);
    assert!(x.contains(&r(d(2016, 11, 15), d(2016, 11, 16), g::Day)));
    assert!(x.contains(&r(d(2016, 10, 18), d(2016, 10, 19), g::Day)));
}

#[test]
fn t_intersect() {
    let tm = TimeMachine::new();
    assert_eq!(tm.eval1(d(2016, 9, 5), "27th feb 1984"),
               r(d(1984, 2, 27), d(1984, 2, 28), g::Day));
    assert_eq!(tm.eval1(d(2016, 10, 24), "friday 18th"),
               r(d(2016, 11, 18), d(2016, 11, 19), g::Day));
    assert_eq!(tm.eval1(d(2016, 10, 24), "18th of june"),
               r(d(2017, 6, 18), d(2017, 6, 19), g::Day));
    assert_eq!(tm.eval1(d(2016, 10, 24), "feb 27th"),
               r(d(2017, 2, 27), d(2017, 2, 28), g::Day));
    assert_eq!(tm.eval1(d(2017, 9, 5), "mon feb 28th"),
               r(d(2022, 2, 28), d(2022, 3, 1), g::Day));
}

#[test]
fn t_anchored() {
    let tm = TimeMachine::new();
    assert_eq!(tm.eval1(d(2016, 9, 5), "10th week of 1984"),
               r(d(1984, 3, 4), d(1984, 3, 11), g::Week));
    assert_eq!(tm.eval1(d(2016, 9, 5), "the 2nd day of the 3rd week of 1987"),
               r(d(1987, 1, 12), d(1987, 1, 13), g::Day));
}

#[test]
fn t_timediff() {
    let tm = TimeMachine::new();
    // until
    assert_eq!(tm.eval1(d(2016, 9, 5), "days until tomorrow"), TimeEl::Count(1));
    assert_eq!(tm.eval1(d(2016, 9, 5), "months until 2018"), TimeEl::Count(16));
    assert_eq!(tm.eval1(d(2016, 9, 5), "weeks until dec"), TimeEl::Count(13));
    assert_eq!(tm.eval1(d(2016, 10, 25), "mon until nov 14th"), TimeEl::Count(2));
    // since
    assert_eq!(tm.eval1(d(2016, 9, 5), "feb 29th since 2000"), TimeEl::Count(5));
    assert_eq!(tm.eval1(d(2016, 9, 5), "years since 2000"), TimeEl::Count(17));
    assert_eq!(tm.eval1(d(2016, 9, 5), "days since sep"), TimeEl::Count(5));
    // between
    assert_eq!(tm.eval1(d(2016, 9, 5), "days between mar and apr"), TimeEl::Count(31));
}

#[test]
fn t_shifts() {
    let tm = TimeMachine::new();
    assert_eq!(tm.eval1(d(2016, 10, 26), "2 weeks ago"),
               r(d(2016, 10, 12), d(2016, 10, 13), g::Day));
    assert_eq!(tm.eval1(d(2016, 10, 26), "a week after feb 14th"),
               r(d(2017, 2, 21), d(2017, 2, 22), g::Day));
    assert_eq!(tm.eval1(d(2016, 10, 26), "a week before feb 28th"),
               r(d(2017, 2, 21), d(2017, 2, 22), g::Day));
    assert_eq!(tm.eval1(d(2016, 10, 26), "in a year"),
               r(d(2017, 10, 26), d(2017, 10, 27), g::Day));
}
