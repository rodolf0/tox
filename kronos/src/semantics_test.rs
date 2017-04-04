use semantics::{Range, Grain, Seq};
use utils::{DateTime, Date};

fn dt(year: i32, month: u32, day: u32) -> DateTime {
    Date::from_ymd(year, month, day).and_hms(0, 0, 0)
}

fn dttm(year: i32, month: u32, day: u32, h: u32, m: u32, s: u32) -> DateTime {
    Date::from_ymd(year, month, day).and_hms(h, m, s)
}

#[test]
fn test_seq_weekday() {
    let mut sunday = Seq::weekday(0)(dt(2016, 8, 27));
    assert_eq!(sunday.next().unwrap(),
               Range{start: dt(2016, 8, 28), end: dt(2016, 8, 29),
                     grain: Grain::Day});
    assert_eq!(sunday.next().unwrap(),
               Range{start: dt(2016, 9, 4), end: dt(2016, 9, 5),
                     grain: Grain::Day});
}

#[test]
fn test_seq_month() {
    let mut august = Seq::month(8)(dt(2016, 8, 27));
    assert_eq!(august.next().unwrap(),
               Range{start: dt(2016, 8, 1), end: dt(2016, 9, 1),
                     grain: Grain::Month});
    assert_eq!(august.next().unwrap(),
               Range{start: dt(2017, 8, 1), end: dt(2017, 9, 1),
                     grain: Grain::Month});

    let mut february = Seq::month(2)(dt(2016, 8, 27));
    assert_eq!(february.next().unwrap(),
               Range{start: dt(2017, 2, 1), end: dt(2017, 3, 1),
                     grain: Grain::Month});
    assert_eq!(february.next().unwrap(),
               Range{start: dt(2018, 2, 1), end: dt(2018, 3, 1),
                     grain: Grain::Month});
}

#[test]
fn test_seq_weekend() {
    let mut weekend = Seq::weekend()(dt(2016, 3, 23));
    assert_eq!(weekend.next().unwrap(),
               Range{start: dt(2016, 3, 26), end: dt(2016, 3, 28),
                     grain: Grain::Day});
    assert_eq!(weekend.next().unwrap(),
               Range{start: dt(2016, 4, 2), end: dt(2016, 4, 4),
                     grain: Grain::Day});

    let mut weekend = Seq::weekend()(dt(2016, 3, 12));
    assert_eq!(weekend.next().unwrap(),
               Range{start: dt(2016, 3, 12), end: dt(2016, 3, 14),
                     grain: Grain::Day});

    let mut weekend = Seq::weekend()(dt(2016, 3, 20));
    assert_eq!(weekend.next().unwrap(),
               Range{start: dt(2016, 3, 19), end: dt(2016, 3, 21),
                     grain: Grain::Day});
}

#[test]
fn test_seq_grain() {
    let mut days = Seq::from_grain(Grain::Day)(dt(2015, 2, 27));
    assert_eq!(days.next().unwrap(),
               Range{start: dt(2015, 2, 27), end: dt(2015, 2, 28),
                     grain: Grain::Day});
    assert_eq!(days.next().unwrap(),
               Range{start: dt(2015, 2, 28), end: dt(2015, 3, 1),
                     grain: Grain::Day});

    let mut weeks = Seq::from_grain(Grain::Week)(dt(2016, 1, 1));
    assert_eq!(weeks.next().unwrap(),
               Range{start: dt(2015, 12, 27), end: dt(2016, 1, 3),
                     grain: Grain::Week});
    assert_eq!(weeks.next().unwrap(),
               Range{start: dt(2016, 1, 3), end: dt(2016, 1, 10),
                     grain: Grain::Week});

    let mut months = Seq::from_grain(Grain::Month)(dt(2015, 2, 27));
    assert_eq!(months.next().unwrap(),
               Range{start: dt(2015, 2, 1), end: dt(2015, 3, 1),
                     grain: Grain::Month});
    assert_eq!(months.next().unwrap(),
               Range{start: dt(2015, 3, 1), end: dt(2015, 4, 1),
                     grain: Grain::Month});

    let mut quarters = Seq::from_grain(Grain::Quarter)(dt(2015, 2, 27));
    assert_eq!(quarters.next().unwrap(),
               Range{start: dt(2015, 1, 1), end: dt(2015, 4, 1),
                     grain: Grain::Quarter});
    assert_eq!(quarters.next().unwrap(),
               Range{start: dt(2015, 4, 1), end: dt(2015, 7, 1),
                     grain: Grain::Quarter});

    let mut years = Seq::from_grain(Grain::Year)(dt(2015, 2, 27));
    assert_eq!(years.next().unwrap(),
               Range{start: dt(2015, 1, 1), end: dt(2016, 1, 1),
                     grain: Grain::Year});
    assert_eq!(years.next().unwrap(),
               Range{start: dt(2016, 1, 1), end: dt(2017, 1, 1),
                     grain: Grain::Year});
}

#[test]
fn test_seq_shift() {
    let mut weekend =
        Seq::shift(Seq::weekend(), Grain::Day, 1)(dt(2016, 3, 23));
    assert_eq!(weekend.next().unwrap(),
               Range{start: dt(2016, 3, 27), end: dt(2016, 3, 29),
                     grain: Grain::Day});
    assert_eq!(weekend.next().unwrap(),
               Range{start: dt(2016, 4, 3), end: dt(2016, 4, 5),
                     grain: Grain::Day});
}

#[test]
fn test_seq_summer() {
    let mut summer = Seq::summer()(dt(2015, 9, 22));
    assert_eq!(summer.next().unwrap(),
               Range{start: dt(2016, 6, 21), end: dt(2016, 9, 21),
                     grain: Grain::Quarter});
    assert_eq!(summer.next().unwrap(),
               Range{start: dt(2017, 6, 21), end: dt(2017, 9, 21),
                     grain: Grain::Quarter});
}

#[test]
fn test_nth_basic() {
    // 3rd day of the month
    let day3 = Seq::nthof(
        3, Seq::from_grain(Grain::Day), Seq::from_grain(Grain::Month));
    let mut day3 = day3(dt(2016, 2, 2));
    assert_eq!(day3.next().unwrap(),
               Range{start: dt(2016, 2, 3), end: dt(2016, 2, 4),
                     grain: Grain::Day});
    assert_eq!(day3.next().unwrap(),
               Range{start: dt(2016, 3, 3), end: dt(2016, 3, 4),
                     grain: Grain::Day});

    // 3rd tuesday of the month
    let tue3mo = Seq::nthof(
        3, Seq::weekday(2), Seq::from_grain(Grain::Month));
    let mut tue3mo = tue3mo(dt(2016, 2, 10));
    assert_eq!(tue3mo.next().unwrap(),
               Range{start: dt(2016, 2, 16), end: dt(2016, 2, 17),
                     grain: Grain::Day});
    assert_eq!(tue3mo.next().unwrap(),
               Range{start: dt(2016, 3, 15), end: dt(2016, 3, 16),
                     grain: Grain::Day});

    // 2nd monday of april
    let secmonapr = Seq::nthof(2, Seq::weekday(1), Seq::month(4));
    let mut secmonapr = secmonapr(dt(2016, 2, 25));
    assert_eq!(secmonapr.next().unwrap(),
               Range{start: dt(2016, 4, 11), end: dt(2016, 4, 12),
                     grain: Grain::Day});
    assert_eq!(secmonapr.next().unwrap(),
               Range{start: dt(2017, 4, 10), end: dt(2017, 4, 11),
                     grain: Grain::Day});

    // 4th month of the year
    let years4thmo = Seq::nthof(
        4, Seq::from_grain(Grain::Month), Seq::from_grain(Grain::Year));
    let mut years4thmo =  years4thmo(dt(2016, 2, 23));
    assert_eq!(years4thmo.next().unwrap(),
               Range{start: dt(2016, 4, 1), end: dt(2016, 5, 1),
                     grain: Grain::Month});
    assert_eq!(years4thmo.next().unwrap(),
               Range{start: dt(2017, 4, 1), end: dt(2017, 5, 1),
                     grain: Grain::Month});

    // 1st day every month
    let first = Seq::nthof(
        1, Seq::from_grain(Grain::Day), Seq::from_grain(Grain::Month));
    let mut first = first(dt(2016, 8, 31));
    assert_eq!(first.next().unwrap(),
               Range{start: dt(2016, 8, 1), end: dt(2016, 8, 2),
                     grain: Grain::Day});
    assert_eq!(first.next().unwrap(),
               Range{start: dt(2016, 9, 1), end: dt(2016, 9, 2),
                     grain: Grain::Day});

    // 3rd week of june
    let thirdwkjune =
        Seq::nthof(3, Seq::from_grain(Grain::Week), Seq::month(6));
    let mut thirdwkjune = thirdwkjune(dt(2016, 9, 4));
    assert_eq!(thirdwkjune.next().unwrap(),
               Range{start: dt(2017, 6, 11), end: dt(2017, 6, 18),
                     grain: Grain::Week});
    assert_eq!(thirdwkjune.next().unwrap(),
               Range{start: dt(2018, 6, 10), end: dt(2018, 6, 17),
                     grain: Grain::Week});

    // 28th of june
    let jun28th =
        Seq::nthof(28, Seq::from_grain(Grain::Day), Seq::month(6));
    let mut jun28th = jun28th(dt(2016, 2, 25));
    assert_eq!(jun28th.next().unwrap(),
               Range{start: dt(2016, 6, 28), end: dt(2016, 6, 29),
                     grain: Grain::Day});
    assert_eq!(jun28th.next().unwrap(),
               Range{start: dt(2017, 6, 28), end: dt(2017, 6, 29),
                     grain: Grain::Day});
}

#[test]
fn test_nth_discontinuous() {
    // 29th of february
    let feb29th = Seq::nthof(
        29, Seq::from_grain(Grain::Day), Seq::month(2));
    let mut feb29th = feb29th(dt(2015, 2, 25));
    assert_eq!(feb29th.next().unwrap(),
               Range{start: dt(2016, 2, 29), end: dt(2016, 3, 1),
                     grain: Grain::Day});
    assert_eq!(feb29th.next().unwrap(),
               Range{start: dt(2020, 2, 29), end: dt(2020, 3, 1),
                     grain: Grain::Day});

    let thirtyfirst = Seq::nthof(
        31, Seq::from_grain(Grain::Day), Seq::from_grain(Grain::Month));
    let mut thirtyfirst = thirtyfirst(dt(2016, 8, 31));
    assert_eq!(thirtyfirst.next().unwrap(),
               Range{start: dt(2016, 8, 31), end: dt(2016, 9, 1),
                     grain: Grain::Day});
    assert_eq!(thirtyfirst.next().unwrap(),
               Range{start: dt(2016, 10, 31), end: dt(2016, 11, 1),
                     grain: Grain::Day});
    assert_eq!(thirtyfirst.next().unwrap(),
               Range{start: dt(2016, 12, 31), end: dt(2017, 1, 1),
                     grain: Grain::Day});
    assert_eq!(thirtyfirst.next().unwrap(),
               Range{start: dt(2017, 1, 31), end: dt(2017, 2, 1),
                     grain: Grain::Day});
}

#[test]
#[should_panic]
fn test_nthof_fuse() {
    let thirtysecond = Seq::nthof(
        32, Seq::from_grain(Grain::Day), Seq::from_grain(Grain::Month));
    let mut thirtysecond = thirtysecond(dt(2016, 8, 31));
    thirtysecond.next();
}

#[test]
#[should_panic]
fn test_lastof_fuse() {
    // 32nd-to-last day of month
    let badlastof = Seq::lastof(
        32, Seq::from_grain(Grain::Day), Seq::from_grain(Grain::Month));
    let mut badlastof = badlastof(dt(2015, 2, 25));
    badlastof.next();
}

#[test]
fn test_nth_non_aligned() {
    let firstwkendjan = Seq::nthof(1, Seq::weekend(), Seq::month(1));
    let mut firstwkendjan = firstwkendjan(dt(2016, 9, 4));
    assert_eq!(firstwkendjan.next().unwrap(),
               Range{start: dt(2016, 12, 31), end: dt(2017, 1, 2),
                     grain: Grain::Day});
    assert_eq!(firstwkendjan.next().unwrap(),
               Range{start: dt(2018, 1, 6), end: dt(2018, 1, 8),
                     grain: Grain::Day});
}

#[test]
fn test_nth_composed() {
    // the 5th instance of 10th-day-of-the-month (each year) aka May 10th
    let mo10th = Seq::nthof(
        10, Seq::from_grain(Grain::Day), Seq::from_grain(Grain::Month));
    let y5th10thday = Seq::nthof(5, mo10th, Seq::from_grain(Grain::Year));
    let mut y5th10thday = y5th10thday(dt(2015, 3, 11));
    assert_eq!(y5th10thday.next().unwrap(),
               Range{start: dt(2015, 5, 10), end: dt(2015, 5, 11),
                     grain: Grain::Day});
    assert_eq!(y5th10thday.next().unwrap(),
               Range{start: dt(2016, 5, 10), end: dt(2016, 5, 11),
                     grain: Grain::Day});
}

#[test]
fn test_lastof() {
    // last weekend of the year
    let weekendofyear = Seq::lastof(
        1, Seq::weekend(), Seq::from_grain(Grain::Year));
    let mut weekendofyear = weekendofyear(dt(2015, 2, 25));
    assert_eq!(weekendofyear.next().unwrap(),
               Range{start: dt(2015, 12, 26), end: dt(2015, 12, 28),
                     grain: Grain::Day});
    assert_eq!(weekendofyear.next().unwrap(),
               Range{start: dt(2016, 12, 31), end: dt(2017, 1, 2),
                     grain: Grain::Day});

    // 2nd-to-last day of february
    let daybeforelastfeb = Seq::lastof(
        2, Seq::from_grain(Grain::Day), Seq::month(2));
    let mut daybeforelastfeb = daybeforelastfeb(dt(2015, 2, 25));
    assert_eq!(daybeforelastfeb.next().unwrap(),
               Range{start: dt(2015, 2, 27), end: dt(2015, 2, 28),
                     grain: Grain::Day});
    assert_eq!(daybeforelastfeb.next().unwrap(),
               Range{start: dt(2016, 2, 28), end: dt(2016, 2, 29),
                     grain: Grain::Day});

    // 29th-to-last day of feb
    let daybeforelastfeb = Seq::lastof(
        29, Seq::from_grain(Grain::Day), Seq::month(2));
    let mut daybeforelastfeb = daybeforelastfeb(dt(2015, 2, 25));
    assert_eq!(daybeforelastfeb.next().unwrap(),
               Range{start: dt(2016, 2, 1), end: dt(2016, 2, 2),
                     grain: Grain::Day});
    assert_eq!(daybeforelastfeb.next().unwrap(),
               Range{start: dt(2020, 2, 1), end: dt(2020, 2, 2),
                     grain: Grain::Day});
}

#[test]
fn test_intersect() {
    // monday 28th
    let mon28th = Seq::intersect(Seq::weekday(1), Seq::nthof(
        28, Seq::from_grain(Grain::Day), Seq::from_grain(Grain::Month)));
    let mut mon28th = mon28th(dt(2016, 2, 25));
    assert_eq!(mon28th.next().unwrap(),
               Range{start: dt(2016, 3, 28), end: dt(2016, 3, 29),
                     grain: Grain::Day});
    assert_eq!(mon28th.next().unwrap(),
               Range{start: dt(2016, 11, 28), end: dt(2016, 11, 29),
                     grain: Grain::Day});
    assert_eq!(mon28th.next().unwrap(),
               Range{start: dt(2017, 8, 28), end: dt(2017, 8, 29),
                     grain: Grain::Day});

    // tuesdays 3pm
    let tue3pm = Seq::intersect(Seq::weekday(2), Seq::nthof(
        16, Seq::from_grain(Grain::Hour), Seq::from_grain(Grain::Day)));
    let mut tue3pm = tue3pm(dt(2016, 2, 25));
    assert_eq!(tue3pm.next().unwrap(),
               Range{start: dttm(2016, 3, 1, 15, 0, 0),
                     end: dttm(2016, 3, 1, 16, 0, 0),
                     grain: Grain::Hour});
    assert_eq!(tue3pm.next().unwrap(),
               Range{start: dttm(2016, 3, 8, 15, 0, 0),
                     end: dttm(2016, 3, 8, 16, 0, 0),
                     grain: Grain::Hour});
    assert_eq!(tue3pm.next().unwrap(),
               Range{start: dttm(2016, 3, 15, 15, 0, 0),
                     end: dttm(2016, 3, 15, 16, 0, 0),
                     grain: Grain::Hour});

    // thursdays of june
    let junthurs = Seq::intersect(Seq::weekday(4), Seq::month(6));
    let mut junthurs = junthurs(dt(2016, 2, 25));
    assert_eq!(junthurs.next().unwrap(),
               Range{start: dt(2016, 6, 2), end: dt(2016, 6, 3),
                     grain: Grain::Day});
    assert_eq!(junthurs.next().unwrap(),
               Range{start: dt(2016, 6, 9), end: dt(2016, 6, 10),
                     grain: Grain::Day});
    assert_eq!(junthurs.next().unwrap(),
               Range{start: dt(2016, 6, 16), end: dt(2016, 6, 17),
                     grain: Grain::Day});
    assert_eq!(junthurs.next().unwrap(),
               Range{start: dt(2016, 6, 23), end: dt(2016, 6, 24),
                     grain: Grain::Day});
    assert_eq!(junthurs.next().unwrap(),
               Range{start: dt(2016, 6, 30), end: dt(2016, 7, 1),
                     grain: Grain::Day});
    assert_eq!(junthurs.next().unwrap(),
               Range{start: dt(2017, 6, 1), end: dt(2017, 6, 2),
                     grain: Grain::Day});
}

#[test]
fn test_interval() {
    // monday to friday
    let mon2fri = Seq::interval(Seq::weekday(1), Seq::weekday(5), true);
    let mut mon2fri = mon2fri(dt(2016, 2, 25));
    assert_eq!(mon2fri.next().unwrap(),
               Range{start: dt(2016, 2, 29), end: dt(2016, 3, 5),
                     grain: Grain::Day});
    assert_eq!(mon2fri.next().unwrap(),
               Range{start: dt(2016, 3, 7), end: dt(2016, 3, 12),
                     grain: Grain::Day});

    // 2nd of june until end of month
    let june2ndtileom = Seq::interval(
        Seq::nthof(2, Seq::from_grain(Grain::Day), Seq::month(6)),
        Seq::month(6), true);
    let mut june2ndtileom = june2ndtileom(dt(2016, 2, 25));
    assert_eq!(june2ndtileom.next().unwrap(),
               Range{start: dt(2016, 6, 2), end: dt(2016, 7, 1),
                     grain: Grain::Day});
    assert_eq!(june2ndtileom.next().unwrap(),
               Range{start: dt(2017, 6, 2), end: dt(2017, 7, 1),
                     grain: Grain::Day});

    // afternoon
    let afternoon = Seq::interval(
        Seq::nthof(13, Seq::from_grain(Grain::Hour),
                   Seq::from_grain(Grain::Day)),
        Seq::nthof(19, Seq::from_grain(Grain::Hour),
                   Seq::from_grain(Grain::Day)), false);
    let mut afternoon = afternoon(dt(2016, 2, 25));
    assert_eq!(afternoon.next().unwrap(),
               Range{start: dttm(2016, 2, 25, 12, 0, 0),
                     end: dttm(2016, 2, 25, 18, 0, 0),
                     grain: Grain::Hour});
    assert_eq!(afternoon.next().unwrap(),
               Range{start: dttm(2016, 2, 26, 12, 0, 0),
                     end: dttm(2016, 2, 26, 18, 0, 0),
                     grain: Grain::Hour});

    // spring south hem
    let southspring = Seq::interval(
        Seq::nthof(21, Seq::from_grain(Grain::Day), Seq::month(9)),
        Seq::nthof(21, Seq::from_grain(Grain::Day), Seq::month(12)), true);
    let mut sspring = southspring(dt(2016, 2, 25));
    assert_eq!(sspring.next().unwrap(),
               Range{start: dt(2016, 9, 21), end: dt(2016, 12, 22),
                     grain: Grain::Day});
}

#[test]
fn test_merge() {
    let twoweeks = Seq::merge(Seq::from_grain(Grain::Week), 2);
    let mut twoweeks = twoweeks(dt(2015, 2, 27));
    assert_eq!(twoweeks.next().unwrap(),
               Range{start: dt(2015, 2, 22), end: dt(2015, 3, 8),
                     grain: Grain::Week});
    assert_eq!(twoweeks.next().unwrap(),
               Range{start: dt(2015, 3, 8), end: dt(2015, 3, 22),
                     grain: Grain::Week});

    let threedays = Seq::merge(Seq::from_grain(Grain::Day), 3);
    let mut threedays = threedays(dt(2015, 2, 27));
    assert_eq!(threedays.next().unwrap(),
               Range{start: dt(2015, 2, 27), end: dt(2015, 3, 2),
                     grain: Grain::Day});
    assert_eq!(threedays.next().unwrap(),
               Range{start: dt(2015, 3, 2), end: dt(2015, 3, 5),
                     grain: Grain::Day});
    assert_eq!(threedays.next().unwrap(),
               Range{start: dt(2015, 3, 5), end: dt(2015, 3, 8),
                     grain: Grain::Day});
}

#[test]
fn test_multi() {
    // 3 days after mon feb 28th
    let monfeb28th3d = Seq::nthof(
        28, Seq::from_grain(Grain::Day), Seq::from_grain(Grain::Month));
    let monfeb28th3d = Seq::intersect(monfeb28th3d, Seq::weekday(1));
    let monfeb28th3d = Seq::intersect(monfeb28th3d, Seq::month(2));
    let monfeb28th3d = Seq::intersect(monfeb28th3d, Seq::month(2));
    let monfeb28th3d = Seq::shift(monfeb28th3d, Grain::Day, 3);
    let mut monfeb28th3d = monfeb28th3d(dt(2021, 9, 5));
    assert_eq!(monfeb28th3d.next().unwrap(),
               Range{start: dt(2022, 3, 3), end: dt(2022, 3, 4),
                     grain: Grain::Day});
    assert_eq!(monfeb28th3d.next().unwrap(),
               Range{start: dt(2028, 3, 2), end: dt(2028, 3, 3),
                     grain: Grain::Day});
}

#[test]
fn test_next() {
    let reftime = dt(2016, 2, 25);
    assert_eq!(Seq::from_grain(Grain::Month).next(reftime, 1),
               Range{start: dt(2016, 3, 1), end: dt(2016, 4, 1),
                     grain: Grain::Month});
    assert_eq!(Seq::weekday(4).next(reftime, 1),
               Range{start: dt(2016, 3, 3), end: dt(2016, 3, 4),
                     grain: Grain::Day});
    assert_eq!(Seq::weekday(5).next(reftime, 1),
               Range{start: dt(2016, 2, 26), end: dt(2016, 2, 27),
                     grain: Grain::Day});

    let mon28th = Seq::intersect(Seq::weekday(1), Seq::nthof(
        28, Seq::from_grain(Grain::Day), Seq::from_grain(Grain::Month)));
    assert_eq!(mon28th.next(reftime, 1),
               Range{start: dt(2016, 3, 28), end: dt(2016, 3, 29),
                     grain: Grain::Day});
    let thu25th = Seq::intersect(Seq::weekday(4), Seq::nthof(
        25, Seq::from_grain(Grain::Day), Seq::from_grain(Grain::Month)));
    assert_eq!(thu25th.next(reftime, 1),
               Range{start: dt(2016, 8, 25), end: dt(2016, 8, 26),
                     grain: Grain::Day});
}
