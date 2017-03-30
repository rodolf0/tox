extern crate chrono;
use chrono::{Datelike, Timelike};
use semantics::Grain;

pub type DateTime = chrono::NaiveDateTime;
pub type Date = chrono::NaiveDate;
pub type Duration = chrono::Duration;

pub fn truncate(d: DateTime, g: Grain) -> DateTime {
    use Grain::*;
    match g {
        Second => d.with_nanosecond(0).unwrap(),
        Minute => d.date().and_hms(d.hour(), d.minute(), 0),
        Hour => d.date().and_hms(d.hour(), 0, 0),
        Day => d.date().and_hms(0, 0, 0),
        Week => {
            let days_from_sun = d.weekday().num_days_from_sunday();
            (d.date() - Duration::days(days_from_sun as i64)).and_hms(0, 0, 0)
        },
        Month => Date::from_ymd(d.year(), d.month(), 1).and_hms(0, 0, 0),
        Quarter => {
            let qstart = 1 + 3 * (d.month()/4);
            Date::from_ymd(d.year(), qstart, 1).and_hms(0, 0, 0)
        },
        Year => Date::from_ymd(d.year(), 1, 1).and_hms(0, 0, 0),
    }
}

pub fn find_dow(mut date: Date, dow: u32) -> Date {
    while date.weekday().num_days_from_sunday() != dow {
        date = date.succ();
    }
    date
}

pub fn find_month(mut date: Date, month: u32) -> Date {
    while date.month() != month {
        date = dtshift::add(date, 0, 1, 0);
    }
    date
}

pub fn shift_datetime(d: DateTime, grain: Grain, n: i32) -> DateTime {
    use Grain::*;
    let m = if n >= 0 {n as u32} else {(-n) as u32};
    let shiftfn = if n >= 0 {dtshift::add} else {dtshift::sub};
    match grain {
        Second => d + Duration::seconds(n as i64),
        Minute => d + Duration::minutes(n as i64),
        Hour => d + Duration::hours(n as i64),
        Day => d + Duration::days(n as i64),
        Week => d + Duration::weeks(n as i64),
        Month => DateTime::new(shiftfn(d.date(), 0, m, 0), d.time()),
        Quarter => DateTime::new(shiftfn(d.date(), 0, 3 * m, 0), d.time()),
        Year => DateTime::new(shiftfn(d.date(), m, 0, 0), d.time()),
    }
}

mod dtshift {
    use super::*;
    use std::cmp;

    fn days_in_month(m: u32, y: i32) -> u32 {
        static DIM: [u8;12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
        assert!(m > 0 && m <= 12);
        // check when february has 29 days
        if m == 2 && y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) {return 29;}
        DIM[(m-1) as usize] as u32
    }

    pub fn add(dt: Date, y: u32, mut m: u32, mut d: u32) -> Date {
        let (mut day, mut month, mut year) = (dt.day(), dt.month(), dt.year());
        while d > 0 {
            let diff = cmp::min(days_in_month(month, year)-day, d);
            day += diff;
            d -= diff;
            if d > 0 {
                day = 0;
                month += 1;
                if month > 12 {
                    year += 1;
                    month = 1;
                }
            }
        }
        while m > 0 {
            let diff = cmp::min(12 - month, m);
            month += diff;
            m -= diff;
            if m > 0 {
                month = 0;
                year += 1;
            }
        }
        year += y as i32;
        day = cmp::min(day, days_in_month(month, year));
        Date::from_ymd(year, month, day)
    }

    pub fn sub(dt: Date, y: u32, mut m: u32, mut d: u32) -> Date {
        let (mut day, mut month, mut year) = (dt.day(), dt.month(), dt.year());
        while d > 0 {
            let diff = cmp::min(day-1, d);
            day -= diff;
            d -= diff;
            if d > 0 {
                month -= 1;
                if month < 1 {
                    month = 12;
                    year -= 1;
                }
                day = 1 + days_in_month(month, year);
            }
        }
        while m > 0 {
            let diff = cmp::min(month-1, m);
            month -= diff;
            m -= diff;
            if m > 0 {
                month = 13;
                year -= 1;
            }
        }
        year -= y as i32;
        day = cmp::min(day, days_in_month(month, year));
        Date::from_ymd(year, month, day)
    }
}


// Build a Range enclosing <t> wiht granularity <g>
//pub fn enclose(t: DateTime, g: Granularity) -> Range {
    //match g {
        //Granularity::Year => {
            //Range{
                //start: Date::from_ymd(t.year(), 1, 1).and_hms(0, 0, 0),
                //end: Date::from_ymd(t.year()+1, 1, 1).and_hms(0, 0, 0),
                //grain: Granularity::Year
            //}
        //},
        //Granularity::Quarter => {
            //let start = Date::from_ymd(t.year(), 1 + 3 * (t.month()/4), 1);
            //Range{
                //start: start.and_hms(0, 0, 0),
                //end: date_add(start, 0, 3, 0).and_hms(0, 0, 0),
                //grain: Granularity::Quarter
            //}
        //},
        //Granularity::Month => {
            //let start = Date::from_ymd(t.year(), t.month(), 1);
            //Range{
                //start: start.and_hms(0, 0, 0),
                //end: date_add(start, 0, 1, 0).and_hms(0, 0, 0),
                //grain: Granularity::Month
            //}
        //},
        //Granularity::Week => {
            //let wdiff = t.weekday().num_days_from_sunday();
            //let start = date_sub(t.date(), 0, 0, wdiff).and_hms(0, 0, 0);
            //Range{
                //start: start,
                //end: start + Duration::days(7),
                //grain: Granularity::Week
            //}
        //},
        //Granularity::Day => {
            //let start = Date::from_ymd(t.year(), t.month(), t.day());
            //Range{
                //start: start.and_hms(0, 0, 0),
                //end: date_add(start, 0, 0, 1).and_hms(0, 0, 0),
                //grain: Granularity::Day
            //}
        //},
    //}
//}

#[cfg(test)]
mod tests {
    use super::*;
    fn dt(year: i32, month: u32, day: u32) -> Date {
        Date::from_ymd(year, month, day)
    }
    fn dttm(year: i32, month: u32, day: u32) -> DateTime {
        Date::from_ymd(year, month, day).and_hms(0, 0, 0)
    }

    #[test]
    fn test_dateadd() {
        let d = dt(2016, 9, 5);
        assert_eq!(dtshift::add(d, 0, 0, 30), dt(2016, 10, 5));
        assert_eq!(dtshift::add(d, 0, 0, 1234), dt(2020, 1, 22));
        assert_eq!(dtshift::add(d, 0, 0, 365), dt(2017, 9, 5));
        assert_eq!(dtshift::add(d, 0, 0, 2541), dt(2023, 8, 21));
        assert_eq!(dtshift::add(d, 0, 1, 0), dt(2016, 10, 5));
        let d = dt(2016, 1, 30);
        assert_eq!(dtshift::add(d, 0, 1, 0), dt(2016, 2, 29));
        assert_eq!(dtshift::add(d, 0, 2, 0), dt(2016, 3, 30));
        assert_eq!(dtshift::add(d, 0, 12, 0), dt(2017, 1, 30));
        let d = dt(2016, 12, 31);
        assert_eq!(dtshift::add(d, 0, 1, 0), dt(2017, 1, 31));
    }

    #[test]
    fn test_datesub() {
        let d = dt(2016, 9, 5);
        assert_eq!(dtshift::sub(d, 0, 0, 3), dt(2016, 9, 2));
        assert_eq!(dtshift::sub(d, 0, 0, 6), dt(2016, 8, 30));
        assert_eq!(dtshift::sub(d, 0, 0, 36), dt(2016, 7, 31));
        assert_eq!(dtshift::sub(d, 0, 0, 1234), dt(2013, 4, 20));
        assert_eq!(dtshift::sub(d, 0, 0, 365), dt(2015, 9, 6));
        assert_eq!(dtshift::sub(d, 0, 1, 0), dt(2016, 8, 5));
        let d = dt(2016, 9, 1);
        assert_eq!(dtshift::sub(d, 0, 0, 1), dt(2016, 8, 31));
        let d = dt(2016, 1, 31);
        assert_eq!(dtshift::sub(d, 0, 1, 0), dt(2015, 12, 31));
        let d = dt(2016, 3, 31);
        assert_eq!(dtshift::sub(d, 0, 1, 0), dt(2016, 2, 29));
        assert_eq!(dtshift::sub(d, 0, 2, 0), dt(2016, 1, 31));
        assert_eq!(dtshift::sub(d, 0, 13, 0), dt(2015, 2, 28));
    }

    #[test]
    fn test_shifts() {
        let d = dttm(2016, 9, 5);
        assert_eq!(shift_datetime(d, Grain::Day, -3), dttm(2016, 9, 2));
        assert_eq!(shift_datetime(d, Grain::Day, -36), dttm(2016, 7, 31));
        assert_eq!(shift_datetime(d, Grain::Day, -1234), dttm(2013, 4, 20));
        assert_eq!(shift_datetime(d, Grain::Month, -1), dttm(2016, 8, 5));
        let d = dttm(2016, 1, 31);
        assert_eq!(shift_datetime(d, Grain::Month, -1), dttm(2015, 12, 31));
        let d = dttm(2016, 3, 31);
        assert_eq!(shift_datetime(d, Grain::Month, -27), dttm(2013, 12, 31));
        assert_eq!(shift_datetime(d, Grain::Month, -2), dttm(2016, 1, 31));
        assert_eq!(shift_datetime(d, Grain::Month, -13), dttm(2015, 2, 28));
        let d = dttm(2016, 3, 31);
        assert_eq!(shift_datetime(d, Grain::Week, 7), dttm(2016, 5, 19));
        assert_eq!(shift_datetime(d, Grain::Year, -7), dttm(2009, 3, 31));
        assert_eq!(shift_datetime(d, Grain::Quarter, 2), dttm(2016, 9, 30));
    }

    //#[test]
    //fn test_enclose() {
        //let dt = Date::from_ymd(2016, 9, 5).and_hms(0, 0, 0);
        //assert_eq!(enclose(dt, Granularity::Year),
                   //Range{
                       //start: Date::from_ymd(2016, 1, 1).and_hms(0, 0, 0),
                       //end: Date::from_ymd(2017, 1, 1).and_hms(0, 0, 0),
                       //grain: Granularity::Year
                   //});
        //assert_eq!(enclose(dt, Granularity::Quarter),
                   //Range{
                       //start: Date::from_ymd(2016, 7, 1).and_hms(0, 0, 0),
                       //end: Date::from_ymd(2016, 10, 1).and_hms(0, 0, 0),
                       //grain: Granularity::Quarter
                   //});
        //assert_eq!(enclose(dt, Granularity::Month),
                   //Range{
                       //start: Date::from_ymd(2016, 9, 1).and_hms(0, 0, 0),
                       //end: Date::from_ymd(2016, 10, 1).and_hms(0, 0, 0),
                       //grain: Granularity::Month
                   //});
        //assert_eq!(enclose(dt, Granularity::Week),
                   //Range{
                       //start: Date::from_ymd(2016, 9, 4).and_hms(0, 0, 0),
                       //end: Date::from_ymd(2016, 9, 11).and_hms(0, 0, 0),
                       //grain: Granularity::Week
                   //});
        //assert_eq!(enclose(dt, Granularity::Day),
                   //Range{
                       //start: Date::from_ymd(2016, 9, 5).and_hms(0, 0, 0),
                       //end: Date::from_ymd(2016, 9, 6).and_hms(0, 0, 0),
                       //grain: Granularity::Day
                   //});
    //}
}
