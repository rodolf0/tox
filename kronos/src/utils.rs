use chrono::naive::date::NaiveDate as Date;
use chrono::Datelike;

pub fn startof_next_month(d: Date) -> Date {
    let m = d.month();
    let mut next_month = d.clone();
    while m == next_month.month() {
        next_month = next_month.succ();
    }
    next_month
}

pub fn startof_next_year(mut d: Date) -> Date {
    let thisyear = d.year();
    while thisyear == d.year() { d = d.succ(); }
    d
}
