extern crate chrono;
type DateTime = chrono::NaiveDateTime;
type Date = chrono::NaiveDate;

use lexers::DelimTokenizer;
use earlgrey::{ParserBuilder, EarleyParser, EarleyEvaler};
use kronos::{Range, Seq, Grain, TimeDir};
use std::str::FromStr;

#[derive(Clone)]
enum TmEl {
    Year(i32),
    Ordinal(u32),
    Seq(Seq),
    Next(Seq, u32),
    Last(Seq, u32),
    This(Seq),
    Fixed(Seq, i32),
    Grain(Grain),
    Since(Seq, Box<TmEl>),
    Until(Seq, Box<TmEl>),
    Between(Seq, Box<TmEl>, Box<TmEl>),
    SmallInt(u32),
    CompGrain(u32, Grain),
    Ago(u32, Grain),
    In(u32, Grain),
    After(u32, Grain, Box<TmEl>),
    Before(u32, Grain, Box<TmEl>),
    Nop,
}

impl TmEl {
    fn range(&self, reftime: DateTime) -> Range {
        match self {
            &TmEl::Next(ref x, n) => x.next(reftime, TimeDir::Future, n),
            &TmEl::Last(ref x, n) => x.next(reftime, TimeDir::Past, n),
            &TmEl::This(ref x) => x.this(reftime),
            &TmEl::Fixed(ref x, y) =>
                x.this(Date::from_ymd(y, 1, 1).and_hms(0, 0, 0)),
            _ => panic!("Can't resolve a range")
        }
    }
}

#[derive(Debug,PartialEq)]
pub enum TimeEl {
    Time(Range),
    Count(u32),
}

macro_rules! pull {
    ($p:path, $e:expr) => (match $e {
        $p(value) => value,
        _ => panic!("Bad pull match")
    })
}

macro_rules! pull2 {
    ($p:path, $e:expr) => (match $e {
        $p(v0, v1) => (v0, v1),
        _ => panic!("Bad pull match")
    })
}

pub struct TimeMachine<'a>(EarleyParser, EarleyEvaler<'a, TmEl>);

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

impl<'a> TimeMachine<'a> {
    pub fn new() -> TimeMachine<'a> {
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

        use kronos::constants as kc;

        let parser = ParserBuilder::new()
            .plug_terminal("weekday", |d| kc::weekday(d).is_some())
            .plug_terminal("month", |d| kc::month(d).is_some())
            .plug_terminal("ordinal", |d|
                kc::ordinal(d).or(kc::short_ordinal(d)).is_some())
            .plug_terminal("grain", |g| Grain::from_str(g).is_some())
            .plug_terminal("year", |y| match i32::from_str(y) {
                Ok(year) if year > 999 && year < 2200 => true,
                _ => false
            })
            .plug_terminal("small_int", |u| match u32::from_str(u) {
                Ok(u) if u < 100 => true, _ => false
            })
            .into_parser("TimeEl", &grammar);

        let mut ev = EarleyEvaler::new(|terminal, t| {
            match terminal {
                "weekday" => TmEl::Seq(
                    Seq::weekday(kc::weekday(t).unwrap())),
                "month" => TmEl::Seq(Seq::month(kc::month(t).unwrap())),
                "ordinal" => TmEl::Ordinal(
                    kc::ordinal(t).or(kc::short_ordinal(t)).unwrap()),
                "grain" => TmEl::Grain(Grain::from_str(t).unwrap()),
                "year" => TmEl::Year(i32::from_str(t).unwrap()),
                "small_int" => TmEl::SmallInt(u32::from_str(t).unwrap()),
                "this" | "next" | "the" | "of" | "christmas" | "last" | "and" |
                "after" | "today" | "tomorrow" | "until" | "since" | "ago" |
                "between" | "in" | "a" | "before" | "yesterday"
                  => TmEl::Nop,
                _ => panic!("Unknown terminal={:?} lexeme={:?}", terminal, t)
            }
        });
        //////////////////////////////////////////////////////////////////////
        ev.action("mday -> ordinal", |mut n| {
            let n = pull!(TmEl::Ordinal, n.swap_remove(0));
            TmEl::Seq(Seq::nthof(n, Seq::grain(Grain::Day),
                       Seq::grain(Grain::Month)))
        });
        //////////////////////////////////////////////////////////////////////
        ev.action("seq -> mday", |mut n| n.swap_remove(0));
        ev.action("seq -> month", |mut n| n.swap_remove(0));
        ev.action("seq -> weekday", |mut n| n.swap_remove(0));
        ev.action("seq -> mday month", |mut n| {
            let seq_month = pull!(TmEl::Seq, n.remove(1));
            let seq_day = pull!(TmEl::Seq, n.remove(0));
            TmEl::Seq(Seq::intersect(seq_month, seq_day))
        });
        ev.action("seq -> mday of month", |mut n| {
            let seq_month = pull!(TmEl::Seq, n.remove(2));
            let seq_day = pull!(TmEl::Seq, n.remove(0));
            TmEl::Seq(Seq::intersect(seq_month, seq_day))
        });
        ev.action("seq -> month mday", |mut n| {
            let seq_month = pull!(TmEl::Seq, n.remove(0));
            let seq_day = pull!(TmEl::Seq, n.remove(0));
            TmEl::Seq(Seq::intersect(seq_month, seq_day))
        });
        ev.action("seq -> weekday mday", |mut n| {
            let seqa = pull!(TmEl::Seq, n.remove(0));
            let seqb = pull!(TmEl::Seq, n.remove(0));
            TmEl::Seq(Seq::intersect(seqa, seqb))
        });
        ev.action("seq -> weekday ordinal of month", |mut n| {
            let seqwday = pull!(TmEl::Seq, n.remove(0));
            let mday = pull!(TmEl::Ordinal, n.remove(0));
            let seqmonth = pull!(TmEl::Seq, n.remove(1));
            let dom = Seq::nthof(mday, Seq::grain(Grain::Day), seqmonth);
            TmEl::Seq(Seq::intersect(dom, seqwday))
        });
        ev.action("seq -> weekday month ordinal", |mut n| {
            let seqwday = pull!(TmEl::Seq, n.remove(0));
            let seqmonth = pull!(TmEl::Seq, n.remove(0));
            let mday = pull!(TmEl::Ordinal, n.remove(0));
            let dom = Seq::nthof(mday, Seq::grain(Grain::Day), seqmonth);
            TmEl::Seq(Seq::intersect(dom, seqwday))
        });
        ev.action("seq -> christmas", |_| TmEl::Seq(
            Seq::nthof(25, Seq::grain(Grain::Day), Seq::month(12))));
        ev.action("seq -> grain", |mut n| {
            let grain = pull!(TmEl::Grain, n.remove(0));
            TmEl::Seq(Seq::grain(grain))
        });
        ev.action("seq -> year", |mut n| {
            TmEl::Seq(Seq::year(pull!(TmEl::Year, n.remove(0))))
        });
        //////////////////////////////////////////////////////////////////////
        ev.action("the_seq -> seq", |mut n| n.swap_remove(0));
        ev.action("the_seq -> the seq", |mut n| n.swap_remove(1));
        ev.action("the_seq -> nthof", |mut n| n.swap_remove(0));
        ev.action("the_seq -> the nthof", |mut n| n.swap_remove(1));
        //////////////////////////////////////////////////////////////////////
        ev.action("shifted_seq -> the_seq", |mut n| n.swap_remove(0));
        ev.action("shifted_seq -> the_seq after next", |mut n| {
            let shifted = pull!(TmEl::Seq, n.swap_remove(0));
            TmEl::Seq(Seq::after_next(shifted, 1))
        });
        ev.action("shifted_seq -> the_seq before last", |mut n| {
            let shifted = pull!(TmEl::Seq, n.swap_remove(0));
            TmEl::Seq(Seq::before_last(shifted, 1))
        });
        //////////////////////////////////////////////////////////////////////
        ev.action("nthof -> ordinal seq of shifted_seq", |mut n| {
            let nth = pull!(TmEl::Ordinal, n.remove(0));
            let win = pull!(TmEl::Seq, n.remove(0));
            let frame = pull!(TmEl::Seq, n.remove(1));
            TmEl::Seq(Seq::nthof(nth, win, frame))
        });
        ev.action("nthof -> last seq of shifted_seq", |mut n| {
            let win = pull!(TmEl::Seq, n.remove(1));
            let frame = pull!(TmEl::Seq, n.remove(2));
            TmEl::Seq(Seq::lastof(1, win, frame))
        });
        //////////////////////////////////////////////////////////////////////
        ev.action("time -> today", |_|
            TmEl::This(Seq::grain(Grain::Day)));
        ev.action("time -> tomorrow", |_|
            TmEl::Next(Seq::grain(Grain::Day), 1));
        ev.action("time -> yesterday", |_|
            TmEl::Last(Seq::grain(Grain::Day), 1));
        ev.action("time -> this seq", |mut n|
            TmEl::This(pull!(TmEl::Seq, n.swap_remove(1))));
        ev.action("time -> next seq", |mut n|
            TmEl::Next(pull!(TmEl::Seq, n.swap_remove(1)), 1));
        ev.action("time -> last shifted_seq", |mut n|
            TmEl::Last(pull!(TmEl::Seq, n.swap_remove(1)), 1));
        ev.action("time -> shifted_seq", |mut n|
            TmEl::This(pull!(TmEl::Seq, n.swap_remove(0))));
        ev.action("time -> the_seq year", |mut n| {
            let seq = pull!(TmEl::Seq, n.remove(0));
            let year = pull!(TmEl::Year, n.remove(0));
            TmEl::Fixed(seq, year)
        });
        //////////////////////////////////////////////////////////////////////
        ev.action("count -> seq until time", |mut n| {
            let seq = pull!(TmEl::Seq, n.remove(0));
            TmEl::Until(seq, Box::new(n.remove(1)))
        });
        ev.action("count -> seq since time", |mut n| {
            let seq = pull!(TmEl::Seq, n.remove(0));
            TmEl::Since(seq, Box::new(n.remove(1)))
        });
        ev.action("count -> seq between time and time", |mut n| {
            let t1 = Box::new(n.swap_remove(4));
            let t0 = Box::new(n.swap_remove(2));
            let seq = pull!(TmEl::Seq, n.swap_remove(0));
            TmEl::Between(seq, t0, t1)
        });
        //////////////////////////////////////////////////////////////////////
        ev.action("comp_grain -> small_int grain", |mut n| {
            let grain = pull!(TmEl::Grain, n.swap_remove(1));
            let small_int = pull!(TmEl::SmallInt, n.swap_remove(0));
            TmEl::CompGrain(small_int, grain)
        });
        ev.action("comp_grain -> a grain", |mut n| {
            let grain = pull!(TmEl::Grain, n.swap_remove(1));
            TmEl::CompGrain(1, grain)
        });
        //////////////////////////////////////////////////////////////////////
        ev.action("time_shift -> comp_grain ago", |mut n| {
            let (n, grain) = pull2!(TmEl::CompGrain, n.swap_remove(0));
            TmEl::Ago(n, grain)
        });
        ev.action("time_shift -> in comp_grain", |mut n| {
            let (n, grain) = pull2!(TmEl::CompGrain, n.swap_remove(1));
            TmEl::In(n, grain)
        });
        ev.action("time_shift -> comp_grain after time", |mut n| {
            let t0 = Box::new(n.swap_remove(2));
            let (n, grain) = pull2!(TmEl::CompGrain, n.swap_remove(0));
            TmEl::After(n, grain, t0)
        });
        ev.action("time_shift -> comp_grain before time", |mut n| {
            let t0 = Box::new(n.swap_remove(2));
            let (n, grain) = pull2!(TmEl::CompGrain, n.swap_remove(0));
            TmEl::Before(n, grain, t0)
        });
        //////////////////////////////////////////////////////////////////////
        ev.action("TimeEl -> time", |mut n| n.swap_remove(0));
        ev.action("TimeEl -> count", |mut n| n.swap_remove(0));
        ev.action("TimeEl -> time_shift", |mut n| n.swap_remove(0));
        //////////////////////////////////////////////////////////////////////
        TimeMachine(parser, ev)
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
            Err(e) => panic!("Time parse failed: {:?}", e),
        };
        self.1.eval_all(&state).into_iter().map(|t| {
            assert!(t.len() == 1);
            match &t[0] {
                &TmEl::Until(ref x, ref tm) => {
                    let deadline = tm.range(reftime);
                    TimeEl::Count(x(reftime, TimeDir::Future)
                        .skip_while(|r| r.end <= reftime)
                        .take_while(|r| r.start < deadline.start)
                        .count() as u32)
                },
                &TmEl::Since(ref x, ref tm) => {
                    let fromtm = tm.range(reftime);
                    TimeEl::Count(x(fromtm.start, TimeDir::Future)
                        .skip_while(|r| r.end <= fromtm.start)
                        .take_while(|r| r.start <= reftime)
                        .count() as u32)
                },
                &TmEl::Between(ref x, ref t0, ref t1) => {
                    let t0 = t0.range(reftime);
                    let t1 = t1.range(reftime);
                    TimeEl::Count(x(t0.start, TimeDir::Future)
                        .skip_while(|r| r.end <= t0.start)
                        .take_while(|r| r.start < t1.start)
                        .count() as u32)
                },
                &TmEl::Ago(n, grain) =>
                    TimeEl::Time(Seq::grain(lower_grain(grain))
                                 .this(reftime).shift(grain, -(n as i32))),
                &TmEl::In(n, grain) =>
                    TimeEl::Time(Seq::grain(lower_grain(grain))
                                 .this(reftime).shift(grain, (n as i32))),
                &TmEl::After(n, grain, ref t0) =>
                    TimeEl::Time(t0.range(reftime).shift(grain, (n as i32))),
                &TmEl::Before(n, grain, ref t0) =>
                    TimeEl::Time(t0.range(reftime).shift(grain, -(n as i32))),
                other => TimeEl::Time(other.range(reftime))
            }
        }).collect()
    }
}
