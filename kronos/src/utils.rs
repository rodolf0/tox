use chrono::naive::date::NaiveDate as Date;
use chrono::Datelike;
use std::cmp;

pub fn startof_next_month(d: Date) -> Date {
    date_add(Date::from_ymd(d.year(), d.month(), 1), 0, 1, 0)
}

pub fn startof_next_year(d: Date) -> Date {
    Date::from_ymd(d.year() + 1, 1, 1)
}

pub fn days_in_month(m: u32, y: i32) -> u32 {
    static DIM: [u32;12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    assert!(m > 0 && m <= 12);
    // check when february has 29 days
    if m == 2 && y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) { return 29; }
    DIM[(m-1) as usize]
}

pub fn date_add(dt: Date, y: i32, mut m: u32, mut d: u32) -> Date {
    let mut day = dt.day();
    let mut month = dt.month();
    let mut year = dt.year();
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
    year += y;
    day = cmp::min(day, days_in_month(month, year));
    Date::from_ymd(year, month, day)
}

pub fn date_sub(dt: Date, y: i32, mut m: u32, mut d: u32) -> Date {
    let mut day = dt.day();
    let mut month = dt.month();
    let mut year = dt.year();
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
    year -= y;
    day = cmp::min(day, days_in_month(month, year));
    Date::from_ymd(year, month, day)
}

#[cfg(test)]
mod tests {
    use chrono::naive::date::NaiveDate as Date;
    use super::{date_add, date_sub};
    #[test]
    fn test_dateadd() {
        let dt = Date::from_ymd(2016, 9, 5);
        assert_eq!(date_add(dt, 0, 0, 30), Date::from_ymd(2016, 10, 5));
        assert_eq!(date_add(dt, 0, 0, 1234), Date::from_ymd(2020, 1, 22));
        assert_eq!(date_add(dt, 0, 0, 365), Date::from_ymd(2017, 9, 5));
        assert_eq!(date_add(dt, 0, 0, 2541), Date::from_ymd(2023, 8, 21));
        assert_eq!(date_add(dt, 0, 1, 0), Date::from_ymd(2016, 10, 5));
        let dt = Date::from_ymd(2016, 1, 30);
        assert_eq!(date_add(dt, 0, 1, 0), Date::from_ymd(2016, 2, 29));
        assert_eq!(date_add(dt, 0, 2, 0), Date::from_ymd(2016, 3, 30));
        assert_eq!(date_add(dt, 0, 12, 0), Date::from_ymd(2017, 1, 30));
        let dt = Date::from_ymd(2016, 12, 31);
        assert_eq!(date_add(dt, 0, 1, 0), Date::from_ymd(2017, 1, 31));
    }
    #[test]
    fn test_datesub() {
        let dt = Date::from_ymd(2016, 9, 5);
        assert_eq!(date_sub(dt, 0, 0, 3), Date::from_ymd(2016, 9, 2));
        assert_eq!(date_sub(dt, 0, 0, 6), Date::from_ymd(2016, 8, 30));
        assert_eq!(date_sub(dt, 0, 0, 36), Date::from_ymd(2016, 7, 31));
        assert_eq!(date_sub(dt, 0, 0, 1234), Date::from_ymd(2013, 4, 20));
        assert_eq!(date_sub(dt, 0, 0, 365), Date::from_ymd(2015, 9, 6));
        assert_eq!(date_sub(dt, 0, 1, 0), Date::from_ymd(2016, 8, 5));
        let dt = Date::from_ymd(2016, 9, 1);
        assert_eq!(date_sub(dt, 0, 0, 1), Date::from_ymd(2016, 8, 31));
        let dt = Date::from_ymd(2016, 1, 31);
        assert_eq!(date_sub(dt, 0, 1, 0), Date::from_ymd(2015, 12, 31));
        let dt = Date::from_ymd(2016, 3, 31);
        assert_eq!(date_sub(dt, 0, 1, 0), Date::from_ymd(2016, 2, 29));
        assert_eq!(date_sub(dt, 0, 2, 0), Date::from_ymd(2016, 1, 31));
        assert_eq!(date_sub(dt, 0, 13, 0), Date::from_ymd(2015, 2, 28));
    }
}
