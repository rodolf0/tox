use chrono::{Duration, Datelike};
use chrono::naive::datetime::NaiveDateTime as DateTime;
use chrono::naive::date::NaiveDate as Date;

use utils;

use std::rc::Rc;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Granularity {
    //Second,
    //Minute,
    //Hour,
    //TimeOfDay, // ??
    Day,
    Month,
    //Season,
    //Quarter,
    //Weekend,
    //Week,
    Year,
    //Decade,
    //Century,
    //TempD, // constante dependent duration
}

#[derive(Clone, Debug, PartialEq)]
pub struct Range {
    pub start: DateTime,
    pub end: DateTime,
    pub grain: Granularity,
}

// A generator of Ranges
pub type Seq = Rc<Fn()->Box<Iterator<Item=Range>>>;

//enum TmDirection {
    //Future,
    //Past,
//}

//struct RefTime {
    //start: DateTime,
    //dir: TmDirection,
//}

pub fn day_of_week(tm: DateTime, dow: usize) -> Seq {
    Rc::new(move || {
        let mut tm = tm.date();
        while (dow as u32) != tm.weekday().num_days_from_sunday() {
            tm = tm.succ();
        }
        let tm = tm.and_hms(0, 0, 0);
        Box::new((0..).map(move |x| {
            Range{
                start: tm + Duration::days(x * 7),
                end: tm + Duration::days(x * 7 + 1),
                grain: Granularity::Day
            }
        }))
    })
}

pub fn month_of_year(tm: DateTime, moy: usize) -> Seq {
    Rc::new(move || {
        let mut tm = Date::from_ymd(tm.year(), tm.month(), 1);
        Box::new((0..).map(move |_| {
            while tm.month() != (moy as u32) {
                tm = utils::startof_next_month(tm);
            }
            let t0 = tm;
            tm = utils::startof_next_month(tm);
            Range{
                start: t0.and_hms(0, 0, 0),
                end: tm.and_hms(0, 0, 0),
                grain: Granularity::Month
            }
        }))
    })
}

pub fn day(tm: DateTime) -> Seq {
    Rc::new(move || {
        let tm = tm.date().and_hms(0, 0, 0);
        Box::new((0..).map(move |x| {
            Range{
                start: tm + Duration::days(x),
                end: tm + Duration::days(x+1),
                grain: Granularity::Day
            }
        }))
    })
}

pub fn month(tm: DateTime) -> Seq {
    Rc::new(move || {
        let mut tm = Date::from_ymd(tm.year(), tm.month(), 1);
        Box::new((0..).map(move |_| {
            let t0 = tm;
            tm = utils::startof_next_month(tm);
            Range{
                start: t0.and_hms(0, 0, 0),
                end: tm.and_hms(0, 0, 0),
                grain: Granularity::Month
            }
        }))
    })
}

pub fn year(tm: DateTime) -> Seq {
    Rc::new(move || {
        let mut tm = Date::from_ymd(tm.year(), 1, 1);
        Box::new((0..).map(move |x| {
            let t0 = tm;
            tm = utils::startof_next_year(tm);
            Range{
                start: t0.and_hms(0, 0, 0),
                end: tm.and_hms(0, 0, 0),
                grain: Granularity::Year
            }
        }))
    })
}

// useful for sequence alignment
pub fn seq_start(s: Seq) -> DateTime {
    s().next().unwrap().start
}

pub fn nth(n: usize, win: Seq, within: Seq) -> Seq {
    // For a predictable outcome you probably want aligned sequences
    // 1. take an instance of <within>
    // 2. cycle to the n-th instance if <win> within <within>
    // TODO: panic on win.duration > within.duration (currently will return empty seq?)
    Rc::new(move || {
        const FUSE: usize = 10000;
        // we need a clone of win each time instead of continuing because we could have
        // overflowed the outer <within> interval and we don't want to miss items
        let win = win.clone();
        Box::new(within().take(FUSE).filter_map(move |outer| {
            let x = win().skip_while(|inner| inner.start < outer.start)
                         .nth(n - 1).unwrap();
            match x.start >= outer.start && x.end <= outer.end {
                true => Some(x),
                false => None
            }
        }))
    })
}

//fn intersect(a: Seq, b: Seq) -> Seq {
    //Rc::new(move || {
        //let x = a.clone()().next().unwrap();
        //let y = b.clone()().next().unwrap();
        //let (a, b) = match y.1 < x.1 {
            //true => (b.clone(), a.clone()),
            //false => (a.clone(), b.clone())
        //};
        //// TODO: not reseting <a> (and skipping to sync with next <b>) should we?
        //Box::new(b().flat_map(move |x| {
            //let x2 = x.clone();
            //a().skip_while(move |y| y.0 < x.0)
             //.take_while(move |y| (y.0 + y.1) <= (x2.0 + x2.1))
        //}))
    //})
//}