use chrono::naive::date::NaiveDate as Date;
use semantics::{Range, Granularity};
use semantics as s;

#[test]
fn test_dayofweek() {
    let reftime = Date::from_ymd(2016, 8, 27).and_hms(0, 0, 0);
    let mut sunday = s::day_of_week(0)(reftime);
    assert_eq!(sunday.next().unwrap(),
               Range{
                start: Date::from_ymd(2016, 8, 28).and_hms(0, 0, 0),
                end: Date::from_ymd(2016, 8, 29).and_hms(0, 0, 0),
                grain: Granularity::Day});
    assert_eq!(sunday.next().unwrap(),
               Range{
                start: Date::from_ymd(2016, 9, 4).and_hms(0, 0, 0),
                end: Date::from_ymd(2016, 9, 5).and_hms(0, 0, 0),
                grain: Granularity::Day});
}

#[test]
fn test_monthofyear() {
    let reftime = Date::from_ymd(2016, 8, 27).and_hms(0, 0, 0);
    let mut august = s::month_of_year(8)(reftime);
    assert_eq!(august.next().unwrap(),
               Range{
                start: Date::from_ymd(2016, 8, 1).and_hms(0, 0, 0),
                end: Date::from_ymd(2016, 9, 1).and_hms(0, 0, 0),
                grain: Granularity::Month});
    assert_eq!(august.next().unwrap(),
               Range{
                start: Date::from_ymd(2017, 8, 1).and_hms(0, 0, 0),
                end: Date::from_ymd(2017, 9, 1).and_hms(0, 0, 0),
                grain: Granularity::Month});

    let mut february = s::month_of_year(2)(reftime);
    assert_eq!(february.next().unwrap(),
               Range{
                start: Date::from_ymd(2017, 2, 1).and_hms(0, 0, 0),
                end: Date::from_ymd(2017, 3, 1).and_hms(0, 0, 0),
                grain: Granularity::Month});
    assert_eq!(february.next().unwrap(),
               Range{
                start: Date::from_ymd(2018, 2, 1).and_hms(0, 0, 0),
                end: Date::from_ymd(2018, 3, 1).and_hms(0, 0, 0),
                grain: Granularity::Month});
}

#[test]
fn test_day() {
    let reftime = Date::from_ymd(2015, 2, 27).and_hms(0, 0, 0);
    let mut days = s::day()(reftime);
    assert_eq!(days.next().unwrap(),
               Range{
                start: Date::from_ymd(2015, 2, 27).and_hms(0, 0, 0),
                end: Date::from_ymd(2015, 2, 28).and_hms(0, 0, 0),
                grain: Granularity::Day});
    assert_eq!(days.next().unwrap(),
               Range{
                start: Date::from_ymd(2015, 2, 28).and_hms(0, 0, 0),
                end: Date::from_ymd(2015, 3, 1).and_hms(0, 0, 0),
                grain: Granularity::Day});
}

#[test]
fn test_month() {
    let reftime = Date::from_ymd(2015, 2, 27).and_hms(0, 0, 0);
    let mut months = s::month()(reftime);
    assert_eq!(months.next().unwrap(),
               Range{
                start: Date::from_ymd(2015, 2, 1).and_hms(0, 0, 0),
                end: Date::from_ymd(2015, 3, 1).and_hms(0, 0, 0),
                grain: Granularity::Month});
    assert_eq!(months.next().unwrap(),
               Range{
                start: Date::from_ymd(2015, 3, 1).and_hms(0, 0, 0),
                end: Date::from_ymd(2015, 4, 1).and_hms(0, 0, 0),
                grain: Granularity::Month});
}

#[test]
fn test_year() {
    let reftime = Date::from_ymd(2015, 2, 27).and_hms(0, 0, 0);
    let mut years = s::year()(reftime);
    assert_eq!(years.next().unwrap(),
               Range{
                start: Date::from_ymd(2015, 1, 1).and_hms(0, 0, 0),
                end: Date::from_ymd(2016, 1, 1).and_hms(0, 0, 0),
                grain: Granularity::Year});
    assert_eq!(years.next().unwrap(),
               Range{
                start: Date::from_ymd(2016, 1, 1).and_hms(0, 0, 0),
                end: Date::from_ymd(2017, 1, 1).and_hms(0, 0, 0),
                grain: Granularity::Year});
}

#[test]
fn test_nth_1() {
    let reftime = Date::from_ymd(2016, 2, 1).and_hms(0, 0, 0);
    // 3rd day of the month
    let mut day3 = s::nth(3, s::day(), s::month())(reftime);
    assert_eq!(day3.next().unwrap(),
               Range{
                start: Date::from_ymd(2016, 2, 3).and_hms(0, 0, 0),
                end: Date::from_ymd(2016, 2, 4).and_hms(0, 0, 0),
                grain: Granularity::Day});
    assert_eq!(day3.next().unwrap(),
               Range{
                start: Date::from_ymd(2016, 3, 3).and_hms(0, 0, 0),
                end: Date::from_ymd(2016, 3, 4).and_hms(0, 0, 0),
                grain: Granularity::Day});
}

#[test]
fn test_nth_2() {
    // reftime has to generate aligned weekday and month seqs
    let reftime = Date::from_ymd(2016, 2, 1).and_hms(0, 0, 0);
    // 3rd tuesday of the month
    let mut tue3mo = s::nth(3, s::day_of_week(2), s::month())(reftime);
    assert_eq!(tue3mo.next().unwrap(),
               Range{
                start: Date::from_ymd(2016, 2, 16).and_hms(0, 0, 0),
                end: Date::from_ymd(2016, 2, 17).and_hms(0, 0, 0),
                grain: Granularity::Day});
    assert_eq!(tue3mo.next().unwrap(),
               Range{
                start: Date::from_ymd(2016, 3, 15).and_hms(0, 0, 0),
                end: Date::from_ymd(2016, 3, 16).and_hms(0, 0, 0),
                grain: Granularity::Day});
}

#[test]
fn test_nth_3() {
    // reftime has to generate aligned month and year sequence
    let reftime = Date::from_ymd(2016, 1, 1).and_hms(0, 0, 0);
    // 4th month of the year
    let mut years4thmo = s::nth(4, s::month(), s::year())(reftime);
    assert_eq!(years4thmo.next().unwrap(),
               Range{
                start: Date::from_ymd(2016, 4, 1).and_hms(0, 0, 0),
                end: Date::from_ymd(2016, 5, 1).and_hms(0, 0, 0),
                grain: Granularity::Month});
    assert_eq!(years4thmo.next().unwrap(),
               Range{
                start: Date::from_ymd(2017, 4, 1).and_hms(0, 0, 0),
                end: Date::from_ymd(2017, 5, 1).and_hms(0, 0, 0),
                grain: Granularity::Month});
}

#[test]
fn test_nth_4() {
    let reftime = Date::from_ymd(2015, 2, 25).and_hms(0, 0, 0);
    // 29th of february
    let mut feb29th = s::nth(29, s::day(), s::month_of_year(2))(reftime);
    assert_eq!(feb29th.next().unwrap(),
               Range{
                start: Date::from_ymd(2016, 2, 29).and_hms(0, 0, 0),
                end: Date::from_ymd(2016, 3, 1).and_hms(0, 0, 0),
                grain: Granularity::Day});
    assert_eq!(feb29th.next().unwrap(),
               Range{
                start: Date::from_ymd(2020, 2, 29).and_hms(0, 0, 0),
                end: Date::from_ymd(2020, 3, 1).and_hms(0, 0, 0),
                grain: Granularity::Day});
}

#[test]
fn test_nth_5() {
    let reftime = Date::from_ymd(2015, 1, 1).and_hms(0, 0, 0);
    let mo10th = s::nth(10, s::day(), s::month());
    // the 5th 10th-day-of-the-month (each year)
    let mut y5th10thday = s::nth(5, mo10th, s::year())(reftime);
    assert_eq!(y5th10thday.next().unwrap(),
               Range{
                start: Date::from_ymd(2015, 5, 10).and_hms(0, 0, 0),
                end: Date::from_ymd(2015, 5, 11).and_hms(0, 0, 0),
                grain: Granularity::Day});
    assert_eq!(y5th10thday.next().unwrap(),
               Range{
                start: Date::from_ymd(2016, 5, 10).and_hms(0, 0, 0),
                end: Date::from_ymd(2016, 5, 11).and_hms(0, 0, 0),
                grain: Granularity::Day});
}

#[test] #[ignore] // expensive test
fn test_nth_6() {
    let reftime = Date::from_ymd(2016, 9, 1).and_hms(0, 0, 0);
    // BAD test: 2nd month of the day
    let mut flawed = s::nth(2, s::month(), s::day())(reftime);
    assert_eq!(flawed.next(), None);
}

#[test]
fn test_intersect_1() {
    let reftime = Date::from_ymd(2016, 2, 25).and_hms(0, 0, 0);
    // 28th of june
    let mut jun28th = s::intersect(
        s::month_of_year(6),
        s::nth(28, s::day(), s::month()))(reftime);
    assert_eq!(jun28th.next().unwrap(),
               Range{
                start: Date::from_ymd(2016, 6, 28).and_hms(0, 0, 0),
                end: Date::from_ymd(2016, 6, 29).and_hms(0, 0, 0),
                grain: Granularity::Day});
    assert_eq!(jun28th.next().unwrap(),
               Range{
                start: Date::from_ymd(2017, 6, 28).and_hms(0, 0, 0),
                end: Date::from_ymd(2017, 6, 29).and_hms(0, 0, 0),
                grain: Granularity::Day});
}
