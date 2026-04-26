use time::UtcDateTime as DateTime;

use earlgrey::{EarleyForest, EarleyParser};
use kronos::{Grain, TimeSeqSpec, TimeSpan};

#[derive(Clone, Copy, Debug)]
pub enum TimeDir {
    Future,
    Past,
}

#[derive(Clone, Debug)]
pub enum Anchor {
    Now,
    Time(DateTime),
    Deferred(Box<TimeValue>, bool), // bool = use_end
}

#[derive(Clone, Debug)]
pub enum TimeValue {
    Seq(TimeSeqSpec),
    QuantitySeq(TimeSeqSpec),
    Span {
        seq: TimeSeqSpec,
        anchor: Anchor,
        dir: TimeDir,
        skip: usize,
    },
    ShiftedSpan {
        anchor: Anchor,
        dir: TimeDir,
        shifts: Vec<(Grain, i32)>,
    },
    RelAnchor(Anchor, TimeDir),
    Interval {
        start: Box<TimeValue>,
        end: Box<TimeValue>,
    },
    Duration(Vec<(Grain, i32)>),
    Quantity(Grain, i32),
    Ordinal(isize),
    Int(i32),
    Keyword,
}

impl TimeValue {
    fn to_seq(self) -> TimeSeqSpec {
        match self {
            TimeValue::Seq(s) => s,
            TimeValue::QuantitySeq(s) => s,
            _ => panic!("Expected Seq, found {:?}", self),
        }
    }

    fn to_int(self) -> i32 {
        match self {
            TimeValue::Int(i) => i,
            _ => panic!("Expected Int, found {:?}", self),
        }
    }

    fn to_ordinal(self) -> isize {
        match self {
            TimeValue::Ordinal(o) => o,
            _ => panic!("Expected Ordinal, found {:?}", self),
        }
    }

    fn to_duration(self) -> Vec<(Grain, i32)> {
        match self {
            TimeValue::Duration(d) => d,
            _ => panic!("Expected Duration, found {:?}", self),
        }
    }

    fn eval(self, reftime: DateTime) -> TimeSpan {
        match self {
            TimeValue::Span { seq, anchor, dir, skip } => {
                let t0 = match anchor {
                    Anchor::Now => reftime,
                    Anchor::Time(t) => t,
                    Anchor::Deferred(tv, use_end) => {
                        let span = tv.eval(reftime);
                        if use_end {
                            span.end
                        } else {
                            span.start
                        }
                    }
                };
                match dir {
                    TimeDir::Future => seq.future(t0).nth(skip).unwrap(),
                    TimeDir::Past => seq.past(t0).nth(skip).unwrap(),
                }
            }
            TimeValue::ShiftedSpan { anchor, dir, shifts } => {
                let t0 = match anchor {
                    Anchor::Now => reftime,
                    Anchor::Time(t) => t,
                    Anchor::Deferred(tv, use_end) => {
                        let span = tv.eval(reftime);
                        if use_end {
                            span.end
                        } else {
                            span.start
                        }
                    }
                };
                
                // Snap down to Day if the shift includes Day or larger grain
                let mut snap_to_day = false;
                for (g, _) in &shifts {
                    if *g >= Grain::Day {
                        snap_to_day = true;
                    }
                }
                
                let mut span = if snap_to_day {
                    let start_time = time::Date::from_calendar_date(t0.year(), t0.month(), t0.day())
                        .unwrap()
                        .with_hms(0, 0, 0)
                        .unwrap()
                        .assume_utc()
                        .into();
                    // Get the Day span starting at `start_time`
                    TimeSeqSpec::grain(Grain::Day)
                        .future(start_time)
                        .next()
                        .unwrap()
                } else {
                    // Create a 1-second span
                    TimeSeqSpec::grain(Grain::Second)
                        .future(t0)
                        .next()
                        .unwrap()
                };
                
                for (g, amt) in shifts {
                    let shift_amt = match dir {
                        TimeDir::Future => amt,
                        TimeDir::Past => -amt,
                    };
                    span = span.shift(g, shift_amt);
                }
                span
            }
            TimeValue::Interval { start, end } => {
                let start_span = start.eval(reftime);
                let end_span = end.eval(reftime);
                TimeSpan {
                    start: start_span.start,
                    end: end_span.start,
                    grain: Grain::Second,
                }
            }
            _ => panic!("eval called on un-evaluable TimeValue"),
        }
    }
}

fn terminal_eval() -> impl Fn(&str, &str) -> TimeValue {
    use crate::constants::*;
    use std::str::FromStr;
    |terminal, lexeme| match terminal {
        "weekday" => TimeValue::Seq(TimeSeqSpec::weekday(weekday(lexeme).unwrap())),
        "monthname" => TimeValue::Seq(TimeSeqSpec::months(Some(month(lexeme).unwrap() as u16))),
        "ordinal" => TimeValue::Ordinal(ordinal(lexeme).or_else(|| short_ordinal(lexeme)).unwrap() as isize),
        "yearnumber" => TimeValue::Int(i32::from_str(lexeme).unwrap()),
        "hourspec" => TimeValue::Seq(TimeSeqSpec::hours(Some(hour_spec(lexeme).unwrap() as u16))),
        "small_int" => TimeValue::Int(i32::from_str(lexeme).unwrap()),
        "time_quantity" => match lexeme {
            "week" | "weeks" => TimeValue::Quantity(Grain::Day, 7),
            q => TimeValue::Quantity(kronos_grain(q).unwrap(), 1),
        },
        "weekend" => TimeValue::Seq(TimeSeqSpec::weekends()),
        "now" | "today" | "yesterday" | "tomorrow" | "this" | "next" | "last" | "ago" | "hence" | "before" | "after" | "from" | "in" | "the" | "a" | "an" | "of" | "on" | "and" | "since" | "until" | "between" => {
            TimeValue::Keyword
        }
        _ => unreachable!("Unknown terminal {}", terminal),
    }
}

pub struct TimeMachine<'a> {
    parser: EarleyParser,
    evaler: EarleyForest<'a, TimeValue>,
}

impl<'a> TimeMachine<'a> {
    pub fn new() -> TimeMachine<'a> {
        let mut ev = EarleyForest::new(terminal_eval());

        // time_expr
        ev.action("time_expr -> time_span", |mut t| t.remove(0));
        ev.action("time_expr -> on time_span", |mut t| t.remove(1));

        // time_span
        ev.action("time_span -> explicit_span", |mut t| t.remove(0));
        ev.action("time_span -> sequence", |mut t| {
            TimeValue::Span { seq: t.remove(0).to_seq(), anchor: Anchor::Now, dir: TimeDir::Future, skip: 0 }
        });

        // explicit_span (anchors)
        ev.action("explicit_span -> now", |_| {
            TimeValue::Span { seq: TimeSeqSpec::grain(Grain::Second), anchor: Anchor::Now, dir: TimeDir::Future, skip: 0 }
        });
        ev.action("explicit_span -> today", |_| {
            TimeValue::Span { seq: TimeSeqSpec::grain(Grain::Day), anchor: Anchor::Now, dir: TimeDir::Future, skip: 0 }
        });
        ev.action("explicit_span -> yesterday", |_| {
            TimeValue::Span { seq: TimeSeqSpec::grain(Grain::Day), anchor: Anchor::Now, dir: TimeDir::Past, skip: 0 }
        });
        ev.action("explicit_span -> tomorrow", |_| {
            TimeValue::Span { seq: TimeSeqSpec::grain(Grain::Day), anchor: Anchor::Now, dir: TimeDir::Future, skip: 1 }
        });
        
        // explicit_span (relative basic)
        ev.action("explicit_span -> this sequence", |mut t| {
            let seq_val = t.remove(1);
            let skip = match &seq_val {
                TimeValue::QuantitySeq(_) => 0,
                TimeValue::Seq(_) => 0,
                _ => 0,
            };
            TimeValue::Span { seq: seq_val.to_seq(), anchor: Anchor::Now, dir: TimeDir::Future, skip }
        });
        ev.action("explicit_span -> next sequence", |mut t| {
            let seq_val = t.remove(1);
            let skip = match &seq_val {
                TimeValue::QuantitySeq(_) => 1,
                TimeValue::Seq(_) => 0, // Named sequence split!
                _ => 1,
            };
            TimeValue::Span { seq: seq_val.to_seq(), anchor: Anchor::Now, dir: TimeDir::Future, skip }
        });
        ev.action("explicit_span -> last sequence", |mut t| {
            let seq_val = t.remove(1);
            let skip = match &seq_val {
                TimeValue::QuantitySeq(_) => 0,
                TimeValue::Seq(_) => 0,
                _ => 1,
            };
            TimeValue::Span { seq: seq_val.to_seq(), anchor: Anchor::Now, dir: TimeDir::Past, skip }
        });

        // explicit_span (anchored explicit)
        ev.action("explicit_span -> sequence yearnumber", |mut t| {
            let year = t.remove(1).to_int();
            let seq = t.remove(0).to_seq();
            TimeValue::Span { seq, anchor: Anchor::Time(TimeSpan::year(year).start), dir: TimeDir::Future, skip: 0 }
        });
        ev.action("explicit_span -> yearnumber", |mut t| {
            let year = t.remove(0).to_int();
            TimeValue::Span { seq: TimeSeqSpec::years(), anchor: Anchor::Time(TimeSpan::year(year).start), dir: TimeDir::Future, skip: 0 }
        });

        // sequence
        ev.action("sequence -> time_quantity", |mut t| {
            let q = t.remove(0);
            match q {
                TimeValue::Quantity(Grain::Day, 7) => TimeValue::QuantitySeq(TimeSeqSpec::weeks()),
                TimeValue::Quantity(g, 1) => TimeValue::QuantitySeq(TimeSeqSpec::grain(g)),
                _ => panic!("Unexpected time_quantity"),
            }
        });
        ev.action("sequence -> weekend", |mut t| t.remove(0));
        ev.action("sequence -> monthname", |mut t| t.remove(0));
        ev.action("sequence -> weekday", |mut t| t.remove(0));
        ev.action("sequence -> hourspec", |mut t| t.remove(0));

        ev.action("sequence -> monthname ordinal", |mut t| {
            let ordinal = t.remove(1).to_ordinal();
            let seq = TimeSeqSpec::days().within(t.remove(0).to_seq(), ordinal).unwrap();
            TimeValue::Seq(seq)
        });
        ev.action("sequence -> ordinal of monthname", |mut t| {
            let month = t.remove(2).to_seq();
            let ordinal = t.remove(0).to_ordinal();
            let seq = TimeSeqSpec::days().within(month, ordinal).unwrap();
            TimeValue::Seq(seq)
        });
        ev.action("sequence -> weekday monthname ordinal", |mut t| {
            let ordinal = t.remove(2).to_ordinal();
            let month = t.remove(1).to_seq();
            let weekday = t.remove(0).to_seq();
            let seq = TimeSeqSpec::days().within(month, ordinal).unwrap().intersection(weekday);
            TimeValue::Seq(seq)
        });
        ev.action("sequence -> weekday ordinal of monthname", |mut t| {
            let month = t.remove(3).to_seq();
            let ordinal = t.remove(1).to_ordinal();
            let weekday = t.remove(0).to_seq();
            let seq = TimeSeqSpec::days().within(month, ordinal).unwrap().intersection(weekday);
            TimeValue::Seq(seq)
        });
        ev.action("sequence -> weekday hourspec", |mut t| {
            let hourspec = t.remove(1).to_seq();
            let weekday = t.remove(0).to_seq();
            TimeValue::Seq(hourspec.intersection(weekday))
        });
        
        ev.action("sequence -> [the] ordinal", |mut t| {
            let ordinal = t.remove(1).to_ordinal();
            let seq = TimeSeqSpec::days().within(TimeSeqSpec::months(None), ordinal).unwrap();
            TimeValue::Seq(seq)
        });
        ev.action("sequence -> weekday [the] ordinal", |mut t| {
            let ordinal = t.remove(2).to_ordinal();
            let weekday = t.remove(0).to_seq();
            let seq = TimeSeqSpec::days().within(TimeSeqSpec::months(None), ordinal).unwrap().intersection(weekday);
            TimeValue::Seq(seq)
        });
        
        // Generated EBNF rules for counts
        ev.action("[the|a|small_int] -> small_int", |mut t| t.remove(0));
        ev.action("[the|a|small_int] -> the", |_| TimeValue::Int(1));
        ev.action("[the|a|small_int] -> a", |_| TimeValue::Int(1));
        ev.action("[the|a|small_int] -> ", |_| TimeValue::Int(1));

        ev.action("(a|an|small_int) -> small_int", |mut t| t.remove(0));
        ev.action("(a|an|small_int) -> a", |_| TimeValue::Int(1));
        ev.action("(a|an|small_int) -> an", |_| TimeValue::Int(1));

        // explicit_span -> [the|a|small_int] sequence relative_anchor
        ev.action("explicit_span -> [the|a|small_int] sequence relative_anchor", |mut t| {
            let anchor_val = t.remove(2);
            let seq = t.remove(1).to_seq();
            let count = t.remove(0).to_int() as usize;

            match anchor_val {
                TimeValue::RelAnchor(anchor, dir) => {
                    let skip = count.saturating_sub(1);
                    TimeValue::Span { seq, anchor, dir, skip }
                }
                _ => unreachable!("relative_anchor should be RelAnchor"),
            }
        });

        // explicit_span -> in (a|an|small_int) sequence
        ev.action("explicit_span -> in (a|an|small_int) sequence", |mut t| {
            let count = t.remove(1).to_int() as usize;
            let seq = t.remove(1).to_seq();
            TimeValue::Span { seq, anchor: Anchor::Now, dir: TimeDir::Future, skip: count }
        });

        ev.action("[the] -> the", |_| TimeValue::Keyword);
        ev.action("[the] -> ", |_| TimeValue::Keyword);

        ev.action("explicit_span -> [the] ordinal_qualifier sequence of [the] explicit_span", |mut t| {
            let span = t.remove(5);
            let _the2 = t.remove(4);
            let _of = t.remove(3);
            let seq = t.remove(2).to_seq();
            let nth = t.remove(1).to_ordinal();
            let _the1 = t.remove(0);

            let (dir, use_end, skip) = if nth > 0 {
                (TimeDir::Future, false, (nth - 1) as usize)
            } else {
                (TimeDir::Past, true, (-nth - 1) as usize)
            };

            TimeValue::Span {
                seq,
                anchor: Anchor::Deferred(Box::new(span), use_end),
                dir,
                skip,
            }
        });

        ev.action("sequence -> [the] ordinal_qualifier sequence of [the] sequence", |mut t| {
            let frame = t.remove(5).to_seq();
            let _the2 = t.remove(4);
            let _of = t.remove(3);
            let seq = t.remove(2).to_seq();
            let nth = t.remove(1).to_ordinal() as isize;
            let _the1 = t.remove(0);

            TimeValue::Seq(seq.within(frame, nth).unwrap())
        });
        
        ev.action("duration -> small_int time_quantity", |mut t| {
            let q = t.remove(1);
            let amt = t.remove(0).to_int();
            match q {
                TimeValue::Quantity(g, m) => TimeValue::Duration(vec![(g, amt * m)]),
                _ => panic!("Expected Quantity"),
            }
        });
        ev.action("duration -> a time_quantity", |mut t| {
            let q = t.remove(1);
            match q {
                TimeValue::Quantity(g, m) => TimeValue::Duration(vec![(g, m)]),
                _ => panic!("Expected Quantity"),
            }
        });
        ev.action("duration -> duration and small_int time_quantity", |mut t| {
            let q = t.remove(3);
            let amt = t.remove(2).to_int();
            let mut dur = t.remove(0).to_duration();
            match q {
                TimeValue::Quantity(g, m) => {
                    dur.push((g, amt * m));
                    TimeValue::Duration(dur)
                }
                _ => panic!("Expected Quantity"),
            }
        });
        ev.action("duration -> duration and a time_quantity", |mut t| {
            let q = t.remove(3);
            let mut dur = t.remove(0).to_duration();
            match q {
                TimeValue::Quantity(g, m) => {
                    dur.push((g, m));
                    TimeValue::Duration(dur)
                }
                _ => panic!("Expected Quantity"),
            }
        });

        ev.action("explicit_span -> duration ago", |mut t| {
            let dur = t.remove(0).to_duration();
            TimeValue::ShiftedSpan { anchor: Anchor::Now, dir: TimeDir::Past, shifts: dur }
        });
        ev.action("explicit_span -> in duration", |mut t| {
            let dur = t.remove(1).to_duration();
            TimeValue::ShiftedSpan { anchor: Anchor::Now, dir: TimeDir::Future, shifts: dur }
        });
        
        ev.action("explicit_span -> since time_span", |mut t| {
            let start = t.remove(1);
            let end = TimeValue::Span { seq: TimeSeqSpec::grain(Grain::Second), anchor: Anchor::Now, dir: TimeDir::Future, skip: 0 };
            TimeValue::Interval { start: Box::new(start), end: Box::new(end) }
        });
        ev.action("explicit_span -> until time_span", |mut t| {
            let end = t.remove(1);
            let start = TimeValue::Span { seq: TimeSeqSpec::grain(Grain::Second), anchor: Anchor::Now, dir: TimeDir::Future, skip: 0 };
            TimeValue::Interval { start: Box::new(start), end: Box::new(end) }
        });
        ev.action("explicit_span -> between time_span and time_span", |mut t| {
            let end = t.remove(3);
            let start = t.remove(1);
            TimeValue::Interval { start: Box::new(start), end: Box::new(end) }
        });

        ev.action("relative_anchor -> ago", |_| TimeValue::RelAnchor(Anchor::Now, TimeDir::Past));
        ev.action("relative_anchor -> hence", |_| TimeValue::RelAnchor(Anchor::Now, TimeDir::Future));
        ev.action("relative_anchor -> before last", |_| TimeValue::RelAnchor(Anchor::Now, TimeDir::Past));
        ev.action("relative_anchor -> before time_span", |mut t| {
            let span = t.remove(1);
            TimeValue::RelAnchor(Anchor::Deferred(Box::new(span), false), TimeDir::Past)
        });
        ev.action("relative_anchor -> from next", |_| TimeValue::RelAnchor(Anchor::Now, TimeDir::Future));
        ev.action("relative_anchor -> after next", |_| TimeValue::RelAnchor(Anchor::Now, TimeDir::Future));
        ev.action("relative_anchor -> from time_span", |mut t| {
            let span = t.remove(1);
            TimeValue::RelAnchor(Anchor::Deferred(Box::new(span), true), TimeDir::Future)
        });
        ev.action("relative_anchor -> after time_span", |mut t| {
            let span = t.remove(1);
            TimeValue::RelAnchor(Anchor::Deferred(Box::new(span), true), TimeDir::Future)
        });
        
        ev.action("ordinal_qualifier -> next", |_| TimeValue::Ordinal(1));
        ev.action("ordinal_qualifier -> last", |_| TimeValue::Ordinal(-1));
        ev.action("ordinal_qualifier -> ordinal", |mut t| t.remove(0));
        ev.action("ordinal_qualifier -> last ordinal", |mut t| {
            TimeValue::Ordinal(-t.remove(1).to_ordinal())
        });

        TimeMachine {
            parser: crate::time_parser::time_parser(),
            evaler: ev,
        }
    }

    pub fn eval(&self, time: &str, reftime: Option<DateTime>) -> Result<Vec<TimeSpan>, String> {
        let mut tokenizer = time.split(&[' ', ','][..]).filter(|w| !w.is_empty());
        let state = self
            .parser
            .parse(&mut tokenizer)
            .map_err(|e| format!("TimeMachine {:?} for '{}'", e, time))?;

        // Default reftime to local time
        let reftime = reftime.unwrap_or_else(|| {
            time::OffsetDateTime::now_local()
                .unwrap_or_else(|_| time::OffsetDateTime::now_utc())
                .into()
        });

        Ok(self
            .evaler
            .eval_all(&state)
            .map_err(|e| format!("TimeMachine {:?} for '{}'", e, time))?
            .into_iter()
            .map(|tree| tree.eval(reftime))
            .collect())
    }
}
