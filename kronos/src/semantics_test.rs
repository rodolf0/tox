use chrono::naive::date::NaiveDate as Date;
use semantics::{Range, Granularity};
use semantics as s;

#[test]
fn test_dayofweek() {
    let reftime = Date::from_ymd(2016, 8, 27).and_hms(0, 0, 0);
    let mut sunday = s::day_of_week(reftime, 0)();
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
    let mut august = s::month_of_year(reftime, 8)();
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

    let mut february = s::month_of_year(reftime, 2)();
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
    let mut days = s::day(reftime)();
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
    let mut months = s::month(reftime)();
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
    let mut years = s::year(reftime)();
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
    let reftime = Date::from_ymd(2016, 2, 25).and_hms(0, 0, 0);
    // 3rd day of the month
    let mo = s::month(reftime);
    let mut day3 = s::nth(3, s::day(s::seq_start(mo.clone())), mo)();
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
    let reftime = Date::from_ymd(2016, 2, 25).and_hms(0, 0, 0);
    // 3rd tuesday of the month
    let mo = s::month(reftime);
    let tue = s::day_of_week(s::seq_start(mo.clone()), 2);
    let mut tue3mo = s::nth(3, tue, mo)();
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
    let reftime = Date::from_ymd(2016, 2, 25).and_hms(0, 0, 0);
    // 4th month of the year
    let years = s::year(reftime);
    let months = s::month(s::seq_start(years.clone()));
    let mut years4thmo = s::nth(4, months, years)();
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
    let feb = s::month_of_year(reftime, 2);
    let mut feb29th = s::nth(29, s::day(s::seq_start(feb.clone())), feb)();
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
    let reftime = Date::from_ymd(2015, 2, 25).and_hms(0, 0, 0);
    let years = s::year(reftime);
    let reftime = s::seq_start(years.clone());
    let mo10th = s::nth(10, s::day(reftime), s::month(reftime));
    // the 5th 10th-day-of-the-month (each year)
    let mut y5th10thday = s::nth(5, mo10th, years)();
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
