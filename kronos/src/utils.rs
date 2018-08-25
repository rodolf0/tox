#![deny(warnings)]

extern crate chrono;
use self::chrono::Timelike;
use self::chrono::Datelike;
use self::chrono::Weekday;

use types::{Grain, Date, DateTime, Duration, Season};


pub fn enclosing_grain_from_duration(duration: Duration) -> Grain {
    if duration <= Duration::seconds(1) { return Grain::Second }
    if duration <= Duration::minutes(1) { return Grain::Minute }
    if duration <= Duration::hours(1) { return Grain::Hour }
    if duration <= Duration::days(1) { return Grain::Day }
    if duration <= Duration::days(7) { return Grain::Week }
    if duration <= Duration::days(31) { return Grain::Month }
    if duration <= Duration::days(92) { return Grain::Quarter }
    if duration <= Duration::days(183) { return Grain::Half }
    if duration <= Duration::days(366) { return Grain::Year }
    if duration <= Duration::days(5 * 366) { return Grain::Lustrum }
    if duration <= Duration::days(10 * 366) { return Grain::Decade }
    if duration <= Duration::days(100 * 366) { return Grain::Century }
    Grain::Millenium
}

pub fn truncate(d: DateTime, granularity: Grain) -> DateTime {
    use types::Grain::*;
    match granularity {
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
            let quarter_start = 1 + 3 * ((d.month()-1) / 3);
            Date::from_ymd(d.year(), quarter_start, 1).and_hms(0, 0, 0)
        },
        Half => {
            let half_start = 1 + 6 * ((d.month() - 1) / 6);
            Date::from_ymd(d.year(), half_start, 1).and_hms(0, 0, 0)
        },
        Year => Date::from_ymd(d.year(), 1, 1).and_hms(0, 0, 0),
        Lustrum =>
            Date::from_ymd(d.year() - d.year() % 5, 1, 1).and_hms(0, 0, 0),
        Decade =>
            Date::from_ymd(d.year() - d.year() % 10, 1, 1).and_hms(0, 0, 0),
        Century =>
            Date::from_ymd(d.year() - d.year() % 100, 1, 1).and_hms(0, 0, 0),
        Millenium =>
            Date::from_ymd(d.year() - d.year() % 1000, 1, 1).and_hms(0, 0, 0),
    }
}

pub fn find_dow(mut date: Date, dow: u32, future: bool) -> Date {
    while date.weekday().num_days_from_sunday() != dow {
        date = if future { date.succ() } else { date.pred() }
    }
    date
}

pub fn find_month(mut date: Date, month: u32, future: bool) -> Date {
    while date.month() != month {
        date = if future {
            dtshift::add(date, 0, 1, 0)
        } else {
            dtshift::sub(date, 0, 1, 0)
        }
    }
    date
}

pub fn find_season(dt: Date, season: Season, future: bool, north: bool)
    -> (Date, Date)
{
    let season_lookup = |date: Date| {
        if date.month() < 3 || date.month() == 3 && date.day() < 21 {
            Season::Winter
        } else if date.month() < 6 || date.month() == 6 && date.day() < 21 {
            Season::Spring
        } else if date.month() < 9 || date.month() == 9 && date.day() < 21 {
            Season::Summer
        } else if date.month() < 12 || date.month() == 12 && date.day() < 21 {
            Season::Autumn
        } else {
            Season::Winter
        }
    };
    let search = |end_mo, mut date: Date| {
        let inc = if future { Duration::days(1) } else { Duration::days(-1) };
        while season_lookup(date) != season {
            date += inc;
        }
        let end_date = Date::from_ymd(date.year(), end_mo, 21);
        let start_date = dtshift::sub(end_date, 0, 3, 0);
        (start_date, end_date)
    };
    match (season, north) {
        (Season::Spring, true) |
        (Season::Autumn, false) => search(6, dt),
        (Season::Summer, true) |
        (Season::Winter, false) => search(9, dt),
        (Season::Autumn, true) |
        (Season::Spring, false) => search(12, dt),
        (Season::Winter, true) |
        (Season::Summer, false) => search(3, dt),
    }
}

pub fn find_weekend(mut date: Date, future: bool) -> Date {
    if date.weekday() == Weekday::Sun { date = date.pred(); }
    while date.weekday() != Weekday::Sat {
        date = if future {
            date.succ()
        } else {
            date.pred()
        };
    }
    date
}

pub fn shift_datetime(d: DateTime, granularity: Grain, n: i32) -> DateTime {
    use types::Grain::*;
    let m = if n >= 0 {n as u32} else {(-n) as u32};
    let shiftfn = if n >= 0 {dtshift::add} else {dtshift::sub};
    match granularity {
        Second => d + Duration::seconds(n as i64),
        Minute => d + Duration::minutes(n as i64),
        Hour => d + Duration::hours(n as i64),
        Day => d + Duration::days(n as i64),
        Week => d + Duration::weeks(n as i64),
        Month => shiftfn(d.date(), 0, m, 0).and_time(d.time()),
        Quarter => shiftfn(d.date(), 0, 3 * m, 0).and_time(d.time()),
        Half => shiftfn(d.date(), 0, 6 * m, 0).and_time(d.time()),
        Year => shiftfn(d.date(), m, 0, 0).and_time(d.time()),
        Lustrum => shiftfn(d.date(), 5 * m, 0, 0).and_time(d.time()),
        Decade => shiftfn(d.date(), 10 * m, 0, 0).and_time(d.time()),
        Century => shiftfn(d.date(), 100 * m, 0, 0).and_time(d.time()),
        Millenium => shiftfn(d.date(), 1000 * m, 0, 0).and_time(d.time()),
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
}
