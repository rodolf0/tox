#![deny(warnings)]

type DateTime = chrono::NaiveDateTime;
type Date = chrono::NaiveDate;

use earlgrey::{EarleyForest, EarleyParser};
use kronos as k;
type Shim = kronos::Shim<'static>;

#[derive(Debug, PartialEq)]
pub enum TimeEl {
    Time(k::Range),
    Count(u32),
}

impl TimeEl {
    fn range(self) -> k::Range {
        if let TimeEl::Time(x) = self {
            x
        } else {
            panic!("BUG")
        }
    }
}

#[derive(Clone)]
enum TimeNode {
    Int(i32),
    Grain(k::Grain),
    Shifts(Vec<(k::Grain, i32)>),
    Nop,
    Seq(Shim),
    This(Shim),
    Next(Shim, usize),
    Last(Shim, usize),
    RefNext(Shim, DateTime),
    RefPrev(Shim, DateTime),
    Until(Shim, DateTime),
    Since(Shim, DateTime),
    Between(Shim, DateTime, DateTime),
}

// Shift a sequence by multiple shifts
fn build_shifter(shifts: Vec<(k::Grain, i32)>, sign: i32, grain: k::Grain) -> Shim {
    // get the finest grain of the composition to anchor the lookback
    let grain = std::cmp::min(shifts.iter().min_by_key(|g| g.0).unwrap().0, grain);
    // cap to day granularity at most
    let mut shifted = Shim::new(kronos::Grains(grain));
    // shift the initial sequence by composed shifts
    for s in shifts {
        shifted = Shim::new(kronos::shift(shifted, s.0, sign * s.1));
    }
    shifted
}

macro_rules! s {
    ($e:expr) => {
        TimeNode::Seq(Shim::new($e))
    };
}

impl TimeNode {
    fn i32(&self) -> i32 {
        if let TimeNode::Int(x) = self {
            *x as i32
        } else {
            panic!("BUG")
        }
    }
    fn u32(&self) -> u32 {
        if let TimeNode::Int(x) = self {
            *x as u32
        } else {
            panic!("BUG")
        }
    }
    fn usize(&self) -> usize {
        if let TimeNode::Int(x) = self {
            *x as usize
        } else {
            panic!("BUG")
        }
    }
    fn grain(&self) -> k::Grain {
        if let TimeNode::Grain(x) = self {
            *x
        } else {
            panic!("BUG")
        }
    }
    fn seq(&self) -> Shim {
        if let TimeNode::Seq(x) = self {
            x.clone()
        } else {
            panic!("BUG")
        }
    }
    fn shifts(self) -> Vec<(k::Grain, i32)> {
        if let TimeNode::Shifts(x) = self {
            x
        } else {
            panic!("BUG")
        }
    }

    fn eval(&self, reftime: DateTime) -> TimeEl {
        use kronos::TimeSequence;
        use TimeNode::*;
        match self {
            This(seq) => TimeEl::Time(seq._future_raw(&reftime).next().unwrap()),
            Next(seq, n) => TimeEl::Time(
                seq.future(&reftime)
                    // skip_while needed to go over 'This'
                    .skip_while(|x| x.start <= reftime)
                    .nth(*n)
                    .unwrap(),
            ),
            Last(seq, n) => TimeEl::Time(seq.past(&reftime).nth(*n).unwrap()),
            RefNext(seq, t0) => TimeEl::Time(seq.future(t0).next().unwrap()),
            RefPrev(seq, t0) => TimeEl::Time(seq.past(t0).next().unwrap()),
            Until(seq, tn) => TimeEl::Count(
                seq.future(&reftime)
                    .take_while(|x| x.start < *tn && x.end <= *tn)
                    .count() as u32,
            ),
            Since(seq, tn) => TimeEl::Count(
                seq.future(tn)
                    .take_while(|x| x.start < reftime && x.end <= reftime)
                    .count() as u32,
            ),
            Between(seq, t0, tn) => TimeEl::Count(
                seq.future(t0)
                    .take_while(|x| x.start < *tn && x.end <= *tn)
                    .count() as u32,
            ),
            _ => unreachable!(),
        }
    }
}

fn terminal_eval() -> impl Fn(&str, &str) -> TimeNode {
    use crate::constants::*;
    use std::str::FromStr;
    use TimeNode::*;
    |terminal, lex| match terminal {
        "day_ordinal" | "ordinal" => {
            Int(ordinal(lex).or_else(|| short_ordinal(lex)).unwrap() as i32)
        }
        "weekday" => Int(weekday(lex).unwrap() as i32),
        "month" => Int(month(lex).unwrap() as i32),
        "grain" => Grain(k::Grain::from_str(lex).unwrap()),
        "year" => Int(i32::from_str(lex).unwrap()),
        "small_int" => Int(i32::from_str(lex).unwrap()),
        _ => Nop,
    }
}

fn evaler_sequence(ev: &mut EarleyForest<'_, TimeNode>) {
    use kronos::*;

    ev.action("named_seq -> day_ordinal", |t| {
        s!(NthOf(
            t[0].usize(),
            Grains(Grain::Day),
            Grains(Grain::Month)
        ))
    });
    ev.action("named_seq -> weekday", |t| s!(Weekday(t[0].u32())));
    ev.action("named_seq -> month", |t| s!(Month(t[0].u32())));
    ev.action("named_seq -> day_ordinal of month", |t| {
        s!(NthOf(t[0].usize(), Grains(Grain::Day), Month(t[2].u32())))
    });
    ev.action("named_seq -> month day_ordinal", |t| {
        s!(NthOf(t[1].usize(), Grains(Grain::Day), Month(t[0].u32())))
    });
    ev.action("named_seq -> weekday day_ordinal", |t| {
        s!(Intersect(
            Weekday(t[0].u32()),
            NthOf(t[1].usize(), Grains(Grain::Day), Grains(Grain::Month))
        ))
    });
    ev.action("named_seq -> weekday day_ordinal of month", |t| {
        s!(Intersect(
            Weekday(t[0].u32()),
            NthOf(t[1].usize(), Grains(Grain::Day), Month(t[3].u32()))
        ))
    });
    ev.action("named_seq -> weekday month day_ordinal", |t| {
        s!(Intersect(
            Weekday(t[0].u32()),
            NthOf(t[2].usize(), Grains(Grain::Day), Month(t[1].u32()))
        ))
    });

    ev.action("named_seq -> year", |t| s!(Year(t[0].i32())));

    ev.action("named_seq -> weekend", |_| s!(Weekend));
    ev.action("named_seq -> weekends", |_| s!(Weekend));

    ev.action("sequence -> named_seq", |mut t| t.remove(0));
    ev.action("sequence -> grain", |t| s!(Grains(t[0].grain())));
}

fn evaler_comp_seq(ev: &mut EarleyForest<'_, TimeNode>) {
    use kronos::*;

    ev.action("@opt_the -> the", |_| TimeNode::Nop);
    ev.action("@opt_the -> ", |_| TimeNode::Nop);

    ev.action("comp_seq -> ordinal sequence of @opt_the comp_seq", |t| {
        s!(NthOf(t[0].usize(), t[1].seq(), t[4].seq()))
    });
    ev.action("comp_seq -> last sequence of @opt_the comp_seq", |t| {
        s!(LastOf(1, t[1].seq(), t[4].seq()))
    });
    ev.action("comp_seq -> sequence", |mut t| t.remove(0));
}

fn evaler_comp_grain(ev: &mut EarleyForest<'_, TimeNode>) {
    ev.action("comp_grain -> small_int grain", |t| {
        TimeNode::Shifts(vec![(t[1].grain(), t[0].i32())])
    });
    ev.action("comp_grain -> a grain", |t| {
        TimeNode::Shifts(vec![(t[1].grain(), 1)])
    });
    ev.action("comp_grain -> comp_grain and small_int grain", |mut t| {
        let mut shifts = t.remove(0).shifts();
        shifts.push((t[2].grain(), t[1].i32()));
        TimeNode::Shifts(shifts)
    });
    ev.action("comp_grain -> comp_grain and a grain", |mut t| {
        let mut shifts = t.remove(0).shifts();
        shifts.push((t[2].grain(), 1));
        TimeNode::Shifts(shifts)
    });
}

fn evaler_time(ev: &mut EarleyForest<'_, TimeNode>, reftime: DateTime) {
    use kronos::*;
    use TimeNode::*;
    ev.action("time -> today", |_| This(Shim::new(Grains(k::Grain::Day))));
    ev.action("time -> tomorrow", |_| {
        Next(Shim::new(Grains(k::Grain::Day)), 0)
    });
    ev.action("time -> yesterday", |_| {
        Last(Shim::new(Grains(k::Grain::Day)), 0)
    });
    ev.action("time -> on weekday", |t| {
        Next(Shim::new(Weekday(t[1].u32())), 0)
    });
    ev.action("time -> named_seq", |t| This(t[0].seq()));

    ev.action("time -> the comp_seq", |t| This(t[1].seq()));
    ev.action("time -> this comp_seq", |t| This(t[1].seq()));
    ev.action("time -> next comp_seq", |t| Next(t[1].seq(), 0));
    ev.action("time -> last comp_seq", |t| Last(t[1].seq(), 0));

    ev.action("time -> comp_seq after next", |t| Next(t[0].seq(), 1));
    ev.action("time -> comp_seq before last", |t| Last(t[0].seq(), 1));

    ev.action("time -> a named_seq ago", |t| Last(t[1].seq(), 0));
    ev.action("time -> small_int named_seq ago", |t| {
        Last(t[1].seq(), t[0].usize() - 1)
    });
    ev.action("time -> in small_int named_seq", |t| {
        Next(t[2].seq(), t[1].usize() - 1)
    });

    ev.action("time -> comp_grain ago", |mut t| {
        let shifts = t.remove(0).shifts();
        Last(build_shifter(shifts, -1, k::Grain::Second), 0)
    });

    ev.action("time -> in comp_grain", |mut t| {
        let shifts = t.remove(1).shifts();
        Next(build_shifter(shifts, 1, k::Grain::Second), 0)
    });

    ev.action("time -> month year", |t| {
        RefNext(
            Shim::new(Grains(k::Grain::Month)),
            Date::from_ymd(t[1].i32(), t[0].u32(), 1).and_hms(0, 0, 0),
        )
    });

    ev.action("time -> month day_ordinal year", |t| {
        RefNext(
            Shim::new(Grains(k::Grain::Day)),
            Date::from_ymd(t[2].i32(), t[0].u32(), t[1].u32()).and_hms(0, 0, 0),
        )
    });

    ev.action("time -> comp_grain after time", move |mut t| {
        let r = t.remove(2).eval(reftime).range();
        let shifts = t.remove(0).shifts();
        RefNext(build_shifter(shifts, 1, r.grain), r.start)
    });

    ev.action("time -> comp_grain before time", move |mut t| {
        let r = t.remove(2).eval(reftime).range();
        let shifts = t.remove(0).shifts();
        RefPrev(build_shifter(shifts, -1, r.grain), r.start)
    });

    ev.action("time -> sequence until time", move |mut t| {
        let time = t.remove(2).eval(reftime).range().start;
        Until(t.remove(0).seq(), time)
    });

    ev.action("time -> sequence since time", move |mut t| {
        let time = t.remove(2).eval(reftime).range().start;
        Since(t.remove(0).seq(), time)
    });

    ev.action("time -> sequence between time and time", move |mut t| {
        let tn = t.remove(4).eval(reftime).range().start;
        let t0 = t.remove(2).eval(reftime).range().start;
        Between(t.remove(0).seq(), t0, tn)
    });
}

pub struct TimeMachine<'a> {
    parser: EarleyParser,
    evaler: EarleyForest<'a, TimeNode>,
    reftime: DateTime,
}

impl<'a> TimeMachine<'a> {
    pub fn new(reftime: DateTime) -> TimeMachine<'a> {
        use crate::time_parser;
        let mut evaler = EarleyForest::new(terminal_eval());
        evaler_sequence(&mut evaler);
        evaler_comp_seq(&mut evaler);
        evaler_comp_grain(&mut evaler);
        evaler_time(&mut evaler, reftime);
        TimeMachine {
            parser: time_parser::time_parser(),
            evaler,
            reftime,
        }
    }

    pub fn eval(&self, time: &str) -> Result<Vec<TimeEl>, String> {
        let mut tokenizer = lexers::DelimTokenizer::new(time.chars(), ", ", true);

        let state = self
            .parser
            .parse(&mut tokenizer)
            .map_err(|e| format!("TimeMachine {:?} for '{}'", e, time))?;

        Ok(self
            .evaler
            .eval_all(&state)
            .map_err(|e| format!("TimeMachine {:?} for '{}'", e, time))?
            .into_iter()
            .map(|tree| tree.eval(self.reftime))
            .collect())
    }
}
