#![deny(warnings)]

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
    Within(Box<TimeValue>, bool), // bool = use_end
    Relative(Box<TimeValue>, bool), // bool = use_end
}

#[derive(Clone, Debug, PartialEq)]
pub struct CountResult {
    pub unit: String,
    pub span: TimeSpan,
    pub total: f64,
    pub full_spans: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TimeResult {
    Span(TimeSpan),
    Count(CountResult),
}

#[derive(Clone, Debug)]
pub enum TimeValue {
    Seq(Option<String>, TimeSeqSpec),
    QuantitySeq(Option<String>, TimeSeqSpec),
    Count(String, TimeSeqSpec, Box<TimeValue>),
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
    RelAnchor(Anchor, TimeDir, usize),
    Interval {
        start: Box<TimeValue>,
        end: Box<TimeValue>,
    },
    Duration(Vec<(Grain, i32)>),
    Quantity(Option<String>, Grain, i32),
    Ordinal(isize),
    Int(i32),
    Keyword,
}

impl TimeValue {
    fn to_label(&self) -> Option<String> {
        match self {
            TimeValue::Seq(Some(l), _) => Some(l.clone()),
            TimeValue::QuantitySeq(Some(l), _) => Some(l.clone()),
            TimeValue::Quantity(Some(l), _, _) => Some(l.clone()),
            _ => None,
        }
    }

    fn to_seq(self) -> TimeSeqSpec {
        match self {
            TimeValue::Seq(_, s) => s,
            TimeValue::QuantitySeq(_, s) => s,
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

    fn eval_top(self, reftime: DateTime) -> Option<TimeResult> {
        match self {
            TimeValue::Count(label, seq, bound_val) => {
                let bounds = bound_val.eval(reftime)?;
                let mut full_spans = 0;
                let mut total = 0.0;
                for span in seq.future(bounds.start).take_while(|s| s.start < bounds.end) {
                    let overlap_start = span.start.max(bounds.start);
                    let overlap_end = span.end.min(bounds.end);
                    let overlap_duration = (overlap_end - overlap_start).whole_seconds() as f64;
                    let span_duration = (span.end - span.start).whole_seconds() as f64;
                    if span_duration > 0.0 {
                        total += overlap_duration / span_duration;
                    }
                    if span.start >= bounds.start && span.end <= bounds.end {
                        full_spans += 1;
                    }
                }
                Some(TimeResult::Count(CountResult {
                    unit: label,
                    span: bounds,
                    total,
                    full_spans,
                }))
            }
            other => other.eval(reftime).map(TimeResult::Span),
        }
    }

    fn eval(self, reftime: DateTime) -> Option<TimeSpan> {
        match self {
            TimeValue::Span { seq, anchor, dir, skip } => {
                match anchor {
                    Anchor::Within(tv, _) => {
                        let bounds = tv.eval(reftime)?;
                        let mut iter: Box<dyn Iterator<Item = TimeSpan>> = match dir {
                            TimeDir::Future => {
                                Box::new(seq.future(bounds.start).take_while(move |s| s.start < bounds.end))
                            }
                            TimeDir::Past => {
                                Box::new(seq.past(bounds.end).take_while(move |s| s.end > bounds.start))
                            }
                        };
                        iter.nth(skip)
                    }
                    Anchor::Relative(tv, use_end) => {
                        let bounds = tv.eval(reftime)?;
                        let t0 = if use_end { bounds.end } else { bounds.start };
                        match dir {
                            TimeDir::Future => seq.future(t0).nth(skip),
                            TimeDir::Past => seq.past(t0).nth(skip),
                        }
                    }
                    _ => {
                        let t0 = match anchor {
                            Anchor::Now => reftime,
                            Anchor::Time(t) => t,
                            _ => unreachable!(),
                        };
                        match dir {
                            TimeDir::Future => seq.future(t0).nth(skip),
                            TimeDir::Past => seq.past(t0).nth(skip),
                        }
                    }
                }
            }
            TimeValue::ShiftedSpan { anchor, dir, shifts } => {
                let t0 = match anchor {
                    Anchor::Now => reftime,
                    Anchor::Time(t) => t,
                    Anchor::Within(tv, use_end) | Anchor::Relative(tv, use_end) => {
                        let span = tv.eval(reftime)?;
                        if use_end {
                            span.end
                        } else {
                            span.start
                        }
                    }
                };
                
                // Snap down to Day only if all shifts are Day or larger grain
                let mut snap_to_day = true;
                for (g, _) in &shifts {
                    if *g < Grain::Day {
                        snap_to_day = false;
                        break;
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
                Some(span)
            }
            TimeValue::Interval { start, end } => {
                let start_span = start.eval(reftime)?;
                let end_span = end.eval(reftime)?;
                Some(TimeSpan {
                    start: start_span.start,
                    end: end_span.start,
                    grain: Grain::Second,
                })
            }
            _ => panic!("eval called on un-evaluable TimeValue"),
        }
    }
}

fn terminal_eval() -> impl Fn(&str, &str) -> TimeValue {
    use crate::constants::*;
    use std::str::FromStr;
    |terminal, lexeme| match terminal {
        "weekday" => TimeValue::Seq(Some(lexeme.to_string()), TimeSeqSpec::weekday(weekday(lexeme).unwrap())),
        "monthname" => TimeValue::Seq(Some(lexeme.to_string()), TimeSeqSpec::months(Some(month(lexeme).unwrap() as u16))),
        "ordinal" => TimeValue::Ordinal(ordinal(lexeme).or_else(|| short_ordinal(lexeme)).unwrap() as isize),
        "yearnumber" => TimeValue::Int(i32::from_str(lexeme).unwrap()),
        "small_int" => TimeValue::Int(i32::from_str(lexeme).unwrap()),
        "time_quantity" => match lexeme {
            "week" | "weeks" => TimeValue::Quantity(Some(lexeme.to_string()), Grain::Day, 7),
            "fortnight" | "fortnights" => TimeValue::Quantity(Some(lexeme.to_string()), Grain::Day, 14),
            "quarter" | "quarters" => TimeValue::Quantity(Some(lexeme.to_string()), Grain::Month, 3),
            "half" | "halfs" | "halves" => TimeValue::Quantity(Some(lexeme.to_string()), Grain::Month, 6),
            "lustrum" | "lustrums" | "lustra" => TimeValue::Quantity(Some(lexeme.to_string()), Grain::Year, 5),
            "decade" | "decades" => TimeValue::Quantity(Some(lexeme.to_string()), Grain::Year, 10),
            "century" | "centuries" => TimeValue::Quantity(Some(lexeme.to_string()), Grain::Year, 100),
            "millennium" | "millennia" | "millenium" | "milleniums" => TimeValue::Quantity(Some(lexeme.to_string()), Grain::Year, 1000),
            q => TimeValue::Quantity(Some(lexeme.to_string()), kronos_grain(q).unwrap(), 1),
        },
        "weekend" => TimeValue::Seq(Some(lexeme.to_string()), TimeSeqSpec::weekends()),
        "clock_time" => {
            let (h, m, s, grain) = parse_clock_time(lexeme).unwrap();
            let mut seq = TimeSeqSpec::hours(Some(h as u16));
            if grain <= Grain::Minute {
                seq = seq.intersection(TimeSeqSpec::minutes(Some(m as u16)));
            }
            if grain <= Grain::Second {
                seq = seq.intersection(TimeSeqSpec::seconds(Some(s as u16)));
            }
            TimeValue::Seq(Some(lexeme.to_string()), seq)
        }
        "numeric_date" => {
            let (y, m, d) = parse_date(lexeme).unwrap();
            let start_time = time::Date::from_calendar_date(y, m.try_into().unwrap(), d).unwrap()
                .with_hms(0, 0, 0).unwrap()
                .assume_utc().into();
            TimeValue::Span {
                seq: TimeSeqSpec::grain(Grain::Day),
                anchor: Anchor::Time(start_time),
                dir: TimeDir::Future,
                skip: 0,
            }
        }
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
        // sequence counting
        ev.action("time_expr -> sequence since time_span", |mut t| {
            let span = t.remove(2);
            let seq_val = t.remove(0);
            let label = seq_val.to_label().unwrap_or_else(|| "units".to_string());
            let interval = TimeValue::Interval {
                start: Box::new(span),
                end: Box::new(TimeValue::Span { seq: TimeSeqSpec::grain(Grain::Second), anchor: Anchor::Now, dir: TimeDir::Future, skip: 0 })
            };
            TimeValue::Count(label, seq_val.to_seq(), Box::new(interval))
        });
        
        ev.action("time_expr -> sequence until time_span", |mut t| {
            let span = t.remove(2);
            let seq_val = t.remove(0);
            let label = seq_val.to_label().unwrap_or_else(|| "units".to_string());
            let interval = TimeValue::Interval {
                start: Box::new(TimeValue::Span { seq: TimeSeqSpec::grain(Grain::Second), anchor: Anchor::Now, dir: TimeDir::Future, skip: 0 }),
                end: Box::new(span)
            };
            TimeValue::Count(label, seq_val.to_seq(), Box::new(interval))
        });

        ev.action("time_expr -> sequence between time_span and time_span", |mut t| {
            let end = t.remove(4);
            let start = t.remove(2);
            let seq_val = t.remove(0);
            let label = seq_val.to_label().unwrap_or_else(|| "units".to_string());
            let interval = TimeValue::Interval { start: Box::new(start), end: Box::new(end) };
            TimeValue::Count(label, seq_val.to_seq(), Box::new(interval))
        });
        
        ev.action("time_expr -> sequence in time_span", |mut t| {
            let span = t.remove(2);
            let seq_val = t.remove(0);
            let label = seq_val.to_label().unwrap_or_else(|| "units".to_string());
            TimeValue::Count(label, seq_val.to_seq(), Box::new(span))
        });


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
        ev.action("explicit_span -> numeric_date", |mut t| t.remove(0));
        
        // explicit_span (relative basic)
        ev.action("explicit_span -> this sequence", |mut t| {
            let seq_val = t.remove(1);
            let skip = match &seq_val {
                TimeValue::QuantitySeq(..) => 0,
                TimeValue::Seq(..) => 0,
                _ => 0,
            };
            TimeValue::Span { seq: seq_val.to_seq(), anchor: Anchor::Now, dir: TimeDir::Future, skip }
        });
        ev.action("explicit_span -> next sequence", |mut t| {
            let seq_val = t.remove(1);
            let skip = match &seq_val {
                TimeValue::QuantitySeq(..) => 1,
                TimeValue::Seq(..) => 0, // Named sequence split!
                _ => 1,
            };
            TimeValue::Span { seq: seq_val.to_seq(), anchor: Anchor::Now, dir: TimeDir::Future, skip }
        });
        ev.action("explicit_span -> last sequence", |mut t| {
            let seq_val = t.remove(1);
            let skip = match &seq_val {
                TimeValue::QuantitySeq(..) => 0,
                TimeValue::Seq(..) => 0,
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
                TimeValue::Quantity(_, Grain::Day, 7) => TimeValue::QuantitySeq(q.to_label(), TimeSeqSpec::weeks()),
                TimeValue::Quantity(_, g, 1) => TimeValue::QuantitySeq(q.to_label(), TimeSeqSpec::grain(g)),
                TimeValue::Quantity(_, Grain::Month, m) => TimeValue::QuantitySeq(q.to_label(), TimeSeqSpec::month_group(m as u32)),
                TimeValue::Quantity(_, Grain::Year, 5) => TimeValue::QuantitySeq(q.to_label(), TimeSeqSpec::grain(Grain::Year).merge(5)),
                TimeValue::Quantity(_, Grain::Year, m) => TimeValue::QuantitySeq(q.to_label(), TimeSeqSpec::year_group(m as u32)),
                TimeValue::Quantity(_, g, m) => TimeValue::QuantitySeq(q.to_label(), TimeSeqSpec::grain(g).merge(m as u16)),
                _ => panic!("Unexpected time_quantity"),
            }
        });
        ev.action("sequence -> named_sequence", |mut t| t.remove(0));
        
        ev.action("named_sequence -> weekend", |mut t| t.remove(0));
        ev.action("named_sequence -> monthname", |mut t| t.remove(0));
        ev.action("named_sequence -> weekday", |mut t| t.remove(0));
        ev.action("named_sequence -> clock_time", |mut t| t.remove(0));

        ev.action("named_sequence -> monthname ordinal", |mut t| {
            let ordinal = t.remove(1).to_ordinal();
            let seq = TimeSeqSpec::days().within(t.remove(0).to_seq(), ordinal).unwrap();
            TimeValue::Seq(None, seq)
        });
        ev.action("named_sequence -> ordinal of monthname", |mut t| {
            let month = t.remove(2).to_seq();
            let ordinal = t.remove(0).to_ordinal();
            let seq = TimeSeqSpec::days().within(month, ordinal).unwrap();
            TimeValue::Seq(None, seq)
        });
        ev.action("named_sequence -> weekday monthname ordinal", |mut t| {
            let ordinal = t.remove(2).to_ordinal();
            let month = t.remove(1).to_seq();
            let weekday = t.remove(0).to_seq();
            let seq = TimeSeqSpec::days().within(month, ordinal).unwrap().intersection(weekday);
            TimeValue::Seq(None, seq)
        });
        ev.action("named_sequence -> weekday ordinal of monthname", |mut t| {
            let month = t.remove(3).to_seq();
            let ordinal = t.remove(1).to_ordinal();
            let weekday = t.remove(0).to_seq();
            let seq = TimeSeqSpec::days().within(month, ordinal).unwrap().intersection(weekday);
            TimeValue::Seq(None, seq)
        });
        ev.action("named_sequence -> weekday clock_time", |mut t| {
            let clock_time = t.remove(1).to_seq();
            let weekday = t.remove(0).to_seq();
            TimeValue::Seq(None, clock_time.intersection(weekday))
        });
        
        ev.action("named_sequence -> [the] ordinal", |mut t| {
            let ordinal = t.remove(1).to_ordinal();
            let seq = TimeSeqSpec::days().within(TimeSeqSpec::months(None), ordinal).unwrap();
            TimeValue::Seq(None, seq)
        });
        ev.action("named_sequence -> weekday [the] ordinal", |mut t| {
            let ordinal = t.remove(2).to_ordinal();
            let weekday = t.remove(0).to_seq();
            let seq = TimeSeqSpec::days().within(TimeSeqSpec::months(None), ordinal).unwrap().intersection(weekday);
            TimeValue::Seq(None, seq)
        });
        
        // Generated EBNF rules for counts
        ev.action("[the|a|small_int] -> small_int", |mut t| t.remove(0));
        ev.action("[the|a|small_int] -> the", |_| TimeValue::Int(1));
        ev.action("[the|a|small_int] -> a", |_| TimeValue::Int(1));
        ev.action("[the|a|small_int] -> ", |_| TimeValue::Int(1));

        ev.action("(a|an|small_int) -> small_int", |mut t| t.remove(0));
        ev.action("(a|an|small_int) -> a", |_| TimeValue::Int(1));
        ev.action("(a|an|small_int) -> an", |_| TimeValue::Int(1));

        // explicit_span -> [the|a|small_int] named_sequence relative_anchor
        ev.action("explicit_span -> [the|a|small_int] named_sequence relative_anchor", |mut t| {
            let anchor_val = t.remove(2);
            let seq = t.remove(1).to_seq();
            let count = t.remove(0).to_int() as usize;

            match anchor_val {
                TimeValue::RelAnchor(anchor, dir, offset) => {
                    let skip = count.saturating_sub(1) + offset;
                    TimeValue::Span { seq, anchor, dir, skip }
                }
                _ => unreachable!("relative_anchor should be RelAnchor"),
            }
        });

        // explicit_span -> in (a|an|small_int) named_sequence
        ev.action("explicit_span -> in (a|an|small_int) named_sequence", |mut t| {
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
                anchor: Anchor::Within(Box::new(span), use_end),
                dir,
                skip,
            }
        });

        ev.action("explicit_span -> sequence on explicit_span", |mut t| {
            let anchor = Anchor::Within(Box::new(t.remove(2)), false);
            let _on = t.remove(1);
            let seq = t.remove(0).to_seq();
            TimeValue::Span { seq, anchor, dir: TimeDir::Future, skip: 0 }
        });

        ev.action("named_sequence -> [the] ordinal_qualifier sequence of [the] sequence", |mut t| {
            let frame = t.remove(5).to_seq();
            let _the2 = t.remove(4);
            let _of = t.remove(3);
            let seq = t.remove(2).to_seq();
            let nth = t.remove(1).to_ordinal() as isize;
            let _the1 = t.remove(0);

            TimeValue::Seq(None, seq.within(frame, nth).unwrap())
        });
        
        ev.action("duration -> small_int time_quantity", |mut t| {
            let q = t.remove(1);
            let amt = t.remove(0).to_int();
            match q {
                TimeValue::Quantity(_, g, m) => TimeValue::Duration(vec![(g, amt * m)]),
                _ => panic!("Expected Quantity"),
            }
        });
        ev.action("(a|an) -> a", |_| TimeValue::Keyword);
        ev.action("(a|an) -> an", |_| TimeValue::Keyword);

        ev.action("duration -> (a|an) time_quantity", |mut t| {
            let q = t.remove(1);
            match q {
                TimeValue::Quantity(_, g, m) => TimeValue::Duration(vec![(g, m)]),
                _ => panic!("Expected Quantity"),
            }
        });
        ev.action("duration -> duration and small_int time_quantity", |mut t| {
            let q = t.remove(3);
            let amt = t.remove(2).to_int();
            let mut dur = t.remove(0).to_duration();
            match q {
                TimeValue::Quantity(_, g, m) => {
                    dur.push((g, amt * m));
                    TimeValue::Duration(dur)
                }
                _ => panic!("Expected Quantity"),
            }
        });
        ev.action("duration -> duration and (a|an) time_quantity", |mut t| {
            let q = t.remove(3);
            let mut dur = t.remove(0).to_duration();
            match q {
                TimeValue::Quantity(_, g, m) => {
                    dur.push((g, m));
                    TimeValue::Duration(dur)
                }
                _ => panic!("Expected Quantity"),
            }
        });

        ev.action("explicit_span -> duration shift_anchor", |mut t| {
            let anchor_val = t.remove(1);
            let dur = t.remove(0).to_duration();
            
            match anchor_val {
                TimeValue::RelAnchor(anchor, dir, _) => {
                    TimeValue::ShiftedSpan { anchor, dir, shifts: dur }
                }
                _ => unreachable!("shift_anchor should be RelAnchor"),
            }
        });
        ev.action("explicit_span -> in duration", |mut t| {
            let dur = t.remove(1).to_duration();
            TimeValue::ShiftedSpan { anchor: Anchor::Now, dir: TimeDir::Future, shifts: dur }
        });
        
        ev.action("explicit_span -> [the|a|small_int] time_quantity before last", |mut t| {
            let _last = t.remove(3);
            let _before = t.remove(2);
            let q = t.remove(1);
            let count = t.remove(0).to_int() as usize;

            let seq = match q {
                TimeValue::Quantity(_, Grain::Day, 7) => TimeSeqSpec::weeks(),
                TimeValue::Quantity(_, g, 1) => TimeSeqSpec::grain(g),
                TimeValue::Quantity(_, Grain::Month, m) => TimeSeqSpec::month_group(m as u32),
                TimeValue::Quantity(_, Grain::Year, 5) => TimeSeqSpec::grain(Grain::Year).merge(5),
                TimeValue::Quantity(_, Grain::Year, m) => TimeSeqSpec::year_group(m as u32),
                TimeValue::Quantity(_, g, m) => TimeSeqSpec::grain(g).merge(m as u16),
                _ => panic!("Unexpected time_quantity"),
            };

            TimeValue::Span { seq, anchor: Anchor::Now, dir: TimeDir::Past, skip: count }
        });

        ev.action("explicit_span -> [the|a|small_int] time_quantity (from|after) next", |mut t| {
            let _next = t.remove(3);
            let _from_after = t.remove(2);
            let q = t.remove(1);
            let count = t.remove(0).to_int() as usize;

            let seq = match q {
                TimeValue::Quantity(_, Grain::Day, 7) => TimeSeqSpec::weeks(),
                TimeValue::Quantity(_, g, 1) => TimeSeqSpec::grain(g),
                TimeValue::Quantity(_, Grain::Month, m) => TimeSeqSpec::month_group(m as u32),
                TimeValue::Quantity(_, Grain::Year, 5) => TimeSeqSpec::grain(Grain::Year).merge(5),
                TimeValue::Quantity(_, Grain::Year, m) => TimeSeqSpec::year_group(m as u32),
                TimeValue::Quantity(_, g, m) => TimeSeqSpec::grain(g).merge(m as u16),
                _ => panic!("Unexpected time_quantity"),
            };

            TimeValue::Span { seq, anchor: Anchor::Now, dir: TimeDir::Future, skip: count }
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

        ev.action("relative_anchor -> shift_anchor", |mut t| t.remove(0));

        ev.action("shift_anchor -> ago", |_| TimeValue::RelAnchor(Anchor::Now, TimeDir::Past, 0));
        ev.action("shift_anchor -> hence", |_| TimeValue::RelAnchor(Anchor::Now, TimeDir::Future, 0));
        ev.action("shift_anchor -> before time_span", |mut t| {
            let span = t.remove(1);
            TimeValue::RelAnchor(Anchor::Relative(Box::new(span), false), TimeDir::Past, 0)
        });
        ev.action("shift_anchor -> (from|after) time_span", |mut t| {
            let span = t.remove(1);
            TimeValue::RelAnchor(Anchor::Relative(Box::new(span), true), TimeDir::Future, 0)
        });

        ev.action("relative_anchor -> before last", |_| TimeValue::RelAnchor(Anchor::Now, TimeDir::Past, 1));
        
        ev.action("(from|after) -> from", |_| TimeValue::Keyword);
        ev.action("(from|after) -> after", |_| TimeValue::Keyword);

        ev.action("relative_anchor -> (from|after) next", |_| TimeValue::RelAnchor(Anchor::Now, TimeDir::Future, 1));
        
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

    pub fn eval(&self, time: &str, reftime: Option<DateTime>) -> Result<Vec<TimeResult>, String> {
        let mut tokenizer = time.split(&[' ', ','][..]).filter(|w| !w.is_empty());
        let state = self
            .parser
            .parse(&mut tokenizer)
            .map_err(|e| format!("TimeMachine {:?} for '{}'", e, time))?;

        // Default reftime to local time
        let reftime = reftime.unwrap_or_else(|| {
            let local_now = time::OffsetDateTime::now_local()
                .unwrap_or_else(|_| time::OffsetDateTime::now_utc());
            let local_naive = time::PrimitiveDateTime::new(local_now.date(), local_now.time());
            local_naive.assume_utc().into()
        });

        Ok(self
            .evaler
            .eval_all(&state)
            .map_err(|e| format!("TimeMachine {:?} for '{}'", e, time))?
            .into_iter()
            .filter_map(|tree| tree.eval_top(reftime))
            .collect())
    }
}
