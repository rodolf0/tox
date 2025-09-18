use core::fmt;
use time::UtcDateTime as DateTime;

use earlgrey::{EarleyForest, EarleyParser};
use kronos::{Grain, TimeSeq, TimeSpan};

#[derive(Clone, Debug)]
enum TimeAstNode {
    Weekday(u8),
    MonthName(u8),
    Ordinal(u8),
    YearNumber(u16),
    HourSpec(u8), // 0-23
    SmallInt(u16),
    TimeGrain(Grain),
    Week,
    Weekend,
    TimeSeq(kronos::TimeSeq),
    TimeSpan(kronos::TimeSpan),
    RelTimeSpan { shift: (Grain, i32), grain: Grain },
}

pub struct TimeStream<'a>(Box<dyn Iterator<Item = TimeSpan> + 'a>);

impl TimeAstNode {
    fn i32(self) -> i32 {
        match self {
            TimeAstNode::Weekday(x) => x as i32,
            TimeAstNode::MonthName(x) => x as i32,
            TimeAstNode::Ordinal(x) => x as i32,
            TimeAstNode::YearNumber(x) => x as i32,
            TimeAstNode::HourSpec(x) => x as i32,
            _ => panic!("BUG: cannot convert {:?} to i32", self),
        }
    }

    fn eval(&self, reftime: DateTime) -> TimeStream {
        match self {
            TimeAstNode::TimeSeq(s) => TimeStream(s.future(reftime)),
            _ => todo!(),
        }
    }
}

fn terminal_eval() -> impl Fn(&str, &str) -> TimeAstNode {
    use crate::constants::*;
    use TimeAstNode::*;
    use std::str::FromStr;
    |terminal, lexeme| match terminal {
        "weekday" => Weekday(weekday(lexeme).unwrap()),
        "monthname" => MonthName(month(lexeme).unwrap()),
        "ordinal" => Ordinal(ordinal(lexeme).or_else(|| short_ordinal(lexeme)).unwrap()),
        "yearnumber" => YearNumber(u16::from_str(lexeme).unwrap()),
        "hourspec" => HourSpec(hour_spec(lexeme).unwrap()),
        "small_int" => SmallInt(u16::from_str(lexeme).unwrap()),
        "time_quantity" => match lexeme {
            "week" | "weeks" => Week,
            q => TimeGrain(kronos_grain(q).unwrap()),
        },
        "weekend" => Weekend,
        _ => unreachable!("Unknown terminal {}", terminal),
    }
}

// #[derive(Debug, PartialEq, Clone)]
// pub enum TimeEl {
//     Time(k::Range),
//     Count(u32),
// }
//
// impl fmt::Display for TimeEl {
//     fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
//         match self {
//             TimeEl::Count(number) => write!(f, "{}", number),
//             TimeEl::Time(r) => {
//                 use kronos::Grain::*;
//                 let fmt_pattern = match r.grain {
//                     Second => "%A, %e %B %Y %H:%M:%S",
//                     Minute => "%A, %e %B %Y %H:%M",
//                     Hour => "%A, %e %B %Y %Hhs",
//                     Day | Week => "%A, %e %B %Y",
//                     Month | Quarter | Half => "%B %Y",
//                     Year | Lustrum | Decade | Century | Millenium => "%Y",
//                 };
//                 if r.grain <= Day {
//                     write!(f, "{}", r.start.format(fmt_pattern))
//                 } else {
//                     write!(
//                         f,
//                         "{} - {}",
//                         r.start.format(fmt_pattern),
//                         r.end.format(fmt_pattern)
//                     )
//                 }
//             }
//         }
//     }
// }
//
// impl TimeEl {
//     fn range(self) -> k::Range {
//         if let TimeEl::Time(x) = self {
//             x
//         } else {
//             panic!("BUG")
//         }
//     }
// }
//
// #[derive(Clone)]
// enum TimeNode {
//     Int(i32),
//     Grain(k::Grain),
//     Shifts(Vec<(k::Grain, i32)>),
//     Nop,
//     Seq(Shim),
//     This(Shim),
//     Next(Shim, usize),
//     Last(Shim, usize),
//     RefNext(Shim, DateTime),
//     RefPrev(Shim, DateTime),
//     Until(Shim, DateTime),
//     Since(Shim, DateTime),
//     Between(Shim, DateTime, DateTime),
// }
//
// // Shift a sequence by multiple shifts
// fn build_shifter(shifts: Vec<(k::Grain, i32)>, sign: i32, grain: k::Grain) -> Shim {
//     // get the finest grain of the composition to anchor the lookback
//     let grain = std::cmp::min(shifts.iter().min_by_key(|g| g.0).unwrap().0, grain);
//     // cap to day granularity at most
//     let mut shifted = Shim::new(kronos::Grains(grain));
//     // shift the initial sequence by composed shifts
//     for s in shifts {
//         shifted = Shim::new(kronos::shift(shifted, s.0, sign * s.1));
//     }
//     shifted
// }
//
// macro_rules! s {
//     ($e:expr) => {
//         TimeNode::Seq(Shim::new($e))
//     };
// }
//
// impl TimeNode {
//     fn i32(&self) -> i32 {
//         if let TimeNode::Int(x) = self {
//             *x as i32
//         } else {
//             panic!("BUG")
//         }
//     }
//     fn u32(&self) -> u32 {
//         if let TimeNode::Int(x) = self {
//             *x as u32
//         } else {
//             panic!("BUG")
//         }
//     }
//     fn usize(&self) -> usize {
//         if let TimeNode::Int(x) = self {
//             *x as usize
//         } else {
//             panic!("BUG")
//         }
//     }
//     fn grain(&self) -> k::Grain {
//         if let TimeNode::Grain(x) = self {
//             *x
//         } else {
//             panic!("BUG")
//         }
//     }
//     fn seq(&self) -> Shim {
//         if let TimeNode::Seq(x) = self {
//             x.clone()
//         } else {
//             panic!("BUG")
//         }
//     }
//     fn shifts(self) -> Vec<(k::Grain, i32)> {
//         if let TimeNode::Shifts(x) = self {
//             x
//         } else {
//             panic!("BUG")
//         }
//     }
//
//     fn eval(&self, reftime: DateTime) -> TimeEl {
//         use TimeNode::*;
//         use kronos::TimeSequence;
//         match self {
//             This(seq) => TimeEl::Time(seq._future_raw(&reftime).next().unwrap()),
//             Next(seq, n) => TimeEl::Time(
//                 seq.future(&reftime)
//                     // skip_while needed to go over 'This'
//                     .skip_while(|x| x.start <= reftime)
//                     .nth(*n)
//                     .unwrap(),
//             ),
//             Last(seq, n) => TimeEl::Time(seq.past(&reftime).nth(*n).unwrap()),
//             RefNext(seq, t0) => TimeEl::Time(seq.future(t0).next().unwrap()),
//             RefPrev(seq, t0) => TimeEl::Time(seq.past(t0).next().unwrap()),
//             Until(seq, tn) => TimeEl::Count(
//                 seq.future(&reftime)
//                     .take_while(|x| x.start < *tn && x.end <= *tn)
//                     .count() as u32,
//             ),
//             Since(seq, tn) => TimeEl::Count(
//                 seq.future(tn)
//                     .take_while(|x| x.start < reftime && x.end <= reftime)
//                     .count() as u32,
//             ),
//             Between(seq, t0, tn) => TimeEl::Count(
//                 seq.future(t0)
//                     .take_while(|x| x.start < *tn && x.end <= *tn)
//                     .count() as u32,
//             ),
//             _ => unreachable!(),
//         }
//     }
// }
//
// fn evaler_time(ev: &mut EarleyForest<'_, TimeNode>, reftime: DateTime) {
//     use TimeNode::*;
//     use kronos::*;
//     ev.action("time -> today", |_| This(Shim::new(Grains(k::Grain::Day))));
//     ev.action("time -> tomorrow", |_| {
//         Next(Shim::new(Grains(k::Grain::Day)), 0)
//     });
//     ev.action("time -> yesterday", |_| {
//         Last(Shim::new(Grains(k::Grain::Day)), 0)
//     });
//     ev.action("time -> on weekday", |t| {
//         Next(Shim::new(Weekday(t[1].u32())), 0)
//     });
//     ev.action("time -> named_seq", |t| This(t[0].seq()));
//
//     ev.action("time -> the comp_seq", |t| This(t[1].seq()));
//     ev.action("time -> this comp_seq", |t| This(t[1].seq()));
//     ev.action("time -> next comp_seq", |t| Next(t[1].seq(), 0));
//     ev.action("time -> last comp_seq", |t| Last(t[1].seq(), 0));
//
//     ev.action("time -> comp_seq after next", |t| Next(t[0].seq(), 1));
//     ev.action("time -> comp_seq before last", |t| Last(t[0].seq(), 1));
//
//     ev.action("time -> a named_seq ago", |t| Last(t[1].seq(), 0));
//     ev.action("time -> small_int named_seq ago", |t| {
//         Last(t[1].seq(), t[0].usize() - 1)
//     });
//     ev.action("time -> in small_int named_seq", |t| {
//         Next(t[2].seq(), t[1].usize() - 1)
//     });
//
//     ev.action("time -> comp_grain ago", |mut t| {
//         let shifts = t.remove(0).shifts();
//         Last(build_shifter(shifts, -1, k::Grain::Second), 0)
//     });
//
//     ev.action("time -> in comp_grain", |mut t| {
//         let shifts = t.remove(1).shifts();
//         Next(build_shifter(shifts, 1, k::Grain::Second), 0)
//     });
//
//     ev.action("time -> month year", |t| {
//         RefNext(
//             Shim::new(Grains(k::Grain::Month)),
//             Date::from_ymd(t[1].i32(), t[0].u32(), 1).and_hms(0, 0, 0),
//         )
//     });
//
//     ev.action("time -> month day_ordinal year", |t| {
//         RefNext(
//             Shim::new(Grains(k::Grain::Day)),
//             Date::from_ymd(t[2].i32(), t[0].u32(), t[1].u32()).and_hms(0, 0, 0),
//         )
//     });
//
//     ev.action("time -> comp_grain after time", move |mut t| {
//         let r = t.remove(2).eval(reftime).range();
//         let shifts = t.remove(0).shifts();
//         RefNext(build_shifter(shifts, 1, r.grain), r.start)
//     });
//
//     ev.action("time -> comp_grain before time", move |mut t| {
//         let r = t.remove(2).eval(reftime).range();
//         let shifts = t.remove(0).shifts();
//         RefPrev(build_shifter(shifts, -1, r.grain), r.start)
//     });
//
//     ev.action("time -> sequence until time", move |mut t| {
//         let time = t.remove(2).eval(reftime).range().start;
//         Until(t.remove(0).seq(), time)
//     });
//
//     ev.action("time -> sequence since time", move |mut t| {
//         let time = t.remove(2).eval(reftime).range().start;
//         Since(t.remove(0).seq(), time)
//     });
//
//     ev.action("time -> sequence between time and time", move |mut t| {
//         let tn = t.remove(4).eval(reftime).range().start;
//         let t0 = t.remove(2).eval(reftime).range().start;
//         Between(t.remove(0).seq(), t0, tn)
//     });
// }

pub struct TimeMachine<'a> {
    parser: EarleyParser,
    evaler: EarleyForest<'a, TimeAstNode>,
}

impl<'a> TimeMachine<'a> {
    pub fn new() -> TimeMachine<'a> {
        let mut ev = EarleyForest::new(terminal_eval());

        // time_spec
        ev.action("time_spec -> anchored_spec", |mut t| t.remove(0));
        ev.action("time_spec -> relative_spec", |mut t| t.remove(0));
        ev.action("time_spec -> scoped_spec", |mut t| t.remove(0));

        // anchored_spec
        ev.action("anchored_spec -> weekday", |mut t| {
            TimeAstNode::TimeSeq(TimeSeq::weekday(t.remove(0).i32() as u8))
        });
        ev.action("anchored_spec -> weekday monthname ordinal", |mut t| {
            let ordinal = t.remove(2).i32() as isize;
            let month = t.remove(1).i32() as u16;
            let weekday = t.remove(0).i32() as u8;
            let seq = TimeSeq::days()
                .within(TimeSeq::months(Some(month)), ordinal)
                .unwrap_or_else(|err| panic!("BUG: invalid within {}", err))
                .intersection(TimeSeq::weekday(weekday));
            TimeAstNode::TimeSeq(seq)
        });
        ev.action("anchored_spec -> weekday ordinal of monthname", |mut t| {
            let month = t.remove(3).i32() as u16;
            let ordinal = t.remove(1).i32() as isize;
            let weekday = t.remove(0).i32() as u8;
            let seq = TimeSeq::days()
                .within(TimeSeq::months(Some(month)), ordinal)
                .unwrap_or_else(|err| panic!("BUG: invalid within {}", err))
                .intersection(TimeSeq::weekday(weekday));
            TimeAstNode::TimeSeq(seq)
        });
        ev.action(
            "anchored_spec -> weekday ordinal of monthname yearnumber",
            |mut t| {
                // TODO
                let year = t.remove(4).i32() as u16;
                let month = t.remove(3).i32() as u16;
                let ordinal = t.remove(1).i32() as isize;
                let weekday = t.remove(0).i32() as u8;
                let seq = TimeSeq::days()
                    .within(TimeSeq::months(Some(month)), ordinal)
                    .unwrap_or_else(|err| panic!("BUG: invalid within {}", err))
                    .intersection(TimeSeq::weekday(weekday));
                TimeAstNode::TimeSeq(seq)
            },
        );
        ev.action("anchored_spec -> weekday hourspec", |mut t| {
            let hour = t.remove(1).i32() as u16;
            let weekday = t.remove(0).i32() as u8;
            TimeAstNode::TimeSeq(TimeSeq::hours(Some(hour)).intersection(TimeSeq::weekday(weekday)))
        });
        ev.action("anchored_spec -> monthname", |mut t| {
            let month = t.remove(0).i32() as u16;
            TimeAstNode::TimeSeq(TimeSeq::months(Some(month)))
        });
        ev.action("anchored_spec -> monthname yearnumber", |mut t| {
            // TODO
            let year = t.remove(1).i32() as u16;
            let month = t.remove(0).i32() as u16;
            TimeAstNode::TimeSeq(TimeSeq::months(Some(month)))
        });
        ev.action("anchored_spec -> monthname ordinal", |mut t| {
            let ordinal = t.remove(1).i32() as isize;
            let month = t.remove(0).i32() as u16;
            TimeAstNode::TimeSeq(
                TimeSeq::days()
                    .within(TimeSeq::months(Some(month)), ordinal)
                    .unwrap(), // TODO: how should we deal with this?
            )
        });
        ev.action("anchored_spec -> monthname ordinal yearnumber", |mut t| {
            // TODO
            let year = t.remove(2).i32() as u16;
            let ordinal = t.remove(1).i32() as isize;
            let month = t.remove(0).i32() as u16;
            TimeAstNode::TimeSeq(
                TimeSeq::days()
                    .within(TimeSeq::months(Some(month)), ordinal)
                    .unwrap_or_else(|err| panic!("BUG: invalid within {}", err)),
            )
        });
        ev.action("anchored_spec -> ordinal 'of' monthname", |mut t| {
            let month = t.remove(2).i32() as u16;
            let ordinal = t.remove(0).i32() as isize;
            TimeAstNode::TimeSeq(
                TimeSeq::days()
                    .within(TimeSeq::months(Some(month)), ordinal)
                    .unwrap_or_else(|err| panic!("BUG: invalid within {}", err)),
            )
        });
        ev.action(
            "anchored_spec -> ordinal 'of' monthname yearnumber",
            |mut t| {
                // TODO
                let year = t.remove(2).i32() as u16;
                let month = t.remove(2).i32() as u16;
                let ordinal = t.remove(0).i32() as isize;
                TimeAstNode::TimeSeq(
                    TimeSeq::days()
                        .within(TimeSeq::months(Some(month)), ordinal)
                        .unwrap_or_else(|err| panic!("BUG: invalid within {}", err)),
                )
            },
        );
        ev.action("anchored_spec -> yearnumber", |mut t| {
            let year = t.remove(0).i32() as i32;
            TimeAstNode::TimeSpan(TimeSpan::year(year))
        });
        ev.action("anchored_spec -> hourspec", |mut t| {
            let hour = t.remove(0).i32() as u16;
            TimeAstNode::TimeSeq(TimeSeq::hours(Some(hour)))
        });
        ev.action("anchored_spec -> now", |_| TimeAstNode::RelTimeSpan {
            shift: (Grain::Second, 0),
            grain: Grain::Second,
        });
        ev.action("anchored_spec -> today", |_| TimeAstNode::RelTimeSpan {
            shift: (Grain::Day, 0),
            grain: Grain::Day,
        });
        ev.action("anchored_spec -> yesterday", |_| TimeAstNode::RelTimeSpan {
            shift: (Grain::Day, -1),
            grain: Grain::Day,
        });
        ev.action("anchored_spec -> tomorrow", |_| TimeAstNode::RelTimeSpan {
            shift: (Grain::Day, 1),
            grain: Grain::Day,
        });

        TimeMachine {
            parser: crate::time_parser::time_parser(),
            evaler: ev,
        }
    }

    pub fn eval(&self, time: &str, reftime: Option<DateTime>) -> Result<Vec<TimeStream<'a>>, String> {
        let mut tokenizer = time.split(&[' ', ','][..]).filter(|w| !w.is_empty());
        let state = self
            .parser
            .parse(&mut tokenizer)
            .map_err(|e| format!("TimeMachine {:?} for '{}'", e, time))?;

        // Default reftim to now
        let reftime = reftime.unwrap_or(DateTime::now());

        Ok(self
            .evaler
            .eval_all(&state)
            .map_err(|e| format!("TimeMachine {:?} for '{}'", e, time))?
            .into_iter()
            .map(|tree| tree.eval(reftime))
            .collect())
    }
}
