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
