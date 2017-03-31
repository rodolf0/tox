use semantics::{Range, Grain, Seq};
use utils::{DateTime, Date};

fn dt(year: i32, month: u32, day: u32) -> DateTime {
    Date::from_ymd(year, month, day).and_hms(0, 0, 0)
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
fn test_seq_summer() {
    let mut summer = Seq::summer()(dt(2015, 9, 22));
    assert_eq!(summer.next().unwrap(),
               Range{start: dt(2016, 6, 21), end: dt(2016, 9, 21),
                     grain: Grain::Quarter});
    assert_eq!(summer.next().unwrap(),
               Range{start: dt(2017, 6, 21), end: dt(2017, 9, 21),
                     grain: Grain::Quarter});
}

//#[test]
//fn test_merge() {
    //let reftime = Date::from_ymd(2015, 2, 27).and_hms(0, 0, 0);
    //let mut twoweeks = s::merge(2, s::week())(reftime);
    //assert_eq!(twoweeks.next().unwrap(),
               //Range{
                //start: Date::from_ymd(2015, 2, 22).and_hms(0, 0, 0),
                //end: Date::from_ymd(2015, 3, 8).and_hms(0, 0, 0),
                //grain: Granularity::Week});
    //assert_eq!(twoweeks.next().unwrap(),
               //Range{
                //start: Date::from_ymd(2015, 3, 8).and_hms(0, 0, 0),
                //end: Date::from_ymd(2015, 3, 22).and_hms(0, 0, 0),
                //grain: Granularity::Week});
    //let mut threedays= s::merge(3, s::day())(reftime);
    //assert_eq!(threedays.next().unwrap(),
               //Range{
                //start: Date::from_ymd(2015, 2, 27).and_hms(0, 0, 0),
                //end: Date::from_ymd(2015, 3, 2).and_hms(0, 0, 0),
                //grain: Granularity::Day});
    //assert_eq!(threedays.next().unwrap(),
               //Range{
                //start: Date::from_ymd(2015, 3, 2).and_hms(0, 0, 0),
                //end: Date::from_ymd(2015, 3, 5).and_hms(0, 0, 0),
                //grain: Granularity::Day});
//}

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

//#[test]
//fn test_intersect_2() {
    //let reftime = Date::from_ymd(2016, 2, 25).and_hms(0, 0, 0);
    //// monday 28th
    //let mut mon28th = s::intersect(
        //s::day_of_week(1),
        //s::nthof(28, s::day(), s::month()))(reftime);
    //assert_eq!(mon28th.next().unwrap(),
               //Range{
                //start: Date::from_ymd(2016, 3, 28).and_hms(0, 0, 0),
                //end: Date::from_ymd(2016, 3, 29).and_hms(0, 0, 0),
                //grain: Granularity::Day});
    //assert_eq!(mon28th.next().unwrap(),
               //Range{
                //start: Date::from_ymd(2016, 11, 28).and_hms(0, 0, 0),
                //end: Date::from_ymd(2016, 11, 29).and_hms(0, 0, 0),
                //grain: Granularity::Day});
//}

//#[test]
//fn test_intersect_4() {
    //let reftime = Date::from_ymd(2016, 8, 31).and_hms(0, 0, 0);
    //// 1st day of month
    //let first = s::nthof(1, s::day(), s::month());
    //let mut firstofmonth = s::intersect(first, s::month())(reftime);
    //assert_eq!(firstofmonth.next().unwrap(),
               //Range{
                //start: Date::from_ymd(2016, 8, 1).and_hms(0, 0, 0),
                //end: Date::from_ymd(2016, 8, 2).and_hms(0, 0, 0),
                //grain: Granularity::Day});
//}

//#[test]
//fn test_intersect_3() {
    //let reftime = Date::from_ymd(2016, 2, 25).and_hms(0, 0, 0);
    //// thursdays of june
    //let junthurs = s::intersect(s::day_of_week(4), s::month_of_year(6));
    //let mut junthurs = junthurs(reftime);
    //assert_eq!(junthurs.next().unwrap(),
               //Range{
                //start: Date::from_ymd(2016, 6, 2).and_hms(0, 0, 0),
                //end: Date::from_ymd(2016, 6, 3).and_hms(0, 0, 0),
                //grain: Granularity::Day});
    //assert_eq!(junthurs.next().unwrap(),
               //Range{
                //start: Date::from_ymd(2016, 6, 9).and_hms(0, 0, 0),
                //end: Date::from_ymd(2016, 6, 10).and_hms(0, 0, 0),
                //grain: Granularity::Day});
    //assert_eq!(junthurs.next().unwrap(),
               //Range{
                //start: Date::from_ymd(2016, 6, 16).and_hms(0, 0, 0),
                //end: Date::from_ymd(2016, 6, 17).and_hms(0, 0, 0),
                //grain: Granularity::Day});
    //assert_eq!(junthurs.next().unwrap(),
               //Range{
                //start: Date::from_ymd(2016, 6, 23).and_hms(0, 0, 0),
                //end: Date::from_ymd(2016, 6, 24).and_hms(0, 0, 0),
                //grain: Granularity::Day});
    //assert_eq!(junthurs.next().unwrap(),
               //Range{
                //start: Date::from_ymd(2016, 6, 30).and_hms(0, 0, 0),
                //end: Date::from_ymd(2016, 7, 1).and_hms(0, 0, 0),
                //grain: Granularity::Day});
    //assert_eq!(junthurs.next().unwrap(),
               //Range{
                //start: Date::from_ymd(2017, 6, 1).and_hms(0, 0, 0),
                //end: Date::from_ymd(2017, 6, 2).and_hms(0, 0, 0),
                //grain: Granularity::Day});
//}

//#[test]
//fn test_interval_1() {
    //let reftime = Date::from_ymd(2016, 2, 25).and_hms(0, 0, 0);
    //let mut mon2fri = s::interval(s::day_of_week(1), s::day_of_week(5))(reftime);
    //assert_eq!(mon2fri.next().unwrap(),
               //Range{
                //start: Date::from_ymd(2016, 2, 29).and_hms(0, 0, 0),
                //end: Date::from_ymd(2016, 3, 4).and_hms(0, 0, 0),
                //grain: Granularity::Day});
    //assert_eq!(mon2fri.next().unwrap(),
               //Range{
                //start: Date::from_ymd(2016, 3, 7).and_hms(0, 0, 0),
                //end: Date::from_ymd(2016, 3, 11).and_hms(0, 0, 0),
                //grain: Granularity::Day});
//}

//#[test]
//fn test_interval_2() {
    //let reftime = Date::from_ymd(2016, 9, 25).and_hms(0, 0, 0);
    //let jun21st = s::intersect(
        //s::month_of_year(6), s::nthof(21, s::day(), s::month()));
    //let sep21st = s::intersect(
        //s::month_of_year(9), s::nthof(21, s::day(), s::month()));
    //let mut summer = s::interval(jun21st, sep21st)(reftime);
    //assert_eq!(summer.next().unwrap(),
               //Range{
                //start: Date::from_ymd(2017, 6, 21).and_hms(0, 0, 0),
                //end: Date::from_ymd(2017, 9, 21).and_hms(0, 0, 0),
                //grain: Granularity::Day});
    //assert_eq!(summer.next().unwrap(),
               //Range{
                //start: Date::from_ymd(2018, 6, 21).and_hms(0, 0, 0),
                //end: Date::from_ymd(2018, 9, 21).and_hms(0, 0, 0),
                //grain: Granularity::Day});
//}

//#[test]
//fn test_this() {
    //let reftime = Date::from_ymd(2016, 2, 25).and_hms(0, 0, 0);
    //assert_eq!(s::this(s::month(), reftime),
               //Range{
                //start: Date::from_ymd(2016, 2, 1).and_hms(0, 0, 0),
                //end: Date::from_ymd(2016, 3, 1).and_hms(0, 0, 0),
                //grain: Granularity::Month});
    //assert_eq!(s::this(s::day_of_week(5), reftime),
               //Range{
                //start: Date::from_ymd(2016, 2, 26).and_hms(0, 0, 0),
                //end: Date::from_ymd(2016, 2, 27).and_hms(0, 0, 0),
                //grain: Granularity::Day});
    //let mon28th = s::intersect(
        //s::day_of_week(1), s::nthof(28, s::day(), s::month()));
    //assert_eq!(s::this(mon28th, reftime),
               //Range{
                //start: Date::from_ymd(2016, 3, 28).and_hms(0, 0, 0),
                //end: Date::from_ymd(2016, 3, 29).and_hms(0, 0, 0),
                //grain: Granularity::Day});
    //assert_eq!(s::this(s::weekend(), reftime),
               //Range{
                //start: Date::from_ymd(2016, 2, 27).and_hms(0, 0, 0),
                //end: Date::from_ymd(2016, 2, 29).and_hms(0, 0, 0),
                //grain: Granularity::Day});
//}

//#[test]
//fn test_next() {
    //let reftime = Date::from_ymd(2016, 2, 25).and_hms(0, 0, 0);
    //assert_eq!(s::next(s::month(), 1, reftime),
               //Range{
                //start: Date::from_ymd(2016, 3, 1).and_hms(0, 0, 0),
                //end: Date::from_ymd(2016, 4, 1).and_hms(0, 0, 0),
                //grain: Granularity::Month});
    //assert_eq!(s::next(s::day_of_week(4), 1, reftime),
               //Range{
                //start: Date::from_ymd(2016, 3, 3).and_hms(0, 0, 0),
                //end: Date::from_ymd(2016, 3, 4).and_hms(0, 0, 0),
                //grain: Granularity::Day});
    //assert_eq!(s::next(s::day_of_week(5), 1, reftime),
               //Range{
                //start: Date::from_ymd(2016, 2, 26).and_hms(0, 0, 0),
                //end: Date::from_ymd(2016, 2, 27).and_hms(0, 0, 0),
                //grain: Granularity::Day});
    //let mon28th = s::intersect(
        //s::day_of_week(1), s::nthof(28, s::day(), s::month()));
    //assert_eq!(s::next(mon28th, 1, reftime),
               //Range{
                //start: Date::from_ymd(2016, 3, 28).and_hms(0, 0, 0),
                //end: Date::from_ymd(2016, 3, 29).and_hms(0, 0, 0),
                //grain: Granularity::Day});
    //let thu25th = s::intersect(
        //s::day_of_week(4), s::nthof(25, s::day(), s::month()));
    //assert_eq!(s::next(thu25th, 1, reftime),
               //Range{
                //start: Date::from_ymd(2016, 8, 25).and_hms(0, 0, 0),
                //end: Date::from_ymd(2016, 8, 26).and_hms(0, 0, 0),
                //grain: Granularity::Day});
//}

//#[test]
//fn test_multi_1() {
    //// 3 days after mon feb 28th
    //let rt = Date::from_ymd(2021, 9, 5).and_hms(0, 0, 0);
    //let monfeb28th = s::nthof(28, s::day(), s::month());
    //let monfeb28th = s::intersect(monfeb28th, s::day_of_week(1));
    //let monfeb28th = s::intersect(monfeb28th, s::month_of_year(2));
    //assert_eq!(monfeb28th(rt).next().unwrap(),
               //Range{
                //start: Date::from_ymd(2022, 2, 28).and_hms(0, 0, 0),
                //end: Date::from_ymd(2022, 3, 1).and_hms(0, 0, 0),
                //grain: Granularity::Day});
    //let after3 = s::shift(monfeb28th(rt).next().unwrap(), 3, Granularity::Day);
    //assert_eq!(after3,
               //Range{
                //start: Date::from_ymd(2022, 3, 3).and_hms(0, 0, 0),
                //end: Date::from_ymd(2022, 3, 4).and_hms(0, 0, 0),
                //grain: Granularity::Day});
//}
