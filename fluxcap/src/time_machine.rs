#![deny(warnings)]

extern crate chrono;
type DateTime = chrono::NaiveDateTime;
type Date = chrono::NaiveDate;

use earlgrey::{EarleyParser, EarleyEvaler, ParserBuilder};
use kronos::constants as kc;
use kronos::{Seq, Grain, TimeDir, Range};
use lexers::DelimTokenizer;
use std::str::FromStr;


fn time_parser() -> EarleyParser {
    let grammar = r#"
    mday := ordinal ;
    seq := mday
         | month
         | weekday
         | mday month
         | mday 'of' month
         | month mday
         | weekday mday
         | weekday ordinal 'of' month
         | weekday month ordinal
         | 'christmas'
         | grain
         | year
         ;
    the_seq := seq
             | 'the' seq
             | nthof
             | 'the' nthof
             ;
    shifted_seq := the_seq
                 | the_seq 'after' 'next'
                 | the_seq 'before' 'last'
                 ;
    nthof := ordinal seq 'of' shifted_seq
           | 'last' seq 'of' shifted_seq
           ;
    time := 'today'
          | 'tomorrow'
          | 'yesterday'
          | 'this' seq
          | 'next' seq
          | 'last' shifted_seq
          | shifted_seq
          | the_seq year
          ;
    count := seq 'until' time
           | seq 'since' time
           | seq 'between' time 'and' time
           ;
    comp_grain := small_int grain
                | 'a' grain
                ;
    time_shift := comp_grain 'ago'
                | 'in' comp_grain
                | comp_grain 'after' time
                | comp_grain 'before' time
                ;
    TimeEl := time | count | time_shift ;
    "#;

    ParserBuilder::new()
        .plug_terminal("weekday", |d| kc::weekday(d).is_some())
        .plug_terminal("month", |d| kc::month(d).is_some())
        .plug_terminal("ordinal",
                       |d| kc::ordinal(d).or(kc::short_ordinal(d)).is_some())
        .plug_terminal("grain", |g| Grain::from_str(g).is_some())
        .plug_terminal("year", |y| match i32::from_str(y) {
            Ok(year) if year > 999 && year < 2200 => true, _ => false})
        .plug_terminal("small_int", |u| match u32::from_str(u) {
            Ok(u) if u < 100 => true, _ => false})
        .into_parser("TimeEl", &grammar)
        .unwrap_or_else(|e| panic!("TimeMachine grammar BUG: {:?}", e))
}

fn time_evaler<'a>() -> EarleyEvaler<'a, T> {
    // provide a function that evaluates tokens
    let mut ev = EarleyEvaler::new(|terminal, t| match terminal {
        "weekday" => T::Seq(Seq::weekday(kc::weekday(t).unwrap())),
        "month" => T::Seq(Seq::month(kc::month(t).unwrap())),
        "ordinal" => T::Ord(kc::ordinal(t).or(kc::short_ordinal(t)).unwrap()),
        "grain" => T::Grain(Grain::from_str(t).unwrap()),
        "year" => T::Year(i32::from_str(t).unwrap()),
        "small_int" => T::SmallInt(u32::from_str(t).unwrap()),
        "this" | "next" | "the" | "of" | "christmas" | "last" | "and" |
        "after" | "today" | "tomorrow" | "until" | "since" | "ago" |
        "between" | "in" | "a" | "before" | "yesterday"
          => T::Nop,
        _ => panic!("Unknown terminal={:?} lexeme={:?}", terminal, t)
    });

    macro_rules! x {
        ($p:path, $e:expr) => (match $e {
            $p(value) => value, _ => panic!("Bad pull match") })}

    macro_rules! x2 {
        ($p:path, $e:expr) => (match $e {
            $p(v0, v1) => (v0, v1), _ => panic!("Bad pull match") })}

    // add semantic time rules
    ev.action("mday -> ordinal", |mut n| {
        let n = x!(T::Ord, n.remove(0));
        T::Seq(Seq::nthof(n, Seq::grain(Grain::Day),
                   Seq::grain(Grain::Month)))
    });

    ev.action("seq -> mday", |mut n| n.remove(0));
    ev.action("seq -> month", |mut n| n.remove(0));
    ev.action("seq -> weekday", |mut n| n.remove(0));
    ev.action("seq -> mday month", |mut n| {
        let seq_month = x!(T::Seq, n.remove(1));
        let seq_day = x!(T::Seq, n.remove(0));
        T::Seq(Seq::intersect(seq_month, seq_day))
    });
    ev.action("seq -> mday of month", |mut n| {
        let seq_month = x!(T::Seq, n.remove(2));
        let seq_day = x!(T::Seq, n.remove(0));
        T::Seq(Seq::intersect(seq_month, seq_day))
    });
    ev.action("seq -> month mday", |mut n| {
        let seq_day = x!(T::Seq, n.remove(1));
        let seq_month = x!(T::Seq, n.remove(0));
        T::Seq(Seq::intersect(seq_month, seq_day))
    });
    ev.action("seq -> weekday mday", |mut n| {
        let seqb = x!(T::Seq, n.remove(1));
        let seqa = x!(T::Seq, n.remove(0));
        T::Seq(Seq::intersect(seqa, seqb))
    });
    ev.action("seq -> weekday ordinal of month", |mut n| {
        let seqmonth = x!(T::Seq, n.remove(3));
        let mday = x!(T::Ord, n.remove(1));
        let seqwday = x!(T::Seq, n.remove(0));
        let dom = Seq::nthof(mday, Seq::grain(Grain::Day), seqmonth);
        T::Seq(Seq::intersect(dom, seqwday))
    });
    ev.action("seq -> weekday month ordinal", |mut n| {
        let mday = x!(T::Ord, n.remove(2));
        let seqmonth = x!(T::Seq, n.remove(1));
        let seqwday = x!(T::Seq, n.remove(0));
        let dom = Seq::nthof(mday, Seq::grain(Grain::Day), seqmonth);
        T::Seq(Seq::intersect(dom, seqwday))
    });
    ev.action("seq -> christmas", |_| T::Seq(
        Seq::nthof(25, Seq::grain(Grain::Day), Seq::month(12))));
    ev.action("seq -> grain", |mut n| {
        let grain = x!(T::Grain, n.remove(0));
        T::Seq(Seq::grain(grain))
    });
    ev.action("seq -> year", |mut n| {
        T::Seq(Seq::year(x!(T::Year, n.remove(0))))
    });

    ev.action("the_seq -> seq", |mut n| n.remove(0));
    ev.action("the_seq -> the seq", |mut n| n.remove(1));
    ev.action("the_seq -> nthof", |mut n| n.remove(0));
    ev.action("the_seq -> the nthof", |mut n| n.remove(1));

    ev.action("shifted_seq -> the_seq", |mut n| n.remove(0));
    ev.action("shifted_seq -> the_seq after next", |mut n| {
        let shifted = x!(T::Seq, n.remove(0));
        T::Seq(Seq::after_next(shifted, 1))
    });
    ev.action("shifted_seq -> the_seq before last", |mut n| {
        let shifted = x!(T::Seq, n.remove(0));
        T::Seq(Seq::before_last(shifted, 1))
    });

    ev.action("nthof -> ordinal seq of shifted_seq", |mut n| {
        let frame = x!(T::Seq, n.remove(3));
        let win = x!(T::Seq, n.remove(1));
        let nth = x!(T::Ord, n.remove(0));
        T::Seq(Seq::nthof(nth, win, frame))
    });
    ev.action("nthof -> last seq of shifted_seq", |mut n| {
        let frame = x!(T::Seq, n.remove(3));
        let win = x!(T::Seq, n.remove(1));
        T::Seq(Seq::lastof(1, win, frame))
    });

    ev.action("time -> today", |_|
        T::This(Seq::grain(Grain::Day)));
    ev.action("time -> tomorrow", |_|
        T::Next(Seq::grain(Grain::Day), 1));
    ev.action("time -> yesterday", |_|
        T::Last(Seq::grain(Grain::Day), 1));
    ev.action("time -> this seq", |mut n|
        T::This(x!(T::Seq, n.remove(1))));
    ev.action("time -> next seq", |mut n|
        T::Next(x!(T::Seq, n.remove(1)), 1));
    ev.action("time -> last shifted_seq", |mut n|
        T::Last(x!(T::Seq, n.remove(1)), 1));
    ev.action("time -> shifted_seq", |mut n|
        T::This(x!(T::Seq, n.remove(0))));
    ev.action("time -> the_seq year", |mut n| {
        let year = x!(T::Year, n.remove(1));
        let seq = x!(T::Seq, n.remove(0));
        T::Fixed(seq, year)
    });

    ev.action("count -> seq until time", |mut n| {
        let reftime = Box::new(n.remove(2));
        T::Until(x!(T::Seq, n.remove(0)), reftime)
    });
    ev.action("count -> seq since time", |mut n| {
        let reftime = Box::new(n.remove(2));
        T::Since(x!(T::Seq, n.remove(0)), reftime)
    });
    ev.action("count -> seq between time and time", |mut n| {
        let t1 = Box::new(n.remove(4));
        let t0 = Box::new(n.remove(2));
        T::Between(x!(T::Seq, n.remove(0)), t0, t1)
    });

    ev.action("comp_grain -> small_int grain", |mut n| {
        let grain = x!(T::Grain, n.remove(1));
        let small_int = x!(T::SmallInt, n.remove(0));
        T::CompGrain(small_int, grain)
    });
    ev.action("comp_grain -> a grain", |mut n| {
        let grain = x!(T::Grain, n.remove(1));
        T::CompGrain(1, grain)
    });

    ev.action("time_shift -> comp_grain ago", |mut n| {
        let (n, grain) = x2!(T::CompGrain, n.remove(0));
        T::Ago(n, grain)
    });
    ev.action("time_shift -> in comp_grain", |mut n| {
        let (n, grain) = x2!(T::CompGrain, n.remove(1));
        T::In(n, grain)
    });
    ev.action("time_shift -> comp_grain after time", |mut n| {
        let t0 = Box::new(n.remove(2));
        let (n, grain) = x2!(T::CompGrain, n.remove(0));
        T::After(n, grain, t0)
    });
    ev.action("time_shift -> comp_grain before time", |mut n| {
        let t0 = Box::new(n.remove(2));
        let (n, grain) = x2!(T::CompGrain, n.remove(0));
        T::Before(n, grain, t0)
    });

    ev.action("TimeEl -> time", |mut n| n.remove(0));
    ev.action("TimeEl -> count", |mut n| n.remove(0));
    ev.action("TimeEl -> time_shift", |mut n| n.remove(0));

    ev
}

fn lower_grain(g: Grain) -> Grain {
    match g {
        Grain::Year => Grain::Day,
        Grain::Quarter => Grain::Day,
        Grain::Month => Grain::Day,
        Grain::Week => Grain::Day,
        Grain::Day => Grain::Day,
        Grain::Hour => Grain::Hour,
        Grain::Minute => Grain::Minute,
        Grain::Second => Grain::Second,
    }
}

#[derive(Clone)]
enum T {
    This(Seq),
    Year(i32),
    Ord(u32),
    Seq(Seq),
    Next(Seq, u32),
    Last(Seq, u32),
    Fixed(Seq, i32),
    Grain(Grain),
    Since(Seq, Box<T>),
    Until(Seq, Box<T>),
    Between(Seq, Box<T>, Box<T>),
    SmallInt(u32),
    CompGrain(u32, Grain),
    Ago(u32, Grain),
    In(u32, Grain),
    After(u32, Grain, Box<T>),
    Before(u32, Grain, Box<T>),
    Nop,
}

#[derive(Debug,PartialEq)]
pub enum TimeEl {
    Time(Range),
    Count(u32),
}

impl T {
    fn range(&self, reftime: DateTime) -> Range {
        match self {
            &T::Next(ref x, n) => x.next(reftime, TimeDir::Future, n),
            &T::Last(ref x, n) => x.next(reftime, TimeDir::Past, n),
            &T::This(ref x) => x.this(reftime),
            &T::Fixed(ref x, y) =>
                x.this(Date::from_ymd(y, 1, 1).and_hms(0, 0, 0)),
            _ => panic!("Can't resolve a range")
        }
    }

    fn eval(&self, reftime: DateTime) -> TimeEl {
        match self {
            &T::Until(ref x, ref tm) => {
                let deadline = tm.range(reftime);
                TimeEl::Count(x(reftime, TimeDir::Future)
                    .skip_while(|r| r.end <= reftime)
                    .take_while(|r| r.start < deadline.start)
                    .count() as u32)
            },
            &T::Since(ref x, ref tm) => {
                let fromtm = tm.range(reftime);
                TimeEl::Count(x(fromtm.start, TimeDir::Future)
                    .skip_while(|r| r.end <= fromtm.start)
                    .take_while(|r| r.start <= reftime)
                    .count() as u32)
            },
            &T::Between(ref x, ref t0, ref t1) => {
                let t0 = t0.range(reftime);
                let t1 = t1.range(reftime);
                TimeEl::Count(x(t0.start, TimeDir::Future)
                    .skip_while(|r| r.end <= t0.start)
                    .take_while(|r| r.start < t1.start)
                    .count() as u32)
            },
            &T::Ago(n, grain) =>
                TimeEl::Time(Seq::grain(lower_grain(grain))
                             .this(reftime).shift(grain, -(n as i32))),
            &T::In(n, grain) =>
                TimeEl::Time(Seq::grain(lower_grain(grain))
                             .this(reftime).shift(grain, (n as i32))),
            &T::After(n, grain, ref t0) =>
                TimeEl::Time(t0.range(reftime).shift(grain, (n as i32))),
            &T::Before(n, grain, ref t0) =>
                TimeEl::Time(t0.range(reftime).shift(grain, -(n as i32))),
            other => TimeEl::Time(other.range(reftime))
        }
    }
}


pub struct TimeMachine<'a>(EarleyParser, EarleyEvaler<'a, T>);

impl<'a> TimeMachine<'a> {
    pub fn new() -> TimeMachine<'a> {
        TimeMachine(time_parser(), time_evaler())
    }

    pub fn eval1(&self, reftime: DateTime, time: &str) -> TimeEl {
        let mut vrange = self.eval(reftime, time);
        assert!(vrange.len() == 1);
        vrange.swap_remove(0)
    }

    pub fn eval(&self, reftime: DateTime, time: &str) -> Vec<TimeEl> {
        let mut tokenizer = DelimTokenizer::from_str(time, ", ", true);
        let state = match self.0.parse(&mut tokenizer) {
            Ok(state) => state,
            Err(e) => {
                eprintln!("TimeMachine {:?} for '{}'", e, time);
                return Vec::new();
            }
        };
        self.1.eval_all(&state)
              .unwrap_or_else(|e| panic!("TimeMachine Error: {:?}", e))
              .into_iter()
              .map(|tree| tree.eval(reftime))
              .collect()
    }
}
