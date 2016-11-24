use chrono::{Datelike, Weekday};
use chrono::naive::datetime::NaiveDateTime as DateTime;
use chrono::naive::date::NaiveDate as Date;
use std::collections::VecDeque;
use std::rc::Rc;
use std::cmp;
use chrono;
use utils;

const EMPTY_FUSE: usize = 1000;

#[derive(Debug,PartialEq,Eq,PartialOrd,Ord,Clone,Copy)]
pub enum Granularity {
    Day,
    Week,
    Month,
    Quarter,
    Year,
}

impl<S: AsRef<str>> From<S> for Granularity {
    fn from(s: S) -> Granularity {
        let s = s.as_ref();
        match s {
            "day" | "Day" => Granularity::Day,
            "week" | "Week" => Granularity::Week,
            "month" | "Month" => Granularity::Month,
            "quarter" | "Quarter" => Granularity::Quarter,
            "year" | "Year" => Granularity::Year,
            _ => panic!("Can't build Granularity from [{}]", s)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Range {
    pub start: DateTime,
    pub end: DateTime,
    pub grain: Granularity,
}

//enum TmDir {
    //Future,
    //Past,
//}

//struct RefTime {
    //start: DateTime,
    //dir: TmDir,
//}

// A generator of Ranges
pub type Seq = Rc<Fn(DateTime)->Box<Iterator<Item=Range>>>;


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
        let tm = reftime.date();
        let ddiff = tm.weekday().num_days_from_sunday();
        let mut tm = utils::date_sub(tm, 0, 0, ddiff);
        Box::new((0..).map(move |_| {
            let t0 = tm;
            tm = t0 + chrono::Duration::days(7);
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
                    .take(EMPTY_FUSE)
                    .filter_map(move |outer| {
            // we restart <win> each time instead of continuing because we
            // could have overflowed the outer interval and we cant miss items
            // See note X on the skip_while filter, could be inner.start < outer.start
            win(align).skip_while(|inner| inner.end <= outer.start)
                      // Could enforce x.start >= outer.start
                      .take_while(|inner| inner.end <= outer.end)
                      .nth(n - 1)
        })) //.skip_while(move |range| range.end < reftime))
    })
}

pub fn lastof(n: usize, win: Seq, within: Seq) -> Seq {
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
                    .take(EMPTY_FUSE)
                    .filter_map(move |outer| {
            // we restart <win> each time instead of continuing because we
            // could have overflowed the outer interval and we cant miss items
            // See note X on the skip_while filter, could be inner.start < outer.start
            let witems = win(align).skip_while(|inner| inner.end <= outer.start);
            let mut buf = VecDeque::new();
            for inner in witems {
                if inner.start >= outer.end {
                    return Some(buf[n-1]);
                }
                buf.push_front(inner);
                if buf.len() > n {
                    buf.pop_back();
                }
            }
            None
        })) //.skip_while(move |range| range.end < reftime))
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
        Box::new(b(reftime)
                 .take(EMPTY_FUSE)
                 .flat_map(move |outer| {
            a(align).skip_while(move |inner| inner.start < outer.start)
                    .take_while(move |inner| inner.end <= outer.end)
        })) //.skip_while(move |range| range.end < reftime))
    })
}

pub fn skip(s: Seq, n: usize) -> Seq {
    Rc::new(move |reftime| { Box::new(s(reftime).skip(n)) })
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

pub fn a_year(y: i32) -> Range {
    Range{
        start: Date::from_ymd(y, 1, 1).and_hms(0, 0, 0),
        end: Date::from_ymd(y + 1, 1, 1).and_hms(0, 0, 0),
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
    let dtfunc = if n >= 0 { utils::date_add } else { utils::date_sub };
    let n = if n < 0 { -n as u32 } else { n as u32 };
    let (s, e) = match g {
        Granularity::Year => {
            (dtfunc(s, n as i32, 0, 0), dtfunc(e, n as i32, 0, 0))
        },
        Granularity::Quarter => {
            (dtfunc(s, 0, 3*n, 0), dtfunc(e, 0, 3*n, 0))
        },
        Granularity::Month => {
            (dtfunc(s, 0, n, 0), dtfunc(e, 0, n, 0))
        },
        Granularity::Week => {
            (dtfunc(s, 0, 0, 7*n), dtfunc(e, 0, 0, 7*n))
        },
        Granularity::Day => {
            (dtfunc(s, 0, 0, n), dtfunc(e, 0, 0, n))
        },
    };
    Range{
        start: s.and_time(r.start.time()),
        end: e.and_time(r.end.time()),
        grain: cmp::min(r.grain, g)
    }
}
