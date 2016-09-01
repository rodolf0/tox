use chrono::naive::date::NaiveDate as Date;
use chrono::Datelike;

// TODO: could be intelligent about the loop
pub fn startof_next_month(d: Date) -> Date {
    let m = d.month();
    let mut next_month = d.clone();
    while m == next_month.month() {
        next_month = next_month.succ();
    }
    next_month
}

// TODO: could be intelligent about the loop
pub fn startof_next_week(d: Date) -> Date {
    let week = d.isoweekdate().1;
    let mut next_week = d.clone();
    while week == next_week.isoweekdate().1 {
        next_week = next_week.succ();
    }
    next_week
}

// TODO: could be intelligent about the loop
pub fn startof_next_year(d: Date) -> Date {
    let y = d.year();
    let mut next_year = d.clone();
    while y == next_year.year() {
        next_year = startof_next_month(next_year);
    }
    next_year
}
