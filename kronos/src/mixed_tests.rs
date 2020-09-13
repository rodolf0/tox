#![deny(warnings)]

use crate::types::{DateTime, Date, Grain, Range, TimeSequence};

use crate::seq_nthof::*;
use crate::seq_intersect::*;
use crate::seq_grain::*;
use crate::seq_named::*;
use crate::seq_func::*;


fn dt(year: i32, month: u32, day: u32) -> DateTime {
    Date::from_ymd(year, month, day).and_hms(0, 0, 0)
}

fn dttm(year: i32, month: u32, day: u32, h: u32, m: u32, s: u32) -> DateTime {
    Date::from_ymd(year, month, day).and_hms(h, m, s)
}


#[test]
fn test_multi() {
    // 3 days after mon feb 28th
    let seq = NthOf(28, Grains(Grain::Day), Grains(Grain::Month));
    let seq = Intersect(seq, Weekday(1));
    let seq = Intersect(seq, Month(2));
    let seq3dafter = shift(seq.clone(), Grain::Day, 3);

    let mut iter = seq3dafter.future(&dt(2021, 9, 5));
    assert_eq!(iter.next().unwrap(),
        Range{start: dt(2022, 3, 3), end: dt(2022, 3, 4), grain: Grain::Day});
    assert_eq!(iter.next().unwrap(),
        Range{start: dt(2028, 3, 2), end: dt(2028, 3, 3), grain: Grain::Day});

    // backward: 3 days after monday feb 28th
    let mut iter = seq3dafter.past(&dt(2021, 9, 5));
    assert_eq!(iter.next().unwrap(),
        Range{start: dt(2011, 3, 3), end: dt(2011, 3, 4), grain: Grain::Day});
    assert_eq!(iter.next().unwrap(),
        Range{start: dt(2005, 3, 3), end: dt(2005, 3, 4), grain: Grain::Day});

    // edge cases, first end-of-range <= reftime
    let mut iter = seq.past(&dttm(2022, 2, 28, 1, 0, 0));
    assert_eq!(iter.next().unwrap(),
        Range{start: dt(2011, 2, 28), end: dt(2011, 3, 1), grain: Grain::Day});

    let mut iter = seq.past(&dttm(2028, 2, 29, 0, 0, 0));
    assert_eq!(iter.next().unwrap(),
        Range{start: dt(2028, 2, 28), end: dt(2028, 2, 29), grain: Grain::Day});
    assert_eq!(iter.next().unwrap(),
        Range{start: dt(2022, 2, 28), end: dt(2022, 3, 1), grain: Grain::Day});
}

#[test]
fn test_every_nmonths_from_offset() {
    // Ref: https://github.com/rodolf0/tox/pull/8/commits
    let t0_april = dt(2018, 4, 3);

    // Filtering once the iterator has been triggered
    let mut every_3months_iter = Grains(Grain::Month)
        .future(&t0_april)
        .step_by(3);

   assert_eq!(every_3months_iter.next().unwrap(), Range{
        start: dt(2018, 4, 1), end: dt(2018, 5, 1), grain: Grain::Month});

    // Specifying the template
    let seq = step_by(Grains(Grain::Month), 3);
   assert_eq!(seq.future(&t0_april).next().unwrap(), Range{
        start: dt(2018, 4, 1), end: dt(2018, 5, 1), grain: Grain::Month});

   // Every n-months but using an offset
    use chrono::Datelike;
    let mut every_3months_from_next_march_iter = Grains(Grain::Month)
        .future(&t0_april)
        .skip_while(|r| r.start.date().month() != 3)
        .step_by(3);
    assert_eq!(every_3months_from_next_march_iter.next().unwrap(), Range{
        start: dt(2019, 3, 1), end: dt(2019, 4, 1), grain: Grain::Month});
}
