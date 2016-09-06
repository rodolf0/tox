use chrono::{Datelike, Weekday};
use chrono::naive::datetime::NaiveDateTime as DateTime;
use chrono::naive::date::NaiveDate as Date;
use std::rc::Rc;
use std::cmp;
use chrono;
use utils;

// shortcircuit bad sequences
const SEQFUSE: usize = 1000;

#[derive(Debug,PartialEq,Eq,PartialOrd,Ord,Clone,Copy)]
pub enum Granularity {
    Day,
    Week,
    Month,
    Quarter,
    Year,
}

// TODO: implement Display for Range to show based on grain
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Range {
    pub start: DateTime,
    pub end: DateTime,
    pub grain: Granularity,
}

// A generator of Ranges
pub type Seq = Rc<Fn(DateTime)->Box<Iterator<Item=Range>>>;

//enum TmDir {
    //Future,
    //Past,
//}

//struct RefTime {
    //start: DateTime,
    //dir: TmDir,
//}

// NOTES
// X: Sequences generate Ranges that have ENDtime after reference-time,
//    they may contain the reference time or start after if discontinuous.
// see duckling http://goo.gl/gxU1Jo

pub fn day_of_week(dow: usize) -> Seq {
    Rc::new(move |reftime| {
        // given X-precondition: (endtime = tm + 1 day) > reftime
        let mut tm = reftime.date();
        while tm.weekday().num_days_from_sunday() != (dow as u32) {
            tm = tm.succ();
        }
        let tm = tm.and_hms(0, 0, 0);
        Box::new((0..).map(move |x| {
            Range{
                start: tm + chrono::Duration::days(x * 7),
                end: tm + chrono::Duration::days(x * 7 + 1),
                grain: Granularity::Day
            }
        }))
    })
}

pub fn month_of_year(moy: usize) -> Seq {
    Rc::new(move |reftime| {
        // X-precondition: (endtime = end-of-month(tm)) > reftime
        let mut tm = Date::from_ymd(reftime.year(), reftime.month(), 1);
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

pub fn day() -> Seq {
    Rc::new(|reftime| {
        // given X-precondition: (endtime = tm + 1 day) > reftime
        let tm = reftime.date().and_hms(0, 0, 0);
        Box::new((0..).map(move |x| {
            Range{
                start: tm + chrono::Duration::days(x),
                end: tm + chrono::Duration::days(x+1),
                grain: Granularity::Day
            }
        }))
    })
}

pub fn weekend() -> Seq {
    Rc::new(|reftime| {
        let mut tm = reftime.date();
        if tm.weekday() == Weekday::Sun {
            tm = tm.pred();
        }
        while tm.weekday() != Weekday::Sat {
            tm = tm.succ();
        }
        let tm = tm.and_hms(0, 0, 0);
        Box::new((0..).map(move |x| {
            Range{
                start: tm + chrono::Duration::days(x * 7),
                end: tm + chrono::Duration::days(x * 7 + 2),
                grain: Granularity::Day
            }
        }))
    })
}

pub fn week() -> Seq {
    Rc::new(|reftime| {
        // X-precondition: (endtime = tm + 1 week) > reftime
        let mut tm = Date::from_isoywd(reftime.isoweekdate().0,
                                       reftime.isoweekdate().1,
                                       Weekday::Mon);
        Box::new((0..).map(move |_| {
            let t0 = tm;
            tm = utils::startof_next_week(tm);
            Range{
                start: t0.and_hms(0, 0, 0),
                end: tm.and_hms(0, 0, 0),
                grain: Granularity::Week
            }
        }))
    })
}

pub fn month() -> Seq {
    Rc::new(|reftime| {
        // X-precondition: (endtime = tm + 1 month) > reftime
        let mut tm = Date::from_ymd(reftime.year(), reftime.month(), 1);
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

pub fn quarter() -> Seq {
    Rc::new(|reftime| {
        // X-precondition: (endtime = tm + 1 quarter) > reftime
        let mut qstart = 1 + 3 * (reftime.month()/4);
        let mut tm = Date::from_ymd(reftime.year(), qstart, 1);
        Box::new((0..).map(move |_| {
            let t0 = tm;
            qstart = (qstart + 3) % 12;
            while tm.month() != qstart {
                tm = utils::startof_next_month(tm);
            }
            Range{
                start: t0.and_hms(0, 0, 0),
                end: tm.and_hms(0, 0, 0),
                grain: Granularity::Quarter
            }
        }))
    })
}

pub fn year() -> Seq {
    Rc::new(|reftime| {
        // X-precondition: (endtime = tm + 1 year) > reftime
        let mut tm = Date::from_ymd(reftime.year(), 1, 1);
        Box::new((0..).map(move |_| {
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

pub fn merge(n: usize, s: Seq) -> Seq {
    struct MergeIt {
        it: Box<Iterator<Item=Range>>,
        tend: Range,
        n: usize,
    };
    impl Iterator for MergeIt {
        type Item = Range;
        fn next(&mut self) -> Option<Range> {
            let t0 = self.tend;
            for _ in 0..self.n {
                self.tend = self.it.next().unwrap();
            }
            Some(Range{start: t0.start, end: self.tend.start, grain: t0.grain})
        }
    }
    Rc::new(move |reftime| {
        let mut ns = s(reftime);
        let tend = ns.next().unwrap();
        Box::new(MergeIt{it: ns, tend: tend, n: n})
    })
}

pub fn nthof(n: usize, win: Seq, within: Seq) -> Seq {
    // For a predictable outcome you probably want aligned sequences
    // 1. take an instance of <within>
    // 2. cycle to the n-th instance if <win> within <within>
    {   // assert win-item.duration < within-item.duration
        let testtm = Date::from_ymd(2000, 1, 1).and_hms(0, 0, 0);
        let a = win(testtm).next().unwrap();
        let b = within(testtm).next().unwrap();
        assert!((a.end - a.start) <= (b.end - b.start));
    }
    Rc::new(move |reftime| {
        let win = win.clone();
        let align = within(reftime).next().unwrap().start;
        //println!("ref={:?} align={:?}, win={:?}",
                 //reftime, align, win(align).next().unwrap());
        Box::new(within(reftime)
                    .take(SEQFUSE) // TODO: panic here ? looks like wrong place
                    .filter_map(move |outer| {
            // we restart <win> each time instead of continuing because we
            // could have overflowed the outer interval and we cant miss items
            // See note X on the skip_while filter, could be inner.start < outer.start
            win(align).skip_while(|inner| inner.end <= outer.start)
                      // Could enforce x.start >= outer.start
                      .take_while(|inner| inner.end <= outer.end)
                      .nth(n - 1)
        }).skip_while(move |range| range.end < reftime)) // overcome alignment
    })
}

pub fn intersect(a: Seq, b: Seq) -> Seq {
    let (a, b) = { // a is the seq with shortest duration items
        let testtm = Date::from_ymd(2000, 1, 1).and_hms(0, 0, 0);
        let x = a(testtm).next().unwrap();
        let y = b(testtm).next().unwrap();
        match (y.end - y.start) < (x.end - x.start) {
            true => (b, a), false => (a, b)
        }
    };
    Rc::new(move |reftime| {
        let a = a.clone();
        let align = b(reftime).next().unwrap().start;
        Box::new(b(reftime).flat_map(move |outer| {
            a(align).skip_while(move |inner| inner.start < outer.start)
                    .take_while(move |inner| inner.end <= outer.end)
        }).skip_while(move |range| range.end < reftime)) // overcome alignment
    })
}

// duckling intervals http://tinyurl.com/hk2vu34
pub fn interval(a: Seq, b: Seq) -> Seq {
    struct IntervalIt {
        s: Box<Iterator<Item=Range>>,
        e: Box<Iterator<Item=Range>>,
    };
    impl Iterator for IntervalIt {
        type Item = Range;
        fn next(&mut self) -> Option<Range> {
            let t0 = self.s.next().unwrap();
            let t1 = self.e.next().unwrap();
            Some(Range{start: t0.start, end: t1.start, grain: t0.grain})
        }
    }
    Rc::new(move |reftime| {
        let align = a(reftime).next().unwrap().start;
        Box::new(IntervalIt{s: a(reftime), e: b(align)})
    })
}

pub fn a_year(y: usize) -> Range {
    Range{
        start: Date::from_ymd(y as i32, 1, 1).and_hms(0, 0, 0),
        end: Date::from_ymd(y as i32 + 1, 1, 1).and_hms(0, 0, 0),
        grain: Granularity::Year
    }
}

pub fn this(s: Seq, r: DateTime) -> Range {
    s(r).next().unwrap()
}

pub fn next(s: Seq, n: usize, r: DateTime) -> Range {
    assert!(n > 0);
    let mut seq = s(r);
    let mut nxt = seq.next();
    // see X note above
    if nxt.unwrap().start <= r { nxt = seq.next(); }
    for _ in 0..n-1 { nxt = seq.next(); }
    nxt.unwrap()
}

// add a duration to a range
pub fn shift(r: Range, n: i32, g: Granularity) -> Range {
    let (s, e) = (r.start.date(), r.end.date());
    let (s, e) = match g {
        Granularity::Year => {
            (utils::date_add(s, n, 0, 0), utils::date_add(e, n, 0, 0))
        },
        Granularity::Quarter => {
            (utils::date_add(s, 0, 3*n as u32, 0), utils::date_add(e, 0, 3*n as u32, 0))
        },
        Granularity::Month => {
            (utils::date_add(s, 0, n as u32, 0), utils::date_add(e, 0, n as u32, 0))
        },
        Granularity::Week => {
            (utils::date_add(s, 0, 0, 7*n as u32), utils::date_add(e, 0, 0, 7*n as u32))
        },
        Granularity::Day => {
            (utils::date_add(s, 0, 0, n as u32), utils::date_add(e, 0, 0, n as u32))
        },
    };
    Range{
        start: s.and_time(r.start.time()),
        end: e.and_time(r.end.time()),
        grain: cmp::min(r.grain, g)
    }
}
