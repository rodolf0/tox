#![deny(warnings)]

type DateTime = chrono::NaiveDateTime;

use crate::time_semantics::{TimeEl, TimeMachine};
use kronos::Grain as g;

fn d(year: i32, month: u32, day: u32) -> DateTime {
    use chrono::naive::NaiveDate as Date;
    Date::from_ymd(year, month, day).and_hms(0, 0, 0)
}

fn r(s: DateTime, e: DateTime, gr: kronos::Grain) -> Vec<TimeEl> {
    vec![TimeEl::Time(kronos::Range {
        start: s,
        end: e,
        grain: gr,
    })]
}

#[test]
fn t_thisnext() {
    let tm = TimeMachine::new(d(2016, 9, 5));
    assert_eq!(tm.eval("next monday"), r(d(2016, 9, 12), d(2016, 9, 13), g::Day));
    assert_eq!(tm.eval("this monday"), r(d(2016, 9, 5), d(2016, 9, 6), g::Day));
    assert_eq!(tm.eval("next march"), r(d(2017, 3, 1), d(2017, 4, 1), g::Month));
    assert_eq!(tm.eval("this march"), r(d(2017, 3, 1), d(2017, 4, 1), g::Month));
    assert_eq!(tm.eval("next month"), r(d(2016, 10, 1), d(2016, 11, 1), g::Month));
    assert_eq!(tm.eval("tue after next"), r(d(2016, 9, 13), d(2016, 9, 14), g::Day));

    let tm = TimeMachine::new(d(2016, 3, 5));
    assert_eq!(tm.eval("this march"), r(d(2016, 3, 1), d(2016, 4, 1), g::Month));
    assert_eq!(tm.eval("next year"), r(d(2017, 1, 1), d(2018, 1, 1), g::Year));
    assert_eq!(tm.eval("next week"), r(d(2016, 3, 6), d(2016, 3, 13), g::Week));
}

#[test]
fn t_direct() {
    let tm = TimeMachine::new(d(2016, 9, 5));
    assert_eq!(tm.eval("2002"), r(d(2002, 1, 1), d(2003, 1, 1), g::Year));
    assert_eq!(tm.eval("the 12th"), r(d(2016, 9, 12), d(2016, 9, 13), g::Day));

    let tm = TimeMachine::new(d(2016, 10, 26));
    assert_eq!(tm.eval("monday"), r(d(2016, 10, 31), d(2016, 11, 1), g::Day));
    assert_eq!(tm.eval("today"), r(d(2016, 10, 26), d(2016, 10, 27), g::Day));
    assert_eq!(tm.eval("tomorrow"), r(d(2016, 10, 27), d(2016, 10, 28), g::Day));

    let tm = TimeMachine::new(d(2016, 9, 12));
    assert_eq!(tm.eval("the 12th"), r(d(2016, 9, 12), d(2016, 9, 13), g::Day));
}

#[test]
fn t_nthof() {
    let tm = TimeMachine::new(d(2016, 9, 5));
    assert_eq!(tm.eval("the 3rd mon of june"), r(d(2017, 6, 19), d(2017, 6, 20), g::Day));
    assert_eq!(tm.eval("the 3rd day of the month"), r(d(2016, 9, 3), d(2016, 9, 4), g::Day));
    assert_eq!(tm.eval("the 2nd week of august"), r(d(2017, 8, 6), d(2017, 8, 13), g::Week));
    assert_eq!(tm.eval("the 3rd day of the 2nd week of may"), r(d(2017, 5, 9), d(2017, 5, 10), g::Day));
    assert_eq!(tm.eval("2nd week of june after next"), r(d(2018, 6, 3), d(2018, 6, 10), g::Week));
    assert_eq!(tm.eval("third tuesday of the month after next"), r(d(2016, 10, 18), d(2016, 10, 19), g::Day));

    let tm = TimeMachine::new(d(2017, 1, 1));
    assert_eq!(tm.eval("the 8th fri of the year"), r(d(2017, 2, 24), d(2017, 2, 25), g::Day));

    let tm = TimeMachine::new(d(2020, 1, 1));
    assert_eq!(tm.eval("the last day of feb"), r(d(2020, 2, 29), d(2020, 3, 1), g::Day));
}

#[test]
fn t_intersect() {
    let tm = TimeMachine::new(d(2016, 9, 5));
    assert_eq!(tm.eval("feb 27th 1984"), r(d(1984, 2, 27), d(1984, 2, 28), g::Day));
    assert_eq!(tm.eval("mon feb 28th"), r(d(2022, 2, 28), d(2022, 3, 1), g::Day));

    let tm = TimeMachine::new(d(2016, 10, 24));
    assert_eq!(tm.eval("friday 18th"), r(d(2016, 11, 18), d(2016, 11, 19), g::Day));
    assert_eq!(tm.eval("18th of june"), r(d(2017, 6, 18), d(2017, 6, 19), g::Day));
    assert_eq!(tm.eval("feb 27th"), r(d(2017, 2, 27), d(2017, 2, 28), g::Day));
}

#[test]
fn t_anchored() {
    let tm = TimeMachine::new(d(2016, 9, 5));
    assert_eq!(tm.eval("the 10th week of 1984"), r(d(1984, 3, 4), d(1984, 3, 11), g::Week));
    assert_eq!(tm.eval("the 2nd day of the 3rd week of 1987"), r(d(1987, 1, 12), d(1987, 1, 13), g::Day));
}

#[test]
fn t_timediff() {
    let tm = TimeMachine::new(d(2016, 9, 5));
    // until
    assert_eq!(tm.eval("days until tomorrow"), vec![TimeEl::Count(1)]);
    assert_eq!(tm.eval("months until 2018"), vec![TimeEl::Count(16)]);
    assert_eq!(tm.eval("weeks until dec"), vec![TimeEl::Count(13)]);
    // since
    assert_eq!(tm.eval("feb 29th since 2000"), vec![TimeEl::Count(5)]);
    assert_eq!(tm.eval("years since 2000"), vec![TimeEl::Count(17)]);
    assert_eq!(tm.eval("days since sep"), vec![TimeEl::Count(4)]); // TODO: CHECK 5?
    // between
    assert_eq!(tm.eval("days between mar and apr"), vec![TimeEl::Count(31)]);

    let tm = TimeMachine::new(d(2016, 10, 25));
    assert_eq!(tm.eval("mon until nov 14th"), vec![TimeEl::Count(2)]);
}

#[test]
fn t_shifts() {
    let tm = TimeMachine::new(d(2016, 10, 26));
    assert_eq!(tm.eval("2 weeks ago"), r(d(2016, 10, 12), d(2016, 10, 13), g::Day));
    assert_eq!(tm.eval("a week after feb 14th"), r(d(2017, 2, 21), d(2017, 2, 22), g::Day));
    assert_eq!(tm.eval("a week before feb 28th"), r(d(2017, 2, 21), d(2017, 2, 22), g::Day));
    assert_eq!(tm.eval("in a year"), r(d(2017, 10, 26), d(2017, 10, 27), g::Day));
}
