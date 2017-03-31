extern crate chrono;
use chrono::{Datelike, Weekday};

use std::ops;
use std::rc::Rc;
use utils::{Duration, DateTime, Date};
use utils;

#[derive(Debug,PartialEq,Eq,PartialOrd,Ord,Clone,Copy)]
pub enum Grain {
    Second,
    Minute,
    Hour,
    Day,
    Week,
    Month,
    Quarter,
    Year,
}

#[derive(Clone,Debug,PartialEq)]
pub struct Range {
    pub start: DateTime,
    pub end: DateTime,
    pub grain: Grain,
}

#[derive(Clone)]
pub struct Seq(Rc<Fn(DateTime)->Box<Iterator<Item=Range>>>);

impl ops::Deref for Seq {
    type Target = Rc<Fn(DateTime)->Box<Iterator<Item=Range>>>;
    fn deref(&self) -> &Self::Target { &self.0 }
}

//// NOTES
//// X: Sequences generate Ranges that have ENDtime after reference-time,
////    they may contain the reference time or start after if discontinuous.
//// see duckling http://goo.gl/gxU1Jo

impl Seq {
    pub fn from_grain(g: Grain) -> Seq {
        Seq(Rc::new(move |reftime| {
            // given X-precondition: end-of-grain(reftime) > reftime
            let base = utils::truncate(reftime, g);
            Box::new((0..).map(move |x| Range{
                start: utils::shift_datetime(base, g, x),
                end: utils::shift_datetime(base, g, x+1),
                grain: g
            }))
        }))
    }

    pub fn weekday(dow: u32) -> Seq {
        // given X-invariant: end-of-day(reftime-shifted-to-dow) > reftime
        Seq(Rc::new(move |reftime| {
            let base = utils::find_dow(reftime.date(), dow).and_hms(0, 0, 0);
            Box::new((0..).map(move |x| Range{
                start: base + Duration::days(x * 7),
                end: base + Duration::days(x * 7 + 1),
                grain: Grain::Day
            }))
        }))
    }

    pub fn month(month: u32) -> Seq {
        // X-invariant: end-of-month(reftime) > reftime
        Seq(Rc::new(move |reftime| {
            let mut m_end = utils::truncate(reftime, Grain::Month);
            Box::new((0..).map(move |_| {
                let m_start =
                    utils::find_month(m_end.date(), month).and_hms(0, 0, 0);
                m_end = utils::shift_datetime(m_start, Grain::Month, 1);
                Range{start: m_start, end: m_end, grain: Grain::Month}
            }))
        }))
    }

    pub fn weekend() -> Seq {
        Seq(Rc::new(|reftime| {
            let mut base = reftime.date();
            if base.weekday() == Weekday::Sun { base = base.pred(); }
            while base.weekday() != Weekday::Sat { base = base.succ(); }
            let base = base.and_hms(0, 0, 0);
            Box::new((0..).map(move |x| {
                Range{
                    start: base + Duration::days(x * 7),
                    end: base + Duration::days(x * 7 + 2),
                    grain: Grain::Day
                }
            }))
        }))
    }

    pub fn nthof(n: usize, win: Seq, frame: Seq) -> Seq {
        // 1. X-invariant: end-of-frame(reftime) > reftime
        // 2. X-invariant: end-of-win-1(outer.start) > outer.start
        Seq(Rc::new(move |reftime| {
            let win = win.clone();
            Box::new(frame(reftime).flat_map(move |outer|
                win(outer.start)
                    // nth window must start within frame of reference
                    .take_while(|inner| {
                        // check inner <win> can be contained within frame
                        // NOTE: most probably not needed
                        assert!(inner.end.signed_duration_since(inner.start) <=
                                outer.end.signed_duration_since(outer.start));
                        inner.start < outer.end
                    }).nth(n-1)
            ))
        }))
    }
}

impl Seq {
    pub fn summer() -> Seq {
        // 21st Jun - 21 Sep
        Seq(Rc::new(move |mut tm| {
            // find summer
            while (tm.month() < 6 || (tm.month() == 6 && tm.day() < 21)) ||
                  (tm.month() > 9 || (tm.month() == 9 && tm.day() >= 21)) {
                tm = utils::shift_datetime(tm, Grain::Day, 1);
            }
            let tm = Date::from_ymd(tm.year(), 6, 21).and_hms(0, 0, 0);
            let tn = Date::from_ymd(tm.year(), 9, 21).and_hms(0, 0, 0);
            Box::new((0..).map(move |x| Range{
                start: utils::shift_datetime(tm, Grain::Year, x),
                end: utils::shift_datetime(tn, Grain::Year, x),
                grain: Grain::Quarter
            }))
        }))
    }
}




//impl<S: AsRef<str>> From<S> for Granularity {
    //fn from(s: S) -> Granularity {
        //let s = s.as_ref();
        //match s {
            //"second" | "Second" => Granularity::Second,
            //"minute" | "Minute" => Granularity::Minute,
            //"hour" | "Hour" => Granularity::Hour,
            //"day" | "Day" => Granularity::Day,
            //"week" | "Week" => Granularity::Week,
            //"month" | "Month" => Granularity::Month,
            //"quarter" | "Quarter" => Granularity::Quarter,
            //"year" | "Year" => Granularity::Year,
            //_ => panic!("Can't build Granularity from [{}]", s)
        //}
    //}
//}


////impl Range {
    //pub fn a_year(y: i32) -> Range {
        //Range{
            //start: Date::from_ymd(y, 1, 1).and_hms(0, 0, 0),
            //end: Date::from_ymd(y + 1, 1, 1).and_hms(0, 0, 0),
            //grain: Granularity::Year
        //}
    //}

    //// add a duration to a range
    //pub fn shift(r: Range, n: i32, g: Granularity) -> Range {
        //let (s, e) = (r.start.date(), r.end.date());
        //let dtfunc = if n >= 0 { utils::date_add } else { utils::date_sub };
        //let n = if n < 0 { -n as u32 } else { n as u32 };
        //let (s, e) = match g {
            //Granularity::Year => {
                //(dtfunc(s, n as i32, 0, 0), dtfunc(e, n as i32, 0, 0))
            //},
            //Granularity::Quarter => {
                //(dtfunc(s, 0, 3*n, 0), dtfunc(e, 0, 3*n, 0))
            //},
            //Granularity::Month => {
                //(dtfunc(s, 0, n, 0), dtfunc(e, 0, n, 0))
            //},
            //Granularity::Week => {
                //(dtfunc(s, 0, 0, 7*n), dtfunc(e, 0, 0, 7*n))
            //},
            //Granularity::Day => {
                //(dtfunc(s, 0, 0, n), dtfunc(e, 0, 0, n))
            //},
        //};
        //Range{
            //start: s.and_time(r.start.time()),
            //end: e.and_time(r.end.time()),
            //grain: cmp::min(r.grain, g)
        //}
    //}

////}


    //pub fn this(s: Seq, r: DateTime) -> Range {
        //s(r).next().unwrap()
    //}

    //pub fn next(s: Seq, n: usize, r: DateTime) -> Range {
        //assert!(n > 0);
        //let mut seq = s(r);
        //let mut nxt = seq.next();
        //// see X note above
        //if nxt.unwrap().start <= r { nxt = seq.next(); }
        //for _ in 0..n-1 { nxt = seq.next(); }
        //nxt.unwrap()
    //}






//pub fn merge(n: usize, s: Seq) -> Seq {
    //struct MergeIt {
        //it: Box<Iterator<Item=Range>>,
        //tend: Range,
        //n: usize,
    //};
    //impl Iterator for MergeIt {
        //type Item = Range;
        //fn next(&mut self) -> Option<Range> {
            //let t0 = self.tend;
            //for _ in 0..self.n {
                //self.tend = self.it.next().unwrap();
            //}
            //Some(Range{start: t0.start, end: self.tend.start, grain: t0.grain})
        //}
    //}
    //Rc::new(move |reftime| {
        //let mut ns = s(reftime);
        //let tend = ns.next().unwrap();
        //Box::new(MergeIt{it: ns, tend: tend, n: n})
    //})
//}

//pub fn lastof(n: usize, win: Seq, within: Seq) -> Seq {
    //// For a predictable outcome you probably want aligned sequences
    //// 1. take an instance of <within>
    //// 2. cycle to the n-th instance if <win> within <within>
    //{   // assert win-item.duration < within-item.duration
        //let testtm = Date::from_ymd(2000, 1, 1).and_hms(0, 0, 0);
        //let a = win(testtm).next().unwrap();
        //let b = within(testtm).next().unwrap();
        //assert!((a.end - a.start) <= (b.end - b.start));
    //}
    //Rc::new(move |reftime| {
        //let win = win.clone();
        //let align = within(reftime).next().unwrap().start;
        ////println!("ref={:?} align={:?}, win={:?}",
                 ////reftime, align, win(align).next().unwrap());
        //Box::new(within(reftime)
                    //.take(EMPTY_FUSE)
                    //.filter_map(move |outer| {
            //// we restart <win> each time instead of continuing because we
            //// could have overflowed the outer interval and we cant miss items
            //// See note X on the skip_while filter, could be inner.start < outer.start
            //let witems = win(align).skip_while(|inner| inner.end <= outer.start);
            //let mut buf = VecDeque::new();
            //for inner in witems {
                //if inner.start >= outer.end {
                    //return Some(buf[n-1]);
                //}
                //buf.push_front(inner);
                //if buf.len() > n {
                    //buf.pop_back();
                //}
            //}
            //None
        //})) //.skip_while(move |range| range.end < reftime))
    //})
//}

//pub fn intersect(a: Seq, b: Seq) -> Seq {
    //let (a, b) = { // a is the seq with shortest duration items
        //let testtm = Date::from_ymd(2000, 1, 1).and_hms(0, 0, 0);
        //let x = a(testtm).next().unwrap();
        //let y = b(testtm).next().unwrap();
        //match (y.end - y.start) < (x.end - x.start) {
            //true => (b, a), false => (a, b)
        //}
    //};
    //Rc::new(move |reftime| {
        //let a = a.clone();
        //let align = b(reftime).next().unwrap().start;
        //Box::new(b(reftime)
                 //.take(EMPTY_FUSE)
                 //.flat_map(move |outer| {
            //a(align).skip_while(move |inner| inner.start < outer.start)
                    //.take_while(move |inner| inner.end <= outer.end)
        //})) //.skip_while(move |range| range.end < reftime))
    //})
//}

//pub fn skip(s: Seq, n: usize) -> Seq {
    //Rc::new(move |reftime| { Box::new(s(reftime).skip(n)) })
//}

//// duckling intervals http://tinyurl.com/hk2vu34
//pub fn interval(a: Seq, b: Seq) -> Seq {
    //struct IntervalIt {
        //s: Box<Iterator<Item=Range>>,
        //e: Box<Iterator<Item=Range>>,
    //};
    //impl Iterator for IntervalIt {
        //type Item = Range;
        //fn next(&mut self) -> Option<Range> {
            //let t0 = self.s.next().unwrap();
            //let t1 = self.e.next().unwrap();
            //Some(Range{start: t0.start, end: t1.start, grain: t0.grain})
        //}
    //}
    //Rc::new(move |reftime| {
        //let align = a(reftime).next().unwrap().start;
        //Box::new(IntervalIt{s: a(reftime), e: b(align)})
    //})
//}
