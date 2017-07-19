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
    Quarter, // TODO: might ned to remove this
    Year,
}

#[derive(Clone,Debug,PartialEq)]
pub struct Range {
    pub start: DateTime,
    pub end: DateTime,
    pub grain: Grain,
}

const INFINITE_FUSE: u16 = 1000;

#[derive(Debug,Clone,Copy,PartialEq)]
pub enum TimeDir {
    Future,
    Past
}

#[derive(Clone)]
pub struct Seq(Rc<Fn(DateTime, TimeDir)->Box<Iterator<Item=Range>>>);

impl ops::Deref for Seq {
    type Target = Rc<Fn(DateTime, TimeDir)->Box<Iterator<Item=Range>>>;
    fn deref(&self) -> &Self::Target { &self.0 }
}

// NOTES
// X: Sequences generate Ranges that have ENDtime after reference-time,
//    they may contain the reference time or start after if discontinuous.
// see duckling http://goo.gl/gxU1Jo
//
// Y: Sequences into the past generate Ranges that have ENDtime before-or-eq
// to reference-time
//
// latent: only applies for TimeDir::Past to hint that a Sequence
// should start without enforcing Y-restriction. This is used mostly when
// a Sequence is going to be used as a frame for another sequence, for example
// in nthof, lastof

impl Seq {
    pub fn _grain(g: Grain, latent: bool) -> Seq {
        Seq(Rc::new(move |reftime, timedir| {
            // Future: X-invariant: end-of-grain(reftime) > reftime
            // Past: Y-invariant: end-of-grain(reftime) <= reftime
            //       unless latent, then it'll encompass reftime
            let sgn = match timedir {TimeDir::Future => 1, TimeDir::Past => -1};
            let start = match (latent, timedir) {
                (false, TimeDir::Past) => 1, _ => 0
            };
            let base = utils::truncate(reftime, g);
            Box::new((start..).map(move |x| Range{
                start: utils::shift_datetime(base, g, sgn * x),
                end: utils::shift_datetime(base, g, sgn * x + 1),
                grain: g
            }))
        }))
    }
    pub fn grain(g: Grain) -> Seq { Seq::_grain(g, false) }

    pub fn _weekday(dow: u32, latent: bool) -> Seq {
        // given X-invariant: end-of-day(reftime-shifted-to-dow) > reftime
        Seq(Rc::new(move |reftime, timedir| {
            // Future: X-invariant: end-of-grain(reftime) > reftime
            // Past: Y-invariant: end-of-grain(reftime) <= reftime
            //       unless latent, then it'll encompass reftime
            let sgn = match timedir {TimeDir::Future => 1, TimeDir::Past => -1};
            let start = match (latent, timedir) {
                (false, TimeDir::Past) => 1, _ => 0
            };
            let base = utils::find_dow(reftime.date(), dow).and_hms(0, 0, 0);
            Box::new((start..).map(move |x| Range{
                start: base + Duration::days(sgn * x * 7),
                end: base + Duration::days(sgn * x * 7 + 1),
                grain: Grain::Day
            }))
        }))
    }

    pub fn weekday(dow: u32) -> Seq { Seq::_weekday(dow, false) }

    pub fn _month(month: u32, latent: bool) -> Seq {
        Seq(Rc::new(move |reftime, timedir| {
            // Future: X-invariant: end-of-grain(reftime) > reftime
            // Past: Y-invariant: end-of-grain(reftime) <= reftime
            //       unless latent, then it'll encompass reftime
            let s = match timedir {TimeDir::Future => 1, TimeDir::Past => -1};
            let start = match (latent, timedir) {
                (false, TimeDir::Past) => 1, _ => 0
            };
            let base = utils::find_month(
                utils::truncate(reftime, Grain::Month).date(), month)
                .and_hms(0, 0, 0);
            Box::new((start..).map(move |x| Range{
                start: utils::shift_datetime(base, Grain::Month, s * 12 * x),
                end: utils::shift_datetime(base, Grain::Month, s * 12 * x + 1),
                grain: Grain::Month
            }))
        }))
    }

    pub fn month(month: u32) -> Seq { Seq::_month(month, false) }

    pub fn _weekend(latent: bool) -> Seq {
        Seq(Rc::new(move |reftime, timedir| {
            // Future: X-invariant: end-of-grain(reftime) > reftime
            // Past: Y-invariant: end-of-grain(reftime) <= reftime
            //       unless latent, then it'll encompass reftime
            let sgn = match timedir {TimeDir::Future => 1, TimeDir::Past => -1};
            let start = match (latent, timedir) {
                (false, TimeDir::Past) => 1, _ => 0
            };
            let mut base = reftime.date();
            if base.weekday() == Weekday::Sun { base = base.pred(); }
            while base.weekday() != Weekday::Sat { base = base.succ(); }
            let base = base.and_hms(0, 0, 0);
            Box::new((start..).map(move |x| Range{
                start: base + Duration::days(sgn * x * 7),
                end: base + Duration::days(sgn * x * 7 + 2),
                grain: Grain::Day
            }))
        }))
    }

    pub fn weekend() -> Seq { Seq::_weekend(false) }

    pub fn _nthof(n: u32, win: Seq, frame: Seq, latent: bool) -> Seq {
        // Future: X-invariant: end-of-grain(reftime) > reftime
        // Past: Y-invariant: end-of-grain(reftime) <= reftime
        //       unless latent, then it'll encompass reftime
        assert!(n > 0);
        // Only allow frame to go in past direction
        Seq(Rc::new(move |reftime, timedir| {
            let win = win.clone();
            let mut fuse = 0;
            Box::new(frame(reftime, timedir)
                .map(move |outer| win(outer.start, TimeDir::Future)
                    // nth window must start within frame of reference
                    .take_while(|inner| {
                        // check inner <win> can be contained within frame
                        // NOTE: most probably not needed
                        assert!(inner.end.signed_duration_since(inner.start) <=
                                outer.end.signed_duration_since(outer.start));
                        inner.start < outer.end
                    }).nth((n-1) as usize))
                // When going to the Past the Frame is instantiated violating
                // Y-invariant (frame=true) to avoid missing 1st element within,
                // so we must skip it here
                .skip_while(move |nth| timedir == TimeDir::Past && !latent &&
                            match nth {
                                &Some(ref x) if x.end > reftime => true,
                                _ => false
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

    pub fn nthof(n: u32, win: Seq, frame: Seq) -> Seq {
        Seq::_nthof(n, win, frame, false)
    }

    pub fn _lastof(n: u32, win: Seq, frame: Seq, latent: bool) -> Seq {
        // Future: X-invariant: end-of-grain(reftime) > reftime
        // Past: Y-invariant: end-of-grain(reftime) <= reftime
        //       unless latent, then it'll encompass reftime
        assert!(n > 0);
        Seq(Rc::new(move |reftime, timedir| {
            let win = win.clone();
            let mut fuse = 0;
            Box::new(frame(reftime, timedir)
                .map(move |outer| {
                    let mut buf = VecDeque::new();
                    for inner in win(outer.start, TimeDir::Future) {
                        if inner.start >= outer.end {
                            return buf.remove((n-1) as usize);
                        }
                        buf.push_front(inner);
                        if buf.len() > n as usize {
                            buf.pop_back();
                        }
                    }
                    None
                })
                // When going to the Past the Frame is instantiated violating
                // Y-invariant (frame=true) to avoid missing 1st element within,
                // so we must skip it here
                .skip_while(move |nth| timedir == TimeDir::Past && !latent &&
                            match nth {
                                &Some(ref x) if x.end > reftime => true,
                                _ => false
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

    pub fn lastof(n: u32, win: Seq, frame: Seq) -> Seq {
        Seq::_lastof(n, win, frame, false)
    }

    pub fn intersect(a: Seq, b: Seq) -> Seq {
        // Both Seqs a, b must have the same Time direction
        Seq(Rc::new(move |reftime, tdir| {
            let mut astream = a(reftime, tdir).peekable();
            let mut bstream = b(reftime, tdir).peekable();
            let mut anext = astream.peek().unwrap().clone();
            let mut bnext = bstream.peek().unwrap().clone();
            // |--- a ---|
            //   |--- b ---|
            Box::new((0..).map(move |_| {
                for _ in 0..INFINITE_FUSE {
                    let overlap = anext.intersect(&bnext);
                    if (tdir == TimeDir::Past && anext.start >= bnext.start) ||
                       (tdir == TimeDir::Future && anext.end <= bnext.end) {
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
        Seq(Rc::new(move |reftime, timedir| Box::new(
            seq(reftime, timedir).map(move |r| Range{
                start: utils::shift_datetime(r.start, g, n),
                end: utils::shift_datetime(r.end, g, n),
                grain: r.grain
            }))))
    }

    pub fn after_next(seq: Seq, n: u32) -> Seq {
        assert!(n > 0);
        Seq(Rc::new(move |reftime, _| {
            let mut seq = seq(reftime, TimeDir::Future).peekable();
            if seq.peek().unwrap().start <= reftime { seq.next(); }
            Box::new(seq.skip(n as usize))
        }))
    }

    pub fn before_last(seq: Seq, n: u32) -> Seq {
        assert!(n > 0);
        Seq(Rc::new(move |reftime, _| {
            Box::new(seq(reftime, TimeDir::Past).skip(n as usize))
        }))
    }

    // apply a transform to each range emited by seq
    // to suppress a value emit Option::None
    // TimeDir is assumed to stay the same
    pub fn map(seq: Seq, f: Rc<Fn(Range)->Option<Range>>) -> Seq {
        Seq(Rc::new(move |reftime, timedir| {
            let f = f.clone();
            Box::new(seq(reftime, timedir).filter_map(move |r| f(r)))
        }))
    }

    // duckling intervals http://tinyurl.com/hk2vu34
    // eg: 2nd monday of june to next month, tuesday to friday
    // TODO: not thorougly tested
    pub fn interval(from: Seq, to: Seq, inclusive: bool) -> Seq {
        let grain =
            from(Date::from_ymd(2016, 1, 1).and_hms(0, 0, 0), TimeDir::Future)
            .next().unwrap().grain;
        Seq(Rc::new(move |reftime, timedir| {
            let reftime = utils::truncate(reftime, utils::next_grain(grain));
            let to = to.clone();
            let mut fuse = 0;
            Box::new(from(reftime, timedir).map(move |ibegin| {
                let iend = to(ibegin.start, TimeDir::Future).next().unwrap();
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

    // TODO: latent?
    pub fn merge(merged: Seq, n: u32) -> Seq {
        assert!(n > 0);
        Seq(Rc::new(move |reftime, timedir| {
            let mut merged = merged(reftime, timedir);
            Box::new((0..).map(move |_| {
                let first = merged.next().unwrap();
                for _ in 1..n-1 { merged.next(); }
                let last = merged.next().unwrap();
                match timedir {
                    TimeDir::Future => Range{
                        start: first.start,
                        end: last.end,
                        grain: first.grain},
                    TimeDir::Past => Range{
                        start: last.start,
                        end: first.end,
                        grain: first.grain},
                }
            }))
        }))
    }
}

impl Seq {
    pub fn _summer(latent: bool) -> Seq {
        // 21st Jun - 21 Sep
        Seq(Rc::new(move |mut tm, timedir| {
            // find summer
            while (tm.month() < 6 || (tm.month() == 6 && tm.day() < 21)) ||
                  (tm.month() > 9 || (tm.month() == 9 && tm.day() >= 21)) {
                tm = utils::shift_datetime(tm, Grain::Day, 1);
            }
            let tm = Date::from_ymd(tm.year(), 6, 21).and_hms(0, 0, 0);
            let tn = Date::from_ymd(tm.year(), 9, 21).and_hms(0, 0, 0);
            // Future: X-invariant: end-of-grain(reftime) > reftime
            // Past: Y-invariant: end-of-grain(reftime) <= reftime
            //       unless latent, then it'll encompass reftime
            let s = match timedir {TimeDir::Future => 1, TimeDir::Past => -1};
            let start = match (latent, timedir) {
                (false, TimeDir::Past) => 1, _ => 0
            };
            Box::new((start..).map(move |x| Range{
                start: utils::shift_datetime(tm, Grain::Year, s * x),
                end: utils::shift_datetime(tn, Grain::Year, s * x),
                grain: Grain::Quarter
            }))
        }))
    }

    pub fn year(y: i32) -> Seq {
        Seq(Rc::new(move |_, _| Box::new((0..1).map(move |_| Range{
            start: Date::from_ymd(y, 1, 1).and_hms(0, 0, 0),
            end: Date::from_ymd(y+1, 1, 1).and_hms(0, 0, 0),
            grain: Grain::Year,
        }))))
    }

    pub fn this(&self, reftime: DateTime) -> Range {
        self.0(reftime, TimeDir::Future).next().unwrap()
    }

    pub fn next(&self, reftime: DateTime, timedir: TimeDir, n: u32) -> Range {
        assert!(n > 0);
        let mut seq = self.0(reftime, timedir);
        let mut base = seq.next().unwrap();
        // All sequences (except Seq::interval) return a first Range that
        // wraps reftime (if the the sequence is not discontinuous). The 'next'
        // method explicitly avoids this first Range if it exists.
        match timedir {
            TimeDir::Future => {
                if base.start <= reftime { base = seq.next().unwrap(); }
                for _ in 0..n-1 { base = seq.next().unwrap(); }
                base
            },
            TimeDir::Past => {
                for _ in 0..n-1 { base = seq.next().unwrap(); }
                base
            }
        }
    }
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

    pub fn truncate(&self, g: Grain) -> Range {
        Range{start: utils::truncate(self.start, g),
              end: utils::truncate(self.end, g), grain: g}
    }
}

impl Grain {
    pub fn from_str<S: AsRef<str>>(s: S) -> Option<Grain> {
        match s.as_ref().to_lowercase().as_ref() {
            "second" | "seconds" => Some(Grain::Second),
            "minute" | "minutes" => Some(Grain::Minute),
            "hour" | "hours" => Some(Grain::Hour),
            "day" | "days" => Some(Grain::Day),
            "week" | "weeks" => Some(Grain::Week),
            "month" | "months" => Some(Grain::Month),
            "quarter" | "quarters" => Some(Grain::Quarter),
            "year" | "years" => Some(Grain::Year),
            _ => None,
        }
    }
}
