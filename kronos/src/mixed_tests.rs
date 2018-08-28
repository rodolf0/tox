#![deny(warnings)]

use types::{DateTime, Date, Grain, Range, TimeSequence};

use seq_nthof::*;
use seq_intersect::*;
use seq_grain::*;
use seq_named::*;
use seq_func::*;


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
