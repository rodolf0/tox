#![deny(warnings)]

use time::UtcDateTime as DateTime;

use earlgrey::ParserBuilder;
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
    Within(Box<TimeValue>, bool),   // bool = use end of the span
    Relative(Box<TimeValue>, bool), // bool = use end of the span
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
    // Helper: span anchored to Now
    fn now_span(seq: TimeSeqSpec, dir: TimeDir, skip: usize) -> Self {
        TimeValue::Span {
            seq,
            anchor: Anchor::Now,
            dir,
            skip,
        }
    }

    // Helper: span with explicit anchor
    fn with_anchor(seq: TimeSeqSpec, anchor: Anchor, dir: TimeDir, skip: usize) -> Self {
        TimeValue::Span {
            seq,
            anchor,
            dir,
            skip,
        }
    }

    // Helper: interval between two time values
    fn interval(start: Self, end: Self) -> Self {
        TimeValue::Interval {
            start: Box::new(start),
            end: Box::new(end),
        }
    }

    // Helper: convert Quantity to TimeSeqSpec (used for time_quantity rules)
    fn quantity_to_seq(q: Self) -> TimeSeqSpec {
        match q {
            TimeValue::Quantity(_, Grain::Day, 7) => TimeSeqSpec::weeks(),
            TimeValue::Quantity(_, g, 1) => TimeSeqSpec::grain(g),
            TimeValue::Quantity(_, Grain::Month, m) => TimeSeqSpec::month_group(m as u32),
            TimeValue::Quantity(_, Grain::Year, 5) => TimeSeqSpec::grain(Grain::Year).merge(5),
            TimeValue::Quantity(_, Grain::Year, m) => TimeSeqSpec::year_group(m as u32),
            TimeValue::Quantity(_, g, m) => TimeSeqSpec::grain(g).merge(m as u16),
            _ => panic!("Unexpected time_quantity"),
        }
    }

    fn to_label(&self) -> Option<String> {
        match self {
            TimeValue::Seq(Some(l), _) => Some(l.clone()),
            TimeValue::QuantitySeq(Some(l), _) => Some(l.clone()),
            TimeValue::Quantity(Some(l), _, _) => Some(l.clone()),
            _ => None,
        }
    }

    fn into_seq(self) -> TimeSeqSpec {
        match self {
            TimeValue::Seq(_, s) => s,
            TimeValue::QuantitySeq(_, s) => s,
            _ => panic!("Expected Seq, found {:?}", self),
        }
    }

    fn into_int(self) -> i32 {
        match self {
            TimeValue::Int(i) => i,
            _ => panic!("Expected Int, found {:?}", self),
        }
    }

    fn into_ordinal(self) -> isize {
        match self {
            TimeValue::Ordinal(o) => o,
            _ => panic!("Expected Ordinal, found {:?}", self),
        }
    }

    fn into_duration(self) -> Vec<(Grain, i32)> {
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
                for span in seq
                    .future(bounds.start)
                    .take_while(|s| s.start < bounds.end)
                {
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
            TimeValue::Span {
                seq,
                anchor,
                dir,
                skip,
            } => match anchor {
                Anchor::Within(tv, _) => {
                    let bounds = tv.eval(reftime)?;
                    let mut iter: Box<dyn Iterator<Item = TimeSpan>> = match dir {
                        TimeDir::Future => Box::new(
                            seq.future(bounds.start)
                                .take_while(move |s| s.start < bounds.end),
                        ),
                        TimeDir::Past => Box::new(
                            seq.past(bounds.end)
                                .take_while(move |s| s.end > bounds.start),
                        ),
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
            },
            TimeValue::ShiftedSpan {
                anchor,
                dir,
                shifts,
            } => {
                let t0 = match anchor {
                    Anchor::Now => reftime,
                    Anchor::Time(t) => t,
                    Anchor::Within(tv, use_end) | Anchor::Relative(tv, use_end) => {
                        let span = tv.eval(reftime)?;
                        if use_end { span.end } else { span.start }
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
                    let start_time =
                        time::Date::from_calendar_date(t0.year(), t0.month(), t0.day())
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
                    TimeSeqSpec::grain(Grain::Second).future(t0).next().unwrap()
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

pub struct TimeMachine {
    parser: earlgrey::Parser<'static, TimeValue>,
}

impl TimeMachine {
    pub fn new() -> TimeMachine {
        use crate::constants::*;
        use std::str::FromStr;

        // === Terminals ===
        let mut builder =
            ParserBuilder::<TimeValue>::new(super::time_parser::time_grammar(), "time_expr")
                .terminal("weekday", |lexeme| {
                    weekday(lexeme)
                        .map(|w| TimeValue::Seq(Some(lexeme.to_string()), TimeSeqSpec::weekday(w)))
                })
                .terminal("monthname", |lexeme| {
                    month(lexeme).map(|m| {
                        TimeValue::Seq(
                            Some(lexeme.to_string()),
                            TimeSeqSpec::months(Some(m as u16)),
                        )
                    })
                })
                .terminal("ordinal", |lexeme| {
                    ordinal(lexeme)
                        .or_else(|| short_ordinal(lexeme))
                        .map(|o| TimeValue::Ordinal(o as isize))
                })
                .terminal("yearnumber", |lexeme| {
                    i32::from_str(lexeme)
                        .ok()
                        .filter(|&y| 999 < y && y < 3000)
                        .map(TimeValue::Int)
                })
                .terminal("small_int", |lexeme| {
                    u16::from_str(lexeme)
                        .ok()
                        .filter(|&s| s <= 999)
                        .map(|s| TimeValue::Int(s as i32))
                })
                .terminal("time_quantity", |lexeme| match lexeme {
                    "week" | "weeks" => {
                        Some(TimeValue::Quantity(Some(lexeme.to_string()), Grain::Day, 7))
                    }
                    "fortnight" | "fortnights" => Some(TimeValue::Quantity(
                        Some(lexeme.to_string()),
                        Grain::Day,
                        14,
                    )),
                    "quarter" | "quarters" => Some(TimeValue::Quantity(
                        Some(lexeme.to_string()),
                        Grain::Month,
                        3,
                    )),
                    "half" | "halfs" | "halves" => Some(TimeValue::Quantity(
                        Some(lexeme.to_string()),
                        Grain::Month,
                        6,
                    )),
                    "lustrum" | "lustrums" | "lustra" => Some(TimeValue::Quantity(
                        Some(lexeme.to_string()),
                        Grain::Year,
                        5,
                    )),
                    "decade" | "decades" => Some(TimeValue::Quantity(
                        Some(lexeme.to_string()),
                        Grain::Year,
                        10,
                    )),
                    "century" | "centuries" => Some(TimeValue::Quantity(
                        Some(lexeme.to_string()),
                        Grain::Year,
                        100,
                    )),
                    "millennium" | "millennia" | "millenium" | "milleniums" => Some(
                        TimeValue::Quantity(Some(lexeme.to_string()), Grain::Year, 1000),
                    ),
                    q => {
                        kronos_grain(q).map(|g| TimeValue::Quantity(Some(lexeme.to_string()), g, 1))
                    }
                })
                .terminal("weekend", |lexeme| {
                    if lexeme == "weekend" || lexeme == "weekends" {
                        Some(TimeValue::Seq(
                            Some(lexeme.to_string()),
                            TimeSeqSpec::weekends(),
                        ))
                    } else {
                        None
                    }
                })
                .terminal("clock_time", |lexeme| {
                    parse_clock_time(lexeme).map(|(h, m, s, grain)| {
                        let mut seq = TimeSeqSpec::hours(Some(h as u16));
                        if grain <= Grain::Minute {
                            seq = seq.intersection(TimeSeqSpec::minutes(Some(m as u16)));
                        }
                        if grain <= Grain::Second {
                            seq = seq.intersection(TimeSeqSpec::seconds(Some(s as u16)));
                        }
                        TimeValue::Seq(Some(lexeme.to_string()), seq)
                    })
                })
                .terminal("numeric_date", |lexeme| {
                    parse_date(lexeme).map(|(y, m, d)| {
                        let start_time =
                            time::Date::from_calendar_date(y, m.try_into().unwrap(), d)
                                .unwrap()
                                .with_hms(0, 0, 0)
                                .unwrap()
                                .assume_utc()
                                .into();
                        TimeValue::with_anchor(
                            TimeSeqSpec::grain(Grain::Day),
                            Anchor::Time(start_time),
                            TimeDir::Future,
                            0,
                        )
                    })
                });

        // === Literals (keywords) ===
        let keywords = [
            "now",
            "today",
            "yesterday",
            "tomorrow",
            "this",
            "next",
            "last",
            "ago",
            "hence",
            "before",
            "after",
            "from",
            "in",
            "the",
            "a",
            "an",
            "of",
            "on",
            "and",
            "since",
            "until",
            "between",
        ];
        for keyword in keywords.iter() {
            builder = builder.literal(keyword, TimeValue::Keyword);
        }

        // === time_expr actions ===
        builder = builder
            .action("time_expr -> time_span", |mut t| t.remove(0))
            .action("time_expr -> on time_span", |mut t| t.remove(1));

        // sequence counting
        builder = builder
            .action("time_expr -> sequence since time_span", |mut t| {
                let span = t.remove(2);
                let seq_val = t.remove(0);
                let label = seq_val.to_label().unwrap_or_else(|| "units".to_string());
                let interval = TimeValue::interval(
                    span,
                    TimeValue::now_span(TimeSeqSpec::grain(Grain::Second), TimeDir::Future, 0),
                );
                TimeValue::Count(label, seq_val.into_seq(), Box::new(interval))
            })
            .action("time_expr -> sequence until time_span", |mut t| {
                let span = t.remove(2);
                let seq_val = t.remove(0);
                let label = seq_val.to_label().unwrap_or_else(|| "units".to_string());
                let interval = TimeValue::interval(
                    TimeValue::now_span(TimeSeqSpec::grain(Grain::Second), TimeDir::Future, 0),
                    span,
                );
                TimeValue::Count(label, seq_val.into_seq(), Box::new(interval))
            })
            .action(
                "time_expr -> sequence between time_span and time_span",
                |mut t| {
                    let end = t.remove(4);
                    let start = t.remove(2);
                    let seq_val = t.remove(0);
                    let label = seq_val.to_label().unwrap_or_else(|| "units".to_string());
                    let interval = TimeValue::interval(start, end);
                    TimeValue::Count(label, seq_val.into_seq(), Box::new(interval))
                },
            )
            .action("time_expr -> sequence in time_span", |mut t| {
                let span = t.remove(2);
                let seq_val = t.remove(0);
                let label = seq_val.to_label().unwrap_or_else(|| "units".to_string());
                TimeValue::Count(label, seq_val.into_seq(), Box::new(span))
            });

        // === time_span actions ===
        builder = builder
            .action("time_span -> explicit_span", |mut t| t.remove(0))
            .action("time_span -> sequence", |mut t| {
                TimeValue::now_span(t.remove(0).into_seq(), TimeDir::Future, 0)
            });

        // === explicit_span actions (anchors) ===
        builder = builder
            .action("explicit_span -> now", |_| {
                TimeValue::now_span(TimeSeqSpec::grain(Grain::Second), TimeDir::Future, 0)
            })
            .action("explicit_span -> today", |_| {
                TimeValue::now_span(TimeSeqSpec::grain(Grain::Day), TimeDir::Future, 0)
            })
            .action("explicit_span -> yesterday", |_| {
                TimeValue::now_span(TimeSeqSpec::grain(Grain::Day), TimeDir::Past, 0)
            })
            .action("explicit_span -> tomorrow", |_| {
                TimeValue::now_span(TimeSeqSpec::grain(Grain::Day), TimeDir::Future, 1)
            })
            .action("explicit_span -> numeric_date", |mut t| t.remove(0));

        // explicit_span (relative basic)
        builder = builder
            .action("explicit_span -> this sequence", |mut t| {
                let seq_val = t.remove(1);
                TimeValue::now_span(seq_val.into_seq(), TimeDir::Future, 0)
            })
            .action("explicit_span -> next sequence", |mut t| {
                let seq_val = t.remove(1);
                let skip = match &seq_val {
                    TimeValue::QuantitySeq(..) => 1,
                    TimeValue::Seq(..) => 0, // Named sequence split!
                    _ => 1,
                };
                TimeValue::now_span(seq_val.into_seq(), TimeDir::Future, skip)
            })
            .action("explicit_span -> last sequence", |mut t| {
                let seq_val = t.remove(1);
                let skip = match &seq_val {
                    TimeValue::QuantitySeq(..) => 0,
                    TimeValue::Seq(..) => 0,
                    _ => 1,
                };
                TimeValue::now_span(seq_val.into_seq(), TimeDir::Past, skip)
            });

        // explicit_span (anchored to year)
        builder = builder
            .action("explicit_span -> sequence yearnumber", |mut t| {
                let year = t.remove(1).into_int();
                let seq = t.remove(0).into_seq();
                TimeValue::with_anchor(
                    seq,
                    Anchor::Time(TimeSpan::year(year).start),
                    TimeDir::Future,
                    0,
                )
            })
            .action("explicit_span -> yearnumber", |mut t| {
                let year = t.remove(0).into_int();
                TimeValue::with_anchor(
                    TimeSeqSpec::years(),
                    Anchor::Time(TimeSpan::year(year).start),
                    TimeDir::Future,
                    0,
                )
            });

        // === sequence actions ===
        builder = builder
            .action("sequence -> time_quantity", |mut t| {
                let q = t.remove(0);
                match q {
                    TimeValue::Quantity(_, Grain::Day, 7) => {
                        TimeValue::QuantitySeq(q.to_label(), TimeSeqSpec::weeks())
                    }
                    TimeValue::Quantity(_, g, 1) => {
                        TimeValue::QuantitySeq(q.to_label(), TimeSeqSpec::grain(g))
                    }
                    TimeValue::Quantity(_, Grain::Month, m) => {
                        TimeValue::QuantitySeq(q.to_label(), TimeSeqSpec::month_group(m as u32))
                    }
                    TimeValue::Quantity(_, Grain::Year, 5) => TimeValue::QuantitySeq(
                        q.to_label(),
                        TimeSeqSpec::grain(Grain::Year).merge(5),
                    ),
                    TimeValue::Quantity(_, Grain::Year, m) => {
                        TimeValue::QuantitySeq(q.to_label(), TimeSeqSpec::year_group(m as u32))
                    }
                    TimeValue::Quantity(_, g, m) => {
                        TimeValue::QuantitySeq(q.to_label(), TimeSeqSpec::grain(g).merge(m as u16))
                    }
                    _ => panic!("Unexpected time_quantity"),
                }
            })
            .action("sequence -> named_sequence", |mut t| t.remove(0));

        // === named_sequence actions ===
        builder = builder
            .action("named_sequence -> weekend", |mut t| t.remove(0))
            .action("named_sequence -> monthname", |mut t| t.remove(0))
            .action("named_sequence -> weekday", |mut t| t.remove(0))
            .action("named_sequence -> clock_time", |mut t| t.remove(0));

        builder = builder
            .action("named_sequence -> monthname ordinal", |mut t| {
                let ordinal = t.remove(1).into_ordinal();
                let seq = TimeSeqSpec::days()
                    .within(t.remove(0).into_seq(), ordinal)
                    .unwrap();
                TimeValue::Seq(None, seq)
            })
            .action("named_sequence -> ordinal of monthname", |mut t| {
                let month = t.remove(2).into_seq();
                let ordinal = t.remove(0).into_ordinal();
                let seq = TimeSeqSpec::days().within(month, ordinal).unwrap();
                TimeValue::Seq(None, seq)
            })
            .action("named_sequence -> weekday monthname ordinal", |mut t| {
                let ordinal = t.remove(2).into_ordinal();
                let month = t.remove(1).into_seq();
                let weekday = t.remove(0).into_seq();
                let seq = TimeSeqSpec::days()
                    .within(month, ordinal)
                    .unwrap()
                    .intersection(weekday);
                TimeValue::Seq(None, seq)
            })
            .action("named_sequence -> weekday ordinal of monthname", |mut t| {
                let month = t.remove(3).into_seq();
                let ordinal = t.remove(1).into_ordinal();
                let weekday = t.remove(0).into_seq();
                let seq = TimeSeqSpec::days()
                    .within(month, ordinal)
                    .unwrap()
                    .intersection(weekday);
                TimeValue::Seq(None, seq)
            })
            .action("named_sequence -> weekday clock_time", |mut t| {
                let clock_time = t.remove(1).into_seq();
                let weekday = t.remove(0).into_seq();
                TimeValue::Seq(None, clock_time.intersection(weekday))
            });

        builder = builder
            .action("named_sequence -> [the] ordinal", |mut t| {
                let ordinal = t.remove(1).into_ordinal();
                let seq = TimeSeqSpec::days()
                    .within(TimeSeqSpec::months(None), ordinal)
                    .unwrap();
                TimeValue::Seq(None, seq)
            })
            .action("named_sequence -> weekday [the] ordinal", |mut t| {
                let ordinal = t.remove(2).into_ordinal();
                let weekday = t.remove(0).into_seq();
                let seq = TimeSeqSpec::days()
                    .within(TimeSeqSpec::months(None), ordinal)
                    .unwrap()
                    .intersection(weekday);
                TimeValue::Seq(None, seq)
            });

        // === EBNF auxiliary rules (counts) ===
        builder = builder
            .action("[the|a|small_int] -> small_int", |mut t| t.remove(0))
            .action("[the|a|small_int] -> the", |_| TimeValue::Int(1))
            .action("[the|a|small_int] -> a", |_| TimeValue::Int(1))
            .action("[the|a|small_int] -> ", |_| TimeValue::Int(1))
            .action("(a|an|small_int) -> small_int", |mut t| t.remove(0))
            .action("(a|an|small_int) -> a", |_| TimeValue::Int(1))
            .action("(a|an|small_int) -> an", |_| TimeValue::Int(1));

        // === Remaining actions (all chained) ===
        builder = builder
            // explicit_span with counts/anchors
            .action("[the] -> the", |_| TimeValue::Keyword)
            .action("[the] -> ", |_| TimeValue::Keyword)
            .action(
                "explicit_span -> [the|a|small_int] named_sequence relative_anchor",
                |mut t| {
                    let anchor_val = t.remove(2);
                    let seq = t.remove(1).into_seq();
                    let count = t.remove(0).into_int() as usize;

                    match anchor_val {
                        TimeValue::RelAnchor(anchor, dir, offset) => {
                            let skip = count.saturating_sub(1) + offset;
                            TimeValue::Span {
                                seq,
                                anchor,
                                dir,
                                skip,
                            }
                        }
                        _ => unreachable!("relative_anchor should be RelAnchor"),
                    }
                },
            )
            .action(
                "explicit_span -> in (a|an|small_int) named_sequence",
                |mut t| {
                    let count = t.remove(1).into_int() as usize;
                    let seq = t.remove(1).into_seq();
                    TimeValue::now_span(seq, TimeDir::Future, count)
                },
            )
            .action(
                "explicit_span -> [the] ordinal_qualifier sequence of [the] explicit_span",
                |mut t| {
                    let span = t.remove(5);
                    let _the2 = t.remove(4);
                    let _of = t.remove(3);
                    let seq = t.remove(2).into_seq();
                    let nth = t.remove(1).into_ordinal();
                    let _the1 = t.remove(0);

                    let (dir, use_end, skip) = if nth > 0 {
                        (TimeDir::Future, false, (nth - 1) as usize)
                    } else {
                        (TimeDir::Past, true, (-nth - 1) as usize)
                    };

                    TimeValue::with_anchor(seq, Anchor::Within(Box::new(span), use_end), dir, skip)
                },
            )
            .action("explicit_span -> sequence on explicit_span", |mut t| {
                let anchor = Anchor::Within(Box::new(t.remove(2)), false);
                let _on = t.remove(1);
                let seq = t.remove(0).into_seq();
                TimeValue::with_anchor(seq, anchor, TimeDir::Future, 0)
            })
            // named_sequence with qualifiers
            .action(
                "named_sequence -> [the] ordinal_qualifier sequence of [the] sequence",
                |mut t| {
                    let frame = t.remove(5).into_seq();
                    let _the2 = t.remove(4);
                    let _of = t.remove(3);
                    let seq = t.remove(2).into_seq();
                    let nth = t.remove(1).into_ordinal();
                    let _the1 = t.remove(0);

                    TimeValue::Seq(None, seq.within(frame, nth).unwrap())
                },
            )
            // duration actions
            .action("(a|an) -> a", |_| TimeValue::Keyword)
            .action("(a|an) -> an", |_| TimeValue::Keyword)
            .action("duration -> small_int time_quantity", |mut t| {
                let q = t.remove(1);
                let amt = t.remove(0).into_int();
                match q {
                    TimeValue::Quantity(_, g, m) => TimeValue::Duration(vec![(g, amt * m)]),
                    _ => panic!("Expected Quantity"),
                }
            })
            .action("duration -> (a|an) time_quantity", |mut t| {
                let q = t.remove(1);
                match q {
                    TimeValue::Quantity(_, g, m) => TimeValue::Duration(vec![(g, m)]),
                    _ => panic!("Expected Quantity"),
                }
            })
            .action(
                "duration -> duration and small_int time_quantity",
                |mut t| {
                    let q = t.remove(3);
                    let amt = t.remove(2).into_int();
                    let mut dur = t.remove(0).into_duration();
                    match q {
                        TimeValue::Quantity(_, g, m) => {
                            dur.push((g, amt * m));
                            TimeValue::Duration(dur)
                        }
                        _ => panic!("Expected Quantity"),
                    }
                },
            )
            .action("duration -> duration and (a|an) time_quantity", |mut t| {
                let q = t.remove(3);
                let mut dur = t.remove(0).into_duration();
                match q {
                    TimeValue::Quantity(_, g, m) => {
                        dur.push((g, m));
                        TimeValue::Duration(dur)
                    }
                    _ => panic!("Expected Quantity"),
                }
            })
            // explicit_span with duration/shift
            .action("explicit_span -> duration shift_anchor", |mut t| {
                let anchor_val = t.remove(1);
                let dur = t.remove(0).into_duration();

                match anchor_val {
                    TimeValue::RelAnchor(anchor, dir, _) => TimeValue::ShiftedSpan {
                        anchor,
                        dir,
                        shifts: dur,
                    },
                    _ => unreachable!("shift_anchor should be RelAnchor"),
                }
            })
            .action("explicit_span -> in duration", |mut t| {
                let dur = t.remove(1).into_duration();
                TimeValue::ShiftedSpan {
                    anchor: Anchor::Now,
                    dir: TimeDir::Future,
                    shifts: dur,
                }
            })
            // explicit_span with time_quantity + before/after
            .action(
                "explicit_span -> [the|a|small_int] time_quantity before last",
                |mut t| {
                    let _last = t.remove(3);
                    let _before = t.remove(2);
                    let q = t.remove(1);
                    let count = t.remove(0).into_int() as usize;
                    let seq = TimeValue::quantity_to_seq(q);
                    TimeValue::now_span(seq, TimeDir::Past, count)
                },
            )
            .action(
                "explicit_span -> [the|a|small_int] time_quantity (from|after) next",
                |mut t| {
                    let _next = t.remove(3);
                    let _from_after = t.remove(2);
                    let q = t.remove(1);
                    let count = t.remove(0).into_int() as usize;
                    let seq = TimeValue::quantity_to_seq(q);
                    TimeValue::now_span(seq, TimeDir::Future, count)
                },
            )
            // interval actions (since/until/between)
            .action("explicit_span -> since time_span", |mut t| {
                let start = t.remove(1);
                let end =
                    TimeValue::now_span(TimeSeqSpec::grain(Grain::Second), TimeDir::Future, 0);
                TimeValue::interval(start, end)
            })
            .action("explicit_span -> until time_span", |mut t| {
                let end = t.remove(1);
                let start =
                    TimeValue::now_span(TimeSeqSpec::grain(Grain::Second), TimeDir::Future, 0);
                TimeValue::interval(start, end)
            })
            .action(
                "explicit_span -> between time_span and time_span",
                |mut t| {
                    let end = t.remove(3);
                    let start = t.remove(1);
                    TimeValue::interval(start, end)
                },
            )
            // relative_anchor actions
            .action("relative_anchor -> shift_anchor", |mut t| t.remove(0))
            .action("shift_anchor -> ago", |_| {
                TimeValue::RelAnchor(Anchor::Now, TimeDir::Past, 0)
            })
            .action("shift_anchor -> hence", |_| {
                TimeValue::RelAnchor(Anchor::Now, TimeDir::Future, 0)
            })
            .action("shift_anchor -> before time_span", |mut t| {
                let span = t.remove(1);
                TimeValue::RelAnchor(Anchor::Relative(Box::new(span), false), TimeDir::Past, 0)
            })
            .action("shift_anchor -> (from|after) time_span", |mut t| {
                let span = t.remove(1);
                TimeValue::RelAnchor(Anchor::Relative(Box::new(span), true), TimeDir::Future, 0)
            })
            .action("relative_anchor -> before last", |_| {
                TimeValue::RelAnchor(Anchor::Now, TimeDir::Past, 1)
            })
            .action("(from|after) -> from", |_| TimeValue::Keyword)
            .action("(from|after) -> after", |_| TimeValue::Keyword)
            .action("relative_anchor -> (from|after) next", |_| {
                TimeValue::RelAnchor(Anchor::Now, TimeDir::Future, 1)
            })
            // ordinal_qualifier actions
            .action("ordinal_qualifier -> next", |_| TimeValue::Ordinal(1))
            .action("ordinal_qualifier -> last", |_| TimeValue::Ordinal(-1))
            .action("ordinal_qualifier -> ordinal", |mut t| t.remove(0))
            .action("ordinal_qualifier -> last ordinal", |mut t| {
                TimeValue::Ordinal(-t.remove(1).into_ordinal())
            });

        // === named_sequence with qualifiers ===
        builder = builder.action(
            "named_sequence -> [the] ordinal_qualifier sequence of [the] sequence",
            |mut t| {
                let frame = t.remove(5).into_seq();
                let _the2 = t.remove(4);
                let _of = t.remove(3);
                let seq = t.remove(2).into_seq();
                let nth = t.remove(1).into_ordinal();
                let _the1 = t.remove(0);

                TimeValue::Seq(None, seq.within(frame, nth).unwrap())
            },
        );

        // === duration actions ===
        builder = builder
            .action("(a|an) -> a", |_| TimeValue::Keyword)
            .action("(a|an) -> an", |_| TimeValue::Keyword)
            .action("duration -> small_int time_quantity", |mut t| {
                let q = t.remove(1);
                let amt = t.remove(0).into_int();
                match q {
                    TimeValue::Quantity(_, g, m) => TimeValue::Duration(vec![(g, amt * m)]),
                    _ => panic!("Expected Quantity"),
                }
            })
            .action("duration -> (a|an) time_quantity", |mut t| {
                let q = t.remove(1);
                match q {
                    TimeValue::Quantity(_, g, m) => TimeValue::Duration(vec![(g, m)]),
                    _ => panic!("Expected Quantity"),
                }
            })
            .action(
                "duration -> duration and small_int time_quantity",
                |mut t| {
                    let q = t.remove(3);
                    let amt = t.remove(2).into_int();
                    let mut dur = t.remove(0).into_duration();
                    match q {
                        TimeValue::Quantity(_, g, m) => {
                            dur.push((g, amt * m));
                            TimeValue::Duration(dur)
                        }
                        _ => panic!("Expected Quantity"),
                    }
                },
            )
            .action("duration -> duration and (a|an) time_quantity", |mut t| {
                let q = t.remove(3);
                let mut dur = t.remove(0).into_duration();
                match q {
                    TimeValue::Quantity(_, g, m) => {
                        dur.push((g, m));
                        TimeValue::Duration(dur)
                    }
                    _ => panic!("Expected Quantity"),
                }
            });

        // === explicit_span with duration/shift ===
        builder = builder
            .action("explicit_span -> duration shift_anchor", |mut t| {
                let anchor_val = t.remove(1);
                let dur = t.remove(0).into_duration();

                match anchor_val {
                    TimeValue::RelAnchor(anchor, dir, _) => TimeValue::ShiftedSpan {
                        anchor,
                        dir,
                        shifts: dur,
                    },
                    _ => unreachable!("shift_anchor should be RelAnchor"),
                }
            })
            .action("explicit_span -> in duration", |mut t| {
                let dur = t.remove(1).into_duration();
                TimeValue::ShiftedSpan {
                    anchor: Anchor::Now,
                    dir: TimeDir::Future,
                    shifts: dur,
                }
            });

        // === explicit_span with time_quantity + before/after ===
        builder = builder
            .action(
                "explicit_span -> [the|a|small_int] time_quantity before last",
                |mut t| {
                    let _last = t.remove(3);
                    let _before = t.remove(2);
                    let q = t.remove(1);
                    let count = t.remove(0).into_int() as usize;
                    let seq = TimeValue::quantity_to_seq(q);
                    TimeValue::now_span(seq, TimeDir::Past, count)
                },
            )
            .action(
                "explicit_span -> [the|a|small_int] time_quantity (from|after) next",
                |mut t| {
                    let _next = t.remove(3);
                    let _from_after = t.remove(2);
                    let q = t.remove(1);
                    let count = t.remove(0).into_int() as usize;
                    let seq = TimeValue::quantity_to_seq(q);
                    TimeValue::now_span(seq, TimeDir::Future, count)
                },
            );

        // === interval actions (since/until/between) ===
        builder = builder
            .action("explicit_span -> since time_span", |mut t| {
                let start = t.remove(1);
                let end =
                    TimeValue::now_span(TimeSeqSpec::grain(Grain::Second), TimeDir::Future, 0);
                TimeValue::interval(start, end)
            })
            .action("explicit_span -> until time_span", |mut t| {
                let end = t.remove(1);
                let start =
                    TimeValue::now_span(TimeSeqSpec::grain(Grain::Second), TimeDir::Future, 0);
                TimeValue::interval(start, end)
            })
            .action(
                "explicit_span -> between time_span and time_span",
                |mut t| {
                    let end = t.remove(3);
                    let start = t.remove(1);
                    TimeValue::interval(start, end)
                },
            );

        // === relative_anchor actions ===
        builder = builder
            .action("relative_anchor -> shift_anchor", |mut t| t.remove(0))
            .action("shift_anchor -> ago", |_| {
                TimeValue::RelAnchor(Anchor::Now, TimeDir::Past, 0)
            })
            .action("shift_anchor -> hence", |_| {
                TimeValue::RelAnchor(Anchor::Now, TimeDir::Future, 0)
            })
            .action("shift_anchor -> before time_span", |mut t| {
                let span = t.remove(1);
                TimeValue::RelAnchor(Anchor::Relative(Box::new(span), false), TimeDir::Past, 0)
            })
            .action("shift_anchor -> (from|after) time_span", |mut t| {
                let span = t.remove(1);
                TimeValue::RelAnchor(Anchor::Relative(Box::new(span), true), TimeDir::Future, 0)
            })
            .action("relative_anchor -> before last", |_| {
                TimeValue::RelAnchor(Anchor::Now, TimeDir::Past, 1)
            })
            .action("(from|after) -> from", |_| TimeValue::Keyword)
            .action("(from|after) -> after", |_| TimeValue::Keyword)
            .action("relative_anchor -> (from|after) next", |_| {
                TimeValue::RelAnchor(Anchor::Now, TimeDir::Future, 1)
            });

        // === ordinal_qualifier actions ===
        builder = builder
            .action("ordinal_qualifier -> next", |_| TimeValue::Ordinal(1))
            .action("ordinal_qualifier -> last", |_| TimeValue::Ordinal(-1))
            .action("ordinal_qualifier -> ordinal", |mut t| t.remove(0))
            .action("ordinal_qualifier -> last ordinal", |mut t| {
                TimeValue::Ordinal(-t.remove(1).into_ordinal())
            });

        builder = builder.action(
            "named_sequence -> [the] ordinal_qualifier sequence of [the] sequence",
            |mut t| {
                let frame = t.remove(5).into_seq();
                let _the2 = t.remove(4);
                let _of = t.remove(3);
                let seq = t.remove(2).into_seq();
                let nth = t.remove(1).into_ordinal();
                let _the1 = t.remove(0);

                TimeValue::Seq(None, seq.within(frame, nth).unwrap())
            },
        );

        // === duration actions ===
        builder = builder
            .action("(a|an) -> a", |_| TimeValue::Keyword)
            .action("(a|an) -> an", |_| TimeValue::Keyword);
        builder = builder
            .action("duration -> small_int time_quantity", |mut t| {
                let q = t.remove(1);
                let amt = t.remove(0).into_int();
                match q {
                    TimeValue::Quantity(_, g, m) => TimeValue::Duration(vec![(g, amt * m)]),
                    _ => panic!("Expected Quantity"),
                }
            })
            .action("duration -> (a|an) time_quantity", |mut t| {
                let q = t.remove(1);
                match q {
                    TimeValue::Quantity(_, g, m) => TimeValue::Duration(vec![(g, m)]),
                    _ => panic!("Expected Quantity"),
                }
            })
            .action(
                "duration -> duration and small_int time_quantity",
                |mut t| {
                    let q = t.remove(3);
                    let amt = t.remove(2).into_int();
                    let mut dur = t.remove(0).into_duration();
                    match q {
                        TimeValue::Quantity(_, g, m) => {
                            dur.push((g, amt * m));
                            TimeValue::Duration(dur)
                        }
                        _ => panic!("Expected Quantity"),
                    }
                },
            )
            .action("duration -> duration and (a|an) time_quantity", |mut t| {
                let q = t.remove(3);
                let mut dur = t.remove(0).into_duration();
                match q {
                    TimeValue::Quantity(_, g, m) => {
                        dur.push((g, m));
                        TimeValue::Duration(dur)
                    }
                    _ => panic!("Expected Quantity"),
                }
            });

        builder = builder
            .action("explicit_span -> duration shift_anchor", |mut t| {
                let anchor_val = t.remove(1);
                let dur = t.remove(0).into_duration();

                match anchor_val {
                    TimeValue::RelAnchor(anchor, dir, _) => TimeValue::ShiftedSpan {
                        anchor,
                        dir,
                        shifts: dur,
                    },
                    _ => unreachable!("shift_anchor should be RelAnchor"),
                }
            })
            .action("explicit_span -> in duration", |mut t| {
                let dur = t.remove(1).into_duration();
                TimeValue::ShiftedSpan {
                    anchor: Anchor::Now,
                    dir: TimeDir::Future,
                    shifts: dur,
                }
            });

        // === explicit_span ===
        builder = builder
            .action(
                "explicit_span -> [the|a|small_int] time_quantity before last",
                |mut t| {
                    let _last = t.remove(3);
                    let _before = t.remove(2);
                    let q = t.remove(1);
                    let count = t.remove(0).into_int() as usize;
                    let seq = TimeValue::quantity_to_seq(q);
                    TimeValue::now_span(seq, TimeDir::Past, count)
                },
            )
            .action(
                "explicit_span -> [the|a|small_int] time_quantity (from|after) next",
                |mut t| {
                    let _next = t.remove(3);
                    let _from_after = t.remove(2);
                    let q = t.remove(1);
                    let count = t.remove(0).into_int() as usize;
                    let seq = TimeValue::quantity_to_seq(q);
                    TimeValue::now_span(seq, TimeDir::Future, count)
                },
            )
            .action("explicit_span -> since time_span", |mut t| {
                let start = t.remove(1);
                let end =
                    TimeValue::now_span(TimeSeqSpec::grain(Grain::Second), TimeDir::Future, 0);
                TimeValue::interval(start, end)
            })
            .action("explicit_span -> until time_span", |mut t| {
                let end = t.remove(1);
                let start =
                    TimeValue::now_span(TimeSeqSpec::grain(Grain::Second), TimeDir::Future, 0);
                TimeValue::interval(start, end)
            })
            .action(
                "explicit_span -> between time_span and time_span",
                |mut t| {
                    let end = t.remove(3);
                    let start = t.remove(1);
                    TimeValue::interval(start, end)
                },
            );

        // === relative_anchor actions ===
        builder = builder
            .action("relative_anchor -> shift_anchor", |mut t| t.remove(0))
            .action("relative_anchor -> before last", |_| {
                TimeValue::RelAnchor(Anchor::Now, TimeDir::Past, 1)
            })
            .action("relative_anchor -> (from|after) next", |_| {
                TimeValue::RelAnchor(Anchor::Now, TimeDir::Future, 1)
            });

        builder = builder
            .action("(from|after) -> from", |_| TimeValue::Keyword)
            .action("(from|after) -> after", |_| TimeValue::Keyword);

        builder = builder
            .action("shift_anchor -> ago", |_| {
                TimeValue::RelAnchor(Anchor::Now, TimeDir::Past, 0)
            })
            .action("shift_anchor -> hence", |_| {
                TimeValue::RelAnchor(Anchor::Now, TimeDir::Future, 0)
            })
            .action("shift_anchor -> before time_span", |mut t| {
                let span = t.remove(1);
                TimeValue::RelAnchor(Anchor::Relative(Box::new(span), false), TimeDir::Past, 0)
            })
            .action("shift_anchor -> (from|after) time_span", |mut t| {
                let span = t.remove(1);
                TimeValue::RelAnchor(Anchor::Relative(Box::new(span), true), TimeDir::Future, 0)
            });

        // === ordinal_qualifier actions ===
        builder = builder
            .action("ordinal_qualifier -> next", |_| TimeValue::Ordinal(1))
            .action("ordinal_qualifier -> last", |_| TimeValue::Ordinal(-1))
            .action("ordinal_qualifier -> ordinal", |mut t| t.remove(0))
            .action("ordinal_qualifier -> last ordinal", |mut t| {
                TimeValue::Ordinal(-t.remove(1).into_ordinal())
            });

        TimeMachine {
            parser: builder.build().unwrap(),
        }
    }

    pub fn eval(&self, time: &str, reftime: Option<DateTime>) -> Result<Vec<TimeResult>, String> {
        let mut tokenizer = time.split(&[' ', ','][..]).filter(|w| !w.is_empty());
        let results = self
            .parser
            .parse_all(&mut tokenizer)
            .map_err(|e| format!("TimeMachine {:?} for '{}'", e, time))?;

        // Default reftime to local time
        let reftime = reftime.unwrap_or_else(|| {
            let local_now = time::OffsetDateTime::now_local()
                .unwrap_or_else(|_| time::OffsetDateTime::now_utc());
            let local_naive = time::PrimitiveDateTime::new(local_now.date(), local_now.time());
            local_naive.assume_utc().into()
        });

        Ok(results
            .into_iter()
            .filter_map(|tree| tree.eval_top(reftime))
            .collect())
    }
}
