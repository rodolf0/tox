extern crate chrono;
use chrono::{Datelike, Weekday};

use std::{ops, cmp};
use std::rc::Rc;
use std::collections::VecDeque;
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

const INFINITE_FUSE: u16 = 100;

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
        assert!(n > 0);
        Seq(Rc::new(move |reftime| {
            let win = win.clone();
            let mut fuse = 0;
            Box::new(frame(reftime)
                .map(move |outer| win(outer.start)
                    // nth window must start within frame of reference
                    .take_while(|inner| {
                        // check inner <win> can be contained within frame
                        // NOTE: most probably not needed
                        assert!(inner.end.signed_duration_since(inner.start) <=
                                outer.end.signed_duration_since(outer.start));
                        inner.start < outer.end
                    }).nth(n-1))
                .flat_map(move |nth| {
                    fuse = if nth.is_some() { 0 } else { fuse + 1 };
                    if fuse >= INFINITE_FUSE {
                        panic!("Seq::nthof INFINITE_FUSE blown");
                    }
                    nth
                })
            )
        }))
    }

    pub fn lastof(n: usize, win: Seq, frame: Seq) -> Seq {
        // 1. X-invariant: end-of-frame(reftime) > reftime
        // 2. X-invariant: end-of-win-1(outer.start) > outer.start
        assert!(n > 0);
        Seq(Rc::new(move |reftime| {
            let win = win.clone();
            let mut fuse = 0;
            Box::new(frame(reftime)
                .map(move |outer| {
                    let mut buf = VecDeque::new();
                    for inner in win(outer.start) {
                        if inner.start >= outer.end {
                            return buf.remove(n-1);
                        }
                        buf.push_front(inner);
                        if buf.len() > n {
                            buf.pop_back();
                        }
                    }
                    None
                })
                .flat_map(move |nth| {
                    fuse = if nth.is_some() { 0 } else { fuse + 1 };
                    if fuse >= INFINITE_FUSE {
                        panic!("Seq::nthof INFINITE_FUSE blown");
                    }
                    nth
                })
            )
        }))
    }

    pub fn intersect(a: Seq, b: Seq) -> Seq {
        Seq(Rc::new(move |reftime| {
            let mut astream = a(reftime).peekable();
            let mut bstream = b(reftime).peekable();
            let mut anext = astream.peek().unwrap().clone();
            let mut bnext = bstream.peek().unwrap().clone();
            // |--- a ---|
            //   |--- b ---|
            Box::new((0..).map(move |_| {
                for _ in 0..INFINITE_FUSE {
                    let overlap = anext.intersect(&bnext);
                    if anext.end <= bnext.end {
                        astream.next();
                        anext = astream.peek().unwrap().clone();
                    } else {
                        bstream.next();
                        bnext = bstream.peek().unwrap().clone();
                    }
                    if let Some(ovp) = overlap { return ovp; }
                }
                panic!("Seq::intersect INFINITE_FUSE blown");
            }))
        }))
    }

    pub fn shift(seq: Seq, g: Grain, n: i32) -> Seq {
        Seq(Rc::new(move |reftime| Box::new(
            seq(reftime).map(move |r| Range{
                start: utils::shift_datetime(r.start, g, n),
                end: utils::shift_datetime(r.end, g, n),
                grain: r.grain
            }))))
    }

    // apply a transform to each range emited by seq
    // to suppress a value emit Option::None
    pub fn map(seq: Seq, f: Rc<Fn(Range)->Option<Range>>) -> Seq {
        Seq(Rc::new(move |reftime| {
            let f = f.clone();
            Box::new(seq(reftime).filter_map(move |r| f(r)))
        }))
    }

    // duckling intervals http://tinyurl.com/hk2vu34
    // eg: 2nd monday of june to next month, tuesday to friday
    // NOTE: the first range emitted may not contain 'reftime' if 'reftime'
    // is not contained within the first element of the <from> sequence
    pub fn interval(from: Seq, to: Seq, inclusive: bool) -> Seq {
        Seq(Rc::new(move |reftime| {
            let to = to.clone();
            let mut fuse = 0;
            Box::new(from(reftime).map(move |ibegin| {
                let iend = to(ibegin.start).next().unwrap();
                let range = match inclusive {
                    true => Range{
                        start: ibegin.start,
                        end: iend.end,
                        grain: ibegin.grain},
                    false => Range{
                        start: ibegin.start,
                        end: iend.start,
                        grain: ibegin.grain},
                };
                match range.end < range.start {
                    true => None,
                    false => Some(range)
                }
            }).flat_map(move |ival| {
                fuse = if ival.is_some() { 0 } else { fuse + 1 };
                if fuse >= INFINITE_FUSE {
                    panic!("Seq::interval INFINITE_FUSE blown");
                }
                ival
            }))
        }))
    }

    pub fn merge(merged: Seq, n: usize) -> Seq {
        assert!(n > 0);
        Seq(Rc::new(move |reftime| {
            let mut merged = merged(reftime);
            Box::new((0..).map(move |_| {
                let first = merged.next().unwrap();
                for _ in 1..n-1 { merged.next(); }
                let last = merged.next().unwrap();
                Range{start: first.start, end: last.end, grain: first.grain}
            }))
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

    //pub fn this(&self, r: DateTime) -> Range {
        //self.0(r).next().unwrap()
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
}


impl Range {
    pub fn intersect(&self, other: &Range) -> Option<Range> {
        match self.start < other.end && self.end > other.start {
            false => None,
            true => Some(Range{
                start: cmp::max(self.start, other.start),
                end: cmp::min(self.end, other.end),
                grain: cmp::min(self.grain, other.grain)
            })
        }
    }

    pub fn len(&self) -> Duration {
        self.end.signed_duration_since(self.start)
    }

    pub fn shift(&self, g: Grain, n: i32) -> Range {
        Range{start: utils::shift_datetime(self.start, g, n),
              end: utils::shift_datetime(self.end, g, n),
              grain: self.grain}
    }

    pub fn from_grain(d: DateTime, g: Grain) -> Range {
        let start = utils::truncate(d, g);
        Range{start: start, end: utils::shift_datetime(start, g, 1), grain: g}
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
